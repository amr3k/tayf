use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use tempfile::TempDir;

const MAX_FRAMES: usize = 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ExportFormat {
    Gif,
    Mp4,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ExportOptions {
    pub width: u32,
    pub fps: u32,
    #[allow(dead_code)]
    pub loop_gif: bool,
    pub quality: u8,
}

impl Default for ExportOptions {
    fn default() -> Self {
        Self {
            width: 800,
            fps: 60,
            loop_gif: true,
            quality: 80,
        }
    }
}

pub struct ExportedFrame {
    pub width: usize,
    pub height: usize,
    pub pixels: Vec<u8>,
}

#[allow(dead_code)]
pub trait LottieRenderer {
    fn get_info(&self) -> LottieInfo;
    fn render_frame(&self, frame: usize) -> Result<ExportedFrame, String>;
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LottieInfo {
    pub width: u32,
    pub height: u32,
    pub frame_rate: f64,
    pub total_frames: f64,
}

pub fn render_to_format(
    _file_path: &str,
    format: ExportFormat,
    options: ExportOptions,
    mut render_frame_fn: impl FnMut(usize) -> Result<ExportedFrame, String>,
    get_info_fn: impl FnOnce() -> LottieInfo,
) -> Result<(Vec<u8>, String), String> {
    let info = get_info_fn();
    let aspect_ratio = info.width as f64 / info.height as f64;
    let output_height = (options.width as f64 / aspect_ratio) as u32;

    let temp_dir = TempDir::with_prefix("animaview-export-").map_err(|e| e.to_string())?;
    let temp_dir_path = temp_dir.path();

    let output_filename = match format {
        ExportFormat::Gif => "animation.gif",
        ExportFormat::Mp4 => "animation.mp4",
    };

    let output_path = temp_dir_path.join(output_filename);
    let input_pipe_path = temp_dir_path.join("frames.raw");

    match format {
        ExportFormat::Gif => {
            export_gif(
                &output_path,
                &input_pipe_path,
                options.width,
                output_height,
                options.fps,
                options.loop_gif,
                &mut render_frame_fn,
                &info,
            )?;
        }
        ExportFormat::Mp4 => {
            export_mp4(
                &output_path,
                &input_pipe_path,
                options.width,
                output_height,
                options.fps,
                options.quality,
                &mut render_frame_fn,
                &info,
            )?;
        }
    }

    let mut output_file =
        File::open(&output_path).map_err(|e| format!("Failed to open output file: {}", e))?;

    let mut output_data = Vec::new();
    output_file
        .read_to_end(&mut output_data)
        .map_err(|e| format!("Failed to read output file: {}", e))?;

    let extension = match format {
        ExportFormat::Gif => "gif",
        ExportFormat::Mp4 => "mp4",
    };

    Ok((output_data, extension.to_string()))
}

fn export_gif(
    output_path: &Path,
    _input_pipe_path: &Path,
    width: u32,
    height: u32,
    fps: u32,
    _loop_gif: bool,
    render_frame_fn: &mut impl FnMut(usize) -> Result<ExportedFrame, String>,
    info: &LottieInfo,
) -> Result<(), String> {
    use ffmpeg_sidecar::command::FfmpegCommand;

    let mut cmd = FfmpegCommand::new();

    cmd.hide_banner()
        .overwrite()
        .input("-")
        .format("rawvideo")
        .pix_fmt("rgba")
        .size(width, height)
        .rate(fps as f32)
        .output(output_path.to_str().unwrap())
        .format("gif");

    let frame_count = (info.total_frames as usize).min(MAX_FRAMES);

    let mut child = cmd
        .spawn()
        .map_err(|e| format!("Failed to spawn FFmpeg process: {}", e))?;

    let mut stdin = child
        .take_stdin()
        .ok_or_else(|| String::from("Failed to get stdin from FFmpeg"))?;

    for frame_idx in 0..frame_count {
        let frame = (render_frame_fn)(frame_idx)
            .map_err(|e| format!("Failed to render frame {}: {}", frame_idx, e))?;

        let resized = resize_frame(
            &frame.pixels,
            frame.width,
            frame.height,
            width as usize,
            height as usize,
        );

        stdin
            .write_all(&resized)
            .map_err(|e| format!("Failed to write frame {} to FFmpeg: {}", frame_idx, e))?;
    }

    stdin
        .flush()
        .map_err(|e| format!("Failed to flush stdin: {}", e))?;
    drop(stdin);

    let status = child
        .wait()
        .map_err(|e| format!("FFmpeg process failed: {}", e))?;

    if !status.success() {
        return Err(format!(
            "FFmpeg exited with error code: {:?}",
            status.code()
        ));
    }

    Ok(())
}

fn export_mp4(
    output_path: &Path,
    _input_pipe_path: &Path,
    width: u32,
    height: u32,
    fps: u32,
    quality: u8,
    render_frame_fn: &mut impl FnMut(usize) -> Result<ExportedFrame, String>,
    info: &LottieInfo,
) -> Result<(), String> {
    use ffmpeg_sidecar::command::FfmpegCommand;

    let crf = match quality {
        0..=30 => 18,
        31..=60 => 23,
        61..=80 => 28,
        _ => 23,
    };

    let mut cmd = FfmpegCommand::new();

    cmd.hide_banner()
        .overwrite()
        .input("-")
        .format("rawvideo")
        .pix_fmt("rgba")
        .size(width, height)
        .rate(fps as f32)
        .output(output_path.to_str().unwrap())
        .format("mp4")
        .codec_video("libx264")
        .preset("fast")
        .crf(crf)
        .pix_fmt("yuv420p");

    let frame_count = (info.total_frames as usize).min(MAX_FRAMES);

    let mut child = cmd
        .spawn()
        .map_err(|e| format!("Failed to spawn FFmpeg process: {}", e))?;

    let mut stdin = child
        .take_stdin()
        .ok_or_else(|| String::from("Failed to get stdin from FFmpeg"))?;

    for frame_idx in 0..frame_count {
        let frame = (render_frame_fn)(frame_idx)
            .map_err(|e| format!("Failed to render frame {}: {}", frame_idx, e))?;

        let resized = resize_frame(
            &frame.pixels,
            frame.width,
            frame.height,
            width as usize,
            height as usize,
        );

        stdin
            .write_all(&resized)
            .map_err(|e| format!("Failed to write frame {} to FFmpeg: {}", frame_idx, e))?;
    }

    stdin
        .flush()
        .map_err(|e| format!("Failed to flush stdin: {}", e))?;
    drop(stdin);

    let status = child
        .wait()
        .map_err(|e| format!("FFmpeg process failed: {}", e))?;

    if !status.success() {
        return Err(format!(
            "FFmpeg exited with error code: {:?}",
            status.code()
        ));
    }

    Ok(())
}

fn resize_frame(
    pixels: &[u8],
    src_width: usize,
    src_height: usize,
    dst_width: usize,
    dst_height: usize,
) -> Vec<u8> {
    if src_width == dst_width && src_height == dst_height {
        return pixels.to_vec();
    }

    let mut output = vec![0u8; dst_width * dst_height * 4];

    let x_ratio = (src_width as f64) / (dst_width as f64);
    let y_ratio = (src_height as f64) / (dst_height as f64);

    for y in 0..dst_height {
        for x in 0..dst_width {
            let src_x = (x as f64 * x_ratio) as usize;
            let src_y = (y as f64 * y_ratio) as usize;

            let src_idx = (src_y * src_width + src_x) * 4;
            let dst_idx = (y * dst_width + x) * 4;

            if src_idx + 3 < pixels.len() {
                output[dst_idx] = pixels[src_idx];
                output[dst_idx + 1] = pixels[src_idx + 1];
                output[dst_idx + 2] = pixels[src_idx + 2];
                output[dst_idx + 3] = pixels[src_idx + 3];
            }
        }
    }

    output
}
