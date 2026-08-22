use anyhow::Result;
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

const QUALIFIER: &str = "com";
const ORGANIZATION: &str = "Amr";
const APPLICATION: &str = "AnimaView";
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AppConfig {
    #[serde(default)]
    pub theme: Theme,

    #[serde(default = "default_lang")]
    pub lang: String,

    #[serde(default = "default_canvas_bg_light")]
    pub canvas_background_color: String,

    #[serde(default = "default_canvas_bg_dark")]
    pub canvas_background_color_dark: String,

    #[serde(default = "default_window_width")]
    pub window_width: u32,

    #[serde(default = "default_window_height")]
    pub window_height: u32,
}

fn default_lang() -> String {
    "en".to_string()
}

fn default_canvas_bg_light() -> String {
    "#FFFFFF".to_string()
}

fn default_canvas_bg_dark() -> String {
    "#0F1115".to_string()
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
        assert_eq!(config.canvas_background_color, "#FFFFFF");
        assert_eq!(config.canvas_background_color_dark, "#0F1115");
        assert_eq!(config.window_width, 960);
        assert_eq!(config.window_height, 680);
    }

    #[test]
    fn test_serde_config_roundtrip() {
        let original = AppConfig {
            theme: Theme::Dark,
            lang: "ar".to_string(),
            canvas_background_color: "#123456".to_string(),
            canvas_background_color_dark: "#654321".to_string(),
            window_width: 1200,
            window_height: 800,
        };

        let json = serde_json::to_string(&original).unwrap();
        let loaded: AppConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(original, loaded);
    }
}
