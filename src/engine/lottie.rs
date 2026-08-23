use super::dotlottie::extract_dotlottie;
use super::metadata::AnimationMetadata;
use anyhow::{Context, Result, anyhow};
use std::ffi::CString;
use std::path::Path;
use std::ptr;
use std::sync::atomic::{AtomicBool, Ordering};
use thorvg_sys as sys;

static ENGINE_INITIALIZED: AtomicBool = AtomicBool::new(false);

pub fn ensure_engine_init() -> Result<()> {
    if !ENGINE_INITIALIZED.swap(true, Ordering::SeqCst) {
        unsafe {
            let res = sys::tvg_engine_init(0);
            if res != sys::Tvg_Result::TVG_RESULT_SUCCESS {
                ENGINE_INITIALIZED.store(false, Ordering::SeqCst);
                return Err(anyhow!("Failed to initialize ThorVG engine: {:?}", res));
            }
        }
    }
    Ok(())
}

pub struct LoadedAnimation {
    anim: sys::Tvg_Animation,
    pic: sys::Tvg_Paint,
    canvas: sys::Tvg_Canvas,
    pub metadata: AnimationMetadata,
    current_canvas_size: (u32, u32),
    buffer: Vec<u32>,
}

unsafe impl Send for LoadedAnimation {}

fn is_valid_lottie_value(value: &serde_json::Value) -> bool {
    if let Some(obj) = value.as_object() {
        let has_version = obj.contains_key("v");
        let has_layers_or_assets = obj.contains_key("layers") || obj.contains_key("assets");
        let has_timing = obj.contains_key("fr") || obj.contains_key("op") || obj.contains_key("ip");
        let has_dimensions = obj.contains_key("w") || obj.contains_key("h");
        // A valid Lottie must at least have a version and one of the structural keys
        has_version && (has_layers_or_assets || has_timing || has_dimensions)
    } else {
        false
    }
}

impl LoadedAnimation {
    pub fn from_bytes(bytes: &[u8], file_path: Option<&str>) -> Result<Self> {
        ensure_engine_init()?;

        if bytes.is_empty() {
            return Err(anyhow!("Invalid or unsupported Lottie animation file"));
        }

        if bytes.len() > 100 * 1024 * 1024 {
            return Err(anyhow!("File too large"));
        }

        let file_path_str = file_path.unwrap_or("animation.json").to_string();
        let file_name = Path::new(&file_path_str)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("animation")
            .to_string();

        let is_dotlottie = file_name.ends_with(".lottie")
            || (bytes.len() >= 4 && &bytes[0..4] == b"PK\x03\x04");

        let (json_bytes, _assets) = if is_dotlottie {
            let extracted = extract_dotlottie(bytes)
                .context("Failed to extract .lottie package")?;
            // Validate extracted JSON is valid Lottie
            if let Ok(val) = serde_json::from_slice::<serde_json::Value>(&extracted.animation_json) {
                if !is_valid_lottie_value(&val) {
                    return Err(anyhow!("Invalid or unsupported Lottie animation file"));
                }
            } else {
                return Err(anyhow!("Invalid or unsupported Lottie animation file"));
            }
            (extracted.animation_json, extracted.assets)
        } else {
            // Quick JSON validation before handing to ThorVG for better error messages
            match serde_json::from_slice::<serde_json::Value>(bytes) {
                Ok(val) => {
                    if !is_valid_lottie_value(&val) {
                        return Err(anyhow!("Invalid or unsupported Lottie animation file"));
                    }
                }
                Err(_) => {
                    return Err(anyhow!("Invalid or unsupported Lottie animation file"));
                }
            }
            (bytes.to_vec(), std::collections::HashMap::new())
        };

        unsafe {
            let anim = sys::tvg_lottie_animation_new();
            if anim.is_null() {
                return Err(anyhow!("Failed to create ThorVG animation"));
            }

            let pic = sys::tvg_animation_get_picture(anim);
            if pic.is_null() {
                sys::tvg_animation_del(anim);
                return Err(anyhow!("Failed to retrieve picture from animation"));
            }

            let mime = CString::new("lottie+json").unwrap();
            let res = sys::tvg_picture_load_data(
                pic,
                json_bytes.as_ptr() as *const _,
                json_bytes.len() as u32,
                mime.as_ptr(),
                ptr::null(),
                true,
            );

            if res != sys::Tvg_Result::TVG_RESULT_SUCCESS {
                sys::tvg_animation_del(anim);
                return Err(anyhow!("Invalid or unsupported Lottie animation file"));
            }

            let mut total_frames = 0.0f32;
            sys::tvg_animation_get_total_frame(anim, &mut total_frames);

            let mut duration = 0.0f32;
            sys::tvg_animation_get_duration(anim, &mut duration);

            let mut orig_w = 0.0f32;
            let mut orig_h = 0.0f32;
            sys::tvg_picture_get_size(pic, &mut orig_w, &mut orig_h);

            let fps = if duration > 0.0 && total_frames > 0.0 {
                total_frames / duration
            } else {
                30.0
            };

            let canvas = sys::tvg_swcanvas_create(sys::Tvg_Engine_Option::TVG_ENGINE_OPTION_DEFAULT);
            if canvas.is_null() {
                sys::tvg_animation_del(anim);
                return Err(anyhow!("Failed to create ThorVG software canvas"));
            }

            let add_res = sys::tvg_canvas_add(canvas, pic);
            if add_res != sys::Tvg_Result::TVG_RESULT_SUCCESS {
                sys::tvg_canvas_destroy(canvas);
                sys::tvg_animation_del(anim);
                return Err(anyhow!("Failed to add picture to canvas: {:?}", add_res));
            }

            let metadata = AnimationMetadata {
                file_path: file_path_str,
                file_name,
                file_size_bytes: bytes.len(),
                is_dotlottie,
                width: if orig_w > 0.0 { orig_w } else { 500.0 },
                height: if orig_h > 0.0 { orig_h } else { 500.0 },
                fps,
                total_frames: if total_frames > 0.0 { total_frames } else { 1.0 },
                duration_seconds: if duration > 0.0 { duration } else { 0.0 },
            };

            Ok(Self {
                anim,
                pic,
                canvas,
                metadata,
                current_canvas_size: (0, 0),
                buffer: Vec::new(),
            })
        }
    }

    pub fn render_frame_rgba(&mut self, frame: f32, width: u32, height: u32) -> Result<&[u8]> {
        let width = width.clamp(1, 4096);
        let height = height.clamp(1, 4096);

        let clamped_frame = frame.clamp(0.0, (self.metadata.total_frames - 1.0).max(0.0));

        unsafe {
            if self.current_canvas_size != (width, height) {
                self.buffer.resize((width * height) as usize, 0);
                let res = sys::tvg_swcanvas_set_target(
                    self.canvas,
                    self.buffer.as_mut_ptr(),
                    width,
                    width,
                    height,
                    sys::Tvg_Colorspace::TVG_COLORSPACE_ABGR8888,
                );
                if res != sys::Tvg_Result::TVG_RESULT_SUCCESS {
                    return Err(anyhow!("Failed to set canvas target: {:?}", res));
                }

                sys::tvg_picture_set_size(self.pic, width as f32, height as f32);
                self.current_canvas_size = (width, height);
            }

            let res = sys::tvg_animation_set_frame(self.anim, clamped_frame);
            if res != sys::Tvg_Result::TVG_RESULT_SUCCESS {
                tracing::trace!("set_frame returned {:?}", res);
            }

            sys::tvg_canvas_update(self.canvas);
            sys::tvg_canvas_draw(self.canvas, true);
            sys::tvg_canvas_sync(self.canvas);

            let byte_slice = std::slice::from_raw_parts(
                self.buffer.as_ptr() as *const u8,
                self.buffer.len() * 4,
            );

            Ok(byte_slice)
        }
    }
}

impl Drop for LoadedAnimation {
    fn drop(&mut self) {
        unsafe {
            if !self.canvas.is_null() {
                sys::tvg_canvas_destroy(self.canvas);
                self.canvas = ptr::null_mut();
            }
            if !self.anim.is_null() {
                sys::tvg_animation_del(self.anim);
                self.anim = ptr::null_mut();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_LOTTIE_JSON: &str = r#"{"v":"5.5.2","fr":30,"ip":0,"op":60,"w":500,"h":500,"nm":"test","ddd":0,"assets":[],"layers":[{"ddd":0,"ind":1,"ty":4,"nm":"Shape 1","sr":1,"ks":{"o":{"a":0,"k":100},"r":{"a":0,"k":0},"p":{"a":0,"k":[250,250,0]},"a":{"a":0,"k":[0,0,0]},"s":{"a":0,"k":[100,100,100]}},"ao":0,"shapes":[{"ty":"rc","d":1,"s":{"a":0,"k":[200,200]},"p":{"a":0,"k":[0,0]},"r":{"a":0,"k":20},"nm":"Rectangle","hd":false},{"ty":"fl","c":{"a":0,"k":[1,0,0,1]},"o":{"a":0,"k":100},"r":1,"bm":0,"nm":"Fill","hd":false}],"ip":0,"op":60,"st":0,"bm":0}]}"#;

    #[test]
    fn test_loaded_animation_lifecycle() {
        let mut anim = LoadedAnimation::from_bytes(TEST_LOTTIE_JSON.as_bytes(), Some("test.json")).unwrap();
        assert_eq!(anim.metadata.file_name, "test.json");
        assert_eq!(anim.metadata.total_frames, 60.0);
        assert_eq!(anim.metadata.width, 500.0);
        assert_eq!(anim.metadata.height, 500.0);

        let rgba = anim.render_frame_rgba(0.0, 100, 100).unwrap();
        assert_eq!(rgba.len(), 100 * 100 * 4);
        assert!(rgba.iter().any(|&b| b != 0));

        let rgba2 = anim.render_frame_rgba(30.0, 100, 100).unwrap();
        assert_eq!(rgba2.len(), 100 * 100 * 4);
        assert!(rgba2.iter().any(|&b| b != 0));
    }

    #[test]
    fn test_loaded_dotlottie() {
        for path in &["/tmp/bull.lottie", "/tmp/exploding_pigeon.lottie", "/tmp/text.lottie"] {
            if let Ok(bytes) = std::fs::read(path) {
                let mut anim = LoadedAnimation::from_bytes(&bytes, Some(path)).unwrap();
                for f in 0..60 {
                    let rgba = anim.render_frame_rgba(f as f32, 200, 200).unwrap();
                    assert_eq!(rgba.len(), 200 * 200 * 4);
                }
            }
        }
    }

    #[test]
    fn test_render_many_frames_stable() {
        let mut anim = LoadedAnimation::from_bytes(TEST_LOTTIE_JSON.as_bytes(), Some("test.json")).unwrap();
        for f in 0..500 {
            let rgba = anim.render_frame_rgba((f % 60) as f32, 400, 400).unwrap();
            assert_eq!(rgba.len(), 400 * 400 * 4);
        }
    }

    #[test]
    fn test_rejects_non_lottie_json() {
        // Generic JSON files must be rejected as invalid Lottie
        let cases = [
            (br#"{"hello":"world"}"# as &[u8], "generic.json"),
            (br#"{"foo":123,"bar":[1,2,3]}"#, "data.json"),
            (br#"{"v":1}"#, "partial.json"),
            (b"not json at all", "broken.json"),
            (b"", "empty.json"),
        ];
        for (bytes, name) in cases {
            let res = LoadedAnimation::from_bytes(bytes, Some(name));
            assert!(
                res.is_err(),
                "Expected {} to be rejected as invalid Lottie, got Ok",
                name
            );
            let err_str = match res {
                Ok(_) => unreachable!(),
                Err(e) => e.to_string(),
            };
            assert!(
                err_str.contains("Invalid or unsupported"),
                "Expected invalid-file error for {}, got: {}",
                name,
                err_str
            );
        }
    }

    #[test]
    fn test_valid_lottie_still_loads_after_validation() {
        // Minimal valid Lottie should still pass even though it has no layers
        let minimal = br#"{"v":"5.5.2","fr":30,"w":100,"h":100,"op":10,"ip":0,"layers":[]}"#;
        assert!(LoadedAnimation::from_bytes(minimal, Some("minimal.json")).is_ok());

        // Full valid Lottie from lifecycle test should still load
        assert!(LoadedAnimation::from_bytes(TEST_LOTTIE_JSON.as_bytes(), Some("test.json")).is_ok());
    }
}
