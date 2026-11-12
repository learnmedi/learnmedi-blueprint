//! The owner's private personality and the red envelope (bao lì xì) of the day.
//!
//! The personality is a short self-bio in Vietnamese that only the owner (and
//! the AI working for them) reads: it sets the tone of pages the AI writes and
//! of the site's 120 envelope quotes. It is never rendered on the public site.
//!
//! The worker precomputes one `AlmanacDay` per date (Vietnamese lunar date,
//! UTC+7) and tags it with `DAY_TYPES`. Each quote is written for one day type.
//! A visitor opens one envelope a day: a quote of the day's main type (or a
//! `normal` one), picked from (site, day, visitor) so a reload shows the same.
//! Quotes are neutral sayings: never in the owner's voice or under their name.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub const BIO_MAX_WORDS: usize = 150;
pub const BIO_MAX_CHARS: usize = 1200;
pub const QUOTE_MAX_CHARS: usize = 160;
/// Quotes per language, exactly.
pub const QUOTES_TOTAL: usize = 120;

/// Day types, from the strongest rule to the weakest. A day carries one or
/// more; its main type is the first of this list it has.
/// (key, Vietnamese label, what quotes of this type must say, minimum count)
pub const DAY_TYPES: &[(&str, &str, &str, usize)] = &[
    ("nguyet_ky", "Ngày Nguyệt Kỵ (mùng 5, 14, 23 âm lịch)", "Không nên đi xa, xuất hành hay làm việc lớn. Khuyên ở gần nhà, làm việc quen, cẩn thận. TUYỆT ĐỐI không gợi ý đi chơi, đi xa, ra ngoài, bắt đầu việc mới.", 10),
    ("tam_nuong", "Ngày Tam Nương (mùng 3, 7, 13, 18, 22, 27 âm lịch)", "Không hợp khởi sự, cưới hỏi, khai trương. Khuyên kiên nhẫn, chuẩn bị, để việc lớn sang hôm khác. Không gợi ý bắt đầu việc mới.", 10),
    ("hac_dao", "Ngày hắc đạo", "Ngày nên chậm lại: nghỉ ngơi, làm cho xong việc cũ, lắng nghe, cẩn trọng. Không gợi ý bắt đầu việc lớn.", 15),
    ("hoang_dao", "Ngày hoàng đạo", "Ngày thuận: khích lệ bắt đầu điều mới, gặp gỡ, học thêm, làm điều tốt.", 15),
    ("normal", "Ngày thường (dùng được mọi ngày)", "Một câu nhẹ nhàng hợp với tính cách chủ site, không nhắc ngày tốt xấu, không khuyên đi xa hay khởi sự.", 40),
];

pub fn is_day_type(key: &str) -> bool {
    DAY_TYPES.iter().any(|t| t.0 == key)
}

/// The main type of a day: the strongest one it has, `normal` if none.
pub fn main_type(types: &[String]) -> &'static str {
    DAY_TYPES.iter().find(|t| types.iter().any(|x| x == t.0)).map_or("normal", |t| t.0)
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Quote {
    pub text: String,
    /// One of `DAY_TYPES`.
    #[serde(rename = "type")]
    pub kind: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Personality {
    /// Self-bio in Vietnamese, at most `BIO_MAX_WORDS` words. Private.
    #[serde(default)]
    pub bio: String,
    /// The owner's switch, on by default; the envelope also needs quotes in the language.
    #[serde(default = "yes")]
    pub fortune_enabled: bool,
    /// `QUOTES_TOTAL` quotes per language code (`vi`, `en`…).
    #[serde(default)]
    pub quotes: BTreeMap<String, Vec<Quote>>,
    /// Fingerprint of `bio` when the quotes were saved (stale when it differs).
    #[serde(default)]
    pub quotes_bio_hash: String,
}

fn yes() -> bool {
    true
}

impl Default for Personality {
    fn default() -> Self {
        Personality { bio: String::new(), fortune_enabled: true, quotes: BTreeMap::new(), quotes_bio_hash: String::new() }
    }
}

impl Personality {
    /// The envelope shows only when switched on and quotes exist in `lang`.
    pub fn envelope_on(&self, lang: &str) -> bool {
        self.fortune_enabled && self.quotes.get(lang).is_some_and(|q| !q.is_empty())
    }
    pub fn quotes_stale(&self) -> bool {
        !self.quotes.is_empty() && self.quotes_bio_hash != bio_hash(&self.bio)
    }
}

pub fn bio_hash(bio: &str) -> String {
    format!("{:016x}", fnv(&[bio.trim()]))
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

pub fn validate_day(d: &AlmanacDay) -> Result<(), String> {
    let ok_date = d.date.len() == 10 && chrono::NaiveDate::parse_from_str(&d.date, "%Y-%m-%d").is_ok();
    if !ok_date || !(1..=30).contains(&d.lunar_day) || !(1..=12).contains(&d.lunar_month) {
        return Err(format!("{}: ngày không hợp lệ", d.date));
    }
    if d.types.is_empty() || d.types.iter().any(|t| !is_day_type(t)) {
        return Err(format!("{}: types phải thuộc {}", d.date, DAY_TYPES.iter().map(|t| t.0).collect::<Vec<_>>().join(", ")));
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

pub fn validate_bio(bio: &str) -> Result<String, String> {
    let bio = bio.trim();
    let words = bio.split_whitespace().count();
    if words > BIO_MAX_WORDS || bio.chars().count() > BIO_MAX_CHARS {
        return Err(format!("Tiểu sử tối đa {BIO_MAX_WORDS} chữ (hiện có {words})."));
    }
    Ok(bio.to_string())
}

fn words_lower(s: &str) -> Vec<String> {
    s.to_lowercase().split(|c: char| !c.is_alphanumeric()).filter(|w| !w.is_empty()).map(String::from).collect()
}

/// Checks a full set of quotes for one language: exactly `QUOTES_TOTAL`, every
/// type known and with its minimum, no links, and never in the owner's voice
/// or under their name (`owner_names`: the site name and the owner's name).
pub fn validate_quotes(quotes: &[Quote], owner_names: &[&str]) -> Result<Vec<Quote>, String> {
    let mut out: Vec<Quote> = Vec::new();
    let names: Vec<Vec<String>> = owner_names.iter().map(|n| words_lower(n)).filter(|w| !w.is_empty()).collect();
    for (i, q) in quotes.iter().enumerate() {
        let text = q.text.split_whitespace().collect::<Vec<_>>().join(" ");
        let n = i + 1;
        if text.is_empty() {
            return Err(format!("Câu {n} trống."));
        }
        if !is_day_type(&q.kind) {
            return Err(format!("Câu {n}: type '{}' không có; dùng {}.", q.kind, DAY_TYPES.iter().map(|t| t.0).collect::<Vec<_>>().join(", ")));
        }
        if text.chars().count() > QUOTE_MAX_CHARS {
            return Err(format!("Câu {n} dài quá {QUOTE_MAX_CHARS} ký tự."));
        }
        let low = text.to_lowercase();
        if low.contains("http") || low.contains("www.") || low.contains("://") {
            return Err(format!("Câu {n} không được chứa đường link."));
        }
        let w = words_lower(&text);
        if ["tôi", "i", "me", "my"].iter().any(|p| w.iter().any(|x| x == p)) {
            return Err(format!("Câu {n} nói ở ngôi thứ nhất (\"tôi\"/\"I\"). Câu chúc là lời chung trong bao lì xì, không phải lời của chủ site."));
        }
        // The owner's name: all words of a name, in a row.
        if names.iter().any(|name| w.windows(name.len()).any(|win| win == name.as_slice())) {
            return Err(format!("Câu {n} nhắc tên chủ site. Câu chúc không gắn với người nào."));
        }
        if out.iter().any(|o| o.text == text) {
            return Err(format!("Câu {n} bị trùng."));
        }
        out.push(Quote { text, kind: q.kind.clone() });
    }
    if out.len() != QUOTES_TOTAL {
        return Err(format!("Cần đúng {QUOTES_TOTAL} câu (đang có {}).", out.len()));
    }
    for (key, label, _, min) in DAY_TYPES {
        let c = out.iter().filter(|q| q.kind == *key).count();
        if c < *min {
            return Err(format!("Loại {key} ({label}) cần ít nhất {min} câu (đang có {c})."));
        }
    }
    Ok(out)
}

/// FNV-1a: stable across builds and machines (std's hasher is not).
fn fnv(parts: &[&str]) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for p in parts {
        for b in p.bytes().chain(std::iter::once(0)) {
            h ^= b as u64;
            h = h.wrapping_mul(0x0100_0000_01b3);
        }
    }
    h
}

/// The visitor's quote for a day: one of the day's main type, else a `normal` one.
pub fn pick<'a>(quotes: &'a [Quote], day_types: &[String], site_id: &str, date: &str, visitor: &str) -> Option<&'a Quote> {
    let main = main_type(day_types);
    let of = |k: &str| quotes.iter().filter(|q| q.kind == k).collect::<Vec<_>>();
    let mut pool = of(main);
    if pool.is_empty() {
        pool = of("normal");
    }
    if pool.is_empty() {
        return None;
    }
    Some(pool[(fnv(&[site_id, date, visitor]) % pool.len() as u64) as usize])
}

/// The brief another assistant (or the site's AI) follows to write the 120 quotes.
pub fn brief(bio: &str, lang: &str) -> serde_json::Value {
    let types: Vec<serde_json::Value> = DAY_TYPES.iter().map(|(k, l, rule, min)| serde_json::json!({"type": k, "label": l, "rule": rule, "min": min})).collect();
    serde_json::json!({
        "task": format!("Viết đúng {QUOTES_TOTAL} câu chúc ngắn bằng ngôn ngữ '{lang}' cho bao lì xì mỗi ngày của site. Mỗi câu thuộc đúng một loại ngày (type) dưới đây, đủ số tối thiểu của từng loại."),
        "personality": bio,
        "day_types": types,
        "rules": [
            "Tính cách chỉ quyết định giọng và chủ đề (vd. tử tế, phục vụ cộng đồng; vui vẻ, yêu việc dạy; kỹ lưỡng, có kế hoạch). Câu chúc là lời chung trong bao lì xì, KHÔNG phải lời của chủ site.",
            "Không dùng ngôi thứ nhất (tôi, I, my), không nhắc tên chủ site, không trích dẫn ai.",
            format!("Mỗi câu tối đa {QUOTE_MAX_CHARS} ký tự, nhẹ nhàng, không link, không lời khuyên y tế, tài chính hay cờ bạc."),
            "Không trùng câu.",
        ],
        "send_back": {
            "op": "set_fortune_quotes",
            "args": {"lang": lang, "quotes": [{"text": "…", "type": "normal"}]},
            "how": "POST /api/v1/ops với {\"op\": \"set_fortune_quotes\", \"args\": {…}} (cùng token). Gửi cả 120 câu một lần; sai một câu thì cả lần gửi bị từ chối và báo câu nào sai.",
        },
    })
}
