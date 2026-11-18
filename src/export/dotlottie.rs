use anyhow::Result;
use std::path::Path;

use crate::engine::LoadedAnimation;
use crate::export::ExportOptions;

pub fn export_dotlottie(
    anim: &mut LoadedAnimation,
    output_path: &Path,
    options: &ExportOptions,
    mut on_progress: impl FnMut(usize, usize),
) -> Result<()> {
    on_progress(10, 100);

    let json_bytes = anim.recolored_json_bytes();
    let anim_id = output_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("animation");

    on_progress(30, 100);
    let archive_bytes = crate::engine::dotlottie::pack_dotlottie(
        &json_bytes,
        anim_id,
        options.speed,
        options.loop_playback,
    )?;

    on_progress(80, 100);
    std::fs::write(output_path, archive_bytes)?;

    on_progress(100, 100);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_LOTTIE_JSON: &str = r#"{"v":"5.5.2","fr":30,"ip":0,"op":60,"w":500,"h":500,"nm":"test","ddd":0,"assets":[],"layers":[{"ddd":0,"ind":1,"ty":4,"nm":"Shape 1","sr":1,"ks":{"o":{"a":0,"k":100},"r":{"a":0,"k":0},"p":{"a":0,"k":[250,250,0]},"a":{"a":0,"k":[0,0,0]},"s":{"a":0,"k":[100,100,100]}},"ao":0,"shapes":[{"ty":"rc","d":1,"s":{"a":0,"k":[200,200]},"p":{"a":0,"k":[0,0]},"r":{"a":0,"k":20},"nm":"Rectangle","hd":false},{"ty":"fl","c":{"a":0,"k":[1,0,0,1]},"o":{"a":0,"k":100},"r":1,"bm":0,"nm":"Fill","hd":false}],"ip":0,"op":60,"st":0,"bm":0}]}"#;

    #[test]
    fn test_export_dotlottie_successful() {
        let mut anim =
            LoadedAnimation::from_bytes(TEST_LOTTIE_JSON.as_bytes(), Some("test.json")).unwrap();
        let temp_dir = tempfile::tempdir().unwrap();
        let output_path = temp_dir.path().join("exported.lottie");
        let options = ExportOptions::default();

        let mut progress_calls = Vec::new();
        export_dotlottie(&mut anim, &output_path, &options, |cur, tot| {
            progress_calls.push((cur, tot));
        })
        .expect("export dotlottie");

        assert!(output_path.exists());
        assert!(!progress_calls.is_empty());
        assert_eq!(progress_calls.last(), Some(&(100, 100)));

        let exported_bytes = std::fs::read(&output_path).unwrap();
        assert!(exported_bytes.len() > 4);
        assert_eq!(&exported_bytes[0..4], b"PK\x03\x04");

        // Verify the exported file can be loaded back into LoadedAnimation
        let reloaded =
            LoadedAnimation::from_bytes(&exported_bytes, Some("exported.lottie")).unwrap();
        assert!(reloaded.metadata.is_dotlottie);
        assert_eq!(reloaded.metadata.width, 500.0);
        assert_eq!(reloaded.metadata.height, 500.0);
    }
}
