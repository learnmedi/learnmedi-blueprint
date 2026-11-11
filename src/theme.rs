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
    /// How the menu is laid out (see `HEADER_STYLES`): a top bar, a column on the
    /// left or right, a corner button, a bar at the bottom, a floating pill…
    pub header_style: String,
    /// solid | outline | soft
    pub button_style: String,
    /// 15..20 px
    pub base_font_size: i64,
    /// Default entrance animation for blocks that do not set one.
    pub default_animation: String,
    /// none (no motion at all) | calm (entrance only) | rich (staggered reveals,
    /// parallax, hover depth, count-up numbers, moving decor) | burst (rich plus a
    /// custom cursor, opening curtain, decoding titles, spotlight pictures, giant
    /// ghost words, scroll skew, glare tilt, colour wipes, cursor trail, film grain).
    #[serde(default = "d_motion")]
    pub motion: String,
    /// light | regular | bold | black
    #[serde(default = "d_weight")]
    pub heading_weight: String,
    /// The theme's own effects (keys of `FX`), rendered as `fx-<key>` classes on
    /// <body>. Empty on a "rich"/"burst" look made before effects were split:
    /// `effective_fx` then gives the old shared set.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub fx: Vec<String>,
}

fn d_motion() -> String {
    "calm".into()
}
fn d_weight() -> String {
    "regular".into()
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
    "Space Grotesk",
    "IBM Plex Mono",
    "Baloo 2",
    "Archivo",
    "Noto Serif Display",
];
pub const RADII: &[&str] = &["none", "sm", "md", "lg", "pill"];
/// Menu layouts. Each one follows the menu of a real site (named in the theme YAML that uses it).
pub const HEADER_STYLES: &[&str] = &[
    // a bar across the top
    "simple",    // name left, links right
    "centered",  // name in the middle, links on both sides, small serif
    "bold",      // bar in the primary colour
    "overlay",   // transparent over the first section, solid after scrolling
    "cluster",   // links right after the name, icons pushed right
    "cta",       // caps links and a pill button as the last item
    "split",     // caps links around a round mark in the middle, rule under, active item boxed
    "tabs",      // links as boxed cells with dividers, top right
    "tiles",     // links as tall tiles with a big first letter and a small line
    "bigtype",   // huge bold links on the right, as big as the name
    "bracket",   // faded words in the middle, active one dark, [ brackets ] around the last
    "masthead",  // big title band, a full-width row of caps links under it
    "band",      // thin coloured strip on top with small centred links, page as a sheet
    "columns",   // logo block left, links in grouped columns with a ✣ bullet
    "ledger",    // round badge left, links in two stacked columns of faint caps
    // not a bar
    "stack",     // wordmark top left, links as a small vertical list top right
    "frame",     // spaced caps across the top, small labels in the bottom corners
    "corners",   // name top left, scroll ruler top middle, links bottom right, line bottom left
    "burger",    // only a button in the top-right corner; the menu fills the screen
    "drawer",    // a bar that turns into a round floating button; the menu slides in from the right
    "dock",      // the whole menu in a bar fixed at the bottom of the screen
    "pill",      // a floating rounded bar detached from the top edge
    "sidebar",   // fixed column on the left: name, line, links with a line marker
    "rail-left", // narrow fixed mono index at the left edge
    "rail-right",// vertical strip fixed on the right edge, links turned sideways
    "toggle",    // only a menu button
];
/// Effects a theme can turn on (`Theme.fx`): (key, the lowest motion level that may use it, what it does).
pub const FX: &[(&str, &str, &str)] = &[
    // motion "rich" and up
    ("words", "rich", "big headings rise word by word out of a mask"),
    ("stagger", "rich", "items of grids and lists come in one after another"),
    ("wipe-img", "rich", "pictures open from a clipped, zoomed frame"),
    ("parallax", "rich", "background and hero pictures drift slower than the page"),
    ("countup", "rich", "numbers count up when they come into view"),
    ("magnetic", "rich", "buttons lean toward the pointer"),
    ("ripple", "rich", "a ripple spreads from the click on buttons"),
    ("shimmer", "rich", "emphasised words carry a moving colour gradient"),
    ("rule-draw", "rich", "a short rule draws itself under section titles"),
    ("float", "rich", "badges and icons bob slowly"),
    ("tuck", "rich", "the menu hides while scrolling down and returns going up"),
    ("kenburns", "rich", "slideshow photos zoom slowly"),
    ("decor-move", "rich", "drawn decor (stars, clouds, mesh) moves"),
    ("card-lift", "rich", "cards rise and their picture zooms on hover"),
    ("btn-sheen", "rich", "a light sheen sweeps across buttons on hover"),
    ("nav-underline", "rich", "a line draws under menu links on hover"),
    ("progress", "rich", "a reading-progress line at the top"),
    ("bounce", "rich", "cards and items spring in with an elastic bounce"),
    ("wiggle", "rich", "cards and buttons wiggle on hover like toys"),
    ("sparkle", "rich", "little stars twinkle around emphasised words"),
    ("boop", "rich", "icons and arrows spring when the pointer touches them"),
    // motion "burst" only
    ("curtain", "burst", "an opening curtain with the site name"),
    ("letters", "burst", "letters of the big title drop in one by one"),
    ("decode", "burst", "small eyebrow lines decode from random signs"),
    ("ghost", "burst", "giant outlined words slide behind sections"),
    ("spotlight", "burst", "pictures are grey until the pointer brings their colour back"),
    ("color-wipe", "burst", "a colour block wipes off each picture"),
    ("tilt", "burst", "cards tilt in 3D toward the pointer with a glare"),
    ("clip-sections", "burst", "coloured sections open out from a rounded inset"),
    ("grain", "burst", "film grain over the page"),
    ("skew", "burst", "pictures skew with the scroll speed"),
    ("cursor", "burst", "a dot-and-ring cursor with a fading trail"),
    ("iris", "burst", "the opening picture opens through a growing circle and closes again as you scroll"),
    ("focus-pull", "burst", "pictures are out of focus and sharpen as they reach the middle of the screen"),
    ("glitch", "burst", "pictures arrive in shifted horizontal slices with split colour"),
    ("scramble", "burst", "menu and button words scramble into mono signs on hover"),
    ("flowfield", "burst", "a field of particles flows behind the opening and bends around the pointer"),
    ("paper-tear", "burst", "sections arrive with a torn paper edge"),
    ("ink-underline", "burst", "links get a hand-drawn wavy underline on hover"),
    ("magnet-cursor", "burst", "a big cursor disc that grabs buttons and says “Xem” over pictures"),
    ("hover-marquee", "burst", "a running line of text crosses a card on hover"),
    ("starfield", "burst", "a slow starfield behind the page that turns with the pointer"),
    ("flashlight", "burst", "dark sections are lit only around the pointer"),
    ("sticker", "burst", "pictures and cards land like stickers, then wobble on hover"),
    ("confetti", "burst", "buttons throw paper confetti when clicked"),
    ("doodle", "burst", "a hand-drawn loop circles emphasised words"),
    ("blinds", "burst", "the opening picture opens like venetian blinds"),
    ("typewriter", "burst", "large statements type themselves with a caret"),
    ("constellation", "burst", "a net of points and lines around the opening picture follows the pointer"),
    ("odometer", "burst", "numbers roll into place digit by digit"),
    ("gold-shine", "burst", "a gold-foil sheen passes over titles and award cards"),
    // opening animations (one per burst theme, see OPENERS)
    ("op-collage", "burst", "opening: photos from the page fly in around the name, then the page lifts like a sheet"),
    ("op-hex", "burst", "opening: a hexagon draws itself, the name and key numbers fly out, the hexagon closes onto the page"),
    ("op-marquee", "burst", "opening: rows of the name race across the screen, then the panel slides up"),
    ("op-stars", "burst", "opening: points of light gather into the name, then scatter"),
    ("op-noise", "burst", "opening: a screen of flickering signs resolves into the name, then splits into slices"),
    ("op-paper", "burst", "opening: coloured paper sheets with the name peel away one after another"),
    ("op-leader", "burst", "opening: a film countdown 3-2-1 with a sweeping hand, a flash, then the name"),
];
/// Effects that are opening animations; a page runs at most one, and none when the site has its own intro.
pub const OPENERS: &[&str] = &["curtain", "op-collage", "op-hex", "op-marquee", "op-stars", "op-noise", "op-paper", "op-leader"];
pub const FX_MAX: usize = 7;
/// The shared sets every rich / burst look used before effects were split; a stored look
/// with no `fx` keeps them.
const LEGACY_RICH_FX: &[&str] = &["words", "stagger", "wipe-img", "parallax", "countup", "magnetic", "ripple", "shimmer", "rule-draw", "float", "tuck", "kenburns", "decor-move", "card-lift", "btn-sheen", "nav-underline", "progress"];
const LEGACY_BURST_FX: &[&str] = &["curtain", "letters", "decode", "ghost", "spotlight", "color-wipe", "tilt", "clip-sections", "grain", "skew", "cursor"];
pub const MOTIONS: &[&str] = &["none", "calm", "rich", "burst"];
pub const HEADING_WEIGHTS: &[&str] = &["light", "regular", "bold", "black"];
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
            motion: d_motion(),
            heading_weight: d_weight(),
            fx: Vec::new(),
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

fn motion_rank(m: &str) -> u8 {
    match m {
        "burst" => 3,
        "rich" => 2,
        "calm" => 1,
        _ => 0,
    }
}

/// Whether a look at this motion level may use the effect.
pub fn fx_allowed(fx: &str, motion: &str) -> bool {
    FX.iter().any(|(k, min, _)| *k == fx && motion_rank(motion) >= motion_rank(min))
}

/// The effects a page really runs: the look's own, or the old shared set for a
/// rich / burst look saved before effects were split.
pub fn effective_fx(theme: &Theme) -> Vec<&str> {
    if !theme.fx.is_empty() {
        return theme.fx.iter().map(String::as_str).filter(|f| fx_allowed(f, &theme.motion)).collect();
    }
    match theme.motion.as_str() {
        "burst" => LEGACY_RICH_FX.iter().chain(LEGACY_BURST_FX).copied().collect(),
        "rich" => LEGACY_RICH_FX.to_vec(),
        _ => Vec::new(),
    }
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
            "motion" => {
                set_enum(&mut theme.motion, val, MOTIONS, key, &mut changes)?;
                let motion = theme.motion.clone();
                theme.fx.retain(|f| fx_allowed(f, &motion));
            }
            "fx" => {
                let list = val.as_array().ok_or("fx phải là danh sách")?;
                let mut fx: Vec<String> = Vec::new();
                for v in list {
                    let f = v.as_str().ok_or("fx: mỗi mục là một chuỗi")?;
                    if !FX.iter().any(|(k, _, _)| *k == f) {
                        return Err(format!("fx: '{f}' không tồn tại. Có: {}", FX.iter().map(|x| x.0).collect::<Vec<_>>().join(", ")));
                    }
                    if !fx.iter().any(|x| x == f) {
                        fx.push(f.to_string());
                    }
                }
                if fx.len() > FX_MAX {
                    return Err(format!("fx: tối đa {FX_MAX} hiệu ứng"));
                }
                fx_check(&fx, obj.get("motion").and_then(|m| m.as_str()).unwrap_or(&theme.motion))?;
                changes.push(format!("fx = {}", fx.join(", ")));
                theme.fx = fx;
            }
            "heading_weight" => set_enum(&mut theme.heading_weight, val, HEADING_WEIGHTS, key, &mut changes)?,
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

fn fx_check(fx: &[String], motion: &str) -> Result<(), String> {
    match fx.iter().find(|f| !fx_allowed(f, motion)) {
        Some(f) => Err(format!("fx: '{f}' không dùng được với motion={motion}")),
        None => Ok(()),
    }
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
        "motion": MOTIONS,
        "heading_weight": HEADING_WEIGHTS,
        "fx": FX.iter().map(|(k, min, what)| json!({"key": k, "min_motion": min, "what": what})).collect::<Vec<_>>(),
        "fx_max": FX_MAX,
        "base_font_size": "14..21",
    })
}
