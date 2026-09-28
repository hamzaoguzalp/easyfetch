use serde::Deserialize;
use std::path::PathBuf;

pub mod key {
    pub const QUIT: u8 = b'q';
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct AppConfig {
    pub refresh_interval: Option<u64>,
    pub logo: Option<LogoConfig>,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct LogoConfig {
    pub source: Option<String>,
    pub mode: Option<String>,
    pub colors: Option<Vec<String>>,
}

impl AppConfig {
    pub fn load() -> Self {
        find_config_path()
            .and_then(|p| std::fs::read_to_string(p).ok())
            .and_then(|c| toml::from_str::<AppConfig>(&c).ok())
            .unwrap_or_default()
    }
}

pub const DEFAULT_CONFIG: &str = include_str!("../assets/config.example.toml");

pub fn get_default_config_path() -> Option<PathBuf> {
    if let Ok(xdg) = std::env::var("XDG_CONFIG_HOME") {
        return Some(PathBuf::from(xdg).join("easyfetch/config.toml"));
    }
    if let Ok(home) = std::env::var("HOME") {
        return Some(PathBuf::from(home).join(".config/easyfetch/config.toml"));
    }
    None
}

pub fn generate_default_config() -> Result<PathBuf, String> {
    let path = get_default_config_path().ok_or_else(|| {
        "Could not determine user configuration directory ($HOME or $XDG_CONFIG_HOME is not set)"
            .to_string()
    })?;

    if path.exists() {
        return Err(format!("Config file already exists at {}", path.display()));
    }

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create directory {}: {}", parent.display(), e))?;
    }

    std::fs::write(&path, DEFAULT_CONFIG)
        .map_err(|e| format!("Failed to write {}: {}", path.display(), e))?;
    Ok(path)
}

pub fn find_config_path() -> Option<PathBuf> {
    // 1. Current working directory: easyfetch.toml
    let local = PathBuf::from("easyfetch.toml");
    if local.is_file() {
        return Some(local);
    }

    // 2. $XDG_CONFIG_HOME/easyfetch/config.toml
    if let Ok(xdg) = std::env::var("XDG_CONFIG_HOME") {
        let p = PathBuf::from(xdg).join("easyfetch/config.toml");
        if p.is_file() {
            return Some(p);
        }
    }

    // 3. ~/.config/easyfetch/config.toml
    if let Ok(home) = std::env::var("HOME") {
        let p = PathBuf::from(home).join(".config/easyfetch/config.toml");
        if p.is_file() {
            return Some(p);
        }
    }

    None
}

pub fn expand_tilde(path_str: &str) -> PathBuf {
    if let Some(rest) = path_str.strip_prefix("~/") {
        let home = std::env::var("HOME").unwrap_or_default();
        if !home.is_empty() {
            return PathBuf::from(home).join(rest);
        }
    }
    PathBuf::from(path_str)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_config() {
        let sample = r#"
refresh_interval = 2

[logo]
source = "fedora"
mode = "compact"
colors = ["blue", "white"]
"#;
        let config: AppConfig = toml::from_str(sample).unwrap();
        assert_eq!(config.refresh_interval, Some(2));
        let logo = config.logo.unwrap();
        assert_eq!(logo.source.as_deref(), Some("fedora"));
        assert_eq!(logo.mode.as_deref(), Some("compact"));
        assert_eq!(
            logo.colors,
            Some(vec!["blue".to_string(), "white".to_string()])
        );
    }

    #[test]
    fn test_expand_tilde() {
        let p = expand_tilde("/etc/hosts");
        assert_eq!(p, PathBuf::from("/etc/hosts"));
    }
}
