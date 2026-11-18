use anyhow::{anyhow, Context, Result};
use base64::Engine;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::{Cursor, Read, Write};
use zip::write::{SimpleFileOptions, ZipWriter};
use zip::{CompressionMethod, ZipArchive};

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

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DotLottieExportManifest {
    pub version: String,
    pub generator: String,
    pub animations: Vec<DotLottieExportAnimation>,
    pub active_animation_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DotLottieExportAnimation {
    pub id: String,
    pub speed: f32,
    #[serde(rename = "loop")]
    pub loop_playback: bool,
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
fn inline_assets_into_json(raw_json_bytes: &[u8], assets: &HashMap<String, Vec<u8>>) -> Vec<u8> {
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
            let found_asset = assets
                .get(p_val)
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
            animation_files
                .remove(cleaned)
                .or_else(|| animation_files.remove(active_id))
        } else if let Some(first) = m.animations.first() {
            let cleaned = first
                .id
                .trim_start_matches("animations/")
                .trim_start_matches("a/")
                .trim_end_matches(".json");
            animation_files
                .remove(cleaned)
                .or_else(|| animation_files.remove(&first.id))
        } else {
            None
        }
    } else {
        None
    };

    let chosen_json = chosen_json.or_else(|| animation_files.into_values().next());
    let raw_animation_json =
        chosen_json.ok_or_else(|| anyhow!("No animation JSON found in .lottie archive"))?;

    // Inline assets into JSON
    let animation_json = inline_assets_into_json(&raw_animation_json, &assets);

    Ok(ExtractedDotLottie {
        animation_json,
        assets,
    })
}

/// Sanitizes an arbitrary string into a safe, valid dotLottie animation identifier.
/// Allowed characters in dotLottie identifiers are alphanumeric, hyphens, and underscores.
pub fn sanitize_id(id: &str) -> String {
    let sanitized: String = id
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '_' || c == '-' {
                c
            } else {
                '_'
            }
        })
        .collect();
    let trimmed = sanitized.trim_matches('_');
    if trimmed.is_empty() {
        "animation".to_string()
    } else {
        trimmed.to_string()
    }
}

/// Extracts embedded Base64 data URI image assets from the Lottie JSON's `assets` array.
/// Returns a list of (zip_archive_path, binary_bytes) and mutates the JSON to point
/// to external `images/{filename}` assets with `e: 0` as specified by dotLottie.
fn extract_image_assets(val: &mut serde_json::Value) -> Vec<(String, Vec<u8>)> {
    let mut extracted_images = Vec::new();
    let Some(assets_arr) = val.get_mut("assets").and_then(|a| a.as_array_mut()) else {
        return extracted_images;
    };

    for (idx, asset) in assets_arr.iter_mut().enumerate() {
        let Some(p_val) = asset.get("p").and_then(|p| p.as_str()).map(ToString::to_string) else {
            continue;
        };

        if !p_val.starts_with("data:") {
            continue;
        }

        // Format is data:[<mediatype>][;base64],<data>
        let Some((meta, encoded_data)) = p_val.split_once(',') else {
            continue;
        };

        if !meta.contains(";base64") {
            continue;
        }

        let mime = meta
            .trim_start_matches("data:")
            .split(';')
            .next()
            .unwrap_or("image/png");

        let ext = match mime {
            "image/png" => "png",
            "image/jpeg" | "image/jpg" => "jpg",
            "image/webp" => "webp",
            "image/gif" => "gif",
            "image/svg+xml" => "svg",
            _ => "png",
        };

        let Ok(decoded_bytes) =
            base64::engine::general_purpose::STANDARD.decode(encoded_data.trim())
        else {
            continue;
        };

        let base_id = asset
            .get("id")
            .and_then(|id| id.as_str())
            .map(sanitize_id)
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| format!("img_{}", idx));

        let filename = format!("{}.{}", base_id, ext);

        // Canonical dotLottie v1 image reference: `u: ""` and `p: "images/<file>"`.
        // Players resolve the location by concatenating `u + p`.
        let archive_path = format!("images/{}", filename);
        if let Some(obj) = asset.as_object_mut() {
            obj.insert(
                "p".to_string(),
                serde_json::Value::String(archive_path.clone()),
            );
            obj.insert("u".to_string(), serde_json::Value::String(String::new()));
            obj.insert("e".to_string(), serde_json::Value::Number(0.into()));
        }

        extracted_images.push((archive_path, decoded_bytes));
    }

    extracted_images
}

/// Packages Lottie JSON bytes and optional embedded assets into a standard dotLottie (.lottie) ZIP archive.
///
/// Uses DEFLATE compression with compression level 9 for maximum size reduction.
/// Generates standard dotLottie `manifest.json` (version 1.0.0, generator "Tayf")
/// and records `speed` and `loop_playback` in the manifest's animation entry.
pub fn pack_dotlottie(
    raw_json_bytes: &[u8],
    animation_id: &str,
    speed: f32,
    loop_playback: bool,
) -> Result<Vec<u8>> {
    let mut val: serde_json::Value =
        serde_json::from_slice(raw_json_bytes).context("Invalid JSON bytes for dotLottie packaging")?;

    let anim_id = sanitize_id(animation_id);

    // Extract any embedded base64 image assets to eliminate base64 encoding overhead
    let extracted_images = extract_image_assets(&mut val);

    // Minify JSON
    let minified_json =
        serde_json::to_vec(&val).context("Failed to serialize minified Lottie JSON")?;

    // Prepare manifest
    let manifest = DotLottieExportManifest {
        version: "1.0.0".to_string(),
        generator: "Tayf".to_string(),
        animations: vec![DotLottieExportAnimation {
            id: anim_id.clone(),
            speed,
            loop_playback,
        }],
        active_animation_id: anim_id.clone(),
    };
    let manifest_bytes =
        serde_json::to_vec(&manifest).context("Failed to serialize dotLottie manifest")?;

    // Create zip archive with Deflate level 9
    let mut buffer = Vec::new();
    {
        let mut zip = ZipWriter::new(Cursor::new(&mut buffer));
        let options = SimpleFileOptions::default()
            .compression_method(CompressionMethod::Deflated)
            .compression_level(Some(9));

        // 1. Write manifest.json
        zip.start_file("manifest.json", options)
            .context("Failed to create manifest.json in .lottie archive")?;
        zip.write_all(&manifest_bytes)?;

        // 2. Write animations/<id>.json
        let anim_entry = format!("animations/{}.json", anim_id);
        zip.start_file(&anim_entry, options)
            .context("Failed to create animation entry in .lottie archive")?;
        zip.write_all(&minified_json)?;

        // 3. Write extracted binary images
        for (img_path, img_data) in extracted_images {
            zip.start_file(&img_path, options)
                .with_context(|| format!("Failed to create asset entry {} in .lottie archive", img_path))?;
            zip.write_all(&img_data)?;
        }

        zip.finish().context("Failed to finalize .lottie ZIP archive")?;
    }

    Ok(buffer)
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
            let manifest_content =
                r#"{"version":"1.0","animations":[{"id":"hero"}],"active_animation_id":"hero"}"#;
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
            let manifest_content =
                r#"{"version":"2","generator":"dotlottie","animations":[{"id":"icon"}]}"#;
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

    #[test]
    fn test_sanitize_id() {
        assert_eq!(sanitize_id("my-animation_1"), "my-animation_1");
        assert_eq!(sanitize_id("My Animation (1).json"), "My_Animation__1__json");
        assert_eq!(sanitize_id("___"), "animation");
        assert_eq!(sanitize_id(""), "animation");
    }

    /// Reads a single named entry out of a ZIP archive.
    fn read_zip_entry<R: std::io::Read + std::io::Seek>(
        zip: &mut zip::ZipArchive<R>,
        name: &str,
    ) -> Vec<u8> {
        use std::io::Read as _;
        let mut file = zip
            .by_name(name)
            .unwrap_or_else(|e| panic!("missing zip entry {name}: {e}"));
        let mut buf = Vec::new();
        file.read_to_end(&mut buf).unwrap();
        buf
    }

    #[test]
    fn test_pack_dotlottie_basic() {
        let lottie_json = br#"{"v":"5.5.2","fr":30,"w":100,"h":100,"op":60,"ip":0,"layers":[]}"#;
        let archive_bytes = pack_dotlottie(lottie_json, "hero", 1.5, true).expect("pack dotlottie");

        assert!(archive_bytes.len() >= 4);
        assert_eq!(&archive_bytes[0..4], b"PK\x03\x04");

        // The manifest must follow the standard dotLottie 1.0 structure.
        let mut zip = zip::ZipArchive::new(std::io::Cursor::new(&archive_bytes)).unwrap();
        let manifest: serde_json::Value =
            serde_json::from_slice(&read_zip_entry(&mut zip, "manifest.json")).unwrap();
        assert_eq!(manifest["version"], "1.0.0");
        assert_eq!(manifest["generator"], "Tayf");
        assert_eq!(manifest["activeAnimationId"], "hero");
        assert_eq!(manifest["animations"][0]["id"], "hero");
        assert_eq!(manifest["animations"][0]["speed"], 1.5);
        assert_eq!(manifest["animations"][0]["loop"], true);

        let extracted = extract_dotlottie(&archive_bytes).expect("extract packed dotlottie");
        let parsed: serde_json::Value =
            serde_json::from_slice(&extracted.animation_json).expect("valid json");
        assert_eq!(parsed.get("v").and_then(|v| v.as_str()), Some("5.5.2"));
        assert_eq!(parsed.get("w").and_then(|w| w.as_u64()), Some(100));
    }

    #[test]
    fn test_pack_dotlottie_with_base64_assets() {
        let fake_png_data = b"\x89PNG\r\n\x1a\n\x00fake";
        let encoded = base64::engine::general_purpose::STANDARD.encode(fake_png_data);
        let data_uri = format!("data:image/png;base64,{}", encoded);

        let json_with_assets = serde_json::json!({
            "v": "5.5.2",
            "fr": 30,
            "w": 100,
            "h": 100,
            "assets": [
                {
                    "id": "img_logo",
                    "p": data_uri,
                    "u": "",
                    "e": 1
                }
            ],
            "layers": []
        });

        let json_bytes = serde_json::to_vec(&json_with_assets).unwrap();
        let archive_bytes =
            pack_dotlottie(&json_bytes, "logo_anim", 1.0, false).expect("pack dotlottie");

        // The packed JSON must use the canonical dotLottie v1 image reference:
        // `u: ""` + `p: "images/<file>"`, exactly as official players expect it.
        let mut zip = zip::ZipArchive::new(std::io::Cursor::new(&archive_bytes)).unwrap();
        let packed_json: serde_json::Value =
            serde_json::from_slice(&read_zip_entry(&mut zip, "animations/logo_anim.json")).unwrap();
        let asset = &packed_json["assets"][0];
        assert_eq!(asset["u"], "");
        assert_eq!(asset["p"], "images/img_logo.png");
        assert_eq!(asset["e"], 0);

        // Verify that the archive can be extracted cleanly
        let extracted = extract_dotlottie(&archive_bytes).expect("extract dotlottie");
        assert_eq!(
            extracted
                .assets
                .get("images/img_logo.png")
                .map(|v| v.as_slice()),
            Some(fake_png_data.as_slice())
        );

        // Inlined JSON should contain data URI for ThorVG
        let json_str = String::from_utf8(extracted.animation_json).unwrap();
        assert!(json_str.contains("data:image/png;base64,"));
    }

    #[test]
    fn test_pack_dotlottie_compression_ratio() {
        // Create a larger realistic JSON to verify compression
        let mut layers = Vec::new();
        for i in 0..50 {
            layers.push(serde_json::json!({
                "ddd": 0,
                "ind": i,
                "ty": 4,
                "nm": format!("Shape Layer {}", i),
                "sr": 1,
                "ks": {
                    "o": {"a": 0, "k": 100},
                    "r": {"a": 0, "k": i * 5},
                    "p": {"a": 0, "k": [100.0 + i as f64, 200.0, 0.0]},
                    "a": {"a": 0, "k": [0, 0, 0]},
                    "s": {"a": 0, "k": [100, 100, 100]}
                },
                "shapes": [
                    {
                        "ty": "rc",
                        "d": 1,
                        "s": {"a": 0, "k": [50, 50]},
                        "p": {"a": 0, "k": [0, 0]},
                        "r": {"a": 0, "k": 0},
                        "nm": "Rect"
                    }
                ]
            }));
        }

        let lottie = serde_json::json!({
            "v": "5.5.2",
            "fr": 60,
            "ip": 0,
            "op": 120,
            "w": 1920,
            "h": 1080,
            "layers": layers
        });

        // Real Lottie JSON from authoring tools is compact (minified), not
        // pretty-printed, so the ratio is measured against the compact form.
        let raw_json = serde_json::to_vec(&lottie).unwrap();
        let raw_size = raw_json.len();

        let archive = pack_dotlottie(&raw_json, "large", 1.0, true).unwrap();
        let compressed_size = archive.len();

        // DEFLATE level 9 on repetitive JSON typically reduces size by > 70%
        let reduction_ratio = (raw_size - compressed_size) as f64 / raw_size as f64;
        assert!(
            reduction_ratio > 0.70,
            "Expected >70% compression, got {:.2}% (raw: {}, compressed: {})",
            reduction_ratio * 100.0,
            raw_size,
            compressed_size
        );
    }
}
