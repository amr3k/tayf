#[derive(Debug, Clone, PartialEq)]
pub struct AnimationMetadata {
    pub file_path: String,
    pub file_name: String,
    pub file_size_bytes: usize,
    pub is_dotlottie: bool,
    pub width: f32,
    pub height: f32,
    pub fps: f32,
    pub total_frames: f32,
    pub duration_seconds: f32,
}

impl AnimationMetadata {
    pub fn formatted_size(&self) -> String {
        let kb = self.file_size_bytes as f64 / 1024.0;
        if kb > 1024.0 {
            format!("{:.1} MB", kb / 1024.0)
        } else {
            format!("{:.1} KB", kb)
        }
    }

    pub fn formatted_dimensions(&self) -> String {
        format!("{} x {}", self.width as u32, self.height as u32)
    }

    pub fn formatted_fps(&self) -> String {
        format!("{:.2}", self.fps)
    }

    pub fn formatted_duration(&self) -> String {
        format!("{:.2}s", self.duration_seconds)
    }

    pub fn file_type_str(&self) -> &'static str {
        if self.is_dotlottie {
            "LOTTIE"
        } else {
            "JSON"
        }
    }
}
