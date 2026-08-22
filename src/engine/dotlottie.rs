use anyhow::{Context, Result, anyhow};
use serde::Deserialize;
use std::collections::HashMap;
use std::io::{Cursor, Read};
use zip::ZipArchive;

#[derive(Debug, Deserialize)]
pub struct DotLottieManifest {
    pub version: Option<String>,
    pub author: Option<String>,
    pub generator: Option<String>,
    pub animations: Vec<DotLottieAnimationItem>,
    #[serde(default)]
    pub active_animation_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct DotLottieAnimationItem {
    pub id: String,
    pub speed: Option<f32>,
    pub theme_color: Option<String>,
    pub loop_count: Option<u32>,
}

pub struct ExtractedDotLottie {
    pub animation_json: Vec<u8>,
    pub assets: HashMap<String, Vec<u8>>,
}

pub fn extract_dotlottie(archive_bytes: &[u8]) -> Result<ExtractedDotLottie> {
    let reader = Cursor::new(archive_bytes);
    let mut zip = ZipArchive::new(reader).context("Failed to parse .lottie ZIP archive")?;

    let mut manifest: Option<DotLottieManifest> = None;
    let mut animation_files: HashMap<String, Vec<u8>> = HashMap::new();
    let mut assets: HashMap<String, Vec<u8>> = HashMap::new();

    for i in 0..zip.len() {
        let mut file = zip.by_index(i)?;
        let name = file.name().to_string();

        if file.is_dir() {
            continue;
        }

        let mut buf = Vec::with_capacity(file.size() as usize);
        file.read_to_end(&mut buf)?;

        if name == "manifest.json" {
            if let Ok(m) = serde_json::from_slice::<DotLottieManifest>(&buf) {
                manifest = Some(m);
            }
        } else if name.starts_with("animations/") && name.ends_with(".json") {
            let anim_id = name
                .trim_start_matches("animations/")
                .trim_end_matches(".json")
                .to_string();
            animation_files.insert(anim_id, buf);
        } else if name.ends_with(".json") && !name.contains('/') {
            animation_files.insert(name.trim_end_matches(".json").to_string(), buf);
        } else {
            assets.insert(name, buf);
        }
    }

    let chosen_json = if let Some(m) = manifest {
        if let Some(active_id) = m.active_animation_id {
            animation_files.remove(&active_id)
        } else if let Some(first) = m.animations.first() {
            animation_files.remove(&first.id)
        } else {
            animation_files.into_values().next()
        }
    } else {
        animation_files.into_values().next()
    };

    let animation_json = chosen_json.ok_or_else(|| anyhow!("No animation JSON found in .lottie archive"))?;

    Ok(ExtractedDotLottie {
        animation_json,
        assets,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use zip::write::{FileOptions, ZipWriter};

    #[test]
    fn test_extract_dotlottie() {
        let mut buffer = Vec::new();
        {
            let mut zip = ZipWriter::new(Cursor::new(&mut buffer));
            let options = FileOptions::<()>::default();

            zip.start_file("manifest.json", options).unwrap();
            let manifest_content = r#"{"version":"1.0","animations":[{"id":"hero"}],"active_animation_id":"hero"}"#;
            zip.write_all(manifest_content.as_bytes()).unwrap();

            zip.start_file("animations/hero.json", options).unwrap();
            let anim_content = r#"{"v":"5.5.2","fr":30,"w":100,"h":100}"#;
            zip.write_all(anim_content.as_bytes()).unwrap();

            zip.finish().unwrap();
        }

        let extracted = extract_dotlottie(&buffer).expect("extract dotlottie");
        assert_eq!(
            String::from_utf8(extracted.animation_json).unwrap(),
            r#"{"v":"5.5.2","fr":30,"w":100,"h":100}"#
        );
    }
}
