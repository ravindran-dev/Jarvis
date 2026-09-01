use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{Read, Write};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MacroDef {
    pub description: String,
    pub steps: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub macros: std::collections::HashMap<String, MacroDef>,
    #[serde(default)]
    pub aliases: std::collections::HashMap<String, String>,

    pub refresh_interval_ms: u64,
    pub log_level: String,
    pub theme_index: usize,
    pub storage_min_threshold_mb: u64,
    pub storage_threads: Option<usize>,

    // New JARVIS settings
    pub welcome_screen: bool,
    pub prompt_style: String,
    pub auto_refresh: bool,
    pub shell: String,
    pub terminal: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            macros: Default::default(),
            aliases: Default::default(),
            refresh_interval_ms: 1000,
            log_level: "info".to_string(),
            theme_index: 0,
            storage_min_threshold_mb: 1,
            storage_threads: None,
            welcome_screen: true,
            prompt_style: "Portal".to_string(),
            auto_refresh: true,
            shell: "Zsh".to_string(),
            terminal: "Kitty".to_string(),
        }
    }
}

impl Config {
    fn config_path() -> PathBuf {
        let mut dir = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/home"));
        dir.push(".config");
        dir.push("jarvis");
        fs::create_dir_all(&dir).ok();
        dir.push("config.toml");
        dir
    }

    pub fn load() -> Self {
        let path = Self::config_path();
        if let Ok(mut f) = fs::File::open(&path) {
            let mut s = String::new();
            if f.read_to_string(&mut s).is_ok() {
                if let Ok(cfg) = toml::from_str::<Config>(&s) {
                    return cfg;
                }
            }
        }
        Self::default()
    }

    pub fn save(&self) -> std::io::Result<()> {
        let path = Self::config_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let data = toml::to_string_pretty(self).unwrap_or_else(|_| "".to_string());
        let mut f = fs::File::create(path)?;
        f.write_all(data.as_bytes())
    }
}
