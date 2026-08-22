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

pub fn export_animation(
    anim: &mut LoadedAnimation,
    output_path: &Path,
    format: ExportFormat,
    options: &ExportOptions,
    on_progress: impl FnMut(usize, usize),
) -> Result<()> {
    match format {
        ExportFormat::Gif => gif::export_gif(
            anim,
            output_path,
            options.width,
            options.fps,
            options.loop_gif,
            options.transparent,
            on_progress,
        ),
        ExportFormat::Mp4 => mp4::export_mp4(
            anim,
            output_path,
            options.width,
            options.fps,
            options.quality,
            options.transparent,
            on_progress,
        ),
    }
}
