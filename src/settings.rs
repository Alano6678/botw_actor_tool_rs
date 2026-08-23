//! Application settings, stored as JSON (port of Python `BatSettings`,
//! which used an INI file — JSON is easier with serde).

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    /// Which platform the editor targets: "wiiu" (big-endian) or "switch".
    #[serde(default = "default_platform")]
    pub platform: String,
    // Wii U directories.
    pub game_dir: String,
    pub update_dir: String,
    pub dlc_dir: String,
    // Switch directories.
    #[serde(default)]
    pub switch_game_dir: String,
    #[serde(default)]
    pub switch_update_dir: String,
    #[serde(default)]
    pub switch_dlc_dir: String,
    /// Wii U game text language.
    pub lang: String,
    /// Switch game text language (independent of the Wii U one).
    #[serde(default = "default_lang")]
    pub switch_lang: String,
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

fn default_platform() -> String {
    "wiiu".to_string()
}

fn default_lang() -> String {
    "USen".to_string()
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            platform: default_platform(),
            game_dir: String::new(),
            update_dir: String::new(),
            dlc_dir: String::new(),
            switch_game_dir: String::new(),
            switch_update_dir: String::new(),
            switch_dlc_dir: String::new(),
            lang: "USen".to_string(),
            switch_lang: "USen".to_string(),
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

    /// Is the current platform Switch (little-endian)?
    pub fn is_switch(&self) -> bool {
        self.platform == "switch"
    }

    /// Active game directory for the current platform.
    pub fn game(&self) -> &str {
        if self.is_switch() {
            &self.switch_game_dir
        } else {
            &self.game_dir
        }
    }

    /// Active update directory for the current platform.
    pub fn update(&self) -> &str {
        if self.is_switch() {
            &self.switch_update_dir
        } else {
            &self.update_dir
        }
    }

    /// Active DLC directory for the current platform.
    pub fn dlc(&self) -> &str {
        if self.is_switch() {
            &self.switch_dlc_dir
        } else {
            &self.dlc_dir
        }
    }

    /// Active game text language for the current platform (each platform has
    /// its own language setting).
    pub fn active_lang(&self) -> &str {
        if self.is_switch() {
            &self.switch_lang
        } else {
            &self.lang
        }
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
