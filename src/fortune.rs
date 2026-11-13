//! The owner's private personality and the fortune sticks (xin xăm) of the day.
//!
//! The personality is a short self-bio in Vietnamese that only the owner (and
//! the AI working for them) reads: it sets the tone of pages the AI writes and
//! the themes of the site's 120 fortune slips. It is never rendered on the public site.
//!
//! The worker precomputes one `AlmanacDay` per date (Vietnamese lunar date,
//! UTC+7) and tags it with `DAY_TYPES`. Each slip (quẻ) is written for one day
//! type and carries a rank (`RANKS`). A visitor draws one stick a day: a slip of
//! the day's main type (or a `normal` one), picked from (site, day, visitor) so a
//! reload shows the same. Visitors see the slip's text only, never a rank or a
//! number. Slips foretell; they are never in the owner's voice or under their name.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub const BIO_MAX_WORDS: usize = 150;
pub const BIO_MAX_CHARS: usize = 1200;
pub const QUOTE_MAX_CHARS: usize = 220;
/// Quotes per language, exactly.
pub const QUOTES_TOTAL: usize = 120;

/// Day types, from the strongest rule to the weakest. A day carries one or
/// more; its main type is the first of this list it has.
/// (key, Vietnamese label, what slips of this type must foretell, minimum count)
pub const DAY_TYPES: &[(&str, &str, &str, usize)] = &[
    ("nguyet_ky", "Ngày Nguyệt Kỵ (mùng 5, 14, 23 âm lịch)", "Đường xa chưa thuận, việc gần có lối. Hạng ha hoặc trung. Không báo lợi cho đi xa, khởi sự.", 10),
    ("tam_nuong", "Ngày Tam Nương (mùng 3, 7, 13, 18, 22, 27 âm lịch)", "Việc mới chưa tới lúc, chờ thêm sẽ được. Hạng ha hoặc trung. Không báo lợi cho khởi sự.", 10),
    ("hac_dao", "Ngày hắc đạo", "Nên chậm lại, việc cũ thành, việc mới còn chậm. Hạng trung hoặc ha.", 15),
    ("hoang_dao", "Ngày hoàng đạo", "Vận đang lên, tin mong sắp tới, có người nâng đỡ. Hạng thuong hoặc trung.", 15),
    ("normal", "Ngày thường", "Chờ đợi, lời hẹn, chuyện cũ trở lại, điều còn giấu. Không nhắc ngày tốt xấu, không báo lợi cho đi xa hay khởi sự. Đủ ba hạng.", 40),
];

/// Ranks of a slip, best first: (key, Vietnamese label, English label).
pub const RANKS: &[(&str, &str, &str)] = &[
    ("thuong", "Thượng", "Great fortune"),
    ("trung", "Trung", "Fair fortune"),
    ("ha", "Hạ", "Modest fortune"),
];

pub fn rank_label(key: &str, en: bool) -> Option<&'static str> {
    RANKS.iter().find(|r| r.0 == key).map(|r| if en { r.2 } else { r.1 })
}

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
    /// One of `RANKS`; empty on sets saved before fortune sticks.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub rank: String,
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
    /// Fingerprint of `bio` and `FORMAT` when the quotes were saved (stale when it differs).
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

/// Bumped when slips change shape, so older sets read as stale and the owner is asked to rewrite them.
const FORMAT: &str = "xam-2";

pub fn bio_hash(bio: &str) -> String {
    format!("{:016x}", fnv(&[bio.trim(), FORMAT]))
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
        if text.contains(';') {
            return Err(format!("Câu {n} có dấu chấm phẩy. Viết thành một hoặc hai câu trọn vẹn, không ghép hai vế."));
        }
        if is_labelled(&low) {
            return Err(format!("Câu {n} có số quẻ hoặc nhãn trong text. \"text\" chỉ là lời quẻ; muốn đánh số thì dùng field \"n\" riêng."));
        }
        let w = words_lower(&text);
        if ["tôi", "i", "me", "my"].iter().any(|p| w.iter().any(|x| x == p)) {
            return Err(format!("Câu {n} nói ở ngôi thứ nhất (\"tôi\"/\"I\"). Quẻ xăm là lời của quẻ, không phải lời của chủ site."));
        }
        // The owner's name: all words of a name, in a row.
        if names.iter().any(|name| w.windows(name.len()).any(|win| win == name.as_slice())) {
            return Err(format!("Câu {n} nhắc tên chủ site. Quẻ xăm không gắn với người nào."));
        }
        if rank_label(&q.rank, false).is_none() {
            return Err(format!("Câu {n}: rank '{}' không có; dùng {}.", q.rank, RANKS.iter().map(|r| r.0).collect::<Vec<_>>().join(", ")));
        }
        if out.iter().any(|o| o.text == text) {
            return Err(format!("Câu {n} bị trùng."));
        }
        out.push(Quote { text, kind: q.kind.clone(), rank: q.rank.clone() });
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
    for (key, label, _) in RANKS {
        if !out.iter().any(|q| q.rank == *key) {
            return Err(format!("Cần ít nhất một quẻ hạng {key} ({label})."));
        }
    }
    Ok(out)
}

/// A label the site shows itself, written into the text: "Quẻ số 12", "Quẻ 12", "Quẻ Thượng:", "Stick no. 3"…
fn is_labelled(low: &str) -> bool {
    let w: Vec<&str> = low.split(|c: char| !c.is_alphanumeric()).filter(|x| !x.is_empty()).collect();
    let num = |x: &str| x.chars().all(|c| c.is_ascii_digit());
    w.windows(2).any(|p| (p[0] == "quẻ" || p[0] == "xăm" || p[0] == "stick" || p[0] == "slip") && (p[1] == "số" || p[1] == "no" || num(p[1])))
        || matches!(w.as_slice(), ["quẻ", "thượng" | "trung" | "hạ", ..])
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

/// The brief another assistant (or the site's AI) follows to write the 120 slips.
pub fn brief(bio: &str, lang: &str) -> serde_json::Value {
    let types: Vec<serde_json::Value> = DAY_TYPES.iter().map(|(k, l, rule, min)| serde_json::json!({"type": k, "label": l, "rule": rule, "min": min})).collect();
    let ranks: Vec<serde_json::Value> = RANKS.iter().map(|(k, l, _)| serde_json::json!({"rank": k, "label": l})).collect();
    serde_json::json!({
        "task": format!("Viết đúng {QUOTES_TOTAL} quẻ xăm bằng ngôn ngữ '{lang}'. Mỗi quẻ: {{\"text\", \"type\", \"rank\"}}, thêm \"n\" nếu muốn đánh số."),
        "how_to_write": [
            "Lời quẻ gợi mở, có nét bí ẩn, để người rút tự nhận ra chuyện của mình. Báo chờ đợi, chuyển vận, thuận nghịch, không kể sẵn hoàn cảnh (ai, ở đâu, làm gì).",
            "1–2 câu tiếng Việt tự nhiên, 60–160 ký tự. Câu đầu báo điều sắp tới, câu sau mở ra lối hoặc lý do.",
            "\"text\" chỉ có lời quẻ: không số, không hạng, không nhãn, không dấu chấm phẩy.",
            "Không lời chúc, không lời dạy, không văn báo cáo, không hình ảnh tự chế, không ngôi thứ nhất, không tên chủ site. Không đoán bệnh tật, tai nạn, chia ly, tiền bạc.",
        ],
        "examples": [
            {"text": "Chuyện tưởng đã khép sắp có lời nhắc lại. Điều bạn chờ chưa tới, nhưng lối mở sẽ đến từ chuyện đã qua.", "type": "normal", "rank": "trung"},
            {"text": "Tin mong sẽ đến chậm hơn lòng người đợi. Khi điều còn giấu được nói ra, bạn sẽ hiểu vì sao chưa thể sớm hơn.", "type": "hac_dao", "rank": "trung"},
            {"text": "Việc mới sắp gặp một lần ngăn trở. Chưa phải hết đường, chỉ là lúc này chưa đến lượt cửa mở.", "type": "tam_nuong", "rank": "ha"},
            {"text": "Lời hẹn tưởng đã quên sắp được giữ. Điều từng mong mà chưa dám tin sẽ có một dấu hiệu rõ hơn.", "type": "hoang_dao", "rank": "thuong"},
            {"text": "Đường xa chưa thuận, việc gần sắp có tin. Chỗ tưởng không còn gì để đợi lại là nơi có lời đáp.", "type": "nguyet_ky", "rank": "ha"},
        ],
        "bad_examples": [
            {"text": "Nửa vầng trăng qua thước cửa; báo cáo sắp được duyệt phần kết quả, phần diễn giải sẽ còn một lượt góp ý.", "why": "Hình ảnh tự chế, chấm phẩy, văn báo cáo."},
            {"text": "Bạn sắp đến buổi học mà thiếu một món đồ cần dùng. Người ngồi cạnh sẽ có thứ cho mượn để bạn theo kịp.", "why": "Kể sẵn tình huống cụ thể, không còn là quẻ."},
        ],
        "personality": bio,
        "day_types": types,
        "ranks": ranks,
        "rules": [
            "Đủ số tối thiểu mỗi type và đủ ba hạng. Hoàng đạo phần lớn thuong, Nguyệt Kỵ và Tam Nương phần lớn ha.",
            "Tính cách chủ site chỉ ảnh hưởng nhẹ tới giọng, không đưa chuyện riêng của chủ site vào quẻ.",
            "120 quẻ khác nhau thật, không lặp khuôn câu.",
        ],
        "send_back": {
            "op": "set_fortune_quotes",
            "args": {"lang": lang, "quotes": [{"n": 1, "text": "…", "type": "normal", "rank": "trung"}]},
            "how": "POST /api/v1/ops với {\"op\": \"set_fortune_quotes\", \"args\": {…}}, cả 120 quẻ một lần. Bị từ chối thì sửa đúng quẻ được báo rồi gửi lại.",
        },
    })
}
