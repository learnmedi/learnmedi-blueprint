//! Interactive decks (`site.decks`).
//!
//! A deck is one document, stored by the site as a YAML file per version
//! (`…/<rev>.lmdeck.yaml` in the site's storage): `meta`, `theme`, `segments`
//! (parts of the session with minutes), `data` (named tables shared by slides)
//! and `slides`. Every slide has common fields (`id`, `kind`, `title`, `notes`…)
//! and the fields of its kind, declared once in `ui/slides/*.yaml` with the same
//! field schema as page blocks. The same registry drives validation (every op,
//! the API, the AI tools), the schema the editor and outside assistants read,
//! and the deck player.
//!
//! Nothing here is HTML: text is Markdown (rendered with raw HTML escaped),
//! colours are checked hex values, fonts come from a list.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};
use std::collections::{BTreeMap, HashSet};
use std::sync::LazyLock;

use crate::blocks::{Field, fields_json, kind_short, validate_object};

// ───────────────────────────── Limits ───────────────────────────────────────

pub const MAX_SLIDES: usize = 120;
pub const MAX_SEGMENTS: usize = 20;
pub const MAX_DATASETS: usize = 12;
pub const MAX_ROWS: usize = 300;
pub const MAX_ROW_KEYS: usize = 30;
pub const MAX_CELL: usize = 3000;
/// Size of the YAML a deck may have (bytes).
pub const MAX_YAML: usize = 1_000_000;
/// Words on screen before `lint_deck` warns.
pub const WORDS_WARN: usize = 120;

pub const TONES: &[&str] = &["default", "primary", "accent", "good", "warn", "bad", "muted"];
pub const BACKGROUNDS: &[&str] = &["", "light", "tint", "dark", "primary", "image"];
pub const SIZES: &[&str] = &["s", "m", "l"];
pub const MOTIONS: &[&str] = &["none", "subtle", "lively"];
pub const LANGS: &[&str] = &["vi", "en"];

/// Fonts a deck may use (all read Vietnamese well; loaded from Google Fonts,
/// the offline copy falls back to system fonts).
pub const DECK_FONTS: &[&str] = &[
    "Be Vietnam Pro", "Inter", "Roboto", "Noto Sans", "Noto Serif", "Source Serif 4", "Lora", "Merriweather",
    "Montserrat", "IBM Plex Sans", "Nunito", "Playfair Display", "Space Grotesk",
];

// ───────────────────────────── Data ─────────────────────────────────────────

fn one() -> u32 {
    1
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
#[serde(default, deny_unknown_fields)]
pub struct DeckMeta {
    pub title: String,
    pub subtitle: String,
    /// Free text shown on the cover / footer ("14/11/2026").
    pub date: String,
    pub author: String,
    pub lang: String,
    /// Planned length in minutes (presenter clock); 0 = sum of segments.
    pub duration_min: i64,
    /// "HH:MM" the session starts (agenda times, presenter clock).
    pub start_time: String,
    /// Footer line on every slide (sources, unit…).
    pub footer: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(default, deny_unknown_fields)]
pub struct DeckTheme {
    /// executive | academic | playful | midnight
    pub preset: String,
    /// Hex colours; "" = the preset's.
    pub primary: String,
    pub accent: String,
    pub background: String,
    pub surface: String,
    pub ink: String,
    pub muted: String,
    pub good: String,
    pub warn: String,
    pub bad: String,
    /// "" = the preset's; else one of `DECK_FONTS`.
    pub heading_font: String,
    pub body_font: String,
    /// s | m | l
    pub size: String,
    /// "" = preset's; none | subtle | lively
    pub motion: String,
    /// Asset id or https URL.
    pub logo: String,
    pub page_numbers: bool,
}

impl Default for DeckTheme {
    fn default() -> Self {
        DeckTheme {
            preset: "executive".into(),
            primary: String::new(),
            accent: String::new(),
            background: String::new(),
            surface: String::new(),
            ink: String::new(),
            muted: String::new(),
            good: String::new(),
            warn: String::new(),
            bad: String::new(),
            heading_font: String::new(),
            body_font: String::new(),
            size: "m".into(),
            motion: String::new(),
            logo: String::new(),
            page_numbers: true,
        }
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
#[serde(default, deny_unknown_fields)]
pub struct Segment {
    pub id: String,
    pub name: String,
    pub min: i64,
    pub outcome: String,
}

/// One slide: common fields + the fields of its kind (flattened, so a slide
/// reads `{id, kind, title, question, options…}`).
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct Slide {
    #[serde(default)]
    pub id: String,
    pub kind: String,
    /// Segment id.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub seg: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub eyebrow: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub title: String,
    /// Presenter notes (Markdown); visitors never see them.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub notes: String,
    /// Source line under the slide.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub source: String,
    #[serde(default, skip_serializing_if = "is_false")]
    pub hidden: bool,
    /// Accent tone of this slide ("" = theme).
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub accent: String,
    /// Background variant ("" = theme).
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub bg: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub bg_image: String,
    /// Planned minutes on this slide (presenter clock); 0 = not set.
    #[serde(default, skip_serializing_if = "is_zero")]
    pub minutes: i64,
    #[serde(flatten)]
    pub props: Map<String, Value>,
}

fn is_false(b: &bool) -> bool {
    !*b
}
fn is_zero(n: &i64) -> bool {
    *n == 0
}

/// Common slide keys (the rest are the kind's fields).
pub const COMMON_KEYS: &[&str] = &["id", "kind", "seg", "eyebrow", "title", "notes", "source", "hidden", "accent", "bg", "bg_image", "minutes"];

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
#[serde(default, deny_unknown_fields)]
pub struct Deck {
    #[serde(default = "one")]
    pub lmdeck: u32,
    pub meta: DeckMeta,
    pub theme: DeckTheme,
    pub segments: Vec<Segment>,
    /// Named tables: each a list of rows (flat objects of text, numbers, booleans).
    pub data: BTreeMap<String, Vec<Map<String, Value>>>,
    pub slides: Vec<Slide>,
}

// ───────────────────────────── Slide kinds ──────────────────────────────────

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SlideKind {
    pub key: String,
    pub label: String,
    /// present | explore | decide | learn
    pub group: String,
    /// Records what the audience or presenter does on it (results).
    #[serde(default)]
    pub interactive: bool,
    pub description: String,
    pub fields: Vec<Field>,
    /// A complete slide of this kind (for docs and AI).
    pub example: Value,
}

const SLIDE_FILES: &[(&str, &str)] = &[
    ("01-hero.yaml", include_str!("../ui/slides/01-hero.yaml")),
    ("02-section.yaml", include_str!("../ui/slides/02-section.yaml")),
    ("03-stats.yaml", include_str!("../ui/slides/03-stats.yaml")),
    ("04-bullets.yaml", include_str!("../ui/slides/04-bullets.yaml")),
    ("05-agenda.yaml", include_str!("../ui/slides/05-agenda.yaml")),
    ("06-cards.yaml", include_str!("../ui/slides/06-cards.yaml")),
    ("07-compare.yaml", include_str!("../ui/slides/07-compare.yaml")),
    ("08-quote.yaml", include_str!("../ui/slides/08-quote.yaml")),
    ("09-chart.yaml", include_str!("../ui/slides/09-chart.yaml")),
    ("10-timeline.yaml", include_str!("../ui/slides/10-timeline.yaml")),
    ("11-image.yaml", include_str!("../ui/slides/11-image.yaml")),
    ("12-table.yaml", include_str!("../ui/slides/12-table.yaml")),
    ("13-matrix.yaml", include_str!("../ui/slides/13-matrix.yaml")),
    ("14-decision.yaml", include_str!("../ui/slides/14-decision.yaml")),
    ("15-assign.yaml", include_str!("../ui/slides/15-assign.yaml")),
    ("16-poll.yaml", include_str!("../ui/slides/16-poll.yaml")),
    ("17-rank.yaml", include_str!("../ui/slides/17-rank.yaml")),
    ("18-checklist.yaml", include_str!("../ui/slides/18-checklist.yaml")),
    ("19-note.yaml", include_str!("../ui/slides/19-note.yaml")),
    ("20-summary.yaml", include_str!("../ui/slides/20-summary.yaml")),
    ("21-quiz.yaml", include_str!("../ui/slides/21-quiz.yaml")),
    ("22-classify.yaml", include_str!("../ui/slides/22-classify.yaml")),
    ("23-reveal.yaml", include_str!("../ui/slides/23-reveal.yaml")),
    ("24-flashcards.yaml", include_str!("../ui/slides/24-flashcards.yaml")),
];

pub static SLIDE_KINDS: LazyLock<Vec<SlideKind>> = LazyLock::new(|| {
    let mut keys = HashSet::new();
    SLIDE_FILES
        .iter()
        .map(|(name, src)| {
            let k: SlideKind = serde_yaml_ng::from_str(src).unwrap_or_else(|e| panic!("ui/slides/{name}: {e}"));
            assert!(keys.insert(k.key.clone()), "duplicate slide kind {}", k.key);
            for f in &k.fields {
                assert!(!COMMON_KEYS.contains(&f.key.as_str()), "ui/slides/{name}: field {} clashes with a common slide key", f.key);
            }
            k
        })
        .collect()
});

pub fn slide_kind(key: &str) -> Option<&'static SlideKind> {
    SLIDE_KINDS.iter().find(|k| k.key == key)
}

pub fn kind_keys() -> Vec<&'static str> {
    SLIDE_KINDS.iter().map(|k| k.key.as_str()).collect()
}

// ───────────────────────────── Theme presets ────────────────────────────────

pub struct Preset {
    pub key: &'static str,
    pub label: &'static str,
    pub description: &'static str,
    /// primary, accent, background, surface, ink, muted, good, warn, bad
    pub colors: [&'static str; 9],
    pub heading_font: &'static str,
    pub body_font: &'static str,
    pub motion: &'static str,
}

pub const PRESETS: &[Preset] = &[
    Preset {
        key: "executive",
        label: "Điều hành",
        description: "Trang trọng cho họp lãnh đạo: nền sáng, xanh đậm, nhấn vàng đồng, hiệu ứng tiết chế.",
        colors: ["#12305a", "#b0812b", "#f4f6f9", "#ffffff", "#0f1b2d", "#5a6577", "#1d7a4e", "#b26b00", "#b42318"],
        heading_font: "Be Vietnam Pro",
        body_font: "Be Vietnam Pro",
        motion: "subtle",
    },
    Preset {
        key: "academic",
        label: "Học thuật",
        description: "Bài giảng, hội thảo: tiêu đề có chân, nền giấy ấm, xanh dương và xanh ngọc.",
        colors: ["#1e3a8a", "#0e7490", "#fbfaf7", "#ffffff", "#1c1917", "#57534e", "#15803d", "#b45309", "#b91c1c"],
        heading_font: "Source Serif 4",
        body_font: "Inter",
        motion: "subtle",
    },
    Preset {
        key: "playful",
        label: "Sinh động",
        description: "Lớp học vui: màu tươi, bo tròn, pháo giấy khi trả lời đúng.",
        colors: ["#6c4cff", "#ff6b6b", "#fff7ec", "#ffffff", "#2a2340", "#6d6585", "#12a36f", "#d97706", "#e5484d"],
        heading_font: "Nunito",
        body_font: "Nunito",
        motion: "lively",
    },
    Preset {
        key: "midnight",
        label: "Nền tối",
        description: "Phòng tối, màn chiếu lớn: nền xanh đen, chữ sáng, nhấn vàng.",
        colors: ["#7aa2ff", "#f2c14e", "#0b1220", "#141d30", "#e8edf5", "#9aa6b8", "#4ade80", "#fbbf24", "#f87171"],
        heading_font: "Inter",
        body_font: "Inter",
        motion: "subtle",
    },
];

pub fn preset(key: &str) -> Option<&'static Preset> {
    PRESETS.iter().find(|p| p.key == key)
}

pub const COLOR_KEYS: [&str; 9] = ["primary", "accent", "background", "surface", "ink", "muted", "good", "warn", "bad"];

impl DeckTheme {
    fn color_slot(&mut self, key: &str) -> &mut String {
        match key {
            "primary" => &mut self.primary,
            "accent" => &mut self.accent,
            "background" => &mut self.background,
            "surface" => &mut self.surface,
            "ink" => &mut self.ink,
            "muted" => &mut self.muted,
            "good" => &mut self.good,
            "warn" => &mut self.warn,
            _ => &mut self.bad,
        }
    }

    fn color(&self, key: &str) -> &str {
        match key {
            "primary" => &self.primary,
            "accent" => &self.accent,
            "background" => &self.background,
            "surface" => &self.surface,
            "ink" => &self.ink,
            "muted" => &self.muted,
            "good" => &self.good,
            "warn" => &self.warn,
            _ => &self.bad,
        }
    }

    /// Every value filled in from the preset (what the player uses).
    pub fn resolved(&self) -> Value {
        let p = preset(&self.preset).unwrap_or(&PRESETS[0]);
        let mut colors = Map::new();
        for (i, k) in COLOR_KEYS.iter().enumerate() {
            let own = self.color(k);
            colors.insert(k.to_string(), json!(if own.is_empty() { p.colors[i] } else { own }));
        }
        json!({
            "preset": p.key,
            "colors": colors,
            "heading_font": if self.heading_font.is_empty() { p.heading_font } else { &self.heading_font },
            "body_font": if self.body_font.is_empty() { p.body_font } else { &self.body_font },
            "size": if self.size.is_empty() { "m" } else { &self.size },
            "motion": if self.motion.is_empty() { p.motion } else { &self.motion },
            "logo": self.logo,
            "page_numbers": self.page_numbers,
        })
    }
}

fn hex_rgb(c: &str) -> Option<(f64, f64, f64)> {
    let h = c.strip_prefix('#')?;
    let h = match h.len() {
        3 => h.chars().flat_map(|c| [c, c]).collect::<String>(),
        6 => h.to_string(),
        _ => return None,
    };
    let n = u32::from_str_radix(&h, 16).ok()?;
    Some((((n >> 16) & 255) as f64, ((n >> 8) & 255) as f64, (n & 255) as f64))
}

fn luminance(c: &str) -> Option<f64> {
    let (r, g, b) = hex_rgb(c)?;
    let ch = |v: f64| {
        let v = v / 255.0;
        if v <= 0.03928 { v / 12.92 } else { ((v + 0.055) / 1.055).powf(2.4) }
    };
    Some(0.2126 * ch(r) + 0.7152 * ch(g) + 0.0722 * ch(b))
}

/// WCAG contrast ratio of two hex colours (1..21).
pub fn contrast(a: &str, b: &str) -> Option<f64> {
    let (la, lb) = (luminance(a)?, luminance(b)?);
    let (hi, lo) = if la > lb { (la, lb) } else { (lb, la) };
    Some((hi + 0.05) / (lo + 0.05))
}

fn clean_hex(v: &str, what: &str) -> Result<String, String> {
    let v = v.trim().to_ascii_lowercase();
    if v.is_empty() {
        return Ok(v);
    }
    if crate::theme::is_hex_color(&v) {
        return Ok(v);
    }
    Err(format!("theme.{what}: cần mã màu hex (#12305a), nhận '{v}'"))
}

fn pick(v: &str, allowed: &[&str], what: &str) -> Result<String, String> {
    if allowed.contains(&v) { Ok(v.to_string()) } else { Err(format!("{what}: '{v}' không hợp lệ. Có: {}", allowed.iter().filter(|a| !a.is_empty()).cloned().collect::<Vec<_>>().join(", "))) }
}

/// Validates and normalizes a theme, refusing text colours that are hard to read.
pub fn validate_theme(t: &DeckTheme) -> Result<DeckTheme, String> {
    let mut t = t.clone();
    if t.preset.is_empty() {
        t.preset = "executive".into();
    }
    if preset(&t.preset).is_none() {
        return Err(format!("theme.preset: '{}' không có. Có: {}", t.preset, PRESETS.iter().map(|p| p.key).collect::<Vec<_>>().join(", ")));
    }
    for k in COLOR_KEYS {
        let v = clean_hex(t.color(k), k)?;
        *t.color_slot(k) = v;
    }
    for (f, what) in [(&mut t.heading_font, "heading_font"), (&mut t.body_font, "body_font")] {
        if !f.is_empty() {
            let found = DECK_FONTS.iter().find(|x| x.eq_ignore_ascii_case(f.trim())).ok_or_else(|| format!("theme.{what}: font '{f}' không có. Có: {}", DECK_FONTS.join(", ")))?;
            *f = found.to_string();
        }
    }
    if t.size.is_empty() {
        t.size = "m".into();
    }
    t.size = pick(&t.size, SIZES, "theme.size")?;
    if !t.motion.is_empty() {
        t.motion = pick(&t.motion, MOTIONS, "theme.motion")?;
    }
    t.logo = t.logo.trim().to_string();
    if !t.logo.is_empty() && !crate::blocks::is_asset_ref(&t.logo) && !t.logo.starts_with("https://") {
        return Err("theme.logo: phải là id ảnh đã tải lên hoặc link https://".into());
    }
    let r = t.resolved();
    let c = |k: &str| r["colors"][k].as_str().unwrap_or("").to_string();
    let checks = [("ink", "background", 4.5), ("ink", "surface", 4.5), ("muted", "surface", 3.0), ("primary", "surface", 3.0)];
    for (fg, bg, min) in checks {
        if let Some(ratio) = contrast(&c(fg), &c(bg)) {
            if ratio < min {
                return Err(format!("theme: màu {fg} ({}) trên {bg} ({}) khó đọc (tương phản {ratio:.1}, cần ≥ {min}). Chọn màu đậm/nhạt hơn.", c(fg), c(bg)));
            }
        }
    }
    Ok(t)
}

// ───────────────────────────── Validation ───────────────────────────────────

fn id_ok(id: &str) -> bool {
    !id.is_empty() && id.len() <= 40 && id.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
}

fn clean_id(raw: &str) -> String {
    let s: String = crate::util::slugify(raw).chars().map(|c| if c == '-' { '_' } else { c }).collect();
    s.chars().take(40).collect()
}

fn text_field(v: &str, max: usize, what: &str) -> Result<String, String> {
    let v = v.trim().to_string();
    if v.chars().count() > max {
        return Err(format!("{what}: dài quá {max} ký tự"));
    }
    Ok(v)
}

/// Validates one slide in place: common fields, then the kind's fields
/// (defaults filled, unknown fields refused). Returns asset ids it uses.
pub fn validate_slide(s: &mut Slide, at: &str) -> Result<Vec<String>, String> {
    let kind = slide_kind(&s.kind).ok_or_else(|| format!("{at}: kind '{}' không có. Có: {}", s.kind, kind_keys().join(", ")))?;
    s.title = text_field(&s.title, 300, &format!("{at}.title"))?;
    s.eyebrow = text_field(&s.eyebrow, 200, &format!("{at}.eyebrow"))?;
    s.source = text_field(&s.source, 400, &format!("{at}.source"))?;
    if s.notes.chars().count() > 6000 {
        return Err(format!("{at}.notes: dài quá 6000 ký tự"));
    }
    if !s.accent.is_empty() {
        s.accent = pick(&s.accent, TONES, &format!("{at}.accent"))?;
    }
    s.bg = pick(&s.bg, BACKGROUNDS, &format!("{at}.bg"))?;
    let mut assets = Vec::new();
    s.bg_image = s.bg_image.trim().to_string();
    if !s.bg_image.is_empty() {
        if crate::blocks::is_asset_ref(&s.bg_image) {
            assets.push(s.bg_image.clone());
        } else if !s.bg_image.starts_with("https://") {
            return Err(format!("{at}.bg_image: phải là id ảnh hoặc https://"));
        }
    }
    s.minutes = s.minutes.clamp(0, 600);
    let clean = validate_object(&kind.fields, &s.props, at, &mut assets)?;
    s.props = clean.as_object().cloned().unwrap_or_default();
    Ok(assets)
}

fn validate_data(data: &mut BTreeMap<String, Vec<Map<String, Value>>>) -> Result<(), String> {
    if data.len() > MAX_DATASETS {
        return Err(format!("data: tối đa {MAX_DATASETS} bảng"));
    }
    for (name, rows) in data.iter_mut() {
        if !id_ok(name) {
            return Err(format!("data.{name}: tên bảng chỉ gồm chữ không dấu, số, _ và -"));
        }
        if rows.len() > MAX_ROWS {
            return Err(format!("data.{name}: tối đa {MAX_ROWS} dòng"));
        }
        for (i, row) in rows.iter_mut().enumerate() {
            if row.len() > MAX_ROW_KEYS {
                return Err(format!("data.{name}[{i}]: tối đa {MAX_ROW_KEYS} cột"));
            }
            for (k, v) in row.iter_mut() {
                if !id_ok(k) {
                    return Err(format!("data.{name}[{i}].{k}: tên cột chỉ gồm chữ không dấu, số, _ và -"));
                }
                match v {
                    Value::String(s) if s.chars().count() > MAX_CELL => return Err(format!("data.{name}[{i}].{k}: dài quá {MAX_CELL} ký tự")),
                    Value::String(_) | Value::Number(_) | Value::Bool(_) => {}
                    Value::Null => *v = Value::String(String::new()),
                    Value::Array(a) if a.iter().all(|x| x.is_string() || x.is_number()) => {
                        *v = Value::String(a.iter().map(|x| x.as_str().map(String::from).unwrap_or_else(|| x.to_string())).collect::<Vec<_>>().join(", "));
                    }
                    _ => return Err(format!("data.{name}[{i}].{k}: chỉ nhận chữ, số hoặc true/false")),
                }
            }
        }
    }
    Ok(())
}

/// Validates and normalizes a whole deck: ids filled in (from titles) and made
/// unique, segments checked, every slide checked against its kind, the theme
/// checked. Returns the clean deck and every asset id it uses.
pub fn validate_deck(deck: &Deck) -> Result<(Deck, Vec<String>), String> {
    let mut d = deck.clone();
    d.lmdeck = 1;
    d.meta.title = text_field(&d.meta.title, 200, "meta.title")?;
    if d.meta.title.is_empty() {
        d.meta.title = "Bài trình chiếu".into();
    }
    d.meta.subtitle = text_field(&d.meta.subtitle, 400, "meta.subtitle")?;
    d.meta.date = text_field(&d.meta.date, 60, "meta.date")?;
    d.meta.author = text_field(&d.meta.author, 200, "meta.author")?;
    d.meta.footer = text_field(&d.meta.footer, 300, "meta.footer")?;
    if d.meta.lang.is_empty() {
        d.meta.lang = "vi".into();
    }
    d.meta.lang = pick(&d.meta.lang, LANGS, "meta.lang")?;
    d.meta.duration_min = d.meta.duration_min.clamp(0, 1440);
    d.meta.start_time = d.meta.start_time.trim().to_string();
    if !d.meta.start_time.is_empty() && parse_hhmm(&d.meta.start_time).is_none() {
        return Err(format!("meta.start_time: cần dạng HH:MM, nhận '{}'", d.meta.start_time));
    }
    d.theme = validate_theme(&d.theme)?;
    if d.segments.len() > MAX_SEGMENTS {
        return Err(format!("segments: tối đa {MAX_SEGMENTS} phần"));
    }
    let mut seg_ids = HashSet::new();
    for (i, s) in d.segments.iter_mut().enumerate() {
        if s.id.is_empty() {
            s.id = format!("s{}", i + 1);
        }
        if !id_ok(&s.id) || !seg_ids.insert(s.id.clone()) {
            return Err(format!("segments[{i}].id '{}' không hợp lệ hoặc bị trùng", s.id));
        }
        s.name = text_field(&s.name, 160, &format!("segments[{i}].name"))?;
        s.outcome = text_field(&s.outcome, 300, &format!("segments[{i}].outcome"))?;
        s.min = s.min.clamp(0, 600);
    }
    validate_data(&mut d.data)?;
    if d.slides.len() > MAX_SLIDES {
        return Err(format!("slides: tối đa {MAX_SLIDES} slide"));
    }
    let mut assets = Vec::new();
    let mut ids = HashSet::new();
    for i in 0..d.slides.len() {
        let at = format!("slides[{i}]");
        let s = &mut d.slides[i];
        if s.id.is_empty() {
            let base = clean_id(if s.title.is_empty() { &s.kind } else { &s.title });
            let base: String = base.chars().take(30).collect();
            let base = if base.is_empty() { format!("s{}", i + 1) } else { base };
            let mut id = base.clone();
            let mut n = 2;
            while ids.contains(&id) {
                id = format!("{base}_{n}");
                n += 1;
            }
            s.id = id;
        }
        if !id_ok(&s.id) {
            return Err(format!("{at}.id '{}': chỉ gồm chữ không dấu, số, _ và - (tối đa 40)", s.id));
        }
        if !ids.insert(s.id.clone()) {
            return Err(format!("{at}.id '{}' bị trùng", s.id));
        }
        if !s.seg.is_empty() && !seg_ids.contains(&s.seg) {
            return Err(format!("{at}.seg '{}' không có trong segments", s.seg));
        }
        assets.extend(validate_slide(s, &format!("{at} ({})", s.id))?);
    }
    if crate::blocks::is_asset_ref(&d.theme.logo) {
        assets.push(d.theme.logo.clone());
    }
    assets.sort();
    assets.dedup();
    Ok((d, assets))
}

pub fn parse_hhmm(s: &str) -> Option<(u32, u32)> {
    let (h, m) = s.split_once(':')?;
    let (h, m) = (h.trim().parse::<u32>().ok()?, m.trim().parse::<u32>().ok()?);
    (h < 24 && m < 60).then_some((h, m))
}

/// Reads a deck from YAML or JSON text.
pub fn parse(text: &str) -> Result<Deck, String> {
    if text.len() > MAX_YAML {
        return Err(format!("deck quá lớn (tối đa {} KB)", MAX_YAML / 1000));
    }
    let t = text.trim_start();
    if t.starts_with('{') {
        return serde_json::from_str(t).map_err(|e| format!("JSON không đọc được: {e}"));
    }
    serde_yaml_ng::from_str(text).map_err(|e| format!("YAML không đọc được: {e}"))
}

/// A deck from a JSON value (an object, or a string holding YAML/JSON).
pub fn from_value(v: &Value) -> Result<Deck, String> {
    match v {
        Value::String(s) => parse(s),
        Value::Object(_) => serde_json::from_value(v.clone()).map_err(|e| format!("deck không hợp lệ: {e}")),
        _ => Err("deck phải là object hoặc chuỗi YAML".into()),
    }
}

/// The YAML file the site stores.
pub fn to_yaml(d: &Deck) -> String {
    let body = serde_yaml_ng::to_string(d).unwrap_or_default();
    format!("# Learnmedi interactive deck (lmdeck v1). Slide kinds: GET /api/v1/decks/kinds\n{body}")
}

/// A slide from a JSON value (object).
pub fn slide_from_value(v: &Value) -> Result<Slide, String> {
    serde_json::from_value(v.clone()).map_err(|e| format!("slide không hợp lệ: {e}"))
}

/// Applies a partial change to a slide: common keys and kind fields replace
/// the old ones (lists are replaced whole); `kind` may change.
pub fn patch_slide(s: &Slide, patch: &Value) -> Result<Slide, String> {
    let mut base = serde_json::to_value(s).map_err(|e| e.to_string())?;
    let obj = patch.as_object().ok_or("patch phải là object")?;
    let b = base.as_object_mut().ok_or("slide")?;
    for (k, v) in obj {
        if k == "id" {
            continue;
        }
        if v.is_null() {
            b.remove(k);
        } else {
            b.insert(k.clone(), v.clone());
        }
    }
    if obj.contains_key("kind") {
        // Keep only fields the new kind knows.
        if let Some(kind) = obj.get("kind").and_then(|k| k.as_str()).and_then(slide_kind) {
            b.retain(|k, _| COMMON_KEYS.contains(&k.as_str()) || kind.fields.iter().any(|f| &f.key == k));
        }
    }
    slide_from_value(&base)
}

// ───────────────────────────── Lint ─────────────────────────────────────────

#[derive(Clone, Debug, Serialize, PartialEq)]
pub struct Lint {
    /// error | warn | info
    pub level: &'static str,
    pub slide: String,
    pub message: String,
}

fn words(v: &Value) -> usize {
    match v {
        Value::String(s) => s.split_whitespace().count(),
        Value::Array(a) => a.iter().map(words).sum(),
        Value::Object(o) => o.iter().filter(|(k, _)| !matches!(k.as_str(), "hint" | "note" | "why" | "explain" | "a" | "back" | "body" | "detail")).map(|(_, v)| words(v)).sum(),
        _ => 0,
    }
}

fn str_of<'a>(s: &'a Slide, k: &str) -> &'a str {
    s.props.get(k).and_then(|v| v.as_str()).unwrap_or("")
}

/// Problems an author (or an assistant that cannot see the screen) should fix:
/// broken links between slides, tables pointing at missing data or columns,
/// slides with too many words, missing notes on decision slides.
pub fn lint(d: &Deck) -> Vec<Lint> {
    let mut out = Vec::new();
    let ids: HashSet<&str> = d.slides.iter().map(|s| s.id.as_str()).collect();
    let mut push = |level: &'static str, slide: &str, message: String| out.push(Lint { level, slide: slide.into(), message });
    if d.slides.is_empty() {
        push("error", "", "Deck chưa có slide nào.".into());
    }
    let dataset_keys = |name: &str| -> Option<HashSet<String>> { d.data.get(name).map(|rows| rows.iter().flat_map(|r| r.keys().cloned()).collect()) };
    for s in &d.slides {
        let sid = s.id.as_str();
        let w = words(&Value::Object(s.props.clone())) + s.title.split_whitespace().count();
        let limit = if s.kind == "decision" { WORDS_WARN * 3 / 2 } else { WORDS_WARN };
        if w > limit && !matches!(s.kind.as_str(), "table" | "matrix" | "quiz" | "flashcards" | "reveal" | "summary") {
            push("warn", sid, format!("Khoảng {w} chữ trên màn hình (nên ≤ {limit}). Chuyển bớt sang notes hoặc tách slide."));
        }
        if s.title.is_empty() && !matches!(s.kind.as_str(), "hero" | "image" | "quote") {
            push("info", sid, "Slide chưa có tiêu đề (title).".into());
        }
        // Links between slides.
        let mut gotos: Vec<String> = Vec::new();
        for key in ["table_slide"] {
            if !str_of(s, key).is_empty() {
                gotos.push(str_of(s, key).to_string());
            }
        }
        for list in ["items", "rows"] {
            if let Some(arr) = s.props.get(list).and_then(|v| v.as_array()) {
                for it in arr {
                    if let Some(g) = it.get("goto").and_then(|v| v.as_str()).filter(|g| !g.is_empty()) {
                        gotos.push(g.to_string());
                    }
                }
            }
        }
        for g in gotos {
            if !ids.contains(g.as_str()) {
                push("error", sid, format!("Liên kết tới slide '{g}' không tồn tại."));
            }
        }
        // Data references.
        let ds = str_of(s, "dataset");
        if matches!(s.kind.as_str(), "table" | "matrix") {
            match dataset_keys(ds) {
                None => push("error", sid, format!("Bảng dữ liệu '{ds}' không có trong data. Có: {}", d.data.keys().cloned().collect::<Vec<_>>().join(", "))),
                Some(keys) => {
                    let mut wanted: Vec<String> = Vec::new();
                    for list in ["columns", "detail"] {
                        if let Some(arr) = s.props.get(list).and_then(|v| v.as_array()) {
                            wanted.extend(arr.iter().filter_map(|c| c.get("key").and_then(|v| v.as_str()).map(String::from)));
                        }
                    }
                    for k in ["filter_key", "group_key", "label_key"] {
                        if !str_of(s, k).is_empty() {
                            wanted.push(str_of(s, k).to_string());
                        }
                    }
                    for k in wanted {
                        if !keys.contains(&k) {
                            push("error", sid, format!("Cột '{k}' không có trong data.{ds}."));
                        }
                    }
                    if s.kind == "table" {
                        if let Some(arr) = s.props.get("columns").and_then(|v| v.as_array()) {
                            for c in arr.iter().filter(|c| c.get("format").and_then(|v| v.as_str()) == Some("goto")) {
                                let key = c.get("key").and_then(|v| v.as_str()).unwrap_or("");
                                for row in d.data.get(ds).into_iter().flatten() {
                                    let target = row.get(key).and_then(|v| v.as_str()).unwrap_or("");
                                    let target = target.to_ascii_lowercase();
                                    if !target.is_empty() && !ids.contains(target.as_str()) {
                                        push("warn", sid, format!("Ô {key}='{target}' trỏ tới slide không có (id slide viết thường)."));
                                        break;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        if s.kind == "decision" {
            let rds = str_of(s, "refs_dataset");
            if !str_of(s, "refs").is_empty() {
                match d.data.get(rds) {
                    None => push("warn", sid, "Có refs nhưng refs_dataset chưa trỏ tới bảng data nào.".into()),
                    Some(rows) => {
                        for r in str_of(s, "refs").split(',').map(str::trim).filter(|r| !r.is_empty()) {
                            if !rows.iter().any(|row| row.get("id").and_then(|v| v.as_str()) == Some(r)) {
                                push("warn", sid, format!("refs '{r}' không có trong data.{rds} (cột id)."));
                            }
                        }
                    }
                }
            }
            if s.props.get("options").and_then(|v| v.as_array()).is_some_and(|a| a.len() < 2) {
                push("warn", sid, "Slide quyết định nên có ít nhất 2 phương án.".into());
            }
        }
        if s.kind == "classify" {
            let n = s.props.get("buckets").and_then(|v| v.as_array()).map_or(0, |a| a.len()) as i64;
            if let Some(items) = s.props.get("items").and_then(|v| v.as_array()) {
                if items.iter().any(|it| it.get("answer").and_then(|v| v.as_i64()).unwrap_or(0) >= n) {
                    push("error", sid, "Có mục có answer lớn hơn số nhóm.".into());
                }
            }
        }
        if s.kind == "quiz" {
            if let Some(qs) = s.props.get("questions").and_then(|v| v.as_array()) {
                for (i, q) in qs.iter().enumerate() {
                    let n = q.get("options").and_then(|v| v.as_array()).map_or(0, |a| a.len()) as i64;
                    if q.get("answer").and_then(|v| v.as_i64()).unwrap_or(0) >= n {
                        push("error", sid, format!("Câu {}: answer vượt số lựa chọn.", i + 1));
                    }
                }
            }
        }
        if s.kind == "timeline" && !str_of(s, "countdown_to").is_empty() && chrono::NaiveDate::parse_from_str(str_of(s, "countdown_to"), "%Y-%m-%d").is_err() {
            push("error", sid, "countdown_to cần dạng YYYY-MM-DD.".into());
        }
    }
    let planned: i64 = d.segments.iter().map(|s| s.min).sum();
    if d.meta.duration_min > 0 && planned > 0 && planned != d.meta.duration_min {
        out.push(Lint { level: "info", slide: String::new(), message: format!("Tổng phút các phần ({planned}) khác meta.duration_min ({}).", d.meta.duration_min) });
    }
    out
}

// ───────────────────────────── Schema export ────────────────────────────────

/// Everything an editor or an outside assistant needs to write a deck.
pub fn schema_json() -> Value {
    json!({
        "format": "lmdeck v1: {lmdeck: 1, meta, theme, segments[], data{name: [rows]}, slides[]}",
        "common_slide_fields": {
            "id": "chữ không dấu, số, _ - (tự sinh từ title nếu bỏ trống); dùng cho goto/table_slide",
            "kind": kind_keys(),
            "seg": "id của một segment", "eyebrow": "dòng nhỏ trên tiêu đề", "title": "tiêu đề slide",
            "notes": "ghi chú người trình bày (Markdown)", "source": "dòng nguồn dưới slide", "hidden": "true = bỏ qua khi trình chiếu",
            "accent": TONES, "bg": BACKGROUNDS, "bg_image": "id ảnh hoặc https://", "minutes": "số phút dự kiến cho slide",
        },
        "meta": {"title": "", "subtitle": "", "date": "", "author": "", "lang": LANGS, "duration_min": 0, "start_time": "HH:MM", "footer": ""},
        "segments": {"id": "s1", "name": "Tên phần", "min": 10, "outcome": "Kết quả cần có"},
        "data": "Bảng dùng chung: {\"compare\": [{\"id\": \"c1\", \"topic\": \"…\", \"status\": \"Chưa có\"}]}. Giá trị: chữ, số, true/false. Slide table/matrix đọc bằng dataset; decision.refs trỏ tới cột id.",
        "theme": theme_schema_json(),
        "kinds": SLIDE_KINDS.iter().map(kind_summary).collect::<Vec<_>>(),
    })
}

fn kind_summary(k: &SlideKind) -> Value {
    json!({"key": k.key, "label": k.label, "group": k.group, "interactive": k.interactive, "description": k.description,
        "fields": k.fields.iter().map(|f| format!("{}{}:{}", f.key, if f.required { "*" } else { "" }, kind_short(&f.kind))).collect::<Vec<_>>()})
}

pub fn kinds_json() -> Value {
    json!({"kinds": SLIDE_KINDS.iter().map(kind_summary).collect::<Vec<_>>()})
}

/// Full schema and example of one kind.
pub fn kind_schema_json(key: &str) -> Option<Value> {
    slide_kind(key).map(|k| json!({"key": k.key, "label": k.label, "group": k.group, "interactive": k.interactive, "description": k.description, "fields": fields_json(&k.fields), "example": k.example}))
}

pub fn theme_schema_json() -> Value {
    json!({
        "presets": PRESETS.iter().map(|p| json!({"key": p.key, "label": p.label, "description": p.description,
            "colors": COLOR_KEYS.iter().zip(p.colors).map(|(k, c)| (k.to_string(), json!(c))).collect::<Map<String, Value>>(),
            "heading_font": p.heading_font, "body_font": p.body_font, "motion": p.motion})).collect::<Vec<_>>(),
        "colors": COLOR_KEYS, "color_rule": "mã hex; chữ (ink) trên nền phải tương phản ≥ 4.5",
        "fonts": DECK_FONTS, "size": SIZES, "motion": MOTIONS, "logo": "id ảnh hoặc https://", "page_numbers": "bool",
    })
}

/// A short outline (id, kind, title) for lists and AI context.
pub fn outline(d: &Deck) -> Vec<Value> {
    d.slides.iter().enumerate().map(|(i, s)| json!({"n": i + 1, "id": s.id, "kind": s.kind, "title": s.title, "seg": s.seg, "hidden": s.hidden})).collect()
}

/// A new deck with a cover slide.
pub fn starter(title: &str) -> Deck {
    let title = if title.trim().is_empty() { "Bài trình chiếu mới" } else { title.trim() };
    let mut props = Map::new();
    props.insert("subtitle".into(), json!(""));
    Deck {
        lmdeck: 1,
        meta: DeckMeta { title: title.into(), lang: "vi".into(), ..Default::default() },
        theme: DeckTheme::default(),
        segments: vec![],
        data: BTreeMap::new(),
        slides: vec![Slide { id: "cover".into(), kind: "hero".into(), title: title.into(), props, ..Default::default() }],
    }
}

/// Any value as YAML (docs and examples).
pub fn value_to_yaml(v: &Value) -> String {
    serde_yaml_ng::to_string(v).unwrap_or_default()
}

/// Rough size of a Value in bytes (for result limits).
pub fn json_size(v: &Value) -> usize {
    serde_json::to_string(v).map(|s| s.len()).unwrap_or(usize::MAX)
}
