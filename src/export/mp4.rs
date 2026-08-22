use anyhow::{Context, Result, anyhow};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use crate::engine::LoadedAnimation;

/// Locate FFmpeg binary using multi-tiered fallback discovery:
/// 1. System PATH via `which`
/// 2. Executable parent directory (bundled sidecar)
/// 3. `ffmpeg-sidecar` package download/cache directory
/// 4. Common standard OS binary paths
pub fn find_ffmpeg_binary() -> Option<PathBuf> {
    // 1. Check system PATH
    if let Ok(path) = which::which("ffmpeg") {
        return Some(path);
    }

    // 2. Check next to current executable
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(parent) = exe_path.parent() {
            let candidate = parent.join(if cfg!(windows) { "ffmpeg.exe" } else { "ffmpeg" });
            if candidate.is_file() {
                return Some(candidate);
            }
        }
    }

    // 3. Check ffmpeg-sidecar paths
    if let Ok(sidecar) = ffmpeg_sidecar::paths::sidecar_path() {
        if sidecar.is_file() {
            return Some(sidecar);
        }
    }
    let sidecar_fallback = ffmpeg_sidecar::paths::ffmpeg_path();
    if sidecar_fallback.is_file() {
        return Some(sidecar_fallback);
    }

    // 4. Common standard installation locations
    let common_paths = [
        "/usr/bin/ffmpeg",
        "/usr/local/bin/ffmpeg",
        "/opt/homebrew/bin/ffmpeg",
        "/usr/pkg/bin/ffmpeg",
        "/opt/local/bin/ffmpeg",
        "C:\\ffmpeg\\bin\\ffmpeg.exe",
        "C:\\Program Files\\ffmpeg\\bin\\ffmpeg.exe",
    ];

    for path_str in common_paths {
        let p = PathBuf::from(path_str);
        if p.is_file() {
            return Some(p);
        }
    }

    None
}

pub fn export_mp4(
    anim: &mut LoadedAnimation,
    output_path: &Path,
    width: u32,
    fps: u32,
    quality: u8,
    transparent: bool,
    mut on_progress: impl FnMut(usize, usize),
) -> Result<()> {
    // Ensure even width and height for H.264
    let mut out_width = width;
    if out_width % 2 != 0 {
        out_width += 1;
    }

    let aspect_ratio = anim.metadata.width / anim.metadata.height;
    let mut out_height = ((out_width as f32 / aspect_ratio).round() as u32).max(2);
    if out_height % 2 != 0 {
        out_height += 1;
    }

    let total_frames = (anim.metadata.total_frames.round() as usize).max(1);
    let total_frames = total_frames.min(4096);

    // Map quality (1..100) to CRF (51..18, lower is better)
    let crf = (51.0 - (quality as f32 * 33.0 / 100.0)).round() as u32;

    let ffmpeg_cmd = find_ffmpeg_binary()
        .context("FFmpeg was not found on your system. Please install ffmpeg or ensure it is available in PATH to export MP4 videos.")?;

    let output_str = output_path.to_str().ok_or_else(|| anyhow!("Invalid output path"))?;

    let mut child = Command::new(ffmpeg_cmd)
        .args([
            "-y",
            "-f", "rawvideo",
            "-vcodec", "rawvideo",
            "-s", &format!("{}x{}", out_width, out_height),
            "-pix_fmt", "rgba",
            "-r", &fps.to_string(),
            "-i", "-",
            "-c:v", "libx264",
            "-pix_fmt", "yuv420p",
            "-crf", &crf.to_string(),
            "-preset", "fast",
            "-movflags", "+faststart",
            output_str,
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .context("Failed to spawn FFmpeg process")?;

    let mut stdin = child.stdin.take().ok_or_else(|| anyhow!("Failed to open FFmpeg stdin"))?;

    for frame_idx in 0..total_frames {
        let rgba_bytes = anim
            .render_frame_rgba(frame_idx as f32, out_width, out_height)
            .context("Failed to render frame for MP4")?;

        let mut frame_data = rgba_bytes.to_vec();

        if !transparent {
            // Composite alpha over white background
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

        stdin.write_all(&frame_data).context("Failed to pipe frame to FFmpeg")?;
        on_progress(frame_idx + 1, total_frames);
    }

    drop(stdin);

    let status = child.wait().context("FFmpeg process failed to complete")?;
    if !status.success() {
        return Err(anyhow!("FFmpeg export failed with exit code: {:?}", status.code()));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_ffmpeg_binary_does_not_panic() {
        let _ = find_ffmpeg_binary();
    }
}

