use std::path::Path;
use std::time::Instant;

use crate::config::{AppConfig, Theme};
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
        self.last_tick = Instant::now();
        Ok(())
    }

    pub fn reset(&mut self) {
        self.animation = None;
        self.current_frame = 0.0;
        self.is_playing = false;
        self.is_sidebar_open = false;
        self.status_message = None;
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

    pub fn effective_canvas_background(&self, is_dark_window: bool) -> &str {
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
        let _ = self.config.save();
    }

    pub fn update_language(&mut self, lang: &str) {
        self.config.lang = lang.to_string();
        i18n::set_app_locale(lang);
        let _ = self.config.save();
    }

    pub fn update_canvas_color(&mut self, color: String, is_dark: bool) {
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
}

