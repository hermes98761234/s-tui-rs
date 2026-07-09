use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(default)]
pub struct Config {
    pub refresh_rate: f64,
    pub temp_threshold: f32,
    /// Source names ("Temp", "Freq", ...) hidden in the TUI
    pub hidden_sources: Vec<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            refresh_rate: 2.0,
            temp_threshold: 80.0,
            hidden_sources: Vec::new(),
        }
    }
}

fn path() -> Option<PathBuf> {
    directories::ProjectDirs::from("", "", "s-tui-rs").map(|d| d.config_dir().join("config.toml"))
}

/// Missing or unparsable config falls back to defaults — never errors.
pub fn load() -> Config {
    path()
        .and_then(|p| std::fs::read_to_string(p).ok())
        .and_then(|s| toml::from_str(&s).ok())
        .unwrap_or_default()
}

pub fn save(cfg: &Config) {
    if let Some(p) = path() {
        if let Some(dir) = p.parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        let _ = std::fs::write(p, toml::to_string(cfg).expect("config serializes"));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_toml_round_trip() {
        let cfg = Config {
            refresh_rate: 1.5,
            temp_threshold: 75.0,
            hidden_sources: vec!["Fan".into()],
        };
        let s = toml::to_string(&cfg).unwrap();
        let back: Config = toml::from_str(&s).unwrap();
        assert_eq!(back, cfg);
    }

    #[test]
    fn empty_toml_gives_defaults() {
        let cfg: Config = toml::from_str("").unwrap();
        assert_eq!(cfg, Config::default());
    }
}
