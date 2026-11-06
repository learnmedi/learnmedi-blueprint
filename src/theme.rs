//! Site theme: colours, fonts, shape. Rendered as CSS custom properties so
//! every block picks it up. Every field is validated — the AI (and the editor)
//! can only set values from these closed sets or valid hex colours.

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ThemeColors {
    pub primary: String,
    pub accent: String,
    pub background: String,
    pub surface: String,
    pub text: String,
    pub muted: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Theme {
    pub colors: ThemeColors,
    pub font_heading: String,
    pub font_body: String,
    /// none | sm | md | lg | pill
    pub radius: String,
    /// simple | centered | bold
    pub header_style: String,
    /// solid | outline | soft
    pub button_style: String,
    /// 15..20 px
    pub base_font_size: i64,
    /// Default entrance animation for blocks that do not set one.
    pub default_animation: String,
}

pub const FONTS: &[&str] = &[
    "Be Vietnam Pro",
    "Inter",
    "Fraunces",
    "Playfair Display",
    "Merriweather",
    "Lora",
    "Nunito",
    "Quicksand",
    "Montserrat",
    "Roboto Slab",
    "Noto Serif",
    "Source Sans 3",
];
pub const RADII: &[&str] = &["none", "sm", "md", "lg", "pill"];
pub const HEADER_STYLES: &[&str] = &["simple", "centered", "bold"];
pub const BUTTON_STYLES: &[&str] = &["solid", "outline", "soft"];
pub const ANIMATIONS: &[&str] = &["none", "fade-up", "fade-in", "zoom-in", "slide-left", "slide-right"];
pub const COLOR_TOKENS: &[&str] = &["primary", "accent", "background", "surface", "text", "muted"];

pub struct Preset {
    pub key: &'static str,
    pub label: &'static str,
    pub theme: fn() -> Theme,
}

fn t(colors: [&str; 6], heading: &str, body: &str, radius: &str, header: &str, button: &str) -> Theme {
    Theme {
        colors: ThemeColors {
            primary: colors[0].into(),
            accent: colors[1].into(),
            background: colors[2].into(),
            surface: colors[3].into(),
            text: colors[4].into(),
            muted: colors[5].into(),
        },
        font_heading: heading.into(),
        font_body: body.into(),
        radius: radius.into(),
        header_style: header.into(),
        button_style: button.into(),
        base_font_size: 17,
        default_animation: "fade-up".into(),
    }
}

pub const PRESETS: &[Preset] = &[
    // Warm editorial look, inspired by tranhoangyen.com (Fraunces + Be Vietnam Pro).
    // navy #14223f · sage-deep #47664f · paper #fbf8f2 · beige #f4ede1 · ink #1a2236 (tranhoangyen.com tokens)
    Preset { key: "swiftlet", label: "Swiftlet — ấm áp, biên tập", theme: || t(["#14223f", "#47664f", "#fbf8f2", "#f4ede1", "#1a2236", "#5f6678"], "Fraunces", "Be Vietnam Pro", "lg", "simple", "solid") },
    Preset { key: "ocean", label: "Đại dương — xanh, hiện đại", theme: || t(["#1d4ed8", "#06b6d4", "#f8fafc", "#ffffff", "#0f172a", "#64748b"], "Montserrat", "Inter", "md", "simple", "solid") },
    Preset { key: "classroom", label: "Lớp học — tươi vui", theme: || t(["#7c3aed", "#f59e0b", "#fffdf7", "#ffffff", "#1f1b2e", "#6b6880"], "Quicksand", "Nunito", "pill", "centered", "soft") },
    Preset { key: "academic", label: "Học thuật — trang nhã", theme: || t(["#7f1d1d", "#b45309", "#fafaf9", "#ffffff", "#1c1917", "#57534e"], "Playfair Display", "Source Sans 3", "sm", "simple", "outline") },
    Preset { key: "midnight", label: "Đêm — tối, nổi bật", theme: || t(["#a78bfa", "#f472b6", "#0f0f17", "#1a1a26", "#ececf4", "#a1a1b5"], "Montserrat", "Inter", "lg", "bold", "solid") },
];

impl Default for Theme {
    fn default() -> Self {
        (PRESETS[0].theme)()
    }
}

pub fn preset(key: &str) -> Option<Theme> {
    PRESETS.iter().find(|p| p.key == key).map(|p| (p.theme)())
}

pub fn is_hex_color(s: &str) -> bool {
    let s = s.trim();
    (s.len() == 7 || s.len() == 4) && s.starts_with('#') && s[1..].chars().all(|c| c.is_ascii_hexdigit())
}

/// A colour value used in block styles: a hex colour or a theme token.
pub fn normalize_color(s: &str) -> Option<String> {
    let v = s.trim().to_ascii_lowercase();
    if v.is_empty() {
        return Some(String::new());
    }
    if is_hex_color(&v) || COLOR_TOKENS.contains(&v.as_str()) {
        return Some(v);
    }
    named_color(&v).map(|c| c.to_string())
}

/// A few Vietnamese/English colour names so "đổi sang màu xanh" still works
/// even if the model passes a word instead of hex.
fn named_color(name: &str) -> Option<&'static str> {
    Some(match name {
        "blue" | "xanh" | "xanh dương" | "xanh da trời" => "#2563eb",
        "green" | "xanh lá" | "xanh lá cây" => "#16a34a",
        "red" | "đỏ" => "#dc2626",
        "orange" | "cam" => "#ea580c",
        "yellow" | "vàng" => "#eab308",
        "purple" | "tím" => "#7c3aed",
        "pink" | "hồng" => "#db2777",
        "black" | "đen" => "#111111",
        "white" | "trắng" => "#ffffff",
        "gray" | "grey" | "xám" => "#6b7280",
        "brown" | "nâu" => "#92400e",
        _ => return None,
    })
}

/// Apply a JSON patch (subset of fields) with validation. Returns a list of
/// human-readable changes, or an error naming the bad field.
pub fn apply_patch(theme: &mut Theme, patch: &Value) -> Result<Vec<String>, String> {
    let obj = patch.as_object().ok_or("theme patch phải là object")?;
    let mut changes = Vec::new();
    if let Some(p) = obj.get("preset").and_then(|v| v.as_str()) {
        *theme = preset(p).ok_or_else(|| format!("preset không tồn tại: {p}. Có: {}", PRESETS.iter().map(|p| p.key).collect::<Vec<_>>().join(", ")))?;
        changes.push(format!("áp dụng preset {p}"));
    }
    for (key, val) in obj {
        match key.as_str() {
            "preset" => {}
            "colors" => {
                let c = val.as_object().ok_or("colors phải là object")?;
                for (ck, cv) in c {
                    let cv = cv.as_str().ok_or_else(|| format!("colors.{ck} phải là chuỗi"))?;
                    let hex = normalize_color(cv)
                        .filter(|h| is_hex_color(h))
                        .ok_or_else(|| format!("colors.{ck}: '{cv}' không phải màu hex (#rrggbb)"))?;
                    let slot = match ck.as_str() {
                        "primary" => &mut theme.colors.primary,
                        "accent" => &mut theme.colors.accent,
                        "background" => &mut theme.colors.background,
                        "surface" => &mut theme.colors.surface,
                        "text" => &mut theme.colors.text,
                        "muted" => &mut theme.colors.muted,
                        other => return Err(format!("màu không tồn tại: {other}. Có: {}", COLOR_TOKENS.join(", "))),
                    };
                    *slot = hex.clone();
                    changes.push(format!("màu {ck} = {hex}"));
                }
            }
            "font_heading" | "font_body" => {
                let f = val.as_str().ok_or_else(|| format!("{key} phải là chuỗi"))?;
                let found = FONTS.iter().find(|x| x.eq_ignore_ascii_case(f)).ok_or_else(|| format!("font không hỗ trợ: {f}. Có: {}", FONTS.join(", ")))?;
                if key == "font_heading" { theme.font_heading = found.to_string() } else { theme.font_body = found.to_string() }
                changes.push(format!("{key} = {found}"));
            }
            "radius" => set_enum(&mut theme.radius, val, RADII, key, &mut changes)?,
            "header_style" => set_enum(&mut theme.header_style, val, HEADER_STYLES, key, &mut changes)?,
            "button_style" => set_enum(&mut theme.button_style, val, BUTTON_STYLES, key, &mut changes)?,
            "default_animation" => set_enum(&mut theme.default_animation, val, ANIMATIONS, key, &mut changes)?,
            "base_font_size" => {
                let n = val.as_i64().ok_or("base_font_size phải là số")?;
                theme.base_font_size = n.clamp(14, 21);
                changes.push(format!("cỡ chữ = {}px", theme.base_font_size));
            }
            other => return Err(format!("trường theme không tồn tại: {other}")),
        }
    }
    Ok(changes)
}

fn set_enum(slot: &mut String, val: &Value, allowed: &[&str], key: &str, changes: &mut Vec<String>) -> Result<(), String> {
    let v = val.as_str().ok_or_else(|| format!("{key} phải là chuỗi"))?;
    if !allowed.contains(&v) {
        return Err(format!("{key}: '{v}' không hợp lệ. Có: {}", allowed.join(", ")));
    }
    *slot = v.to_string();
    changes.push(format!("{key} = {v}"));
    Ok(())
}

/// Machine-readable description for the AI prompt / editor.
pub fn schema_json() -> Value {
    json!({
        "presets": PRESETS.iter().map(|p| json!({"key": p.key, "label": p.label})).collect::<Vec<_>>(),
        "colors": COLOR_TOKENS,
        "fonts": FONTS,
        "radius": RADII,
        "header_style": HEADER_STYLES,
        "button_style": BUTTON_STYLES,
        "animations": ANIMATIONS,
        "base_font_size": "14..21",
    })
}
