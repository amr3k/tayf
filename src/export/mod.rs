pub mod dotlottie;
pub mod gif;
pub mod mp4;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;

use crate::engine::LoadedAnimation;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ExportFormat {
    Gif,
    Mp4,
    DotLottie,
}

impl ExportFormat {
    /// File extension (without a leading dot) used when saving this format.
    pub fn extension(self) -> &'static str {
        match self {
            ExportFormat::Gif => "gif",
            ExportFormat::Mp4 => "mp4",
            ExportFormat::DotLottie => "lottie",
        }
    }

    /// Locale key of the human-readable format name.
    pub fn label_key(self) -> &'static str {
        match self {
            ExportFormat::Gif => "format_gif",
            ExportFormat::Mp4 => "format_mp4",
            ExportFormat::DotLottie => "format_dotlottie",
        }
    }

    /// Locale key of the native save-dialog filter name.
    pub fn filter_key(self) -> &'static str {
        match self {
            ExportFormat::Gif => "format_gif_filter",
            ExportFormat::Mp4 => "format_mp4_filter",
            ExportFormat::DotLottie => "format_dotlottie_filter",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportOptions {
    pub width: u32,
    pub fps: u32,
    pub loop_playback: bool,
    pub quality: u8,
    pub transparent: bool,
    /// Playback speed recorded in the dotLottie manifest; raster formats ignore it.
    pub speed: f32,
}

impl Default for ExportOptions {
    fn default() -> Self {
        Self {
            width: 800,
            fps: 60,
            loop_playback: true,
            quality: 80,
            transparent: true,
            speed: 1.0,
        }
    }
}

pub fn composite_white_bg(frame_data: &mut [u8]) {
    for pixel in frame_data.chunks_exact_mut(4) {
        let a = pixel[3] as u32;
        if a < 255 {
            pixel[0] = ((pixel[0] as u32 * a + 255 * (255 - a)) / 255) as u8;
            pixel[1] = ((pixel[1] as u32 * a + 255 * (255 - a)) / 255) as u8;
            pixel[2] = ((pixel[2] as u32 * a + 255 * (255 - a)) / 255) as u8;
            pixel[3] = 255;
        }
    }
}

pub fn export_animation(
    anim: &mut LoadedAnimation,
    output_path: &Path,
    format: ExportFormat,
    options: &ExportOptions,
    on_progress: impl FnMut(usize, usize),
) -> Result<()> {
    match format {
        ExportFormat::Gif => gif::export_gif(anim, output_path, options, on_progress),
        ExportFormat::Mp4 => mp4::export_mp4(anim, output_path, options, on_progress),
        ExportFormat::DotLottie => {
            dotlottie::export_dotlottie(anim, output_path, options, on_progress)
        }
    }
}
