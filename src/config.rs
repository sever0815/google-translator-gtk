use gtk::glib;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct Config {
    pub favorites: Vec<String>,
    pub source: String,
    pub target: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            favorites: vec!["en".into(), "ru".into()],
            source: "auto".into(),
            target: "ru".into(),
        }
    }
}

fn config_path() -> PathBuf {
    glib::user_config_dir()
        .join("translator-gtk")
        .join("config.json")
}

pub fn load() -> Config {
    fs::read_to_string(config_path())
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

pub fn save(cfg: &Config) {
    let path = config_path();
    if let Some(dir) = path.parent() {
        let _ = fs::create_dir_all(dir);
    }
    if let Ok(s) = serde_json::to_string_pretty(cfg) {
        let _ = fs::write(path, s);
    }
}
