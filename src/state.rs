use std::path::Path;
use std::time::Instant;

use gpui::{Bounds, Pixels};

use crate::config::{AppConfig, HexColor, Theme};
use crate::engine::{AnimationMetadata, LoadedAnimation};
use crate::export::{ExportFormat, ExportOptions};
use crate::i18n;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActiveModal {
    None,
    Preferences,
    About,
    Export,
}

/// Which UI surface a custom canvas-color picker belongs to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorPickerSlot {
    Sidebar,
    PreferencesLight,
    PreferencesDark,
}

/// Transient UI state for the custom canvas-color picker.
#[derive(Debug, Clone, Copy, Default)]
pub struct CustomPickerState {
    /// The slot whose picker panel is currently expanded, if any.
    pub open_slot: Option<ColorPickerSlot>,
    /// Hue preserved while the picker is open so grayscale selections
    /// don't snap the hue thumb back to red.
    pub working_hue: f32,
    /// Window-space bounds of the saturation/value area, captured at paint time.
    pub sv_bounds: Option<Bounds<Pixels>>,
    /// Window-space bounds of the hue bar, captured at paint time.
    pub hue_bounds: Option<Bounds<Pixels>>,
}

#[derive(Debug, Clone)]
pub struct ExportProgressState {
    pub is_exporting: bool,
    pub current_frame: usize,
    pub total_frames: usize,
    pub progress_percent: u32,
    pub error: Option<String>,
    pub success: bool,
}

pub struct AppState {
    pub config: AppConfig,
    pub animation: Option<LoadedAnimation>,
    pub current_frame: f32,
    pub is_playing: bool,
    pub loop_playback: bool,
    pub speed: f32,
    pub is_sidebar_open: bool,
    pub active_modal: ActiveModal,
    pub export_format: ExportFormat,
    pub export_options: ExportOptions,
    pub export_progress: Option<ExportProgressState>,
    pub status_message: Option<(String, bool)>, // (message, is_error)
    pub is_theme_dropdown_open: bool,
    pub is_app_menu_open: bool,
    pub custom_picker: CustomPickerState,
    pub last_tick: Instant,
}

impl AppState {
    pub fn new() -> Self {
        let config = AppConfig::load();
        i18n::set_app_locale(&config.lang);

        Self {
            config,
            animation: None,
            current_frame: 0.0,
            is_playing: true,
            loop_playback: true,
            speed: 1.0,
            is_sidebar_open: false,
            active_modal: ActiveModal::None,
            export_format: ExportFormat::Gif,
            export_options: ExportOptions::default(),
            export_progress: None,
            status_message: None,
            is_theme_dropdown_open: false,
            is_app_menu_open: false,
            custom_picker: CustomPickerState::default(),
            last_tick: Instant::now(),
        }
    }

    pub fn load_file(&mut self, path: &Path) -> anyhow::Result<()> {
        let bytes = std::fs::read(path)?;
        let anim = LoadedAnimation::from_bytes(&bytes, path.to_str())?;
        self.animation = Some(anim);
        self.current_frame = 0.0;
        self.is_playing = true;
        self.is_sidebar_open = true;
        self.status_message = None;
        self.is_app_menu_open = false;
        self.custom_picker.open_slot = None;
        self.last_tick = Instant::now();
        Ok(())
    }

    pub fn load_bytes(&mut self, bytes: &[u8], name: Option<&str>) -> anyhow::Result<()> {
        let anim = LoadedAnimation::from_bytes(bytes, name)?;
        self.animation = Some(anim);
        self.current_frame = 0.0;
        self.is_playing = true;
        self.is_sidebar_open = true;
        self.status_message = None;
        self.is_app_menu_open = false;
        self.custom_picker.open_slot = None;
        self.last_tick = Instant::now();
        Ok(())
    }

    pub fn reset(&mut self) {
        self.animation = None;
        self.current_frame = 0.0;
        self.is_playing = false;
        self.is_sidebar_open = false;
        self.status_message = None;
        self.is_theme_dropdown_open = false;
        self.is_app_menu_open = false;
        self.custom_picker.open_slot = None;
    }

    pub fn toggle_play_pause(&mut self) {
        if self.animation.is_some() {
            self.is_playing = !self.is_playing;
            self.last_tick = Instant::now();
        }
    }

    pub fn seek(&mut self, frame: f32) {
        if let Some(anim) = &self.animation {
            let max_frame = (anim.metadata.total_frames - 1.0).max(0.0);
            self.current_frame = frame.clamp(0.0, max_frame);
        }
    }

    pub fn seek_ratio(&mut self, ratio: f32) {
        if let Some(anim) = &self.animation {
            let total = anim.metadata.total_frames;
            let target_frame = (ratio * (total - 1.0)).clamp(0.0, (total - 1.0).max(0.0));
            self.seek(target_frame);
        }
    }

    pub fn step_frame(&mut self, delta: f32) {
        if let Some(anim) = &self.animation {
            let max_frame = (anim.metadata.total_frames - 1.0).max(0.0);
            let next = (self.current_frame + delta).clamp(0.0, max_frame);
            self.current_frame = next;
            self.is_playing = false;
        }
    }

    pub fn set_speed(&mut self, speed: f32) {
        self.speed = speed.clamp(0.1, 5.0);
    }

    pub fn set_loop(&mut self, loop_playback: bool) {
        self.loop_playback = loop_playback;
    }

    pub fn metadata(&self) -> Option<&AnimationMetadata> {
        self.animation.as_ref().map(|a| &a.metadata)
    }

    pub fn tick(&mut self) {
        if !self.is_playing {
            self.last_tick = Instant::now();
            return;
        }

        let now = Instant::now();
        let dt = now.duration_since(self.last_tick).as_secs_f32();
        self.last_tick = now;

        if let Some(anim) = &self.animation {
            let total = anim.metadata.total_frames;
            if total <= 1.0 {
                return;
            }

            let delta_frames = dt * anim.metadata.fps * self.speed;
            let mut next = self.current_frame + delta_frames;

            if next >= total {
                if self.loop_playback {
                    next = next % total;
                } else {
                    next = total - 1.0;
                    self.is_playing = false;
                }
            }

            self.current_frame = next;
        }
    }

    pub fn effective_canvas_background(&self, is_dark_window: bool) -> &HexColor {
        match self.config.theme {
            Theme::Light => &self.config.canvas_background_color,
            Theme::Dark => &self.config.canvas_background_color_dark,
            Theme::System => {
                if is_dark_window {
                    &self.config.canvas_background_color_dark
                } else {
                    &self.config.canvas_background_color
                }
            }
        }
    }

    pub fn update_theme(&mut self, theme: Theme) {
        self.config.theme = theme;
        self.is_theme_dropdown_open = false;
        let _ = self.config.save();
    }

    pub fn update_language(&mut self, lang: &str) {
        self.config.lang = lang.to_string();
        i18n::set_app_locale(lang);
        let _ = self.config.save();
    }

    pub fn canvas_color_for(&self, is_dark: bool) -> &HexColor {
        if is_dark {
            &self.config.canvas_background_color_dark
        } else {
            &self.config.canvas_background_color
        }
    }

    pub fn update_canvas_color(&mut self, color: HexColor, is_dark: bool) {
        if is_dark {
            self.config.canvas_background_color_dark = color;
        } else {
            self.config.canvas_background_color = color;
        }
        let _ = self.config.save();
    }

    pub fn set_export_format(&mut self, format: ExportFormat) {
        self.export_format = format;
    }

    pub fn set_export_width(&mut self, width: u32) {
        self.export_options.width = width;
    }

    pub fn set_export_fps(&mut self, fps: u32) {
        self.export_options.fps = fps;
    }

    pub fn set_export_quality(&mut self, quality: u8) {
        self.export_options.quality = quality;
    }

    pub fn toggle_export_transparent(&mut self) {
        self.export_options.transparent = !self.export_options.transparent;
    }

    pub fn toggle_export_loop(&mut self) {
        self.export_options.loop_gif = !self.export_options.loop_gif;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_LOTTIE_JSON: &str = r#"{"v":"5.5.2","fr":30,"ip":0,"op":10,"w":100,"h":100,"nm":"test","ddd":0,"assets":[],"layers":[{"ddd":0,"ind":1,"ty":4,"nm":"Shape 1","sr":1,"ks":{"o":{"a":0,"k":100},"r":{"a":0,"k":0},"p":{"a":0,"k":[50,50,0]},"a":{"a":0,"k":[0,0,0]},"s":{"a":0,"k":[100,100,100]}},"ao":0,"shapes":[{"ty":"rc","d":1,"s":{"a":0,"k":[50,50]},"p":{"a":0,"k":[0,0]},"r":{"a":0,"k":0},"nm":"Rectangle","hd":false},{"ty":"fl","c":{"a":0,"k":[1,0,0,1]},"o":{"a":0,"k":100},"r":1,"bm":0,"nm":"Fill","hd":false}],"ip":0,"op":10,"st":0,"bm":0}]}"#;

    #[test]
    fn test_app_state_export_options() {
        let mut state = AppState::new();
        assert_eq!(state.export_format, ExportFormat::Gif);

        state.set_export_format(ExportFormat::Mp4);
        assert_eq!(state.export_format, ExportFormat::Mp4);

        state.set_export_width(1080);
        assert_eq!(state.export_options.width, 1080);

        state.set_export_fps(30);
        assert_eq!(state.export_options.fps, 30);

        state.set_export_quality(90);
        assert_eq!(state.export_options.quality, 90);

        let prev_trans = state.export_options.transparent;
        state.toggle_export_transparent();
        assert_eq!(state.export_options.transparent, !prev_trans);

        let prev_loop = state.export_options.loop_gif;
        state.toggle_export_loop();
        assert_eq!(state.export_options.loop_gif, !prev_loop);
    }

    #[test]
    fn test_app_state_seek_ratio() {
        let mut state = AppState::new();
        state.load_bytes(TEST_LOTTIE_JSON.as_bytes(), Some("test.json")).unwrap();

        state.seek_ratio(0.5);
        assert!((state.current_frame - 4.5).abs() < 0.01);

        state.seek_ratio(1.0);
        assert!((state.current_frame - 9.0).abs() < 0.01);

        state.seek_ratio(0.0);
        assert_eq!(state.current_frame, 0.0);
    }

    #[test]
    fn test_app_state_theme_dropdown() {
        let mut state = AppState::new();
        assert!(!state.is_theme_dropdown_open);

        state.is_theme_dropdown_open = true;
        assert!(state.is_theme_dropdown_open);

        state.update_theme(Theme::Dark);
        assert_eq!(state.config.theme, Theme::Dark);
        assert!(!state.is_theme_dropdown_open);

        state.is_theme_dropdown_open = true;
        state.reset();
        assert!(!state.is_theme_dropdown_open);
    }

    #[test]
    fn test_app_state_app_menu() {
        let mut state = AppState::new();
        assert!(!state.is_app_menu_open);

        state.is_app_menu_open = true;
        assert!(state.is_app_menu_open);

        state.reset();
        assert!(!state.is_app_menu_open);
    }

    #[test]
    fn test_all_huge_icons_loadable() {
        use crate::ui::icon::{Icon, IconAssets};
        use gpui::AssetSource;

        let assets = IconAssets;
        let icons = [
            Icon::Play,
            Icon::Pause,
            Icon::Previous,
            Icon::Next,
            Icon::Repeat,
            Icon::Folder,
            Icon::Upload,
            Icon::Download,
            Icon::Settings,
            Icon::SidebarRight,
            Icon::Info,
            Icon::Cancel,
            Icon::Sun,
            Icon::Moon,
            Icon::Computer,
            Icon::ArrowDown,
            Icon::Tick,
            Icon::ArrowUpRight,
            Icon::PaintBoard,
            Icon::Film,
            Icon::Image,
            Icon::WindowMinimize,
            Icon::WindowMaximize,
            Icon::WindowRestore,
            Icon::WindowClose,
        ];

        for icon in icons {
            let path = icon.path();
            let loaded = assets.load(path).expect("Failed to load icon");
            assert!(loaded.is_some(), "Icon {:?} at path {} was not found", icon, path);
            let bytes = loaded.unwrap();
            let content = std::str::from_utf8(&bytes).expect("Icon content is not valid UTF-8");
            assert!(content.starts_with("<svg"), "Icon content must start with <svg");
            assert!(content.ends_with("</svg>"), "Icon content must end with </svg>");
        }
    }
}

