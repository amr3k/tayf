use std::process::Command;
use std::sync::OnceLock;

use gpui::{rgba, Hsla};

/// Returns the desktop environment's global accent color, if one exists.
///
/// On KDE Plasma this reads `[General] AccentColor` from
/// `~/.config/kdeglobals`. Plasma writes the resolved value there even when
/// the user selected "Accent color from wallpaper", so wallpaper-derived
/// accents are picked up automatically.
///
/// On GNOME 47+ it queries `gsettings` for the named accent color.
fn detect_system_accent() -> Option<Hsla> {
    #[cfg(target_os = "linux")]
    {
        read_kde_accent().or_else(read_gnome_accent)
    }
    #[cfg(not(target_os = "linux"))]
    {
        None
    }
}

#[cfg(target_os = "linux")]
fn read_kde_accent() -> Option<Hsla> {
    let config_dir = directories::BaseDirs::new()?
        .config_dir()
        .join("kdeglobals");
    let content = std::fs::read_to_string(config_dir).ok()?;

    let mut in_general = false;
    for line in content.lines() {
        let line = line.trim();
        if line.starts_with('[') {
            in_general = line.eq_ignore_ascii_case("[general]");
            continue;
        }
        if !in_general {
            continue;
        }
        if let Some(value) = line.strip_prefix("AccentColor=") {
            return parse_rgb_triplet(value);
        }
    }
    None
}

#[cfg(target_os = "linux")]
fn parse_rgb_triplet(value: &str) -> Option<Hsla> {
    let parts: Vec<&str> = value.split(',').map(str::trim).collect();
    if parts.len() != 3 {
        return None;
    }
    let r: i32 = parts[0].parse().ok()?;
    let g: i32 = parts[1].parse().ok()?;
    let b: i32 = parts[2].parse().ok()?;
    if !(0..=255).contains(&r) || !(0..=255).contains(&g) || !(0..=255).contains(&b) {
        return None;
    }
    Some(rgba(((r as u32) << 24) | ((g as u32) << 16) | ((b as u32) << 8) | 0xff).into())
}

#[cfg(target_os = "linux")]
const GNOME_ACCENTS: &[(&str, u32)] = &[
    ("blue", 0x3584e4),
    ("teal", 0x2190a4),
    ("green", 0x3a944a),
    ("yellow", 0xc88800),
    ("orange", 0xed5b00),
    ("red", 0xe62d42),
    ("pink", 0xd56199),
    ("purple", 0x9141ac),
    ("slate", 0x6f8396),
];

#[cfg(target_os = "linux")]
fn read_gnome_accent() -> Option<Hsla> {
    let output = Command::new("gsettings")
        .args(["get", "org.gnome.desktop.interface", "accent-color"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let name = String::from_utf8_lossy(&output.stdout);
    let name = name.trim().trim_matches('\'');
    let (&_, hex) = GNOME_ACCENTS.iter().find(|(key, _)| *key == name)?;
    Some(rgba((hex << 8) | 0xff).into())
}

/// Cached accessor for the system accent color.
pub fn system_accent() -> Option<Hsla> {
    static ACCENT: OnceLock<Option<Hsla>> = OnceLock::new();
    *ACCENT.get_or_init(detect_system_accent)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_rgb_triplet() {
        let c = parse_rgb_triplet("159, 98, 196").unwrap();
        let l = c.l;
        assert!(l > 0.0 && l < 1.0);
    }

    #[test]
    fn rejects_invalid_triplet() {
        assert!(parse_rgb_triplet("-1,-1,-1").is_none());
        assert!(parse_rgb_triplet("300,0,0").is_none());
        assert!(parse_rgb_triplet("red").is_none());
    }

    #[test]
    fn knows_gnome_names() {
        assert_eq!(GNOME_ACCENTS.iter().count(), 9);
    }
}
