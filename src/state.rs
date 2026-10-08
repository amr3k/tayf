use std::path::Path;
use std::time::Instant;

use gpui::{Bounds, Pixels, Point};

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

/// Which UI surface a custom color picker belongs to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorPickerSlot {
    Sidebar,
    PreferencesLight,
    PreferencesDark,
    Palette,
}

/// Transient UI state for the custom canvas-color picker.
#[derive(Debug, Clone)]
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
    /// In-progress hex text while the user types/pastes a custom color.
    /// `None` means the field mirrors the current color; `Some` means editing.
    pub hex_draft: Option<String>,
    /// Caret offset within `hex_draft`, in utf-8 bytes (always a char boundary).
    pub hex_cursor: usize,
    /// Selection anchor within `hex_draft` (utf-8 bytes); `None` means caret only.
    pub hex_anchor: Option<usize>,
    /// True while a press-and-drag selection is in progress on the hex field.
    pub hex_selecting: bool,
    /// Bounds of the hex text element, captured at paint time for
    /// click-to-place-caret hit-testing.
    pub hex_text_bounds: Option<Bounds<Pixels>>,
    /// Caret blink phase while editing.
    pub hex_blink_visible: bool,
    pub last_hex_blink: Instant,
    /// Which canvas color the in-progress hex edit applies to.
    pub hex_target_is_dark: bool,
    /// Position of the mouse-down that last dismissed the panel via
    /// outside-click. Used to keep the toggle swatch from reopening the
    /// panel in the same click (capture fires before the swatch bubble).
    pub last_outside_pos: Option<Point<Pixels>>,
}

impl Default for CustomPickerState {
    fn default() -> Self {
        Self {
            open_slot: None,
            working_hue: 0.0,
            sv_bounds: None,
            hue_bounds: None,
            hex_draft: None,
            hex_cursor: 0,
            hex_anchor: None,
            hex_selecting: false,
            hex_text_bounds: None,
            hex_blink_visible: true,
            last_hex_blink: Instant::now(),
            hex_target_is_dark: false,
            last_outside_pos: None,
        }
    }
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
    pub palette_edit_index: Option<usize>,
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
            palette_edit_index: None,
            last_tick: Instant::now(),
        }
    }

    fn clear_transient_pickers(&mut self) {
        self.custom_picker.open_slot = None;
        self.custom_picker.hex_draft = None;
        self.custom_picker.hex_cursor = 0;
        self.custom_picker.hex_anchor = None;
        self.custom_picker.hex_selecting = false;
        self.custom_picker.last_outside_pos = None;
        self.palette_edit_index = None;
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
        self.clear_transient_pickers();
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
        self.clear_transient_pickers();
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
        self.clear_transient_pickers();
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

    pub fn effective_canvas_background(&self, is_dark_window: bool) -> Option<&HexColor> {
        match self.config.theme {
            Theme::Light => self.config.canvas_background_color.as_ref(),
            Theme::Dark => self.config.canvas_background_color_dark.as_ref(),
            Theme::System => {
                if is_dark_window {
                    self.config.canvas_background_color_dark.as_ref()
                } else {
                    self.config.canvas_background_color.as_ref()
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

    pub fn canvas_color_for(&self, is_dark: bool) -> Option<&HexColor> {
        if is_dark {
            self.config.canvas_background_color_dark.as_ref()
        } else {
            self.config.canvas_background_color.as_ref()
        }
    }

    pub fn update_canvas_color(&mut self, color: Option<HexColor>, is_dark: bool) {
        if is_dark {
            self.config.canvas_background_color_dark = color;
        } else {
            self.config.canvas_background_color = color;
        }
        let _ = self.config.save();
    }

    pub fn clear_canvas_color(&mut self, is_dark: bool) {
        self.update_canvas_color(None, is_dark);
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

    // ---- Color palette (LottieFiles-style recolor) ----

    /// Detected original colors for the loaded animation, if any.
    pub fn palette_colors(&self) -> Vec<String> {
        self.animation
            .as_ref()
            .map(|a| a.palette_colors().to_vec())
            .unwrap_or_default()
    }

    pub fn has_palette_override(&self) -> bool {
        self.animation
            .as_ref()
            .map(|a| a.has_palette_override())
            .unwrap_or(false)
    }

    /// Current (possibly overridden) hex for an original palette color.
    pub fn palette_current(&self, original: &str) -> String {
        self.animation
            .as_ref()
            .map(|a| a.palette_current(original))
            .unwrap_or_else(|| original.to_uppercase())
    }

    /// Apply a preset palette (index-mapped, cycling when lengths differ).
    /// Returns true when the preview was reloaded.
    pub fn apply_palette_preset(&mut self, preset: &[String]) -> bool {
        let colors = self.palette_colors();
        if colors.is_empty() || preset.is_empty() {
            return false;
        }
        let map = crate::engine::palette::build_map_from_preset(&colors, preset);
        match self.animation.as_mut().map(|a| a.set_palette_map(map)) {
            Some(Ok(changed)) => changed,
            _ => false,
        }
    }

    /// Set/clear a single per-color override. Returns true on reload.
    pub fn set_palette_override(&mut self, original: &str, replacement: Option<&str>) -> bool {
        match self
            .animation
            .as_mut()
            .map(|a| a.set_palette_override(original, replacement))
        {
            Some(Ok(changed)) => changed,
            _ => false,
        }
    }

    /// Clear all overrides and restore original artwork. Returns true on reload.
    pub fn reset_palette(&mut self) -> bool {
        self.palette_edit_index = None;
        if self.custom_picker.open_slot == Some(ColorPickerSlot::Palette) {
            self.custom_picker.open_slot = None;
            self.custom_picker.hex_draft = None;
        }
        match self.animation.as_mut().map(|a| a.reset_palette()) {
            Some(Ok(changed)) => changed,
            _ => false,
        }
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
        state
            .load_bytes(TEST_LOTTIE_JSON.as_bytes(), Some("test.json"))
            .unwrap();

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
    fn test_app_state_palette_preset_and_reset() {
        let mut state = AppState::new();
        state
            .load_bytes(TEST_LOTTIE_JSON.as_bytes(), Some("test.json"))
            .unwrap();
        assert_eq!(state.palette_colors(), vec!["#FF0000".to_string()]);
        assert!(!state.has_palette_override());

        assert!(state.apply_palette_preset(&["#00FF00".to_string()]));
        assert!(state.has_palette_override());
        assert_eq!(state.palette_current("#FF0000"), "#00FF00");

        assert!(state.set_palette_override("#FF0000", Some("#0000FF")));
        assert_eq!(state.palette_current("#FF0000"), "#0000FF");

        assert!(state.reset_palette());
        assert!(!state.has_palette_override());
        assert!(!state.reset_palette());
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
            assert!(
                loaded.is_some(),
                "Icon {:?} at path {} was not found",
                icon,
                path
            );
            let bytes = loaded.unwrap();
            let content = std::str::from_utf8(&bytes).expect("Icon content is not valid UTF-8");
            assert!(
                content.starts_with("<svg"),
                "Icon content must start with <svg"
            );
            assert!(
                content.ends_with("</svg>"),
                "Icon content must end with </svg>"
            );
        }
    }

    #[test]
    fn test_transparent_canvas_background() {
        let mut state = AppState::new();
        state.config.theme = Theme::System;
        state.update_canvas_color(Some(HexColor::new("#FFFFFF")), false);
        state.update_canvas_color(Some(HexColor::new("#0F1115")), true);
        assert!(state.canvas_color_for(false).is_some());
        assert!(state.effective_canvas_background(false).is_some());

        state.clear_canvas_color(false);
        assert!(state.canvas_color_for(false).is_none());
        assert!(state.effective_canvas_background(true).is_some());

        state.clear_canvas_color(true);
        assert!(state.canvas_color_for(true).is_none());

        state.update_canvas_color(Some(HexColor::new("#123456")), false);
        assert_eq!(
            state.canvas_color_for(false).unwrap().as_str(),
            "#123456"
        );
    }

    #[test]
    fn test_transparent_config_serde() {
        let mut config = AppConfig::default();
        config.canvas_background_color = None;
        let json = serde_json::to_string(&config).unwrap();
        let loaded: AppConfig = serde_json::from_str(&json).unwrap();
        assert!(loaded.canvas_background_color.is_none());

        let legacy = r##"{"theme":"system","lang":"en","canvas_background_color":"#FF0000","canvas_background_color_dark":"#000000","window_width":960,"window_height":680}"##;
        let loaded: AppConfig = serde_json::from_str(legacy).unwrap();
        assert_eq!(
            loaded.canvas_background_color.unwrap().as_str(),
            "#FF0000"
        );
    }
}
