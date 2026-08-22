use anyhow::{Context, Result, anyhow};
use base64::Engine;
use serde::Deserialize;
use std::collections::HashMap;
use std::io::{Cursor, Read};
use zip::ZipArchive;

#[derive(Debug, Deserialize)]
pub struct DotLottieManifest {
    pub version: Option<String>,
    pub author: Option<String>,
    pub generator: Option<String>,
    #[serde(default)]
    pub animations: Vec<DotLottieAnimationItem>,
    #[serde(default, alias = "activeAnimationId", alias = "active_animation_id")]
    pub active_animation_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct DotLottieAnimationItem {
    pub id: String,
    pub speed: Option<f32>,
    #[serde(alias = "themeColor", alias = "theme_color")]
    pub theme_color: Option<String>,
    #[serde(alias = "loopCount", alias = "loop_count")]
    pub loop_count: Option<u32>,
}

pub struct ExtractedDotLottie {
    pub animation_json: Vec<u8>,
    pub assets: HashMap<String, Vec<u8>>,
}

/// Helper to get MIME type from file extension
fn mime_type_from_filename(filename: &str) -> &'static str {
    let lower = filename.to_lowercase();
    if lower.ends_with(".png") {
        "image/png"
    } else if lower.ends_with(".jpg") || lower.ends_with(".jpeg") {
        "image/jpeg"
    } else if lower.ends_with(".webp") {
        "image/webp"
    } else if lower.ends_with(".gif") {
        "image/gif"
    } else if lower.ends_with(".svg") {
        "image/svg+xml"
    } else if lower.ends_with(".ttf") {
        "font/ttf"
    } else if lower.ends_with(".otf") {
        "font/otf"
    } else if lower.ends_with(".woff") {
        "font/woff"
    } else if lower.ends_with(".woff2") {
        "font/woff2"
    } else {
        "application/octet-stream"
    }
}

/// Inline external image assets into the Lottie JSON using base64 data URIs.
/// This allows ThorVG to load and render all image assets directly from memory.
fn inline_assets_into_json(
    raw_json_bytes: &[u8],
    assets: &HashMap<String, Vec<u8>>,
) -> Vec<u8> {
    if assets.is_empty() {
        return raw_json_bytes.to_vec();
    }

    let Ok(mut val) = serde_json::from_slice::<serde_json::Value>(raw_json_bytes) else {
        return raw_json_bytes.to_vec();
    };

    if let Some(assets_arr) = val.get_mut("assets").and_then(|a| a.as_array_mut()) {
        for asset in assets_arr.iter_mut() {
            let Some(p_val) = asset.get("p").and_then(|p| p.as_str()) else {
                continue;
            };

            // Already a data URI
            if p_val.starts_with("data:") {
                continue;
            }

            let u_val = asset.get("u").and_then(|u| u.as_str()).unwrap_or("");
            let full_p = format!("{}{}", u_val, p_val);

            // Look up asset in extracted archive using various possible paths
            let found_asset = assets.get(p_val)
                .or_else(|| assets.get(&full_p))
                .or_else(|| assets.get(&format!("images/{}", p_val)))
                .or_else(|| assets.get(&format!("i/{}", p_val)))
                .or_else(|| {
                    // Fallback to matching by file name alone
                    let file_name = std::path::Path::new(p_val)
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or(p_val);
                    assets.iter().find_map(|(k, v)| {
                        if k.ends_with(file_name) {
                            Some(v)
                        } else {
                            None
                        }
                    })
                });

            if let Some(bytes) = found_asset {
                let mime = mime_type_from_filename(p_val);
                let encoded = base64::engine::general_purpose::STANDARD.encode(bytes);
                let data_uri = format!("data:{};base64,{}", mime, encoded);

                if let Some(obj) = asset.as_object_mut() {
                    obj.insert("p".to_string(), serde_json::Value::String(data_uri));
                    obj.insert("u".to_string(), serde_json::Value::String(String::new()));
                    obj.insert("e".to_string(), serde_json::Value::Number(1.into()));
                }
            }
        }
    }

    serde_json::to_vec(&val).unwrap_or_else(|_| raw_json_bytes.to_vec())
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
        } else if name.starts_with("a/") && name.ends_with(".json") {
            // dotLottie v2 animation path
            let anim_id = name
                .trim_start_matches("a/")
                .trim_end_matches(".json")
                .to_string();
            animation_files.insert(anim_id, buf);
        } else if name.ends_with(".json") && !name.contains('/') {
            animation_files.insert(name.trim_end_matches(".json").to_string(), buf);
        } else {
            assets.insert(name, buf);
        }
    }

    let chosen_json = if let Some(m) = &manifest {
        if let Some(active_id) = &m.active_animation_id {
            let cleaned = active_id
                .trim_start_matches("animations/")
                .trim_start_matches("a/")
                .trim_end_matches(".json");
            animation_files.remove(cleaned).or_else(|| animation_files.remove(active_id))
        } else if let Some(first) = m.animations.first() {
            let cleaned = first.id
                .trim_start_matches("animations/")
                .trim_start_matches("a/")
                .trim_end_matches(".json");
            animation_files.remove(cleaned).or_else(|| animation_files.remove(&first.id))
        } else {
            None
        }
    } else {
        None
    };

    let chosen_json = chosen_json.or_else(|| animation_files.into_values().next());
    let raw_animation_json = chosen_json.ok_or_else(|| anyhow!("No animation JSON found in .lottie archive"))?;

    // Inline assets into JSON
    let animation_json = inline_assets_into_json(&raw_animation_json, &assets);

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
    fn test_extract_dotlottie_v1() {
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

    #[test]
    fn test_extract_dotlottie_v2_with_inlined_assets() {
        let mut buffer = Vec::new();
        {
            let mut zip = ZipWriter::new(Cursor::new(&mut buffer));
            let options = FileOptions::<()>::default();

            zip.start_file("manifest.json", options).unwrap();
            let manifest_content = r#"{"version":"2","generator":"dotlottie","animations":[{"id":"icon"}]}"#;
            zip.write_all(manifest_content.as_bytes()).unwrap();

            zip.start_file("a/icon.json", options).unwrap();
            let anim_content = r#"{"v":"5.5.2","fr":30,"w":100,"h":100,"assets":[{"id":"img_0","p":"img_0.png","u":"images/","e":0}]}"#;
            zip.write_all(anim_content.as_bytes()).unwrap();

            zip.start_file("images/img_0.png", options).unwrap();
            zip.write_all(b"fake_png_data").unwrap();

            zip.finish().unwrap();
        }

        let extracted = extract_dotlottie(&buffer).expect("extract dotlottie v2");
        let json_str = String::from_utf8(extracted.animation_json).unwrap();
        assert!(json_str.contains("data:image/png;base64,"));
    }
}
