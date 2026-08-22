use anyhow::{Context, Result};
use gif::{Encoder, Frame, Repeat};
use std::fs::File;
use std::path::Path;

use crate::engine::LoadedAnimation;

pub fn export_gif(
    anim: &mut LoadedAnimation,
    output_path: &Path,
    width: u32,
    fps: u32,
    loop_gif: bool,
    transparent: bool,
    mut on_progress: impl FnMut(usize, usize),
) -> Result<()> {
    let aspect_ratio = anim.metadata.width / anim.metadata.height;
    let height = ((width as f32 / aspect_ratio).round() as u32).max(1);

    let total_frames = (anim.metadata.total_frames.round() as usize).max(1);
    let total_frames = total_frames.min(2048);

    let file = File::create(output_path).context("Failed to create GIF file")?;
    let mut encoder = Encoder::new(file, width as u16, height as u16, &[])
        .context("Failed to create GIF encoder")?;

    let repeat = if loop_gif {
        Repeat::Infinite
    } else {
        Repeat::Finite(0)
    };
    encoder.set_repeat(repeat).context("Failed to set GIF repeat")?;

    let delay_hundredths = ((100.0 / fps as f32).round() as u16).max(1);

    for frame_idx in 0..total_frames {
        let rgba_bytes = anim
            .render_frame_rgba(frame_idx as f32, width, height)
            .context("Failed to render frame for GIF")?;

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

        let mut gif_frame = Frame::from_rgba_speed(
            width as u16,
            height as u16,
            &mut frame_data,
            10,
        );
        gif_frame.delay = delay_hundredths;

        encoder.write_frame(&gif_frame).context("Failed to write GIF frame")?;
        on_progress(frame_idx + 1, total_frames);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_LOTTIE_JSON: &str = r#"{"v":"5.5.2","fr":30,"ip":0,"op":10,"w":100,"h":100,"nm":"test","ddd":0,"assets":[],"layers":[{"ddd":0,"ind":1,"ty":4,"nm":"Shape 1","sr":1,"ks":{"o":{"a":0,"k":100},"r":{"a":0,"k":0},"p":{"a":0,"k":[50,50,0]},"a":{"a":0,"k":[0,0,0]},"s":{"a":0,"k":[100,100,100]}},"ao":0,"shapes":[{"ty":"rc","d":1,"s":{"a":0,"k":[50,50]},"p":{"a":0,"k":[0,0]},"r":{"a":0,"k":0},"nm":"Rectangle","hd":false},{"ty":"fl","c":{"a":0,"k":[1,0,0,1]},"o":{"a":0,"k":100},"r":1,"bm":0,"nm":"Fill","hd":false}],"ip":0,"op":10,"st":0,"bm":0}]}"#;

    #[test]
    fn test_export_gif_successful() {
        let mut anim = LoadedAnimation::from_bytes(TEST_LOTTIE_JSON.as_bytes(), Some("test.json")).unwrap();
        let temp_dir = tempfile::tempdir().unwrap();
        let out_path = temp_dir.path().join("output.gif");

        let mut progress_count = 0;
        let res = export_gif(
            &mut anim,
            &out_path,
            100,
            30,
            true,
            true,
            |cur, total| {
                progress_count = cur;
                assert!(total >= 10);
            },
        );

        assert!(res.is_ok());
        assert!(out_path.exists());
        assert!(std::fs::metadata(&out_path).unwrap().len() > 0);
        assert_eq!(progress_count, 10);
    }
}
