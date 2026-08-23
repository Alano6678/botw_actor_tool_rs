//! Main actor editing object — port of the Python `actor.py`.

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use roead::byml::{Byml, Map};
use roead::Endian;

use crate::actorinfo;
use crate::data;
use crate::flag::crc32_str;
use crate::pack::{ActorPack, PackSource};
use crate::store::{new_flag, FlagStore};
use crate::texts::ActorTexts;
use crate::util;

pub const FAR_LINKS: &[&str] = &["LifeConditionUser", "ModelUser", "PhysicsUser"];

const FLAG_CLASSES: &[(&str, bool)] = &[
    // (prefix, is_bool) — bool means bool_data, otherwise s32_data
    ("_DispNameFlag", true),
    ("EquipTime_", false),
    ("IsGet_", true),
    ("IsNewPictureBook_", true),
    ("IsRegisteredPictureBook_", true),
    ("PictureBookSize_", false),
    ("PorchTime_", false),
];

const FLAG_TYPES: &[(&str, &[&str])] = &[
    (
        "Animal",
        &["IsNewPictureBook_", "IsRegisteredPictureBook_", "PictureBookSize_"],
    ),
    ("Armor", &["EquipTime_", "IsGet_", "PorchTime_"]),
    (
        "Enemy",
        &["IsNewPictureBook_", "IsRegisteredPictureBook_", "PictureBookSize_"],
    ),
    (
        "Item",
        &[
            "IsGet_",
            "IsNewPictureBook_",
            "IsRegisteredPictureBook_",
            "PictureBookSize_",
        ],
    ),
    ("Npc", &["_DispNameFlag"]),
    (
        "Weapon",
        &[
            "EquipTime_",
            "IsGet_",
            "IsNewPictureBook_",
            "IsRegisteredPictureBook_",
            "PictureBookSize_",
            "PorchTime_",
        ],
    ),
];

/// Try to fetch a vanilla file's data using the generic_link_files table.
pub fn try_retrieve_custom_file(link: &str, file_ref: &str) -> anyhow::Result<String> {
    if let Some(an) = data::get_generic_file(link, file_ref) {
        let found = util::find_file(&format!("Actor/Pack/{an}.sbactorpack"))?;
        let source = match found {
            util::FoundFile::Path(p) => PackSource::Path(p),
            util::FoundFile::Resident { titlebg, inner } => PackSource::Resident {
                titlebg,
                inner,
            },
        };
        let mut pack = ActorPack::new();
        pack.from_actor(&source)?;
        return Ok(pack.get_link_data(link));
    }
    Ok(String::new())
}

pub struct BATActor {
    pub pack: ActorPack,
    pub info: Map,
    pub far_pack: Option<ActorPack>,
    pub far_info: Option<Map>,
    pub needs_info_update: bool,
    pub far_needs_info_update: bool,
    pub texts: ActorTexts,
    pub flags: FlagStore,
    pub flag_hashes: (HashSet<i32>, HashSet<i32>),
    pub resident: bool,
    pub origname: String,
    pub far_origname: String,
    /// Manual ActorInfo overrides (empty string = keep the auto value).
    /// Applied on top of the regenerated entry when saving.
    pub info_overrides: HashMap<String, String>,
    /// Keep old ActorInfo fields that are not in the profile whitelist
    /// (by default the regenerated entry drops them).
    pub info_keep_extra: bool,
}

/// ActorInfo product file loaded as BYML.
pub struct ActorInfoFile {
    pub root: Byml,
    pub path: PathBuf,
}

impl ActorInfoFile {
    pub fn load(path: &Path) -> anyhow::Result<Self> {
        let raw = std::fs::read(path)?;
        let data = util::unyaz_if_needed(&raw);
        let root = Byml::from_binary(data)?;
        Ok(ActorInfoFile {
            root,
            path: path.to_path_buf(),
        })
    }

    pub fn save(&self, be: bool) -> anyhow::Result<()> {
        let endian = if be { Endian::Big } else { Endian::Little };
        let data = self.root.to_binary(endian);
        let out = roead::yaz0::compress(&data);
        std::fs::write(&self.path, out)?;
        Ok(())
    }

    pub fn actors_mut(&mut self) -> Option<&mut Vec<Byml>> {
        match self.root.as_mut_map().ok() {
            Some(m) => m.get_mut("Actors").and_then(|a| a.as_mut_array().ok()),
            None => None,
        }
    }

    pub fn hashes(&self) -> Vec<i32> {
        match self.root.as_map().ok() {
            Some(m) => m
                .get("Hashes")
                .and_then(|h| h.as_array().ok())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_i32().ok().or_else(|| v.as_u32().ok().map(|u| u as i32)))
                        .collect()
                })
                .unwrap_or_default(),
            None => Vec::new(),
        }
    }

    pub fn set_hashes(&mut self, hashes: Vec<i32>) {
        if let Some(m) = self.root.as_mut_map().ok() {
            let arr = hashes
                .iter()
                .map(|h| {
                    if *h > i32::MAX {
                        Byml::U32(*h as u32)
                    } else {
                        Byml::I32(*h)
                    }
                })
                .collect();
            m.insert("Hashes".into(), Byml::Array(arr));
        }
    }
}

impl BATActor {
    pub fn new(source: &PackSource) -> anyhow::Result<Self> {
        let mut pack = ActorPack::new();
        pack.from_actor(source)?;
        let origname = pack.get_name();
        let resident = matches!(source, PackSource::Resident { .. });

        let actorinfo_path = match source {
            PackSource::Path(p) => {
                let mut p = p.parent().map(|p| p.to_path_buf()).unwrap_or_default();
                p.pop();
                p.join("ActorInfo.product.sbyml")
            }
            PackSource::Resident { titlebg, .. } => titlebg
                .parent()
                .map(|p| p.to_path_buf())
                .unwrap_or_default()
                .join("../Actor/ActorInfo.product.sbyml"),
        };
        let mut need_find = false;
        let mut actorinfo_bytes = if actorinfo_path.exists() {
            Some(std::fs::read(&actorinfo_path)?)
        } else {
            need_find = true;
            None
        };
        if need_find {
            let found = util::find_file("Actor/ActorInfo.product.sbyml")?;
            actorinfo_bytes = Some(found.read_bytes()?);
        }
        let actorinfo = Byml::from_binary(util::unyaz_if_needed(&actorinfo_bytes.unwrap_or_default()))?;

        // Find info entries
        let actors = actorinfo
            .as_map()
            .ok()
            .and_then(|m| m.get("Actors").and_then(|a| a.as_array().ok()));
        let mut info: Option<Map> = None;
        let mut far_info: Option<Map> = None;

        let mut far_pack: Option<ActorPack> = None;
        let mut far_origname = String::new();
        if let PackSource::Path(p) = source {
            let far_path = p.with_file_name(format!("{}_Far.sbactorpack", p.file_stem().map(|s| s.to_string_lossy().to_string()).unwrap_or_default()));
            if far_path.exists() {
                let mut fp = ActorPack::new();
                fp.from_actor(&PackSource::Path(far_path))?;
                far_origname = fp.get_name();
                far_pack = Some(fp);
            }
        }

        if let Some(actors) = actors {
            for actor in actors {
                if let Some(m) = actor.as_map().ok() {
                    let name = m
                        .get("name")
                        .and_then(|v| v.as_string().ok())
                        .map(|s| s.as_str())
                        .unwrap_or("");
                    if name == origname {
                        info = Some(m.clone());
                    }
                    if let Some(fp) = &far_pack {
                        if name == fp.get_name() {
                            far_info = Some(m.clone());
                        }
                    }
                }
            }
        }
        let info = info.ok_or_else(|| {
            anyhow::anyhow!(
                "ActorInfo.product.sbyml did not contain an info entry for {origname}"
            )
        })?;
        if far_pack.is_some() && far_info.is_none() {
            return Err(anyhow::anyhow!(
                "ActorInfo.product.sbyml did not contain an info entry for {far_origname}"
            ));
        }

        let profile = pack.get_link("ProfileUser");
        // ActorTexts derives the actor name from the pack file stem and
        // walks up to find the game root, so pass the *file* path here
        // (not the directory). For resident actors the real path is inside
        // TitleBG.pack; joining it keeps the stem correct and the root walk
        // lands on the update directory.
        let text_source = match source {
            PackSource::Path(p) => p.clone(),
            PackSource::Resident { titlebg, inner } => titlebg.join(inner),
        };
        let mut texts = ActorTexts::new(&text_source, &profile);
        load_texts(&mut texts);

        let mut actor = BATActor {
            pack,
            info,
            far_pack,
            far_info,
            needs_info_update: false,
            far_needs_info_update: false,
            texts,
            flags: FlagStore::new(),
            flag_hashes: (HashSet::new(), HashSet::new()),
            resident,
            origname,
            far_origname,
            info_overrides: HashMap::new(),
            info_keep_extra: false,
        };
        actor.set_flags(&actor.origname.clone());
        Ok(actor)
    }

    pub fn get_name(&self) -> String {
        self.pack.get_name()
    }

    pub fn set_name(&mut self, name: &str) {
        self.pack.set_name(name.to_string());
        self.texts.set_actor_name(name.to_string());
        self.set_flags(name);
        self.needs_info_update = true;
        self.resident = false;
    }

    pub fn get_link(&self, link: &str) -> String {
        self.pack.get_link(link)
    }

    pub fn set_link(&mut self, link: &str, linkref: &str) -> bool {
        if self.has_far() {
            if link == "LifeConditionUser" && linkref == "Dummy" {
                return false;
            }
            self.pack.set_link(link, linkref);
            self.needs_info_update = true;
            if FAR_LINKS.contains(&link) {
                if let Some(far) = self.far_pack.as_mut() {
                    far.set_link(link, linkref);
                    self.far_needs_info_update = true;
                }
            }
            return true;
        }
        self.pack.set_link(link, linkref);
        self.needs_info_update = true;
        true
    }

    pub fn has_far(&self) -> bool {
        self.far_pack.is_some()
    }

    pub fn get_link_data(&self, link: &str) -> String {
        self.pack.get_link_data(link)
    }

    pub fn set_link_data(&mut self, link: &str, data: &str) -> anyhow::Result<()> {
        self.pack.set_link_data(link, data)?;
        self.needs_info_update = true;
        Ok(())
    }

    pub fn get_tags(&self) -> String {
        self.pack.get_tags()
    }

    pub fn set_tags(&mut self, tags: &str) {
        self.pack.set_tags(tags);
        self.needs_info_update = true;
    }

    pub fn get_info(&mut self) -> Map {
        if self.needs_info_update {
            self.info = self.info_preview();
            self.needs_info_update = false;
        }
        let mut entry = self.info.clone();
        for (key, val) in &self.info_overrides {
            if !val.is_empty() {
                entry.insert(
                    key.as_str().into(),
                    actorinfo::parse_api_value(val),
                );
            }
        }
        entry
    }

    /// The freshly regenerated ActorInfo entry (without overrides) — this is
    /// what the ActorInfo editor page shows as the "auto" values.
    pub fn info_preview(&mut self) -> Map {
        actorinfo::generate_actor_info(
            &self.pack,
            self.has_far(),
            &self.info,
            self.origname == self.pack.get_name(),
            self.info_keep_extra,
        )
        .unwrap_or_else(|e| {
            eprintln!("failed to generate actor info: {e}");
            self.info.clone()
        })
    }

    #[allow(dead_code)]
    pub fn get_actorlink(&self) -> roead::aamp::ParameterIO {
        self.pack.get_actorlink()
    }

    pub fn get_texts(&self) -> &ActorTexts {
        &self.texts
    }

    pub fn set_texts(&mut self, texts: std::collections::HashMap<String, String>) {
        self.texts.set_texts(texts);
    }

    /// Regenerate the level-1 flags for this actor (port of `set_flags`).
    pub fn set_flags(&mut self, name: &str) {
        let (bools, s32s) = &mut self.flag_hashes;
        for hash in bools.iter() {
            self.flags.remove("bool_data", *hash);
        }
        for hash in s32s.iter() {
            self.flags.remove("s32_data", *hash);
        }
        bools.clear();
        s32s.clear();

        let actor_type = name.split('_').next().unwrap_or("");
        if let Some((_, prefixes)) = FLAG_TYPES.iter().find(|(t, _)| *t == actor_type) {
            for prefix in *prefixes {
                let prefix = *prefix;
                let is_bool = FLAG_CLASSES
                    .iter()
                    .find(|(p, _)| *p == prefix)
                    .map(|(_, b)| *b)
                    .unwrap_or(false);
                let ftype = if is_bool { "bool_data" } else { "s32_data" };
                let mut flag = new_flag(ftype, false);
                let data_name = if prefix.starts_with('_') {
                    format!("{name}{prefix}")
                } else {
                    format!("{prefix}{name}")
                };
                flag.set_data_name(data_name);
                flag.use_name_to_override_params();
                if is_bool {
                    bools.insert(flag.hash_value);
                } else {
                    s32s.insert(flag.hash_value);
                }
                self.flags.add(ftype, flag);
            }
        }
    }

    /// Save to a mod dir (`content` = big endian, `romfs` = little endian).
    pub fn save(&mut self, root_dir: &Path, be: bool) -> anyhow::Result<()> {
        let pack_bytes = self.pack.get_bytes(be)?;
        let compressed = roead::yaz0::compress(&pack_bytes);

        let actor_dir = PathBuf::from("Actor").join("Pack").join(format!("{}.sbactorpack", self.get_name()));
        if self.resident {
            let titlebg = root_dir.join("Pack").join("TitleBG.pack");
            if !titlebg.exists() {
                if let Some(parent) = titlebg.parent() {
                    std::fs::create_dir_all(parent)?;
                }
                let found = util::find_file("Pack/TitleBG.pack")?;
                std::fs::write(&titlebg, found.read_bytes()?)?;
            }
            util::inject_bytes_into_sarc(
                &titlebg,
                &format!("Actor/Pack/{}.sbactorpack", self.get_name()),
                &compressed,
            )?;
        } else {
            let actor_path = root_dir.join(&actor_dir);
            if let Some(parent) = actor_path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::write(actor_path, &compressed)?;
        }

        let hash = crc32_str(&self.get_name());
        let info = self.get_info();

        let mut far_hash: Option<i32> = None;
        if let Some(far) = &self.far_pack {
            let far_pack_bytes = far.get_bytes(be)?;
            let actor_path = root_dir
                .join("Actor")
                .join("Pack")
                .join(format!("{}.sbactorpack", far.get_name()));
            if let Some(parent) = actor_path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::write(actor_path, roead::yaz0::compress(&far_pack_bytes))?;
            far_hash = Some(crc32_str(&far.get_name()));
        }
        let mut final_far_info: Option<Map> = None;
        if self.far_needs_info_update {
            if let Some(far) = &self.far_pack {
                if let Some(far_info) = &self.far_info {
                    final_far_info = actorinfo::generate_actor_info(
                        far,
                        false,
                        far_info,
                        self.far_origname == far.get_name(),
                        self.info_keep_extra,
                    )
                    .ok();
                }
            }
        } else {
            final_far_info = self.far_info.clone();
        }

        // ActorInfo
        let actorinfo_path = root_dir.join("Actor").join("ActorInfo.product.sbyml");
        let mut need_find = false;
        let mut bytes = if actorinfo_path.exists() {
            Some(std::fs::read(&actorinfo_path)?)
        } else {
            need_find = true;
            None
        };
        if need_find {
            if let Some(parent) = actorinfo_path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::write(&actorinfo_path, b"")?;
            let found = util::find_file("Actor/ActorInfo.product.sbyml")?;
            bytes = Some(found.read_bytes()?);
            std::fs::write(&actorinfo_path, bytes.as_ref().unwrap())?;
        }
        let root = Byml::from_binary(util::unyaz_if_needed(&bytes.unwrap_or_default()))?;
        let mut root = root;
        let mut hashes: Vec<i32> = match root.as_map().ok().and_then(|m| m.get("Hashes").and_then(|h| h.as_array().ok())) {
            Some(arr) => arr
                .iter()
                .filter_map(|v| v.as_i32().ok().or_else(|| v.as_u32().ok().map(|u| u as i32)))
                .collect(),
            None => Vec::new(),
        };
        let hash_index = if let Some(i) = hashes.iter().position(|h| *h == hash) {
            i
        } else {
            hashes.push(hash);
            hashes.sort_unstable();
            hashes.iter().position(|h| *h == hash).unwrap()
        };
        let mut far_hash_index: Option<usize> = None;
        if let Some(fh) = far_hash {
            far_hash_index = if let Some(i) = hashes.iter().position(|h| *h == fh) {
                Some(i)
            } else {
                hashes.push(fh);
                hashes.sort_unstable();
                hashes.iter().position(|h| *h == fh)
            };
        }
        if let Some(m) = root.as_mut_map().ok() {
            let arr = hashes
                .iter()
                .map(|h| {
                    if *h > i32::MAX {
                        Byml::U32(*h as u32)
                    } else {
                        Byml::I32(*h)
                    }
                })
                .collect();
            m.insert("Hashes".into(), Byml::Array(arr));
            if let Some(actors) = m.get_mut("Actors").and_then(|a| a.as_mut_array().ok()) {
                while actors.len() < hash_index + 1 {
                    actors.push(Byml::Map(Map::default()));
                }
                actors[hash_index] = Byml::Map(info);
                if let (Some(fi), Some(fix)) = (final_far_info, far_hash_index) {
                    while actors.len() < fix + 1 {
                        actors.push(Byml::Map(Map::default()));
                    }
                    actors[fix] = Byml::Map(fi);
                }
            }
        }
        let endian = if be { Endian::Big } else { Endian::Little };
        let out = roead::yaz0::compress(&root.to_binary(endian));
        std::fs::write(&actorinfo_path, out)?;

        // Texts
        let texts = self.texts.clone();
        texts.write(root_dir, be)?;

        // GameData flags
        let bootup_path = root_dir.join("Pack").join("Bootup.pack");
        if !bootup_path.exists() {
            if let Some(parent) = bootup_path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            let found = util::find_file("Pack/Bootup.pack")?;
            std::fs::write(&bootup_path, found.read_bytes()?)?;
        }
        let gamedata_sarc = util::get_gamedata_sarc(&bootup_path)?;
        for file in gamedata_sarc.files() {
            if let Ok(byml) = Byml::from_binary(file.data()) {
                if let Some(m) = byml.as_map().ok() {
                    let name = file.name().unwrap_or("");
                    self.flags.add_flags_from_hash_no_overwrite(name, m);
                }
            }
        }

        let orig_files = util::get_last_two_savedata_files(&bootup_path)?;
        // The originals YAZ0-compress the generated gamedata/savedata before
        // injecting; without this a second save fails with
        // "found SARC, expected Yaz0" (which also matched the rename flow).
        let gamedata = roead::yaz0::compress(&util::make_new_gamedata(&self.flags, be)?);
        let savedata = roead::yaz0::compress(&util::make_new_savedata(&self.flags, be, orig_files)?);
        util::inject_files_into_bootup(
            &bootup_path,
            &[
                ("GameData/gamedata.ssarc", gamedata),
                ("GameData/savedataformat.ssarc", savedata),
            ],
        )?;

        Ok(())
    }
}

fn load_texts(texts: &mut ActorTexts) {
    if let Err(e) = texts.load() {
        eprintln!("failed to load texts: {e}");
    }
}

/// List all vanilla actor names from the update dir ActorInfo.
pub fn get_all_vanilla_actors(update_dir: &Path) -> anyhow::Result<Vec<String>> {
    let actorinfo_path = update_dir.join("Actor").join("ActorInfo.product.sbyml");
    if !actorinfo_path.exists() {
        return Ok(Vec::new());
    }
    let raw = std::fs::read(&actorinfo_path)?;
    let byml = Byml::from_binary(util::unyaz_if_needed(&raw))?;
    let mut out = Vec::new();
    if let Some(actors) = byml
        .as_map()
        .ok()
        .and_then(|m| m.get("Actors").and_then(|a| a.as_array().ok()))
    {
        for actor in actors {
            if let Some(m) = actor.as_map().ok() {
                if let Some(name) = m.get("name").and_then(|v| v.as_string().ok()) {
                    if !name.ends_with("_Far") {
                        out.push(name.to_string());
                    }
                }
            }
        }
    }
    out.sort();
    Ok(out)
}

/// End-to-end check against a real BotW dump (if configured):
/// `cargo test -- --ignored real_dump`
#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings::Settings;

    #[test]
    #[ignore]
    fn real_dump_texts_load() {
        let settings = Settings::load();
        if settings.update_dir.is_empty() || settings.game_dir.is_empty() {
            eprintln!("game dirs not configured; skipping");
            return;
        }
        let candidates = [
            "Armor_001_Head",
            "Item_Fruit_A",
            "Weapon_Sword_001",
            "Animal_Deer_A",
            "Item_Material_01",
        ];
        let mut found_texts = false;
        for name in candidates {
            match util::find_file(&format!("Actor/Pack/{name}.sbactorpack")) {
                Ok(found) => {
                    let source = match found {
                        util::FoundFile::Path(p) => PackSource::Path(p),
                        util::FoundFile::Resident { titlebg, inner } => {
                            PackSource::Resident { titlebg, inner }
                        }
                    };
                    match BATActor::new(&source) {
                        Ok(actor) => {
                            let texts = actor.get_texts().get_texts().clone();
                            eprintln!(
                                "{name}: {} texts {:?}",
                                texts.len(),
                                texts.keys().take(4).collect::<Vec<_>>()
                            );
                            if !texts.is_empty() {
                                found_texts = true;
                            }
                        }
                        Err(e) => eprintln!("{name}: load failed: {e:#}"),
                    }
                }
                Err(e) => eprintln!("{name}: find failed: {e:#}"),
            }
        }
        assert!(
            found_texts,
            "no actor produced texts (lang={}); check update_dir/Bootup_{}.pack",
            settings.lang,
            settings.lang
        );
    }

    /// Diagnostic: real AAMP/BYML `to_text -> from_text` roundtrip.
    /// run with: cargo test aamp_roundtrip_real -- --ignored --nocapture
    #[test]
    #[ignore]
    fn aamp_roundtrip_real() {
        let settings = Settings::load();
        if settings.update_dir.is_empty() {
            return;
        }
        let candidates = [
            "Armor_001_Head",
            "Weapon_Sword_001",
            "Item_Fruit_A",
            "Animal_Deer_A",
            "Enemy_Guardian",
            "Npc_001",
            "Item_Conductor",
        ];
        let mut parse_failures: Vec<String> = Vec::new();
        let mut diff_failures: Vec<String> = Vec::new();
        for name in candidates {
            let Ok(found) = util::find_file(&format!("Actor/Pack/{name}.sbactorpack")) else {
                eprintln!("{name}: not found");
                continue;
            };
            let source = match found {
                util::FoundFile::Path(p) => PackSource::Path(p),
                util::FoundFile::Resident { titlebg, inner } => {
                    PackSource::Resident { titlebg, inner }
                }
            };
            let Ok(actor) = BATActor::new(&source) else {
                eprintln!("{name}: BATActor::new failed");
                continue;
            };
            for link in crate::util::AAMP_LINK_REFS.iter().map(|(l, _)| *l) {
                let data = actor.get_link_data(link);
                if data.is_empty() {
                    continue;
                }
                match roead::aamp::ParameterIO::from_text(&data) {
                    Ok(pio) => {
                        let back = pio.to_text();
                        if back != data {
                            diff_failures.push(format!("{name}/{link}: text differs"));
                            let a: Vec<&str> = data.lines().collect();
                            let b: Vec<&str> = back.lines().collect();
                            for i in 0..a.len().min(b.len()) {
                                if a[i] != b[i] {
                                    diff_failures.push(format!(
                                        "  first diff line {i}:\n    orig: {}\n    back: {}",
                                        a[i], b[i]
                                    ));
                                    break;
                                }
                            }
                        }
                    }
                    Err(e) => parse_failures.push(format!(
                        "{name}/{link}: from_text failed: {e:#}"
                    )),
                }
            }
            for link in crate::util::BYML_LINK_REFS.iter().map(|(l, _)| *l) {
                let data = actor.get_link_data(link);
                if data.is_empty() {
                    continue;
                }
                match roead::byml::Byml::from_text(&data) {
                    Ok(b) => {
                        let back = b.to_text();
                        if back != data {
                            diff_failures.push(format!("{name}/{link} (byml): text differs"));
                        }
                    }
                    Err(e) => parse_failures.push(format!(
                        "{name}/{link}: byml from_text failed: {e:#}"
                    )),
                }
            }
        }
        for f in &parse_failures {
            eprintln!("PARSE FAIL: {f}");
        }
        for f in &diff_failures {
            eprintln!("TEXT DIFF: {f}");
        }
        eprintln!(
            "summary: {} parse failures, {} text diffs",
            parse_failures.len(),
            diff_failures.len()
        );
    }

    /// Full edit-then-save flow against real data.
    /// run with: cargo test full_flow_real -- --ignored --nocapture
    #[test]
    #[ignore]
    fn full_flow_real() {
        let settings = Settings::load();
        if settings.update_dir.is_empty() {
            return;
        }
        let found = util::find_file("Actor/Pack/Armor_001_Head.sbactorpack").unwrap();
        let source = match found {
            util::FoundFile::Path(p) => PackSource::Path(p),
            util::FoundFile::Resident { titlebg, inner } => {
                PackSource::Resident { titlebg, inner }
            }
        };
        let mut actor = BATActor::new(&source).unwrap();
        eprintln!("actor = {}", actor.get_name());

        // Reproduce the reported bug: rename the actor, then save twice into
        // the same mod dir (the 2nd save used to fail with
        // "Bad magic value: found `SARC`, expected `Yaz0`").
        actor.set_name("Armor_001_Head_Test");
        assert_eq!(actor.get_name(), "Armor_001_Head_Test");

        let link = "GParamUser";
        let data = actor.get_link_data(link);
        eprintln!("original file head: {:?}", &data[..data.len().min(200)]);
        assert!(!data.is_empty(), "GParamUser data should not be empty");

        // Simulate the editor Save button: parse text -> set_link_data.
        let modified = data.replace("StarNum: 1", "StarNum: 2");
        assert!(
            modified != data,
            "expected 'StarNum: 1' present in GParam text for the test"
        );
        actor.set_link_data(link, &modified).unwrap();
        let back = actor.get_link_data(link);
        assert!(
            back.contains("StarNum: 2"),
            "set_link_data must take effect"
        );

        // The in-memory pack must contain the changed file.
        let bytes = actor.pack.get_bytes(false).unwrap();
        let sarc = roead::sarc::Sarc::new(bytes).unwrap();
        let gparam = sarc
            .get_data("Actor/GeneralParamList/Armor_001_Head_Test.bgparamlist")
            .expect("GParam file must exist in rebuilt pack (renamed actor)");
        let text = roead::aamp::ParameterIO::from_binary(gparam)
            .unwrap()
            .to_text();
        assert!(text.contains("StarNum: 2"), "rebuilt pack must contain the change");

        // Full save to a mod dir (romfs = little endian), then re-open.
        let root = std::env::temp_dir().join(format!("bat_flow_{}", std::process::id()));
        let root = root.join("romfs");
        std::fs::create_dir_all(&root).unwrap();
        actor.save(&root, false).unwrap();
        // Second save into the same dir (the failing scenario).
        actor.save(&root, false).unwrap();
        assert!(root.join("Actor").join("Pack").join("Armor_001_Head_Test.sbactorpack").exists());
        let mut reloaded = BATActor::new(&PackSource::Path(
            root.join("Actor").join("Pack").join("Armor_001_Head_Test.sbactorpack"),
        ))
        .unwrap();
        let reloaded_data = reloaded.get_link_data(link);
        assert!(
            reloaded_data.contains("StarNum: 2"),
            "reloaded actor must contain the change"
        );
        // check ActorInfo got written with the new entry
        let info_path = root.join("Actor").join("ActorInfo.product.sbyml");
        assert!(info_path.exists(), "ActorInfo must be written");
        eprintln!("full flow OK; saved under {:?}", root);
        std::fs::remove_dir_all(root.parent().unwrap()).ok();
    }
}
