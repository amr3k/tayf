use serde_json::Value;
use std::collections::{HashMap, HashSet};

/// A preset palette: a named set of replacement colors (hex `#RRGGBB`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PresetPalette {
    pub id: &'static str,
    pub name: &'static str,
    pub colors: Vec<&'static str>,
}

/// Built-in preset palettes (LottieFiles-style).
pub fn preset_palettes() -> Vec<PresetPalette> {
    vec![
        PresetPalette {
            id: "sunset",
            name: "Sunset",
            colors: vec!["#FF6B6B", "#FFA07A", "#FFD93D", "#6BCB77", "#4D96FF"],
        },
        PresetPalette {
            id: "ocean",
            name: "Ocean",
            colors: vec!["#03045E", "#0077B6", "#00B4D8", "#90E0EF", "#CAF0F8"],
        },
        PresetPalette {
            id: "forest",
            name: "Forest",
            colors: vec!["#1B4332", "#2D6A4F", "#40916C", "#95D5B2", "#D8F3DC"],
        },
        PresetPalette {
            id: "candy",
            name: "Candy",
            colors: vec!["#F72585", "#B5179E", "#7209B7", "#3A0CA3", "#4CC9F0"],
        },
        PresetPalette {
            id: "mono",
            name: "Mono",
            colors: vec!["#111111", "#444444", "#888888", "#BBBBBB", "#EEEEEE"],
        },
    ]
}

/// Build an original->replacement map by index: `original[i] -> preset[i % preset.len()]`.
pub fn build_map_from_preset(original: &[String], preset: &[String]) -> HashMap<String, String> {
    let mut map = HashMap::new();
    if preset.is_empty() {
        return map;
    }
    for (i, orig) in original.iter().enumerate() {
        let replacement = &preset[i % preset.len()];
        if replacement.to_uppercase() != orig.to_uppercase() {
            map.insert(orig.clone(), replacement.clone());
        }
    }
    map
}

/// Convert 0-1 float RGB to `#RRGGBB` (uppercase).
pub fn rgb_to_hex(r: f32, g: f32, b: f32) -> String {
    let to_u8 = |v: f32| (v.clamp(0.0, 1.0) * 255.0).round() as u8;
    format!(
        "#{:02X}{:02X}{:02X}",
        to_u8(r),
        to_u8(g),
        to_u8(b)
    )
}

/// Parse `#RRGGBB` / `#RGB` / `#RRGGBBAA` (with or without `#`) to 0-1 floats.
/// Alpha suffix is ignored (transparency is preserved per-occurrence on apply).
pub fn hex_to_rgb_floats(hex: &str) -> Option<(f32, f32, f32)> {
    let clean = hex.trim().trim_start_matches('#');
    let (r, g, b) = match clean.len() {
        3 => {
            let r = u8::from_str_radix(&clean[0..1], 16).ok()? * 17;
            let g = u8::from_str_radix(&clean[1..2], 16).ok()? * 17;
            let b = u8::from_str_radix(&clean[2..3], 16).ok()? * 17;
            (r, g, b)
        }
        6 | 8 => {
            let r = u8::from_str_radix(&clean[0..2], 16).ok()?;
            let g = u8::from_str_radix(&clean[2..4], 16).ok()?;
            let b = u8::from_str_radix(&clean[4..6], 16).ok()?;
            (r, g, b)
        }
        _ => return None,
    };
    Some((r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0))
}

fn push_unique(out: &mut Vec<String>, seen: &mut HashSet<String>, hex: String) {
    let key = hex.to_uppercase();
    if seen.insert(key.clone()) {
        out.push(key);
    }
}

fn color_array_to_hex(arr: &[Value]) -> Option<String> {
    if arr.len() < 3 {
        return None;
    }
    let r = arr[0].as_f64()? as f32;
    let g = arr[1].as_f64()? as f32;
    let b = arr[2].as_f64()? as f32;
    if !r.is_finite() || !g.is_finite() || !b.is_finite() {
        return None;
    }
    Some(rgb_to_hex(r, g, b))
}

fn set_color_array(arr: &mut [Value], r: f32, g: f32, b: f32) {
    if arr.len() > 0 {
        arr[0] = Value::from(r as f64);
    }
    if arr.len() > 1 {
        arr[1] = Value::from(g as f64);
    }
    if arr.len() > 2 {
        arr[2] = Value::from(b as f64);
    }
}

/// Extract ordered unique `#RRGGBB` colors from a Lottie JSON value.
///
/// Covers solid fills (`ty=="fl"` -> `c.k`), solid strokes (`ty=="st"` ->
/// `c.k`) and gradients (`ty=="gf"/"gs"` -> `g.k` + `g.p`), both static
/// (`"a":0,"k":[r,g,b,a]`) and animated (`"a":1` keyframes with `s`/`e`)
/// encodings. Alpha is ignored for identity; it is preserved on apply.
pub fn extract_colors(value: &Value) -> Vec<String> {
    let mut out = Vec::new();
    let mut seen = HashSet::new();
    collect_from_value(value, &mut out, &mut seen);
    out
}

fn collect_from_value(value: &Value, out: &mut Vec<String>, seen: &mut HashSet<String>) {
    match value {
        Value::Object(obj) => {
            if let Some(ty) = obj.get("ty").and_then(|t| t.as_str()) {
                match ty {
                    "fl" | "st" => {
                        if let Some(c) = obj.get("c") {
                            collect_from_color_prop(c, out, seen);
                        }
                    }
                    "gf" | "gs" => {
                        if let Some(g) = obj.get("g") {
                            collect_from_gradient(g, out, seen);
                        }
                    }
                    _ => {}
                }
            }
            for child in obj.values() {
                collect_from_value(child, out, seen);
            }
        }
        Value::Array(arr) => {
            for child in arr {
                collect_from_value(child, out, seen);
            }
        }
        _ => {}
    }
}

fn collect_from_color_prop(prop: &Value, out: &mut Vec<String>, seen: &mut HashSet<String>) {
    let Some(k) = prop.get("k") else {
        return;
    };
    collect_from_color_k(k, out, seen);
}

fn collect_from_color_k(k: &Value, out: &mut Vec<String>, seen: &mut HashSet<String>) {
    let Some(arr) = k.as_array() else {
        return;
    };
    if arr.is_empty() {
        return;
    }
    if arr[0].is_number() {
        // Static: [r,g,b,a?]
        if let Some(hex) = color_array_to_hex(arr) {
            push_unique(out, seen, hex);
        }
    } else if arr[0].is_object() {
        // Animated keyframes: [{s:[...], e:[...]?}, ...]
        for kf in arr {
            let Some(kf_obj) = kf.as_object() else {
                continue;
            };
            if let Some(s) = kf_obj.get("s").and_then(|s| s.as_array()) {
                if let Some(hex) = color_array_to_hex(s) {
                    push_unique(out, seen, hex);
                }
            }
            if let Some(e) = kf_obj.get("e").and_then(|e| e.as_array()) {
                if let Some(hex) = color_array_to_hex(e) {
                    push_unique(out, seen, hex);
                }
            }
        }
    }
}

fn gradient_count(g_obj: &serde_json::Map<String, Value>, data_len: usize) -> usize {
    if let Some(p) = g_obj.get("p").and_then(|p| p.as_u64()) {
        let p = p as usize;
        if p > 0 {
            return p;
        }
    }
    // Fallback when `p` is missing: infer from data length.
    data_len / 4
}

fn collect_from_gradient_data(
    data: &Value,
    count: usize,
    out: &mut Vec<String>,
    seen: &mut HashSet<String>,
) {
    let Some(arr) = data.as_array() else {
        return;
    };
    if arr.is_empty() {
        return;
    }
    if arr[0].is_number() {
        collect_from_gradient_numbers(arr, count, out, seen);
    } else if arr[0].is_object() {
        for kf in arr {
            let Some(kf_obj) = kf.as_object() else {
                continue;
            };
            if let Some(s) = kf_obj.get("s") {
                collect_from_gradient_data(s, count, out, seen);
            }
            if let Some(e) = kf_obj.get("e") {
                // Only recurse when `e` looks like gradient data (array).
                if e.is_array() {
                    collect_from_gradient_data(e, count, out, seen);
                }
            }
        }
    }
}

fn collect_from_gradient_numbers(
    arr: &[Value],
    count: usize,
    out: &mut Vec<String>,
    seen: &mut HashSet<String>,
) {
    // Layout: first `count*4` floats are [offset,r,g,b]*count, remainder are
    // opacity stops [offset,opacity]*n which must not be treated as colors.
    // Require a fully numeric array; otherwise skip to avoid misaligned reads.
    let mut floats = Vec::with_capacity(arr.len());
    for v in arr {
        let Some(f) = v.as_f64() else {
            return;
        };
        let f = f as f32;
        if !f.is_finite() {
            // Allow non-finite to be skipped per-stop below, but keep layout.
        }
        floats.push(f);
    }
    let effective = count.min(floats.len() / 4);
    for i in 0..effective {
        let base = i * 4;
        let r = floats[base + 1];
        let g = floats[base + 2];
        let b = floats[base + 3];
        if r.is_finite() && g.is_finite() && b.is_finite() {
            push_unique(out, seen, rgb_to_hex(r, g, b));
        }
    }
}

fn collect_from_gradient(g: &Value, out: &mut Vec<String>, seen: &mut HashSet<String>) {
    let Some(g_obj) = g.as_object() else {
        return;
    };
    let Some(k_wrapper) = g_obj.get("k") else {
        return;
    };
    // `k` is usually a property object {"a":..,"k":[...]} but handle a bare
    // array defensively.
    if let Some(inner) = k_wrapper.as_object().and_then(|o| o.get("k")) {
        let count = gradient_count(g_obj, inner.as_array().map(|a| a.len()).unwrap_or(0));
        collect_from_gradient_data(inner, count, out, seen);
    } else if k_wrapper.is_array() {
        let count = gradient_count(g_obj, k_wrapper.as_array().map(|a| a.len()).unwrap_or(0));
        collect_from_gradient_data(k_wrapper, count, out, seen);
    }
}

/// Apply a palette map (`original #RRGGBB` -> `replacement #RRGGBB`) to a
/// Lottie JSON value in place. Returns the number of color occurrences
/// rewritten. RGB is replaced; original alpha, gradient offsets and opacity
/// stops are preserved.
pub fn apply_palette(value: &mut Value, map: &HashMap<String, String>) -> usize {
    if map.is_empty() {
        return 0;
    }
    // Normalize map to uppercase keys and parsed replacement RGB for speed.
    let mut normalized: HashMap<String, (f32, f32, f32)> = HashMap::with_capacity(map.len());
    for (k, v) in map {
        let key = k.to_uppercase();
        if let Some(rgb) = hex_to_rgb_floats(v) {
            normalized.insert(key, rgb);
        }
    }
    if normalized.is_empty() {
        return 0;
    }
    apply_to_value(value, &normalized, 0)
}

fn apply_to_value(
    value: &mut Value,
    map: &HashMap<String, (f32, f32, f32)>,
    mut count: usize,
) -> usize {
    match value {
        Value::Object(obj) => {
            let ty = obj
                .get("ty")
                .and_then(|t| t.as_str())
                .unwrap_or("")
                .to_string();
            match ty.as_str() {
                "fl" | "st" => {
                    if let Some(c) = obj.get_mut("c") {
                        count += apply_to_color_prop(c, map);
                    }
                }
                "gf" | "gs" => {
                    if let Some(g) = obj.get_mut("g") {
                        count += apply_to_gradient(g, map);
                    }
                }
                _ => {}
            }
            for child in obj.values_mut() {
                count = apply_to_value(child, map, count);
            }
            count
        }
        Value::Array(arr) => {
            for child in arr.iter_mut() {
                count = apply_to_value(child, map, count);
            }
            count
        }
        _ => count,
    }
}

fn lookup_replacement(arr: &[Value], map: &HashMap<String, (f32, f32, f32)>) -> Option<(f32, f32, f32)> {
    color_array_to_hex(arr)
        .and_then(|hex| map.get(&hex.to_uppercase()).copied())
}

fn apply_to_color_prop(prop: &mut Value, map: &HashMap<String, (f32, f32, f32)>) -> usize {
    let Some(k) = prop.get_mut("k") else {
        return 0;
    };
    apply_to_color_k(k, map)
}

fn apply_to_color_k(k: &mut Value, map: &HashMap<String, (f32, f32, f32)>) -> usize {
    let Some(arr) = k.as_array_mut() else {
        return 0;
    };
    if arr.is_empty() {
        return 0;
    }
    let is_static = arr[0].is_number();
    if is_static {
        if let Some((r, g, b)) = lookup_replacement(arr, map) {
            set_color_array(arr, r, g, b);
            return 1;
        }
        return 0;
    }
    let mut count = 0;
    for kf in arr.iter_mut() {
        let Some(kf_obj) = kf.as_object_mut() else {
            continue;
        };
        if let Some(s) = kf_obj.get_mut("s").and_then(|s| s.as_array_mut()) {
            // Borrow dance: compute replacement from immutable view first.
            let replacement = {
                let view: Vec<Value> = s.clone();
                lookup_replacement(&view, map)
            };
            if let Some((r, g, b)) = replacement {
                set_color_array(s, r, g, b);
                count += 1;
            }
        }
        if let Some(e) = kf_obj.get_mut("e").and_then(|e| e.as_array_mut()) {
            let replacement = {
                let view: Vec<Value> = e.clone();
                lookup_replacement(&view, map)
            };
            if let Some((r, g, b)) = replacement {
                // Only rewrite when `e` looks like a color (3+ numbers).
                if e.len() >= 3 && e[0].is_number() {
                    set_color_array(e, r, g, b);
                    count += 1;
                }
            }
        }
    }
    count
}

fn apply_to_gradient(g: &mut Value, map: &HashMap<String, (f32, f32, f32)>) -> usize {
    let Some(g_obj) = g.as_object_mut() else {
        return 0;
    };
    // Determine count from `p` (fall back to data length when missing).
    let count_hint = g_obj.get("p").and_then(|p| p.as_u64()).map(|p| p as usize);
    let Some(k_wrapper) = g_obj.get_mut("k") else {
        return 0;
    };
    if let Some(inner_obj) = k_wrapper.as_object_mut() {
        if inner_obj.contains_key("k") {
            let count = count_hint.unwrap_or_else(|| {
                inner_obj
                    .get("k")
                    .and_then(|k| k.as_array())
                    .map(|a| a.len() / 4)
                    .unwrap_or(0)
            });
            if count == 0 {
                return 0;
            }
            let Some(inner) = inner_obj.get_mut("k") else {
                return 0;
            };
            return apply_to_gradient_data(inner, count, map);
        }
    }
    if k_wrapper.is_array() {
        let count = count_hint.unwrap_or_else(|| {
            k_wrapper.as_array().map(|a| a.len() / 4).unwrap_or(0)
        });
        if count == 0 {
            return 0;
        }
        return apply_to_gradient_data(k_wrapper, count, map);
    }
    0
}

fn apply_to_gradient_data(
    data: &mut Value,
    count: usize,
    map: &HashMap<String, (f32, f32, f32)>,
) -> usize {
    let Some(arr) = data.as_array_mut() else {
        return 0;
    };
    if arr.is_empty() {
        return 0;
    }
    if arr[0].is_number() {
        return apply_to_gradient_numbers(arr, count, map);
    }
    let mut total = 0;
    for kf in arr.iter_mut() {
        let Some(kf_obj) = kf.as_object_mut() else {
            continue;
        };
        if let Some(s) = kf_obj.get_mut("s") {
            total += apply_to_gradient_data(s, count, map);
        }
        if let Some(e) = kf_obj.get_mut("e") {
            if e.is_array() {
                total += apply_to_gradient_data(e, count, map);
            }
        }
    }
    total
}

fn apply_to_gradient_numbers(
    arr: &mut [Value],
    count: usize,
    map: &HashMap<String, (f32, f32, f32)>,
) -> usize {
    if arr.is_empty() || !arr.iter().all(|v| v.is_number()) {
        return 0;
    }
    let mut floats: Vec<f64> = arr.iter().filter_map(|v| v.as_f64()).collect();
    if floats.len() != arr.len() || floats.is_empty() {
        return 0;
    }
    let effective = count.min(floats.len() / 4);
    let mut replaced = 0;
    for i in 0..effective {
        let base = i * 4;
        let r = floats[base + 1] as f32;
        let g = floats[base + 2] as f32;
        let b = floats[base + 3] as f32;
        if !r.is_finite() || !g.is_finite() || !b.is_finite() {
            continue;
        }
        let hex = rgb_to_hex(r, g, b).to_uppercase();
        if let Some((nr, ng, nb)) = map.get(&hex) {
            floats[base + 1] = *nr as f64;
            floats[base + 2] = *ng as f64;
            floats[base + 3] = *nb as f64;
            replaced += 1;
        }
    }
    if replaced > 0 {
        // Write back only the numeric prefix we parsed; opacity tail is
        // included in `floats` and round-trips unchanged.
        for (slot, val) in arr.iter_mut().zip(floats.iter()) {
            *slot = Value::from(*val);
        }
    }
    replaced
}

/// Extract colors directly from serialized JSON bytes. Returns empty vec on
/// parse failure.
pub fn extract_colors_from_bytes(bytes: &[u8]) -> Vec<String> {
    serde_json::from_slice::<Value>(bytes)
        .map(|v| extract_colors(&v))
        .unwrap_or_default()
}

/// Apply a palette map to serialized JSON bytes. Returns `None` when the
/// input is not valid JSON or serialization fails.
pub fn apply_palette_to_bytes(
    original: &[u8],
    map: &HashMap<String, String>,
) -> Option<Vec<u8>> {
    if map.is_empty() {
        return Some(original.to_vec());
    }
    let mut value: Value = serde_json::from_slice(original).ok()?;
    apply_palette(&mut value, map);
    serde_json::to_vec(&value).ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn map(pairs: &[(&str, &str)]) -> HashMap<String, String> {
        pairs
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect()
    }

    #[test]
    fn extracts_static_fill_and_stroke() {
        let v = json!({
            "layers": [{
                "shapes": [
                    {"ty": "fl", "c": {"a": 0, "k": [1.0, 0.0, 0.0, 1.0]}},
                    {"ty": "st", "c": {"a": 0, "k": [0.0, 1.0, 0.0, 1.0]}},
                    {"ty": "fl", "c": {"a": 0, "k": [1.0, 0.0, 0.0, 1.0]}}
                ]
            }]
        });
        assert_eq!(extract_colors(&v), vec!["#FF0000", "#00FF00"]);
    }

    #[test]
    fn extracts_animated_fill_keyframes() {
        let v = json!({
            "layers": [{
                "shapes": [{
                    "ty": "fl",
                    "c": {"a": 1, "k": [
                        {"t": 0, "s": [1.0, 0.0, 0.0, 1.0], "e": [0.0, 0.0, 1.0, 1.0]},
                        {"t": 10, "s": [0.0, 0.0, 1.0, 1.0]}
                    ]}
                }]
            }]
        });
        assert_eq!(extract_colors(&v), vec!["#FF0000", "#0000FF"]);
    }

    #[test]
    fn extracts_gradient_stops_without_opacity() {
        // 2 stops: offsets 0 and 1, red -> blue, then opacity stops.
        let v = json!({
            "layers": [{
                "shapes": [{
                    "ty": "gf",
                    "g": {"p": 2, "k": {"a": 0, "k": [0.0, 1.0, 0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 0.0, 1.0, 1.0, 1.0]}}
                }]
            }]
        });
        assert_eq!(extract_colors(&v), vec!["#FF0000", "#0000FF"]);
    }

    #[test]
    fn extracts_animated_gradient_keyframes() {
        let v = json!({
            "layers": [{
                "shapes": [{
                    "ty": "gs",
                    "g": {"p": 1, "k": {"a": 1, "k": [
                        {"t": 0, "s": [0.0, 1.0, 0.0, 0.0]},
                        {"t": 10, "s": [0.0, 0.0, 1.0, 0.0]}
                    ]}}
                }]
            }]
        });
        assert_eq!(extract_colors(&v), vec!["#FF0000", "#00FF00"]);
    }

    #[test]
    fn extracts_nested_groups_and_assets() {
        let v = json!({
            "layers": [{
                "shapes": [{
                    "ty": "gr",
                    "it": [
                        {"ty": "fl", "c": {"a": 0, "k": [1.0, 1.0, 0.0, 1.0]}},
                        {"ty": "gr", "it": [
                            {"ty": "st", "c": {"a": 0, "k": [0.0, 1.0, 1.0, 1.0]}}
                        ]}
                    ]
                }]
            }],
            "assets": [{
                "layers": [{
                    "shapes": [{"ty": "fl", "c": {"a": 0, "k": [1.0, 0.0, 1.0, 1.0]}}]
                }]
            }]
        });
        assert_eq!(
            extract_colors(&v),
            vec!["#FFFF00", "#00FFFF", "#FF00FF"]
        );
    }

    #[test]
    fn apply_rewrites_static_and_preserves_alpha() {
        let mut v = json!({
            "layers": [{
                "shapes": [{"ty": "fl", "c": {"a": 0, "k": [1.0, 0.0, 0.0, 0.5]}}]
            }]
        });
        let n = apply_palette(&mut v, &map(&[("#FF0000", "#00FF00")]));
        assert_eq!(n, 1);
        let k = &v["layers"][0]["shapes"][0]["c"]["k"];
        assert!((k[0].as_f64().unwrap() - 0.0).abs() < 1e-6);
        assert!((k[1].as_f64().unwrap() - 1.0).abs() < 1e-6);
        assert!((k[2].as_f64().unwrap() - 0.0).abs() < 1e-6);
        // Alpha preserved.
        assert!((k[3].as_f64().unwrap() - 0.5).abs() < 1e-6);
    }

    #[test]
    fn apply_rewrites_animated_keyframes() {
        let mut v = json!({
            "layers": [{
                "shapes": [{
                    "ty": "fl",
                    "c": {"a": 1, "k": [
                        {"t": 0, "s": [1.0, 0.0, 0.0, 1.0], "e": [1.0, 0.0, 0.0, 1.0]},
                        {"t": 10, "s": [0.0, 1.0, 0.0, 1.0]}
                    ]}
                }]
            }]
        });
        let n = apply_palette(&mut v, &map(&[("#FF0000", "#0000FF")]));
        assert_eq!(n, 2);
        let k0s = &v["layers"][0]["shapes"][0]["c"]["k"][0]["s"];
        assert!((k0s[2].as_f64().unwrap() - 1.0).abs() < 1e-6);
        // Unmapped green keyframe untouched.
        let k1s = &v["layers"][0]["shapes"][0]["c"]["k"][1]["s"];
        assert!((k1s[1].as_f64().unwrap() - 1.0).abs() < 1e-6);
    }

    #[test]
    fn apply_rewrites_gradient_preserving_offsets_and_opacity() {
        let mut v = json!({
            "layers": [{
                "shapes": [{
                    "ty": "gf",
                    "g": {"p": 2, "k": {"a": 0, "k": [0.0, 1.0, 0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 0.0, 1.0, 1.0, 1.0]}}
                }]
            }]
        });
        let n = apply_palette(&mut v, &map(&[("#FF0000", "#00FF00")]));
        assert_eq!(n, 1);
        let arr = v["layers"][0]["shapes"][0]["g"]["k"]["k"]
            .as_array()
            .unwrap();
        // Offsets preserved.
        assert!((arr[0].as_f64().unwrap() - 0.0).abs() < 1e-9);
        assert!((arr[4].as_f64().unwrap() - 1.0).abs() < 1e-9);
        // First stop now green.
        assert!((arr[1].as_f64().unwrap() - 0.0).abs() < 1e-6);
        assert!((arr[2].as_f64().unwrap() - 1.0).abs() < 1e-6);
        // Second stop still blue.
        assert!((arr[6].as_f64().unwrap() - 0.0).abs() < 1e-6);
        assert!((arr[7].as_f64().unwrap() - 1.0).abs() < 1e-6);
        // Opacity tail untouched (length preserved for ThorVG stop count).
        assert_eq!(arr.len(), 12);
    }

    #[test]
    fn apply_is_case_insensitive_and_ignores_unknown() {
        let mut v = json!({
            "layers": [{
                "shapes": [{"ty": "fl", "c": {"a": 0, "k": [1.0, 0.0, 0.0, 1.0]}}]
            }]
        });
        let n = apply_palette(&mut v, &map(&[("#ff0000", "#00ff00"), ("#123456", "#654321")]));
        assert_eq!(n, 1);
        let k = &v["layers"][0]["shapes"][0]["c"]["k"];
        assert!((k[1].as_f64().unwrap() - 1.0).abs() < 1e-6);
    }

    #[test]
    fn roundtrip_bytes_extract_apply() {
        let original = br#"{"v":"5.5.2","fr":30,"w":100,"h":100,"layers":[{"shapes":[{"ty":"fl","c":{"a":0,"k":[1,0,0,1]}}]}]}"#;
        let colors = extract_colors_from_bytes(original);
        assert_eq!(colors, vec!["#FF0000"]);
        let recolored = apply_palette_to_bytes(original, &map(&[("#FF0000", "#0000FF")])).unwrap();
        let colors2 = extract_colors_from_bytes(&recolored);
        assert_eq!(colors2, vec!["#0000FF"]);
    }

    #[test]
    fn preset_map_cycles_when_lengths_differ() {
        let original = vec!["#AAAAAA".to_string(), "#BBBBBB".to_string(), "#CCCCCC".to_string()];
        let preset = vec!["#111111".to_string(), "#222222".to_string()];
        let m = build_map_from_preset(&original, &preset);
        assert_eq!(m.get("#AAAAAA").unwrap(), "#111111");
        assert_eq!(m.get("#BBBBBB").unwrap(), "#222222");
        assert_eq!(m.get("#CCCCCC").unwrap(), "#111111");
    }

    #[test]
    fn hex_helpers_roundtrip() {
        assert_eq!(rgb_to_hex(1.0, 0.0, 0.0), "#FF0000");
        let (r, g, _b) = hex_to_rgb_floats("#00FF00").unwrap();
        assert!((r - 0.0).abs() < 1e-6 && (g - 1.0).abs() < 1e-6);
        assert!(hex_to_rgb_floats("#xyz").is_none());
        assert!(hex_to_rgb_floats("not-a-color").is_none());
    }
}
