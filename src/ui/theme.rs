use gpui::{Hsla, rgba};

pub use crate::config::HexColor;

impl HexColor {
    pub fn to_hsla(&self) -> Hsla {
        parse_hex_color(self.as_str())
    }
}

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
    /// Overrides the accent color (and its derived variants) with the given
    /// system accent color.
    pub fn with_accent(mut self, accent: Hsla, is_dark: bool) -> Self {
        self.accent = accent;
        self.accent_hover = shift_lightness(accent, if is_dark { 0.07 } else { -0.07 });
        self.accent_active = shift_lightness(accent, if is_dark { 0.14 } else { -0.14 });

        let text: Hsla = if relative_luminance(accent) > 0.55 {
            rgba(0x111827ff).into()
        } else {
            rgba(0xffffffff).into()
        };
        self.accent_text = text;
        self
    }

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

fn shift_lightness(color: Hsla, delta: f32) -> Hsla {
    Hsla {
        h: color.h,
        s: color.s,
        l: (color.l + delta).clamp(0.0, 1.0),
        a: color.a,
    }
}

fn relative_luminance(color: Hsla) -> f32 {
    let (r, g, b) = hsl_to_rgb(color);
    0.2126 * r + 0.7152 * g + 0.0722 * b
}

fn hsl_to_rgb(color: Hsla) -> (f32, f32, f32) {
    let h = color.h.fract() * 6.0;
    let s = color.s.clamp(0.0, 1.0);
    let l = color.l.clamp(0.0, 1.0);

    let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
    let x = c * (1.0 - ((h % 2.0) - 1.0).abs());
    let m = l - c / 2.0;

    let (r, g, b) = match h as u32 {
        0 => (c, x, 0.0),
        1 => (x, c, 0.0),
        2 => (0.0, c, x),
        3 => (0.0, x, c),
        4 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };
    (r + m, g + m, b + m)
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

#[cfg(test)]
mod tests {
    use super::*;
    use gpui::rgba;

    #[test]
    fn accent_variants_are_derived() {
        let base = rgba(0x6366f1ff).into();
        let dark = ThemeColors::dark().with_accent(base, true);
        let light = ThemeColors::light().with_accent(base, false);

        assert_eq!(dark.accent, base);
        assert_eq!(light.accent, base);
        // Light theme: hover/active are darker; dark theme: lighter.
        assert!(light.accent_hover.l < light.accent.l);
        assert!(light.accent_active.l < light.accent_hover.l);
        assert!(dark.accent_hover.l > dark.accent.l);
        assert!(dark.accent_active.l > dark.accent_hover.l);
        // Indigo is not bright enough for dark text.
        let white: Hsla = rgba(0xffffffff).into();
        assert_eq!(dark.accent_text, white);
    }

    #[test]
    fn bright_accent_gets_dark_text() {
        let yellow: Hsla = rgba(0xffeb3bff).into();
        let themed = ThemeColors::dark().with_accent(yellow, true);
        let dark_text: Hsla = rgba(0x111827ff).into();
        assert_eq!(themed.accent_text, dark_text);
    }

    #[test]
    fn lightness_is_clamped() {
        let black: Hsla = rgba(0x000000ff).into();
        let themed = ThemeColors::dark().with_accent(black, true);
        assert!(themed.accent_active.l <= 1.0);
        assert!(themed.accent_hover.l >= 0.0);
    }
}
