use gpui::{Hsla, rgba};

#[derive(Debug, Clone, Copy)]
pub struct ThemeColors {
    pub background: Hsla,
    pub surface: Hsla,
    pub surface_hover: Hsla,
    pub surface_active: Hsla,
    pub border: Hsla,
    pub border_subtle: Hsla,
    pub text_primary: Hsla,
    pub text_secondary: Hsla,
    pub text_muted: Hsla,
    pub accent: Hsla,
    pub accent_hover: Hsla,
    pub accent_active: Hsla,
    pub accent_text: Hsla,
    pub danger: Hsla,
    pub success: Hsla,
    pub modal_backdrop: Hsla,
}

impl ThemeColors {
    pub fn dark() -> Self {
        Self {
            background: rgba(0x0f1115ff).into(),
            surface: rgba(0x181a20ff).into(),
            surface_hover: rgba(0x222630ff).into(),
            surface_active: rgba(0x2a303cff).into(),
            border: rgba(0xffffff18).into(),
            border_subtle: rgba(0xffffff0d).into(),
            text_primary: rgba(0xf8fafcff).into(),
            text_secondary: rgba(0x94a3b8ff).into(),
            text_muted: rgba(0x64748bff).into(),
            accent: rgba(0x6366f1ff).into(),
            accent_hover: rgba(0x4f46e5ff).into(),
            accent_active: rgba(0x4338caff).into(),
            accent_text: rgba(0xffffffff).into(),
            danger: rgba(0xef4444ff).into(),
            success: rgba(0x22c55eff).into(),
            modal_backdrop: rgba(0x00000088).into(),
        }
    }

    pub fn light() -> Self {
        Self {
            background: rgba(0xffffffff).into(),
            surface: rgba(0xf4f5f8ff).into(),
            surface_hover: rgba(0xe8ecf2ff).into(),
            surface_active: rgba(0xdce2ecff).into(),
            border: rgba(0x00000018).into(),
            border_subtle: rgba(0x0000000d).into(),
            text_primary: rgba(0x0f172aff).into(),
            text_secondary: rgba(0x475569ff).into(),
            text_muted: rgba(0x94a3b8ff).into(),
            accent: rgba(0x4f46e5ff).into(),
            accent_hover: rgba(0x4338caff).into(),
            accent_active: rgba(0x3730a3ff).into(),
            accent_text: rgba(0xffffffff).into(),
            danger: rgba(0xdc2626ff).into(),
            success: rgba(0x16a34aff).into(),
            modal_backdrop: rgba(0x00000066).into(),
        }
    }
}

pub fn parse_hex_color(hex: &str) -> Hsla {
    let clean = hex.trim().trim_start_matches('#');
    if clean.len() == 6 {
        if let (Ok(r), Ok(g), Ok(b)) = (
            u8::from_str_radix(&clean[0..2], 16),
            u8::from_str_radix(&clean[2..4], 16),
            u8::from_str_radix(&clean[4..6], 16),
        ) {
            return rgba(((r as u32) << 24) | ((g as u32) << 16) | ((b as u32) << 8) | 0xff).into();
        }
    } else if clean.len() == 8 {
        if let (Ok(r), Ok(g), Ok(b), Ok(a)) = (
            u8::from_str_radix(&clean[0..2], 16),
            u8::from_str_radix(&clean[2..4], 16),
            u8::from_str_radix(&clean[4..6], 16),
            u8::from_str_radix(&clean[6..8], 16),
        ) {
            return rgba(((r as u32) << 24) | ((g as u32) << 16) | ((b as u32) << 8) | (a as u32)).into();
        }
    } else if clean.len() == 3 {
        if let (Ok(r), Ok(g), Ok(b)) = (
            u8::from_str_radix(&clean[0..1], 16),
            u8::from_str_radix(&clean[1..2], 16),
            u8::from_str_radix(&clean[2..3], 16),
        ) {
            let r = r * 17;
            let g = g * 17;
            let b = b * 17;
            return rgba(((r as u32) << 24) | ((g as u32) << 16) | ((b as u32) << 8) | 0xff).into();
        }
    }
    rgba(0x0f1115ff).into()
}
