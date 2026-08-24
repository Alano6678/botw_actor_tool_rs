//! Actor pack container - a port of the Python `pack.py`.
//!
//! An `.sbactorpack` is a SARC archive containing the ActorLink AAMP file
//! plus the files referenced by its links (AAMP/BYML) and miscellaneous files.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use roead::aamp::{get_default_name_table, Name, Parameter, ParameterIO, ParameterObject};
use roead::byml::Byml;
use roead::sarc::{Sarc, SarcWriter};
use roead::types::FixedSafeString;
use roead::Endian;

use crate::util;

/// Source of actor data: a plain file path, or a file nested inside the
/// resident `TitleBG.pack` (given as `titlebg // inner path`).
#[derive(Clone, Debug)]
pub enum PackSource {
    Path(PathBuf),
    Resident { titlebg: PathBuf, inner: String },
}

impl PackSource {
    /// Parse the "//"-style string used for resident actors.
    pub fn from_nested(s: &str) -> Self {
        if let Some((t, i)) = s.split_once("//") {
            PackSource::Resident {
                titlebg: PathBuf::from(t),
                inner: i.to_string(),
            }
        } else {
            PackSource::Path(PathBuf::from(s))
        }
    }

    fn stem(&self) -> String {
        match self {
            PackSource::Path(p) => p
                .file_stem()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_default(),
            PackSource::Resident { inner, .. } => Path::new(inner)
                .file_stem()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_default(),
        }
    }

    /// Read raw (still possibly Yaz0-compressed) bytes.
    pub fn read_bytes(&self) -> anyhow::Result<Vec<u8>> {
        match self {
            PackSource::Path(p) => Ok(std::fs::read(p)?),
            PackSource::Resident { titlebg, inner } => {
                let data = std::fs::read(titlebg)?;
                let sarc = Sarc::new(data)?;
                let inner_bytes = sarc
                    .get_data(inner)
                    .ok_or_else(|| anyhow::anyhow!("{inner} not found in {titlebg:?}"))?;
                Ok(inner_bytes.to_vec())
            }
        }
    }

    /// Directory of the source, for locating sibling files (mod context).
    pub fn parent(&self) -> PathBuf {
        match self {
            PackSource::Path(p) => p
                .parent()
                .map(|p| p.to_path_buf())
                .unwrap_or_default(),
            PackSource::Resident { .. } => PathBuf::new(),
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct ActorPack {
    actorname: String,
    aampfiles: HashMap<String, ParameterIO>,
    bymlfiles: HashMap<String, Byml>,
    miscfiles: HashMap<String, Vec<u8>>,
    links: HashMap<String, String>,
    tags: Vec<String>,
    misc_tags: Vec<(Name, ParameterObject)>,
    source: Option<PackSource>,
}

impl ActorPack {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_actor(&mut self, source: &PackSource) -> anyhow::Result<()> {
        let name = source.stem();
        let data = source.read_bytes()?;
        let data = util::unyaz_if_needed(&data);
        let sarc = Sarc::new(data)?;

        let actorlink_name = format!("Actor/ActorLink/{name}.bxml");
        let actorlink_bytes = sarc
            .try_get_data(&actorlink_name)?
            .ok_or_else(|| anyhow::anyhow!("{actorlink_name} not found in actor pack"))?;
        let actorlink = ParameterIO::from_binary(actorlink_bytes)?;

        self.actorname = name.clone();
        self.source = Some(source.clone());
        self.links.clear();
        self.tags.clear();
        self.misc_tags.clear();
        self.aampfiles.clear();
        self.bymlfiles.clear();
        self.miscfiles.clear();

        let nt = get_default_name_table();
        let mut handled: Vec<String> = vec![actorlink_name.clone()];
        for (key, obj) in actorlink.param_root.objects.iter() {
            let key_name = nt
                .get_name(key.hash(), 0, 0)
                .map(|s| s.to_string())
                .unwrap_or_default();
            if key_name == "LinkTarget" {
                for (pkey, param) in obj.0.iter() {
                    let pname = nt
                        .get_name(pkey.hash(), 0, 0)
                        .map(|s| s.to_string())
                        .unwrap_or_default();
                    if !pname.is_empty() {
                        self.links
                            .insert(pname, param.as_str().unwrap_or("").to_string());
                    }
                }
            } else if key_name == "Tags" {
                for (_, param) in obj.0.iter() {
                    self.tags.push(param.as_str().unwrap_or("").to_string());
                }
            } else {
                self.misc_tags.push((*key, obj.clone()));
            }
        }

        for (link, (folder, ext)) in util::AAMP_LINK_REFS.iter() {
            if self.links.get(*link).map(|s| s.as_str()) == Some("Dummy") {
                continue;
            }
            let linkref = &self.links[*link];
            let filename = format!("Actor/{folder}/{linkref}{ext}");
            if let Some(filedata) = sarc.try_get_data(&filename)? {
                self.aampfiles
                    .insert(link.to_string(), ParameterIO::from_binary(filedata)?);
                handled.push(filename);
            }
        }

        for (link, (folder, ext)) in util::BYML_LINK_REFS.iter() {
            if self.links.get(*link).map(|s| s.as_str()) == Some("Dummy") {
                continue;
            }
            let linkref = &self.links[*link];
            let filename = format!("Actor/{folder}/{linkref}{ext}");
            if let Some(filedata) = sarc.try_get_data(&filename)? {
                self.bymlfiles
                    .insert(link.to_string(), Byml::from_binary(filedata)?);
                handled.push(filename);
            }
        }

        for file in sarc.files() {
            let name = file.name().unwrap_or("").to_string();
            if !name.is_empty() && !handled.contains(&name) {
                self.miscfiles.insert(name, file.data().to_vec());
            }
        }

        Ok(())
    }

    pub fn get_name(&self) -> String {
        self.actorname.clone()
    }

    /// Rename the actor, updating links, embedded file data and file names
    /// (port of the Python `set_name`).
    pub fn set_name(&mut self, name: String) {
        let old = self.actorname.clone();
        for (_, linkref) in self.links.iter_mut() {
            if *linkref == old {
                *linkref = name.clone();
            }
        }
        for (_, value) in self.aampfiles.iter_mut() {
            let yaml = value.to_text();
            if yaml.contains(&old) {
                *value = ParameterIO::from_text(yaml.replace(&old, &name)).unwrap_or_default();
            }
        }
        for (_, value) in self.bymlfiles.iter_mut() {
            let text = value.to_text();
            if text.contains(&old) {
                *value = Byml::from_text(text.replace(&old, &name)).unwrap_or_default();
            }
        }
        let old_files: Vec<String> = self.miscfiles.keys().cloned().collect();
        for filename in old_files {
            if filename.contains(&old) {
                let new_filename = filename.replace(&old, &name);
                let data = self.miscfiles.remove(&filename).unwrap_or_default();
                self.miscfiles.insert(new_filename, data);
            }
        }
        self.actorname = name.clone();
        // Armor model folder special case
        if name.starts_with("Armor_")
            && self.links.get("ModelUser").map(|s| s.as_str()) == Some(name.as_str())
        {
            if let Some(model) = self.aampfiles.get_mut("ModelUser") {
                let folder = name
                    .rsplit_once('_')
                    .map(|(f, _)| f.to_string())
                    .unwrap_or_else(|| name.clone());
                if let Some(base) = model
                    .param_root
                    .lists
                    .get_mut("ModelData")
                    .and_then(|l| l.lists.get_mut("ModelData_0"))
                    .and_then(|l| l.objects.get_mut("Base"))
                {
                    base.0.insert(
                        "Folder".into(),
                        Parameter::from(FixedSafeString::<64>::from(folder.as_str())),
                    );
                }
            }
        }
    }

    pub fn get_link(&self, link: &str) -> String {
        self.links
            .get(link)
            .cloned()
            .unwrap_or_else(|| "Dummy".into())
    }

    pub fn set_link(&mut self, link: &str, linkref: &str) {
        let old = self
            .links
            .get(link)
            .cloned()
            .unwrap_or_else(|| "Dummy".into());
        self.links.insert(link.to_string(), linkref.to_string());
        if util::AAMP_LINK_REFS.iter().any(|(l, _)| *l == link) {
            if old == "Dummy" {
                self.aampfiles.insert(link.to_string(), ParameterIO::new());
            } else if linkref == "Dummy" {
                self.aampfiles.remove(link);
            }
        } else if util::BYML_LINK_REFS.iter().any(|(l, _)| *l == link) {
            if old == "Dummy" {
                self.bymlfiles.insert(
                    link.to_string(),
                    Byml::from_iter(std::iter::empty::<(String, Byml)>()),
                );
            } else if linkref == "Dummy" {
                self.bymlfiles.remove(link);
            }
        }
    }

    pub fn get_link_data(&self, link: &str) -> String {
        let linkref = self.get_link(link);
        if linkref != "Dummy" {
            if let Some(pio) = self.aampfiles.get(link) {
                return pio.to_text();
            } else if let Some(byml) = self.bymlfiles.get(link) {
                return byml.to_text();
            }
        }
        String::new()
    }

    /// Parse and store edited YAML for a link. Errors propagate so the UI
    /// can show them (previously parse failures were silently swallowed,
    /// making "Save" appear to have no effect).
    pub fn set_link_data(&mut self, link: &str, data: &str) -> anyhow::Result<()> {
        if util::AAMP_LINK_REFS.iter().any(|(l, _)| *l == link) {
            let pio = ParameterIO::from_text(data)
                .map_err(|e| anyhow::anyhow!("Failed to parse {link} YAML: {e}"))?;
            self.aampfiles.insert(link.to_string(), pio);
        } else if util::BYML_LINK_REFS.iter().any(|(l, _)| *l == link) {
            let byml = Byml::from_text(data)
                .map_err(|e| anyhow::anyhow!("Failed to parse {link} YAML: {e}"))?;
            self.bymlfiles.insert(link.to_string(), byml);
        }
        Ok(())
    }

    pub fn get_tags(&self) -> String {
        self.tags.join(", ")
    }

    /// Names of the editable link files (link names) available in the pack.
    pub fn link_file_names(&self) -> Vec<String> {
        let mut v: Vec<String> = self.aampfiles.keys().cloned().collect();
        v.extend(self.bymlfiles.keys().cloned());
        v
    }

    /// Paths of the non-editable files kept as-is inside the pack
    /// (e.g. `Physics/Cloth/...hkcl`, `Actor/AS/...bas`).
    pub fn misc_paths(&self) -> Vec<String> {
        let mut v: Vec<String> = self.miscfiles.keys().cloned().collect();
        v.sort();
        v
    }

    /// Size in bytes of a misc (non-editable) file.
    pub fn misc_size(&self, path: &str) -> usize {
        self.miscfiles.get(path).map(|d| d.len()).unwrap_or(0)
    }

    /// Replace the contents of a misc file (or insert if it is new).
    pub fn replace_misc(&mut self, path: &str, data: Vec<u8>) {
        self.miscfiles.insert(path.to_string(), data);
    }

    /// Rename a misc file's path key (moving it to a new in-pack location).
    pub fn rename_misc(&mut self, old: &str, new: &str) {
        if let Some(data) = self.miscfiles.remove(old) {
            self.miscfiles.insert(new.to_string(), data);
        }
    }

    pub fn set_tags(&mut self, tags: &str) {
        self.tags = tags.split(", ").map(|s| s.to_string()).collect();
    }

    pub fn get_actorlink(&self) -> ParameterIO {
        let mut actorlink = ParameterIO::new();
        actorlink.data_type = "xml".into();
        let mut link_target = ParameterObject::new();
        for (link, linkref) in self.links.iter() {
            link_target
                .0
                .insert(link.as_str().into(), make_string_param(linkref));
        }
        actorlink
            .param_root
            .objects
            .insert("LinkTarget", link_target);
        if !self.tags.is_empty() {
            let mut tags = ParameterObject::new();
            for (i, tag) in self.tags.iter().enumerate() {
                tags.0
                    .insert(format!("Tag{i}").as_str().into(), make_string_param(tag));
            }
            actorlink
                .param_root
                .objects
                .insert("Tags", tags);
        }
        for (key, obj) in self.misc_tags.iter() {
            actorlink.param_root.objects.insert(*key, obj.clone());
        }
        actorlink
    }

    pub fn get_bytes(&self, be: bool) -> anyhow::Result<Vec<u8>> {
        let endian = if be { Endian::Big } else { Endian::Little };
        let mut writer = SarcWriter::new(endian);

        let filename = format!("Actor/ActorLink/{}.bxml", self.actorname);
        writer.add_file(filename, self.get_actorlink().to_binary());

        for (link, data) in self.aampfiles.iter() {
            if let Some((folder, ext)) =
                util::AAMP_LINK_REFS.iter().find(|(l, _)| *l == link).map(|(_, v)| *v)
            {
                let filename = format!("Actor/{folder}/{}{ext}", self.get_link(link));
                writer.add_file(filename, data.to_binary());
            }
        }

        for (link, data) in self.bymlfiles.iter() {
            if let Some((folder, ext)) =
                util::BYML_LINK_REFS.iter().find(|(l, _)| *l == link).map(|(_, v)| *v)
            {
                let filename = format!("Actor/{folder}/{}{ext}", self.get_link(link));
                writer.add_file(filename, data.to_binary(endian));
            }
        }

        for (filename, data) in self.miscfiles.iter() {
            writer.add_file(filename.clone(), data.clone());
        }

        Ok(writer.to_binary())
    }
}

fn make_string_param(s: &str) -> Parameter {
    if s.len() <= 32 {
        Parameter::from(FixedSafeString::<32>::from(s))
    } else {
        Parameter::from(FixedSafeString::<64>::from(s))
    }
}
