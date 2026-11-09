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
pub const HEADER_STYLES: &[&str] = &["simple", "centered", "bold", "sidebar", "toggle"];
pub const BUTTON_STYLES: &[&str] = &["solid", "outline", "soft"];
pub const ANIMATIONS: &[&str] = &["none", "fade-up", "fade-in", "zoom-in", "slide-left", "slide-right"];
pub const COLOR_TOKENS: &[&str] = &["primary", "accent", "background", "surface", "text", "muted"];

impl Default for Theme {
    /// The look of a site that never picked one. The catalog of looks lives in
    /// the engine (`themes` collection); this is only the rule-level fallback.
    fn default() -> Self {
        Theme {
            colors: ThemeColors {
                primary: "#14223f".into(),
                accent: "#47664f".into(),
                background: "#fbf8f2".into(),
                surface: "#f4ede1".into(),
                text: "#1a2236".into(),
                muted: "#5f6678".into(),
            },
            font_heading: "Fraunces".into(),
            font_body: "Be Vietnam Pro".into(),
            radius: "lg".into(),
            header_style: "simple".into(),
            button_style: "solid".into(),
            base_font_size: 17,
            default_animation: "fade-up".into(),
        }
    }
}

/// Checks every field of a complete look (from the catalog or an admin edit).
pub fn validate(theme: &Theme) -> Result<(), String> {
    let mut copy = Theme::default();
    let patch = serde_json::to_value(theme).map_err(|e| e.to_string())?;
    apply_patch(&mut copy, &patch)?;
    if copy != *theme {
        return Err("giao diện có giá trị ngoài phạm vi cho phép".into());
    }
    Ok(())
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
    apply_patch_with(theme, patch, &|_| None)
}

/// Like `apply_patch`, with `preset` resolving a catalog key (the catalog lives in the engine).
pub fn apply_patch_with(theme: &mut Theme, patch: &Value, preset: &dyn Fn(&str) -> Option<Theme>) -> Result<Vec<String>, String> {
    let obj = patch.as_object().ok_or("theme patch phải là object")?;
    let mut changes = Vec::new();
    if let Some(p) = obj.get("preset").and_then(|v| v.as_str()) {
        *theme = preset(p).ok_or_else(|| format!("giao diện không tồn tại: {p}. Dùng list_themes để xem các key."))?;
        changes.push(format!("áp dụng giao diện {p}"));
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
        "colors": COLOR_TOKENS,
        "fonts": FONTS,
        "radius": RADII,
        "header_style": HEADER_STYLES,
        "button_style": BUTTON_STYLES,
        "animations": ANIMATIONS,
        "base_font_size": "14..21",
    })
}
