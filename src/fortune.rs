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
//! reload shows the same. The slip's number is its place in the list (1–120).
//! Slips foretell; they are never in the owner's voice or under their name.

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
    ("nguyet_ky", "Ngày Nguyệt Kỵ (mùng 5, 14, 23 âm lịch)", "Quẻ báo đường xa có trắc trở, việc lớn dễ dở dang; điều lành đến từ chỗ quen, người gần. Hạng ha hoặc trung. TUYỆT ĐỐI không báo lợi cho đi xa, xuất hành, khởi sự.", 10),
    ("tam_nuong", "Ngày Tam Nương (mùng 3, 7, 13, 18, 22, 27 âm lịch)", "Quẻ báo việc mới chưa đến thời, cưới hỏi, khai trương nên dời; ai chờ thêm sẽ được. Hạng ha hoặc trung. Không báo lợi cho khởi sự.", 10),
    ("hac_dao", "Ngày hắc đạo", "Quẻ báo nước còn đục, nên lặng: việc cũ thành, việc mới chậm, lời nói dễ lỡ. Hạng trung hoặc ha. Không báo lợi cho việc lớn.", 15),
    ("hoang_dao", "Ngày hoàng đạo", "Quẻ báo thời đến: có quý nhân, tin vui, việc học và việc mới hanh thông. Hạng thuong hoặc trung.", 15),
    ("normal", "Ngày thường (dùng được mọi ngày)", "Quẻ về chuyện thường ngày (học trò, đồng nghiệp, trang sách, lời hứa, mùa màng), không nhắc ngày tốt xấu, không báo lợi cho đi xa hay khởi sự. Đủ ba hạng.", 40),
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
const FORMAT: &str = "xam-1";

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
        "task": format!("Viết đúng {QUOTES_TOTAL} quẻ xăm bằng ngôn ngữ '{lang}' cho ống xăm trên site. Mỗi ngày, khách rút một que và đọc quẻ của mình; số quẻ là thứ tự trong danh sách (1–{QUOTES_TOTAL}). Mỗi quẻ thuộc đúng một loại ngày (type) và một hạng (rank) dưới đây, đủ số tối thiểu của từng loại."),
        "what_a_slip_is": "Quẻ xăm là lời đoán, không phải lời khuyên hay lời chúc. Mỗi quẻ có hai phần trong cùng một câu: (1) một hình ảnh như thơ xăm (vd. 'Trăng mười sáu khuyết một bên', 'Thuyền xuôi gặp gió đầu ghềnh'); (2) lời giải nói rõ điều sắp tới ở một mảng cụ thể: việc học, công việc, người cần gặp, tin tức, chuyện chờ đợi, lời hứa. Dùng thì tương lai ('sẽ', 'sắp', 'trong tuần'), nói như quẻ đang báo trước.",
        "examples": [
            {"text": "Mưa dầm thấm đất. Việc đang làm dở sẽ thành nếu đủ kiên nhẫn; tin chờ lâu sắp tới, chưa nên giục.", "type": "hac_dao", "rank": "trung"},
            {"text": "Cửa đông mở, nắng sớm vào nhà. Sắp gặp người giúp mình trong việc học hoặc công việc; lời đề nghị đầu tiên nên nhận.", "type": "hoang_dao", "rank": "thuong"},
            {"text": "Thuyền xuôi gặp gió đầu ghềnh. Đường xa hôm nay nhiều trắc trở; điều tốt nằm ở chỗ quen, người gần.", "type": "nguyet_ky", "rank": "ha"},
        ],
        "not_a_slip": [
            "Lời khuyên chung chung ('Hãy luôn mỉm cười', 'Cứ cố gắng là được').",
            "Lời chúc ('Chúc bạn một ngày vui').",
            "Châm ngôn không đoán điều gì.",
        ],
        "personality": bio,
        "day_types": types,
        "ranks": ranks,
        "rules": [
            "Tính cách chủ site chỉ quyết định giọng và chủ đề hình ảnh (vd. người dạy học → trang sách, phấn bảng, học trò; người làm cộng đồng → giếng làng, bếp chung). Quẻ là lời của ống xăm, KHÔNG phải lời của chủ site.",
            "Không dùng ngôi thứ nhất (tôi, I, my), không nhắc tên chủ site, không trích dẫn ai.",
            "Hạng ha báo điều chưa thuận nhưng nhẹ nhàng và luôn chỉ ra chỗ có lối; không đoán bệnh tật, tai nạn, chết chóc, chia ly.",
            "Không đoán về tiền bạc, đầu tư, xổ số, cờ bạc hay sức khoẻ.",
            format!("Mỗi quẻ tối đa {QUOTE_MAX_CHARS} ký tự, không link, không trùng."),
            "Có đủ ba hạng; quẻ hoàng đạo phần lớn thuong, quẻ Nguyệt Kỵ và Tam Nương phần lớn ha.",
        ],
        "send_back": {
            "op": "set_fortune_quotes",
            "args": {"lang": lang, "quotes": [{"text": "…", "type": "normal", "rank": "trung"}]},
            "how": "POST /api/v1/ops với {\"op\": \"set_fortune_quotes\", \"args\": {…}} (cùng token). Gửi cả 120 quẻ một lần; sai một quẻ thì cả lần gửi bị từ chối và báo quẻ nào sai.",
        },
    })
}
