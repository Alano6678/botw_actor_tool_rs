//! Application settings, stored as JSON (port of Python `BatSettings`,
//! which used an INI file — JSON is easier with serde).

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub game_dir: String,
    pub update_dir: String,
    pub dlc_dir: String,
    pub lang: String,
    #[serde(default = "default_ui_lang")]
    pub ui_lang: String,
    #[serde(default)]
    pub dark_theme: bool,
    #[serde(default)]
    pub show_unsupported_tabs: bool,
    #[serde(default)]
    pub win_pos_x: i32,
    #[serde(default)]
    pub win_pos_y: i32,
    #[serde(default)]
    pub win_width: i32,
    #[serde(default)]
    pub win_height: i32,
}

fn default_ui_lang() -> String {
    "en".to_string()
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            game_dir: String::new(),
            update_dir: String::new(),
            dlc_dir: String::new(),
            lang: "USen".to_string(),
            ui_lang: default_ui_lang(),
            dark_theme: false,
            show_unsupported_tabs: false,
            win_pos_x: 0,
            win_pos_y: 0,
            win_width: 0,
            win_height: 0,
        }
    }
}

pub fn get_data_dir() -> PathBuf {
    if let Ok(local) = std::env::var("LOCALAPPDATA") {
        let dir = PathBuf::from(local).join("botw_actor_tool_rs");
        if !dir.exists() {
            let _ = std::fs::create_dir_all(&dir);
        }
        return dir;
    }
    let dir = std::env::var("USERPROFILE")
        .or_else(|_| std::env::var("HOME"))
        .map(|h| PathBuf::from(h).join(".config").join("botw_actor_tool_rs"))
        .unwrap_or_else(|_| PathBuf::from(".botw_actor_tool_rs"));
    if !dir.exists() {
        let _ = std::fs::create_dir_all(&dir);
    }
    dir
}

fn settings_path() -> PathBuf {
    get_data_dir().join("settings.json")
}

impl Settings {
    pub fn load() -> Self {
        let path = settings_path();
        if path.exists() {
            if let Ok(text) = std::fs::read_to_string(&path) {
                if let Ok(s) = serde_json::from_str::<Settings>(&text) {
                    return s;
                }
            }
        }
        Settings::default()
    }

    pub fn save(&self) -> anyhow::Result<()> {
        let path = settings_path();
        let text = serde_json::to_string_pretty(self)?;
        std::fs::write(path, text)?;
        Ok(())
    }

    pub fn validate_game_dir(&self, game_path: &str) -> bool {
        let p = PathBuf::from(game_path);
        p.is_dir() && p.join("Pack").join("Dungeon000.pack").exists()
    }

    pub fn validate_update_dir(&self, update_path: &str) -> bool {
        let p = PathBuf::from(update_path);
        p.is_dir()
            && p.join("Actor").join("Pack").join("ActorObserverByActorTagTag.sbactorpack")
                .exists()
    }

    pub fn validate_dlc_dir(&self, dlc_path: &str) -> bool {
        let p = PathBuf::from(dlc_path);
        p.is_dir() && p.join("Pack").join("AocMainField.pack").exists()
    }
}
