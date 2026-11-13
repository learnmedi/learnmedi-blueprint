//! The block system.
//!
//! A page is an ordered list of `Block`s. Each block has a `type` from the
//! registry below, `props` validated against that type's field schema, and a
//! `style` (background, colours, spacing, width, alignment, entrance
//! animation). The same registry drives:
//!   - validation of every change (manual editor AND AI tool calls),
//!   - the property panel in the Studio (served as JSON),
//!   - the AI tool docs / system prompt,
//!   - server-side rendering (`render.rs`).
//!
//! Text fields marked "title" accept `*emphasis*`, rendered as an accent-coloured
//! <em> (the editorial look of tranhoangyen.com).

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};

use crate::tabs::{BlockGroup, COLLECTION_TEMPLATES, TAB_KINDS};
use crate::theme::{ANIMATIONS, normalize_color};

// ───────────────────────────── Data ─────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Block {
    pub id: String,
    #[serde(rename = "type")]
    pub kind: String,
    pub props: Value,
    #[serde(default)]
    pub style: BlockStyle,
    #[serde(default)]
    pub hidden: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BlockStyle {
    /// "" (transparent) | theme token (primary, accent, surface, …) | #hex
    #[serde(default)]
    pub background: String,
    /// "" (auto) | theme token | #hex
    #[serde(default)]
    pub text_color: String,
    /// Asset id or https URL of a background image.
    #[serde(default)]
    pub background_image: String,
    /// Dark overlay on background images, 0..80 (%).
    #[serde(default)]
    pub overlay: i64,
    /// none | sm | md | lg | xl
    #[serde(default = "d_md")]
    pub padding: String,
    /// left | center | right
    #[serde(default = "d_left")]
    pub align: String,
    /// narrow | normal | wide | full
    #[serde(default = "d_normal")]
    pub width: String,
    /// "" = theme default, or one of theme::ANIMATIONS
    #[serde(default)]
    pub animation: String,
    /// Optional HTML id so menus can link to `#anchor`.
    #[serde(default)]
    pub anchor: String,
    /// "" or one of DECORS: a CSS-drawn background texture (no image needed).
    #[serde(default)]
    pub decor: String,
}

fn d_md() -> String {
    "md".into()
}
fn d_left() -> String {
    "left".into()
}
fn d_normal() -> String {
    "normal".into()
}

impl Default for BlockStyle {
    fn default() -> Self {
        BlockStyle {
            background: String::new(),
            text_color: String::new(),
            background_image: String::new(),
            overlay: 0,
            padding: d_md(),
            align: d_left(),
            width: d_normal(),
            animation: String::new(),
            anchor: String::new(),
            decor: String::new(),
        }
    }
}

pub const PADDINGS: &[&str] = &["none", "sm", "md", "lg", "xl"];
pub const ALIGNS: &[&str] = &["left", "center", "right"];
pub const WIDTHS: &[&str] = &["narrow", "normal", "wide", "full"];
pub const DECORS: &[&str] = &["grain", "stars", "grid", "clouds", "rays", "lines", "circle", "arches", "dots", "paper", "mesh", "network"];

// ───────────────────────────── Schema ───────────────────────────────────────

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Kind {
    /// Single line. `title` = accepts *emphasis*.
    Text { max: usize, #[serde(default, rename = "emphasis")] title: bool },
    /// Markdown (headings, lists, links, bold…). Raw HTML is escaped.
    Markdown { max: usize },
    Url,
    /// Asset id (uploaded image) or https URL.
    Image,
    /// Asset id (uploaded video) or YouTube/Vimeo/https URL.
    Video,
    Color,
    Select { options: Vec<String> },
    Number { min: f64, max: f64 },
    Bool,
    Icon,
    List { item: Vec<Field>, max: usize },
}

#[derive(Debug, Deserialize)]
pub struct Field {
    pub key: String,
    pub label: String,
    #[serde(flatten)]
    pub kind: Kind,
    #[serde(default)]
    pub required: bool,
    pub default: Value,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BlockType {
    pub group: BlockGroup,
    pub key: String,
    pub label: String,
    pub description: String,
    pub fields: Vec<Field>,
}

pub const ICONS: &[&str] = &[
    "book", "research", "spark", "heart", "star", "school", "globe", "chat", "video", "pen", "users", "award", "leaf",
    "check", "calendar", "mail", "phone", "pin", "music", "code", "chart", "lightbulb", "child", "puzzle",
];

const BLOCK_FILES: &[(&str, &str)] = &[
    ("01-hero.yaml", include_str!("../ui/blocks/01-hero.yaml")),
    ("02-heading.yaml", include_str!("../ui/blocks/02-heading.yaml")),
    ("03-text.yaml", include_str!("../ui/blocks/03-text.yaml")),
    ("04-image.yaml", include_str!("../ui/blocks/04-image.yaml")),
    ("05-gallery.yaml", include_str!("../ui/blocks/05-gallery.yaml")),
    ("06-media_text.yaml", include_str!("../ui/blocks/06-media_text.yaml")),
    ("07-features.yaml", include_str!("../ui/blocks/07-features.yaml")),
    ("08-cards.yaml", include_str!("../ui/blocks/08-cards.yaml")),
    ("09-rows.yaml", include_str!("../ui/blocks/09-rows.yaml")),
    ("10-buttons.yaml", include_str!("../ui/blocks/10-buttons.yaml")),
    ("11-cta.yaml", include_str!("../ui/blocks/11-cta.yaml")),
    ("12-quote.yaml", include_str!("../ui/blocks/12-quote.yaml")),
    ("13-video.yaml", include_str!("../ui/blocks/13-video.yaml")),
    ("14-stats.yaml", include_str!("../ui/blocks/14-stats.yaml")),
    ("15-timeline.yaml", include_str!("../ui/blocks/15-timeline.yaml")),
    ("16-faq.yaml", include_str!("../ui/blocks/16-faq.yaml")),
    ("17-contact.yaml", include_str!("../ui/blocks/17-contact.yaml")),
    ("18-blog_index.yaml", include_str!("../ui/blocks/18-blog_index.yaml")),
    ("19-course_index.yaml", include_str!("../ui/blocks/19-course_index.yaml")),
    ("20-collection_index.yaml", include_str!("../ui/blocks/20-collection_index.yaml")),
    ("21-collection_list.yaml", include_str!("../ui/blocks/21-collection_list.yaml")),
    ("22-post_list.yaml", include_str!("../ui/blocks/22-post_list.yaml")),
    ("23-course_list.yaml", include_str!("../ui/blocks/23-course_list.yaml")),
    ("24-subscribe.yaml", include_str!("../ui/blocks/24-subscribe.yaml")),
    ("25-divider.yaml", include_str!("../ui/blocks/25-divider.yaml")),
    ("26-spacer.yaml", include_str!("../ui/blocks/26-spacer.yaml")),
    ("27-product_list.yaml", include_str!("../ui/blocks/27-product_list.yaml")),
    ("28-slideshow.yaml", include_str!("../ui/blocks/28-slideshow.yaml")),
    ("29-product_index.yaml", include_str!("../ui/blocks/29-product_index.yaml")),
    ("30-community_index.yaml", include_str!("../ui/blocks/30-community_index.yaml")),
    ("31-survey_index.yaml", include_str!("../ui/blocks/31-survey_index.yaml")),
    ("32-associate_index.yaml", include_str!("../ui/blocks/32-associate_index.yaml")),
];

pub static REGISTRY: std::sync::LazyLock<Vec<BlockType>> = std::sync::LazyLock::new(|| {
    let mut keys = std::collections::HashSet::new();
    BLOCK_FILES.iter().map(|(name, yaml)| {
        let block: BlockType = serde_yaml_ng::from_str(yaml)
            .unwrap_or_else(|e| panic!("invalid block catalog entry {name}: {e}"));
        assert!(keys.insert(block.key.clone()), "duplicate block key {}", block.key);
        block
    }).collect()
});

pub fn block_type(key: &str) -> Option<&'static BlockType> {
    REGISTRY.iter().find(|b| b.key == key)
}

pub fn type_keys() -> Vec<&'static str> {
    REGISTRY.iter().map(|b| b.key.as_str()).collect()
}

// ───────────────────────────── Validation ───────────────────────────────────

fn parse_default(field: &Field) -> Value {
    field.default.clone()
}

/// Default props for a block type.
pub fn default_props(bt: &BlockType) -> Value {
    let mut m = Map::new();
    for fd in &bt.fields {
        m.insert(fd.key.clone(), parse_default(fd));
    }
    Value::Object(m)
}

/// `true` when the value is an uploaded asset id (not a URL).
pub fn is_asset_ref(v: &str) -> bool {
    !v.is_empty() && !v.contains(':') && !v.contains('/') && v.len() <= 40 && v.chars().all(|c| c.is_ascii_alphanumeric())
}

pub fn validate_url(v: &str) -> bool {
    let l = v.trim().to_ascii_lowercase();
    l.is_empty()
        || l.starts_with("https://")
        || l.starts_with("http://")
        || l.starts_with("mailto:")
        || l.starts_with("tel:")
        || (l.starts_with('/') && !l.starts_with("//"))
        || l.starts_with('#')
}

fn validate_field(fd: &Field, v: Option<&Value>, path: &str, assets: &mut Vec<String>) -> Result<Value, String> {
    let v = match v {
        None | Some(Value::Null) => parse_default(fd),
        Some(v) => v.clone(),
    };
    let p = if path.is_empty() { fd.key.to_string() } else { format!("{path}.{}", fd.key) };
    let as_string = |v: &Value| -> Result<String, String> {
        match v {
            Value::String(s) => Ok(s.clone()),
            Value::Number(n) => Ok(n.to_string()),
            Value::Bool(b) => Ok(b.to_string()),
            _ => Err(format!("{p}: phải là chuỗi")),
        }
    };
    let out = match &fd.kind {
        Kind::Text { max, .. } => {
            let s = as_string(&v)?.replace(['\n', '\r'], " ");
            let s = s.trim().to_string();
            if s.chars().count() > *max {
                return Err(format!("{p}: dài quá {max} ký tự"));
            }
            Value::String(s)
        }
        Kind::Markdown { max } => {
            let s = as_string(&v)?;
            if s.chars().count() > *max {
                return Err(format!("{p}: dài quá {max} ký tự"));
            }
            Value::String(s)
        }
        Kind::Url => {
            let s = as_string(&v)?.trim().to_string();
            if !validate_url(&s) {
                return Err(format!("{p}: liên kết không hợp lệ ('{s}'). Dùng https://…, mailto:, tel:, /trang hoặc #muc"));
            }
            Value::String(s)
        }
        Kind::Image | Kind::Video => {
            let s = as_string(&v)?.trim().to_string();
            if is_asset_ref(&s) {
                assets.push(s.clone());
            } else if !s.is_empty() && !s.to_ascii_lowercase().starts_with("https://") {
                return Err(format!("{p}: phải là id ảnh/video đã tải lên hoặc link https://"));
            }
            Value::String(s)
        }
        Kind::Color => {
            let s = as_string(&v)?;
            Value::String(normalize_color(&s).ok_or_else(|| format!("{p}: màu không hợp lệ '{s}'"))?)
        }
        Kind::Select { options: opts } => {
            let s = as_string(&v)?;
            if !opts.contains(&s) {
                return Err(format!("{p}: '{s}' không hợp lệ, chọn một trong: {}", opts.join(", ")));
            }
            Value::String(s)
        }
        Kind::Icon => {
            let s = as_string(&v)?;
            Value::String(if ICONS.contains(&s.as_str()) { s } else { "star".into() })
        }
        Kind::Number { min, max } => {
            let n = match &v {
                Value::Number(n) => n.as_f64().unwrap_or(*min),
                Value::String(s) => s.trim().parse::<f64>().map_err(|_| format!("{p}: phải là số"))?,
                _ => return Err(format!("{p}: phải là số")),
            };
            let n = n.clamp(*min, *max);
            if n.fract() == 0.0 { json!(n as i64) } else { json!(n) }
        }
        Kind::Bool => match &v {
            Value::Bool(b) => Value::Bool(*b),
            Value::String(s) => Value::Bool(matches!(s.as_str(), "true" | "1" | "yes")),
            Value::Number(n) => Value::Bool(n.as_f64().unwrap_or(0.0) != 0.0),
            _ => return Err(format!("{p}: phải là true/false")),
        },
        Kind::List { item, max } => {
            let arr = v.as_array().ok_or_else(|| format!("{p}: phải là danh sách"))?;
            if arr.len() > *max {
                return Err(format!("{p}: tối đa {max} mục"));
            }
            let mut out = Vec::with_capacity(arr.len());
            for (i, it) in arr.iter().enumerate() {
                let obj = it.as_object().ok_or_else(|| format!("{p}[{i}]: phải là object"))?;
                out.push(validate_object(item, obj, &format!("{p}[{i}]"), assets)?);
            }
            Value::Array(out)
        }
    };
    if fd.required {
        let empty = match &out {
            Value::String(s) => s.is_empty(),
            Value::Array(a) => a.is_empty() && !matches!(fd.kind, Kind::List { .. }),
            _ => false,
        };
        if empty && !matches!(fd.kind, Kind::Image | Kind::Video) {
            return Err(format!("{p}: bắt buộc"));
        }
    }
    Ok(out)
}

fn validate_object(fields: &[Field], obj: &Map<String, Value>, path: &str, assets: &mut Vec<String>) -> Result<Value, String> {
    for k in obj.keys() {
        if !fields.iter().any(|f| &f.key == k) {
            let allowed: Vec<_> = fields.iter().map(|f| f.key.as_str()).collect();
            let p = if path.is_empty() { k.clone() } else { format!("{path}.{k}") };
            return Err(format!("{p}: trường không tồn tại. Các trường hợp lệ: {}", allowed.join(", ")));
        }
    }
    let mut out = Map::new();
    for fd in fields {
        out.insert(fd.key.clone(), validate_field(fd, obj.get(&fd.key), path, assets)?);
    }
    Ok(Value::Object(out))
}

/// Validate full props for a block type. Missing fields get defaults; unknown
/// fields are rejected. Returns (clean props, referenced asset ids).
pub fn validate_props(kind: &str, props: &Value) -> Result<(Value, Vec<String>), String> {
    let bt = block_type(kind).ok_or_else(|| format!("loại block không tồn tại: {kind}. Có: {}", type_keys().join(", ")))?;
    let empty = Map::new();
    let obj = match props {
        Value::Object(o) => o,
        Value::Null => &empty,
        _ => return Err("props phải là object".into()),
    };
    let mut assets = Vec::new();
    let v = validate_object(&bt.fields, obj, "", &mut assets)?;
    Ok((v, assets))
}

/// Shallow merge: keys in `patch` replace keys in `base` (lists are replaced
/// wholesale — send the full list to change one item).
pub fn merge_props(base: &Value, patch: &Value) -> Value {
    let mut out = base.as_object().cloned().unwrap_or_default();
    if let Some(p) = patch.as_object() {
        for (k, v) in p {
            out.insert(k.clone(), v.clone());
        }
    }
    Value::Object(out)
}

/// Apply a style patch with validation; returns asset refs found.
pub fn apply_style_patch(style: &mut BlockStyle, patch: &Value) -> Result<Vec<String>, String> {
    let obj = patch.as_object().ok_or("style phải là object")?;
    let mut assets = Vec::new();
    for (k, v) in obj {
        let s = match v {
            Value::String(s) => s.clone(),
            Value::Number(n) => n.to_string(),
            Value::Null => String::new(),
            _ => return Err(format!("style.{k}: giá trị không hợp lệ")),
        };
        match k.as_str() {
            "background" => style.background = normalize_color(&s).ok_or_else(|| format!("style.background: màu không hợp lệ '{s}'"))?,
            "text_color" => style.text_color = normalize_color(&s).ok_or_else(|| format!("style.text_color: màu không hợp lệ '{s}'"))?,
            "background_image" => {
                let s = s.trim().to_string();
                if is_asset_ref(&s) {
                    assets.push(s.clone());
                } else if !s.is_empty() && !s.starts_with("https://") {
                    return Err("style.background_image: phải là id ảnh hoặc https://".into());
                }
                style.background_image = s;
            }
            "overlay" => style.overlay = s.parse::<i64>().map_err(|_| "style.overlay: phải là số 0..80")?.clamp(0, 80),
            "padding" => style.padding = one_of(&s, PADDINGS, "padding")?,
            "align" => style.align = one_of(&s, ALIGNS, "align")?,
            "width" => style.width = one_of(&s, WIDTHS, "width")?,
            "animation" => {
                style.animation = if s.is_empty() { s } else { one_of(&s, ANIMATIONS, "animation")? };
            }
            "anchor" => {
                let a = crate::util::slugify(&s);
                style.anchor = a;
            }
            "decor" => {
                style.decor = if s.is_empty() { s } else { one_of(&s, DECORS, "decor")? };
            }
            other => return Err(format!("style.{other}: không tồn tại. Có: background, text_color, background_image, overlay, padding, align, width, animation, anchor, decor")),
        }
    }
    Ok(assets)
}

fn one_of(v: &str, allowed: &[&str], name: &str) -> Result<String, String> {
    if allowed.contains(&v) { Ok(v.to_string()) } else { Err(format!("style.{name}: '{v}' không hợp lệ. Có: {}", allowed.join(", "))) }
}

/// Every asset id referenced by a block (props + style).
pub fn block_asset_refs(b: &Block) -> Vec<String> {
    let mut refs = validate_props(&b.kind, &b.props).map(|(_, a)| a).unwrap_or_default();
    if is_asset_ref(&b.style.background_image) {
        refs.push(b.style.background_image.clone());
    }
    refs
}

pub fn new_block(kind: &str, props: &Value) -> Result<(Block, Vec<String>), String> {
    let (props, assets) = validate_props(kind, props)?;
    Ok((Block { id: crate::util::short_id(), kind: kind.into(), props, style: BlockStyle::default(), hidden: false }, assets))
}

// ───────────────────────────── Schema export ────────────────────────────────

fn kind_json(k: &Kind) -> Value {
    match k {
        Kind::Text { max, title } => json!({"type": "text", "max": max, "emphasis": title}),
        Kind::Markdown { max } => json!({"type": "markdown", "max": max}),
        Kind::Url => json!({"type": "url"}),
        Kind::Image => json!({"type": "image"}),
        Kind::Video => json!({"type": "video"}),
        Kind::Color => json!({"type": "color"}),
        Kind::Select { options: o } => json!({"type": "select", "options": o}),
        Kind::Number { min, max } => json!({"type": "number", "min": min, "max": max}),
        Kind::Bool => json!({"type": "bool"}),
        Kind::Icon => json!({"type": "icon", "options": ICONS}),
        Kind::List { item, max } => json!({"type": "list", "max": max, "item": fields_json(item)}),
    }
}

fn fields_json(fields: &[Field]) -> Value {
    Value::Array(
        fields
            .iter()
            .map(|f| {
                let mut v = kind_json(&f.kind);
                v["key"] = json!(f.key);
                v["label"] = json!(f.label);
                v["required"] = json!(f.required);
                v["default"] = parse_default(f);
                v
            })
            .collect(),
    )
}

/// Full registry for the Studio property panel.
pub fn registry_json() -> Value {
    json!({
        "types": REGISTRY.iter().map(|b| json!({
            "key": b.key, "group": b.group, "label": b.label, "description": b.description, "fields": fields_json(&b.fields)
        })).collect::<Vec<_>>(),
        "style": {
            "padding": PADDINGS, "align": ALIGNS, "width": WIDTHS, "animation": ANIMATIONS, "decor": DECORS,
            "colors": crate::theme::COLOR_TOKENS,
        },
        "icons": ICONS,
        "tab_kinds": TAB_KINDS,
        "collection_templates": COLLECTION_TEMPLATES,
    })
}

fn kind_short(k: &Kind) -> String {
    match k {
        Kind::Text { title: true, .. } => "text(*nhấn*)".into(),
        Kind::Text { .. } => "text".into(),
        Kind::Markdown { .. } => "markdown".into(),
        Kind::Url => "url".into(),
        Kind::Image => "image".into(),
        Kind::Video => "video".into(),
        Kind::Color => "color".into(),
        Kind::Select { options: o } => o.join("|"),
        Kind::Number { min, max } => format!("number {min}..{max}"),
        Kind::Bool => "bool".into(),
        Kind::Icon => "icon".into(),
        Kind::List { item, max } => format!(
            "list≤{max}[{}]",
            item.iter().map(|f| format!("{}:{}", f.key, kind_short(&f.kind))).collect::<Vec<_>>().join(", ")
        ),
    }
}

/// Compact one-line-per-type catalog for the AI system prompt.
pub fn catalog_for_prompt() -> String {
    catalog_for_prompt_without(&[])
}

/// The block catalog without the given block types (switched off, or their
/// feature is not on for the AI).
pub fn catalog_for_prompt_without(skip: &[&str]) -> String {
    let mut s = String::new();
    for b in REGISTRY.iter().filter(|b| !skip.contains(&b.key.as_str())) {
        s.push_str(&format!("- {} ({}): {}\n  props: ", b.key, b.label, b.description));
        s.push_str(
            &b.fields
                .iter()
                .map(|f| format!("{}{}:{}", f.key, if f.required { "*" } else { "" }, kind_short(&f.kind)))
                .collect::<Vec<_>>()
                .join("; "),
        );
        s.push('\n');
    }
    s
}

/// Detailed schema of one type (AI tool `get_block_schema`).
pub fn type_schema_json(key: &str) -> Option<Value> {
    block_type(key).map(|b| json!({"key": b.key, "group": b.group, "label": b.label, "description": b.description, "fields": fields_json(&b.fields), "example_props": default_props(b)}))
}
