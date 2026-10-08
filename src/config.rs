use anyhow::Result;
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

const QUALIFIER: &str = "me";
const ORGANIZATION: &str = "a3k";
const APPLICATION: &str = "tayf";
const CONFIG_FILE_NAME: &str = "configurations.json";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Theme {
    System,
    Light,
    Dark,
}

impl Default for Theme {
    fn default() -> Self {
        Self::System
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct HexColor(String);

impl HexColor {
    pub fn new(hex: impl Into<String>) -> Self {
        let s = hex.into();
        Self(Self::normalize(&s))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn is_valid(hex: &str) -> bool {
        let clean = hex.trim().trim_start_matches('#');
        (clean.len() == 3 || clean.len() == 6 || clean.len() == 8)
            && clean.chars().all(|c| c.is_ascii_hexdigit())
    }

    pub fn normalize(hex: &str) -> String {
        let clean = hex.trim();
        if clean.starts_with('#') {
            clean.to_uppercase()
        } else {
            format!("#{}", clean.to_uppercase())
        }
    }
}

impl Default for HexColor {
    fn default() -> Self {
        Self("#FFFFFF".to_string())
    }
}

impl std::fmt::Display for HexColor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<&str> for HexColor {
    fn from(s: &str) -> Self {
        HexColor::new(s)
    }
}

impl From<String> for HexColor {
    fn from(s: String) -> Self {
        HexColor::new(s)
    }
}

impl std::ops::Deref for HexColor {
    type Target = str;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AppConfig {
    #[serde(default)]
    pub theme: Theme,

    #[serde(default = "default_lang")]
    pub lang: String,

    #[serde(default = "default_canvas_bg_light")]
    pub canvas_background_color: HexColor,

    #[serde(default = "default_canvas_bg_dark")]
    pub canvas_background_color_dark: HexColor,

    #[serde(default = "default_window_width")]
    pub window_width: u32,

    #[serde(default = "default_window_height")]
    pub window_height: u32,
}

fn default_lang() -> String {
    "en".to_string()
}

fn default_canvas_bg_light() -> HexColor {
    HexColor::new("#FFFFFF")
}

fn default_canvas_bg_dark() -> HexColor {
    HexColor::new("#0F1115")
}

fn default_window_width() -> u32 {
    960
}

fn default_window_height() -> u32 {
    680
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            theme: Theme::default(),
            lang: default_lang(),
            canvas_background_color: default_canvas_bg_light(),
            canvas_background_color_dark: default_canvas_bg_dark(),
            window_width: default_window_width(),
            window_height: default_window_height(),
        }
    }
}

impl AppConfig {
    pub fn config_path() -> Option<PathBuf> {
        ProjectDirs::from(QUALIFIER, ORGANIZATION, APPLICATION)
            .map(|dirs| dirs.config_dir().join(CONFIG_FILE_NAME))
    }

    pub fn load() -> Self {
        Self::load_from_path(Self::config_path())
    }

    pub fn load_from_path(path: Option<PathBuf>) -> Self {
        if let Some(path) = path {
            if path.exists() {
                if let Ok(content) = fs::read_to_string(&path) {
                    if let Ok(config) = serde_json::from_str::<Self>(&content) {
                        return config;
                    }
                }
            }
        }
        Self::default()
    }

    pub fn save(&self) -> Result<()> {
        if let Some(path) = Self::config_path() {
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)?;
            }
            let json = serde_json::to_string_pretty(self)?;
            fs::write(path, json)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = AppConfig::default();
        assert_eq!(config.theme, Theme::System);
        assert_eq!(config.lang, "en");
        assert_eq!(config.canvas_background_color.as_str(), "#FFFFFF");
        assert_eq!(config.canvas_background_color_dark.as_str(), "#0F1115");
        assert_eq!(config.window_width, 960);
        assert_eq!(config.window_height, 680);
    }

    #[test]
    fn test_serde_config_roundtrip() {
        let original = AppConfig {
            theme: Theme::Dark,
            lang: "ar".to_string(),
            canvas_background_color: HexColor::new("#123456"),
            canvas_background_color_dark: HexColor::new("#654321"),
            window_width: 1200,
            window_height: 800,
        };

        let json = serde_json::to_string(&original).unwrap();
        let loaded: AppConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(original, loaded);
    }

    #[test]
    fn test_hex_normalize_edge_cases() {
        // Bare hex gains a leading `#` and uppercases (issue #29).
        assert_eq!(HexColor::normalize("ff0000"), "#FF0000");
        assert_eq!(HexColor::normalize("#ff0000"), "#FF0000");
        assert_eq!(HexColor::normalize("34C3EB"), "#34C3EB");
        assert_eq!(HexColor::normalize("#34c3eb"), "#34C3EB");
        assert_eq!(HexColor::normalize("  #abc  "), "#ABC");
        assert_eq!(HexColor::new("ff0000").as_str(), "#FF0000");
        assert_eq!(HexColor::new("#ff0000").as_str(), "#FF0000");
    }

    #[test]
    fn test_hex_is_valid_edge_cases() {
        // Valid: 3/6/8 hex digits with or without `#`.
        for valid in [
            "#FFF", "FFF", "#fff", "#FF0000", "ff0000", "#34C3EB", "34c3eb", "#FF0000FF",
            "ff0000ff",
        ] {
            assert!(HexColor::is_valid(valid), "expected valid: {valid}");
        }
        // Invalid: wrong length, non-hex, empty.
        for invalid in ["#xyz", "#12345", "", "   ", "#", "ff000", "#GGGGGG", "red", "#1234567"] {
            assert!(!HexColor::is_valid(invalid), "expected invalid: {invalid}");
        }
    }
}
