//! The daily tear-off calendar (lịch bloc) in the corner of every public page.
//!
//! The owner picks one deck (`DECKS`): Truyện Kiều, Kinh Dịch or Thích Nhất Hạnh.
//! The worker keeps the decks and, every minute, sends the site one `FortuneSet`:
//! one item of every deck. A visitor who tears today's page gets the item of the
//! site's deck in the set of that minute; it is theirs for the day (the browser
//! keeps it, and a signed-in visitor gets the same one on every device).
//!
//! The worker also precomputes one `AlmanacDay` per date (Vietnamese lunar date,
//! UTC+7): the calendar shows the solar and the lunar date.

use serde::{Deserialize, Serialize};

/// (key, label, what the owner sees in Settings).
pub const DECKS: &[(&str, &str, &str)] = &[
    ("kieu", "Truyện Kiều", "Một cặp lục bát của Nguyễn Du mỗi ngày."),
    ("kinh_dich", "Kinh Dịch", "Một lời quẻ hoặc một hào, kèm âm Hán Việt và lời giải."),
    ("thich_nhat_hanh", "Thích Nhất Hạnh", "Một câu của thiền sư (tiếng Anh)."),
];
pub const DEFAULT_DECK: &str = "kieu";

pub const TEXT_MAX_CHARS: usize = 800;
pub const ORIGINAL_MAX_CHARS: usize = 1000;

pub fn is_deck(key: &str) -> bool {
    DECKS.iter().any(|d| d.0 == key)
}

pub fn deck_label(key: &str) -> Option<&'static str> {
    DECKS.iter().find(|d| d.0 == key).map(|d| d.1)
}

/// The owner's settings (`SiteContent.fortune`).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Fortune {
    #[serde(default = "yes")]
    pub enabled: bool,
    /// One of `DECKS`.
    #[serde(default = "default_deck")]
    pub deck: String,
}

fn yes() -> bool {
    true
}

fn default_deck() -> String {
    DEFAULT_DECK.to_string()
}

impl Default for Fortune {
    fn default() -> Self {
        Fortune { enabled: true, deck: default_deck() }
    }
}

/// One item of a deck, as the worker sends it and the visitor sees it.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DeckItem {
    /// One of `DECKS`.
    pub deck: String,
    /// Unique inside its deck (`kieu-0001`, `kd-01-3`…).
    pub id: String,
    /// What the back of the page shows.
    pub text: String,
    /// The original (Chinese and Sino-Vietnamese for Kinh Dịch), if any.
    #[serde(default)]
    pub original: String,
    pub author: String,
    /// "Truyện Kiều", "Kinh Dịch – quẻ Thuần Càn, hào 1", a book title…
    #[serde(default)]
    pub work: String,
    /// A clickable source (https only), if any.
    #[serde(default)]
    pub source_url: String,
    /// `vi` | `en`.
    #[serde(default)]
    pub lang: String,
}

/// What the worker sends every minute: one item of every deck it has.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FortuneSet {
    /// `YYYY-MM-DDTHH:MM` (Vietnam time).
    pub minute: String,
    pub items: Vec<DeckItem>,
}

impl FortuneSet {
    /// The item of `deck`, else the first one (a deck the worker does not have yet).
    pub fn item_for(&self, deck: &str) -> Option<&DeckItem> {
        self.items.iter().find(|i| i.deck == deck).or_else(|| self.items.first())
    }
}

pub fn validate_set(set: &FortuneSet) -> Result<(), String> {
    let ok_minute = set.minute.len() == 16 && chrono::NaiveDateTime::parse_from_str(&set.minute, "%Y-%m-%dT%H:%M").is_ok();
    if !ok_minute {
        return Err(format!("minute '{}' phải là YYYY-MM-DDTHH:MM", set.minute));
    }
    if set.items.is_empty() {
        return Err("items trống".into());
    }
    let mut seen: Vec<&str> = Vec::new();
    for it in &set.items {
        if !is_deck(&it.deck) {
            return Err(format!("deck '{}' không có; dùng {}", it.deck, DECKS.iter().map(|d| d.0).collect::<Vec<_>>().join(", ")));
        }
        if seen.contains(&it.deck.as_str()) {
            return Err(format!("deck '{}' bị lặp", it.deck));
        }
        seen.push(&it.deck);
        let n = |s: &str| s.chars().count();
        if it.id.trim().is_empty() || n(&it.id) > 40 {
            return Err(format!("{}: id trống hoặc quá dài", it.deck));
        }
        if it.text.trim().is_empty() || n(&it.text) > TEXT_MAX_CHARS {
            return Err(format!("{} {}: text trống hoặc dài quá {TEXT_MAX_CHARS} ký tự", it.deck, it.id));
        }
        if n(&it.original) > ORIGINAL_MAX_CHARS || it.author.trim().is_empty() || n(&it.author) > 80 || n(&it.work) > 160 {
            return Err(format!("{} {}: author/work/original không hợp lệ", it.deck, it.id));
        }
        if !it.source_url.is_empty() && (!it.source_url.starts_with("https://") || n(&it.source_url) > 400 || it.source_url.contains(['"', '<', '>', ' '])) {
            return Err(format!("{} {}: source_url phải là link https://", it.deck, it.id));
        }
        if !matches!(it.lang.as_str(), "" | "vi" | "en") {
            return Err(format!("{} {}: lang phải là vi hoặc en", it.deck, it.id));
        }
    }
    Ok(())
}

/// Day types the worker tags each date with (folk rules on the lunar day and the 12 stars).
/// The calendar shows only the dates; the types are kept for the worker's rows.
pub const DAY_TYPES: &[&str] = &["nguyet_ky", "tam_nuong", "hac_dao", "hoang_dao", "normal"];

pub fn is_day_type(key: &str) -> bool {
    DAY_TYPES.contains(&key)
}

/// One precomputed date (computed by the worker, stored by the site).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AlmanacDay {
    /// `YYYY-MM-DD` (Vietnam).
    #[serde(rename = "_id")]
    pub date: String,
    pub lunar_day: u32,
    pub lunar_month: u32,
    pub lunar_year: i32,
    #[serde(default)]
    pub leap: bool,
    /// "Bính Ngọ"
    pub year_can_chi: String,
    /// "Giáp Tý"
    pub day_can_chi: String,
    /// The day's star of the 12 (Thanh Long…), and whether it is hoàng đạo.
    pub star: String,
    pub star_good: bool,
    /// The day's Trực (Kiến, Trừ…).
    #[serde(default)]
    pub truc: String,
    /// Lucky hours, e.g. "Tý (23–1h)".
    #[serde(default)]
    pub good_hours: Vec<String>,
    /// Keys of `DAY_TYPES`.
    pub types: Vec<String>,
}

impl AlmanacDay {
    /// "7/10", "7/10 nhuận" (leap month).
    pub fn lunar_short(&self, en: bool) -> String {
        let leap = if self.leap { if en { " (leap)" } else { " nhuận" } } else { "" };
        format!("{}/{}{leap}", self.lunar_day, self.lunar_month)
    }
}

pub fn validate_day(d: &AlmanacDay) -> Result<(), String> {
    let ok_date = d.date.len() == 10 && chrono::NaiveDate::parse_from_str(&d.date, "%Y-%m-%d").is_ok();
    if !ok_date || !(1..=30).contains(&d.lunar_day) || !(1..=12).contains(&d.lunar_month) {
        return Err(format!("{}: ngày không hợp lệ", d.date));
    }
    if d.types.is_empty() || d.types.iter().any(|t| !is_day_type(t)) {
        return Err(format!("{}: types phải thuộc {}", d.date, DAY_TYPES.join(", ")));
    }
    let short = |s: &str| s.chars().count() <= 40;
    if !short(&d.year_can_chi) || !short(&d.day_can_chi) || !short(&d.star) || !short(&d.truc) || d.good_hours.len() > 12 || d.good_hours.iter().any(|h| !short(h)) {
        return Err(format!("{}: chuỗi quá dài", d.date));
    }
    Ok(())
}

/// Folk rules on the Vietnamese lunar day (the 12 stars come from the almanac).
pub fn lunar_day_types(lunar_day: u32, star_good: bool) -> Vec<String> {
    let mut t = Vec::new();
    if [5, 14, 23].contains(&lunar_day) {
        t.push("nguyet_ky".to_string());
    }
    if [3, 7, 13, 18, 22, 27].contains(&lunar_day) {
        t.push("tam_nuong".to_string());
    }
    t.push(if star_good { "hoang_dao" } else { "hac_dao" }.to_string());
    t
}
