//! Actor texts (MSBT) handling — port of the Python `texts.py`, using the
//! `msyt` library (MSBT <-> text model) instead of PyMsyt.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use msyt::model::{Content, Entry};
use msyt::{Endianness, Msyt};
use roead::sarc::{Sarc, SarcWriter};

use crate::settings::Settings;
use crate::util;

const PROFILE: &str = "ActorType";

#[derive(Clone, Debug, Default)]
pub struct ActorTexts {
    pub texts: HashMap<String, String>,
    pub actor_name: String,
    pub profile: String,
    pub root_dir: Option<PathBuf>,
    pub lang: String,
}

impl ActorTexts {
    pub fn new(pack: &Path, profile: &str) -> Self {
        Self::new_with_lang(pack, profile, Settings::load().lang.clone())
    }

    pub fn new_with_lang(pack: &Path, profile: &str, lang: String) -> Self {
        let actor_name = pack
            .file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_default();
        let mut root_dir = pack.parent().map(|p| p.to_path_buf()).unwrap_or_default();
        loop {
            if root_dir.join("Actor").exists()
                || (root_dir.file_name().map(|n| n != "Actor").unwrap_or(true)
                    && root_dir.join("Pack").exists())
            {
                break;
            }
            if !root_dir.pop() {
                break;
            }
        }
        ActorTexts {
            texts: HashMap::new(),
            actor_name,
            profile: profile.to_string(),
            root_dir: Some(root_dir),
            lang,
        }
    }

    /// Load texts for this actor from the Bootup pack of the selected language.
    pub fn load(&mut self) -> anyhow::Result<()> {
        let settings = Settings::load();
        let lang = self.lang.clone();
        self.texts.clear();

        let root_dir = match &self.root_dir {
            Some(d) => d.clone(),
            None => return Ok(()),
        };
        let text_pack = root_dir.join(format!("Pack/Bootup_{lang}.pack"));
        let text_pack = if text_pack.exists() {
            text_pack
        } else {
            PathBuf::from(&settings.update_dir).join(format!("Pack/Bootup_{lang}.pack"))
        };
        if !text_pack.exists() {
            return Ok(());
        }

        let text_sarc = Sarc::new(std::fs::read(&text_pack)?)?;
        let message = format!("Message/Msg_{lang}.product.ssarc");
        let message_data = text_sarc
            .try_get_data(&message)?
            .ok_or_else(|| anyhow::anyhow!("{message} not found in Bootup pack"))?;
        let message_sarc = Sarc::new(roead::yaz0::decompress(message_data)?)?;
        let msbt_name = format!("{PROFILE}/{}.msbt", self.profile);
        let msbt_data = match message_sarc.try_get_data(&msbt_name)? {
            Some(d) => d,
            None => return Ok(()),
        };

        let msyt = Msyt::from_msbt_bytes(msbt_data)?;
        let prefix = format!("{}_", self.actor_name);
        for (label, entry) in msyt.entries.iter() {
            if let Some(key) = label.strip_prefix(&prefix) {
                let mut text = String::new();
                for content in &entry.contents {
                    if let Content::Text(s) = content {
                        text.push_str(s);
                    }
                }
                self.texts.insert(key.to_string(), text);
            }
        }
        Ok(())
    }

    pub fn get_texts(&self) -> &HashMap<String, String> {
        &self.texts
    }

    pub fn set_texts(&mut self, texts: HashMap<String, String>) {
        self.texts = texts;
    }

    pub fn set_actor_name(&mut self, name: String) {
        self.actor_name = name;
    }

    /// Write the texts into a mod directory's Bootup pack.
    pub fn write(&self, root_str: &Path, be: bool) -> anyhow::Result<()> {
        if self.texts.is_empty() {
            return Ok(());
        }
        let lang = self.lang.clone();
        let endianness = if be { Endianness::Big } else { Endianness::Little };

        let text_pack = root_str.join(format!("Pack/Bootup_{lang}.pack"));
        let raw = if text_pack.exists() {
            std::fs::read(&text_pack)?
        } else {
            if let Some(parent) = text_pack.parent() {
                std::fs::create_dir_all(parent)?;
            }
            let found = util::find_file(&format!("Pack/Bootup_{lang}.pack"))?;
            let bytes = found.read_bytes()?;
            std::fs::write(&text_pack, &bytes)?;
            bytes
        };

        let text_sarc = Sarc::new(raw)?;
        let mut text_sarc_writer = SarcWriter::from_sarc(&text_sarc);
        let message = format!("Message/Msg_{lang}.product.ssarc");
        let message_data = text_sarc_writer
            .get_file(&message)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("{message} not found in Bootup pack"))?;
        let message_sarc: Sarc = Sarc::new(roead::yaz0::decompress(&message_data)?)?;
        let mut message_sarc_writer = SarcWriter::from_sarc(&message_sarc);
        let msbt_name = format!("{PROFILE}/{}.msbt", self.profile);
        let msbt_bytes = message_sarc_writer
            .get_file(&msbt_name)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("{msbt_name} not found"))?;

        let mut msyt = Msyt::from_msbt_bytes(&msbt_bytes)?;
        for (key, text) in self.texts.iter() {
            msyt.entries.insert(
                format!("{}_{}", self.actor_name, key),
                Entry {
                    attributes: None,
                    contents: vec![Content::Text(text.clone())],
                },
            );
        }
        let new_msbt = msyt.into_msbt_bytes(endianness)?;
        message_sarc_writer.add_file(msbt_name, new_msbt);
        let message_bytes = message_sarc_writer.to_binary();
        text_sarc_writer.add_file(message, roead::yaz0::compress(&message_bytes));
        std::fs::write(&text_pack, text_sarc_writer.to_binary())?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use indexmap::IndexMap;
    use msyt::model::{MsbtInfo, Msyt};
    use roead::Endian as REndian;

    fn make_bootup() -> Vec<u8> {
        let mut msyt = Msyt {
            msbt: MsbtInfo {
                group_count: 1,
                atr1_unknown: None,
                ato1: None,
                tsy1: None,
                nli1: None,
            },
            entries: IndexMap::new(),
        };
        msyt.entries.insert(
            "TestActor_Name".to_string(),
            Entry {
                attributes: None,
                contents: vec![Content::Text("Hello".to_string())],
            },
        );
        msyt.entries.insert(
            "TestActor_Desc".to_string(),
            Entry {
                attributes: None,
                contents: vec![Content::Text("A description".to_string())],
            },
        );
        let msbt = msyt.into_msbt_bytes(Endianness::Little).unwrap();

        let mut inner = SarcWriter::new(REndian::Little);
        inner.add_file("ActorType/Armor.msbt", msbt);
        let inner_bytes = inner.to_binary();

        let mut outer = SarcWriter::new(REndian::Little);
        outer.add_file(
            "Message/Msg_USen.product.ssarc",
            roead::yaz0::compress(&inner_bytes),
        );
        outer.to_binary()
    }

    #[test]
    fn texts_load_and_write_roundtrip() {
        let bootup = make_bootup();
        let root = std::env::temp_dir().join(format!("bat_texts_t1_{}", std::process::id()));
        let pack_dir = root.join("Pack");
        std::fs::create_dir_all(&pack_dir).unwrap();
        std::fs::write(pack_dir.join("Bootup_USen.pack"), &bootup).unwrap();

        let actor_pack = root.join("Actor").join("Pack").join("TestActor.sbactorpack");
        let mut texts = ActorTexts::new_with_lang(&actor_pack, "Armor", "USen".to_string());
        texts.load().unwrap();
        assert_eq!(
            texts.get_texts().get("Name").map(|s| s.as_str()),
            Some("Hello")
        );
        assert_eq!(
            texts.get_texts().get("Desc").map(|s| s.as_str()),
            Some("A description")
        );

        // Rename and write back, then verify the on-disk result round-trips.
        let mut new = HashMap::new();
        new.insert("Name".to_string(), "Renamed".to_string());
        texts.set_texts(new);
        texts.write(&root, false).unwrap();

        let mut texts2 = ActorTexts::new_with_lang(&actor_pack, "Armor", "USen".to_string());
        texts2.load().unwrap();
        assert_eq!(
            texts2.get_texts().get("Name").map(|s| s.as_str()),
            Some("Renamed")
        );
        assert_eq!(
            texts2.get_texts().get("Desc").map(|s| s.as_str()),
            Some("A description")
        );

        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn actor_name_comes_from_file_stem() {
        let bootup = make_bootup();
        let root = std::env::temp_dir().join(format!("bat_texts_t2_{}", std::process::id()));
        let pack_dir = root.join("Pack");
        std::fs::create_dir_all(&pack_dir).unwrap();
        std::fs::write(pack_dir.join("Bootup_USen.pack"), &bootup).unwrap();

        // A directory path (the old bug) must NOT be used; pass the file path.
        let actor_pack = root.join("Actor").join("Pack").join("OtherActor.sbactorpack");
        let texts = ActorTexts::new_with_lang(&actor_pack, "Armor", "USen".to_string());
        assert_eq!(texts.actor_name, "OtherActor");

        let mut texts = texts;
        texts.load().unwrap();
        assert!(texts.get_texts().is_empty());

        std::fs::remove_dir_all(&root).ok();
    }
}
