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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportOptions {
    pub width: u32,
    pub fps: u32,
    pub loop_gif: bool,
    pub quality: u8,
    pub transparent: bool,
}

impl Default for ExportOptions {
    fn default() -> Self {
        Self {
            width: 800,
            fps: 60,
            loop_gif: true,
            quality: 80,
            transparent: true,
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
    }
}
