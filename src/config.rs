use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{Read, Write};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub refresh_interval_ms: u64,
    pub log_level: String,
    pub theme_index: usize,
    pub storage_min_threshold_mb: u64,
    pub storage_threads: Option<usize>,
    pub enabled_plugins: Vec<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            refresh_interval_ms: 1000,
            log_level: "info".to_string(),
            theme_index: 0,
            storage_min_threshold_mb: 1,
            storage_threads: None,
            enabled_plugins: Vec::new(),
        }
    }
}

impl Config {
    fn config_path() -> PathBuf {
        let mut dir = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/home"));
        dir.push(".jarvis");
        fs::create_dir_all(&dir).ok();
        dir.push("config.json");
        dir
    }

    pub fn load() -> Self {
        let path = Self::config_path();
        if let Ok(mut f) = fs::File::open(&path) {
            let mut s = String::new();
            if f.read_to_string(&mut s).is_ok() {
                if let Ok(cfg) = serde_json::from_str::<Config>(&s) {
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
        let data = serde_json::to_string_pretty(self).unwrap_or_else(|_| "{}".to_string());
        let mut f = fs::File::create(path)?;
        f.write_all(data.as_bytes())
    }
}
