//! Journal lists that fill a publication's HĐGSNN points when the owner left them
//! empty: the councils' lists of scored journals (HĐGSNN, by field and ISSN, with
//! the maximum points by publication year) and a Scopus quartile by year (from
//! SCImago SJR) for an estimate of international journals.
//!
//! The data is sent by the worker (`learnmedi_worker/journals/`); this module only
//! holds the types, their validation and the rules. Everything computed here is a
//! **self-assessment upper bound**: the council gives a mark inside the journal's
//! range, never above it.

use serde::{Deserialize, Serialize};

use crate::scholar::Quartile;
use crate::util::fold_text;

/// The list the HĐGSNN rows come from (shown with every looked-up value).
pub const HDGSNN_LIST: &str = "Danh mục HĐGSNN 2025 (QĐ 26/QĐ-HĐGSNN ngày 11/7/2025)";
/// Applying is advised once the total reaches this many times the minimum.
pub const RECOMMEND_FACTOR: f64 = 1.5;

/// A field council (HĐGS ngành / liên ngành).
#[derive(Debug)]
pub struct Nganh {
    /// Key used by the lists (`kinh_te`).
    pub key: &'static str,
    pub label: &'static str,
    /// Other names people type (already folded: lowercase, no diacritics).
    pub aliases: &'static [&'static str],
    /// Estimated maximum points of a Scopus journal in Q1, Q2, Q3, Q4 for this
    /// field, read from the field's rules for international journals (most fields
    /// rank by SCIE/WoS and impact factor, which is not open data, so the quartile
    /// stands in for it).
    pub intl: [f64; 4],
}

/// Used when the field is unknown: the most common international scale.
pub const DEFAULT_INTL: [f64; 4] = [3.0, 2.0, 1.5, 1.5];

pub const NGANH: &[Nganh] = &[
    Nganh { key: "chan_nuoi_thu_y_thuy_san", label: "Chăn nuôi - Thú y - Thủy sản", aliases: &["chan nuoi", "thu y", "thuy san"], intl: [3.0, 2.5, 2.0, 1.5] },
    Nganh { key: "co_hoc", label: "Cơ học", aliases: &["co hoc"], intl: [3.0, 2.0, 1.5, 1.5] },
    Nganh { key: "co_khi_dong_luc", label: "Cơ khí - Động lực", aliases: &["co khi", "dong luc"], intl: [3.0, 2.0, 1.5, 1.5] },
    Nganh { key: "cong_nghe_thong_tin", label: "Công nghệ thông tin", aliases: &["cntt", "cong nghe thong tin", "tin hoc", "khoa hoc may tinh"], intl: [2.0, 1.75, 1.5, 1.0] },
    Nganh { key: "dien_dien_tu_tu_dong_hoa", label: "Điện - Điện tử - Tự động hóa", aliases: &["dien tu", "tu dong hoa", "dien"], intl: [2.0, 2.0, 2.0, 2.0] },
    Nganh { key: "duoc_hoc", label: "Dược học", aliases: &["duoc hoc", "duoc"], intl: [3.0, 2.0, 2.0, 1.5] },
    Nganh { key: "giao_thong_van_tai", label: "Giao thông vận tải", aliases: &["giao thong van tai", "giao thong"], intl: [3.0, 2.0, 1.5, 1.5] },
    Nganh { key: "hoa_hoc_cong_nghe_thuc_pham", label: "Hóa học - Công nghệ thực phẩm", aliases: &["hoa hoc", "cong nghe thuc pham", "thuc pham", "hoa"], intl: [3.0, 2.0, 1.5, 1.5] },
    Nganh { key: "khoa_hoc_an_ninh", label: "Khoa học An ninh", aliases: &["an ninh"], intl: [3.0, 2.0, 1.5, 1.5] },
    Nganh { key: "khoa_hoc_giao_duc", label: "Khoa học Giáo dục", aliases: &["khoa hoc giao duc", "giao duc"], intl: [3.0, 2.0, 1.5, 1.5] },
    Nganh { key: "khoa_hoc_quan_su", label: "Khoa học Quân sự", aliases: &["quan su"], intl: [3.0, 2.0, 1.5, 1.5] },
    Nganh { key: "khoa_hoc_trai_dat_mo", label: "Khoa học Trái đất - Mỏ", aliases: &["trai dat", "dia chat", "mo"], intl: [2.0, 2.0, 1.5, 1.5] },
    Nganh { key: "kinh_te", label: "Kinh tế", aliases: &["kinh te", "tai chinh", "quan tri kinh doanh", "ke toan"], intl: [2.0, 1.5, 1.5, 1.5] },
    Nganh { key: "luat_hoc", label: "Luật học", aliases: &["luat hoc", "luat"], intl: [3.0, 2.0, 1.5, 1.5] },
    Nganh { key: "luyen_kim", label: "Luyện kim", aliases: &["luyen kim"], intl: [3.0, 2.0, 1.5, 1.5] },
    Nganh { key: "ngon_ngu_hoc", label: "Ngôn ngữ học", aliases: &["ngon ngu hoc", "ngon ngu"], intl: [3.0, 2.0, 1.5, 1.5] },
    Nganh { key: "nong_nghiep_lam_nghiep", label: "Nông nghiệp - Lâm nghiệp", aliases: &["nong nghiep", "lam nghiep"], intl: [3.0, 2.0, 2.0, 2.0] },
    Nganh { key: "sinh_hoc", label: "Sinh học", aliases: &["sinh hoc"], intl: [3.0, 2.0, 1.5, 1.5] },
    Nganh { key: "su_hoc_khao_co_hoc_dan_toc_hoc", label: "Sử học - Khảo cổ học - Dân tộc học/Nhân học", aliases: &["su hoc", "khao co", "dan toc hoc", "nhan hoc"], intl: [3.0, 2.0, 1.5, 1.5] },
    Nganh { key: "tam_ly_hoc", label: "Tâm lý học", aliases: &["tam ly hoc", "tam ly"], intl: [3.0, 2.0, 1.5, 1.5] },
    Nganh { key: "thuy_loi", label: "Thủy lợi", aliases: &["thuy loi"], intl: [2.0, 2.0, 2.0, 1.5] },
    Nganh { key: "toan_hoc", label: "Toán học", aliases: &["toan hoc", "toan"], intl: [2.0, 2.0, 1.5, 1.5] },
    Nganh { key: "triet_hoc_xa_hoi_hoc_chinh_tri_hoc", label: "Triết học - Xã hội học - Chính trị học", aliases: &["triet hoc", "xa hoi hoc", "chinh tri hoc"], intl: [2.0, 2.0, 2.0, 2.0] },
    Nganh { key: "van_hoa_nghe_thuat_the_duc_the_thao", label: "Văn hóa - Nghệ thuật - Thể dục thể thao", aliases: &["van hoa", "nghe thuat", "the duc the thao", "the thao"], intl: [3.0, 2.0, 1.5, 1.5] },
    Nganh { key: "van_hoc", label: "Văn học", aliases: &["van hoc"], intl: [3.0, 2.0, 2.0, 2.0] },
    Nganh { key: "vat_ly", label: "Vật lý", aliases: &["vat ly"], intl: [2.5, 2.0, 1.5, 1.5] },
    Nganh { key: "xay_dung_kien_truc", label: "Xây dựng - Kiến trúc", aliases: &["xay dung", "kien truc"], intl: [3.0, 2.0, 2.0, 2.0] },
    Nganh { key: "y_hoc", label: "Y học", aliases: &["y hoc", "y khoa", "y te", "y"], intl: [2.0, 2.0, 1.5, 1.5] },
];

pub fn nganh_by_key(key: &str) -> Option<&'static Nganh> {
    NGANH.iter().find(|n| n.key == key)
}

/// The council for what the owner typed as their field ("Kinh tế", "kinh te",
/// "HĐGS ngành Y học", "CNTT"…). None when nothing matches.
pub fn nganh(field: &str) -> Option<&'static Nganh> {
    let f = fold_text(field);
    if f.is_empty() {
        return None;
    }
    if let Some(n) = nganh_by_key(field.trim()) {
        return Some(n);
    }
    let words: Vec<&str> = f.split(' ').filter(|w| !matches!(*w, "hdgs" | "hoi" | "dong" | "giao" | "su" | "nganh" | "lien")).collect();
    let core = words.join(" ");
    for n in NGANH {
        if fold_text(n.label) == f || fold_text(n.label) == core || n.aliases.contains(&core.as_str()) {
            return Some(n);
        }
    }
    // Longest alias first, matched on whole words ("y" must not match "ky thuat").
    let padded = format!(" {f} ");
    let mut best: Option<(&'static Nganh, usize)> = None;
    for n in NGANH {
        for a in n.aliases {
            if padded.contains(&format!(" {a} ")) && best.is_none_or(|(_, l)| a.len() > l) {
                best = Some((n, a.len()));
            }
        }
    }
    best.map(|(n, _)| n)
}

/// `1234-567x`, `1234567X`, `p-1234-5679` → `1234-567X` when the check digit is right.
pub fn normalize_issn(input: &str) -> Option<String> {
    let s: String = input.chars().filter(|c| c.is_ascii_alphanumeric()).collect::<String>().to_ascii_uppercase();
    let s = s.trim_start_matches(['P', 'E']);
    if s.len() != 8 || !s[..7].chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    let sum: u32 = s[..7].chars().zip((2..=8).rev()).map(|(c, w)| c.to_digit(10).unwrap_or(0) * w).sum();
    let check = (11 - sum % 11) % 11;
    let want = if check == 10 { 'X' } else { char::from_digit(check, 10)? };
    (s.chars().nth(7)? == want).then(|| format!("{}-{}", &s[..4], &s[4..]))
}

/// One line of a journal's points cell: "0 – 0,75 từ 2020" = `{max: 0.75, from: 2020}`.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Tier {
    pub max: f64,
    #[serde(default)]
    pub from: Option<i32>,
    /// Last year the line applies ("trước 2020" → 2019).
    #[serde(default)]
    pub until: Option<i32>,
}

/// The maximum points of a paper published in `year` (None: the newest line).
/// None when no line covers that year (the journal was not on the list then).
pub fn max_for_year(tiers: &[Tier], year: Option<i32>) -> Option<f64> {
    let key = |t: &Tier| t.from.unwrap_or(i32::MIN);
    let pick = |list: Vec<&Tier>| list.into_iter().enumerate().max_by_key(|(i, t)| (key(t), *i)).map(|(_, t)| t.max);
    match year {
        None => pick(tiers.iter().filter(|t| t.until.is_none()).collect()).or_else(|| pick(tiers.iter().collect())),
        Some(y) => pick(tiers.iter().filter(|t| t.from.is_none_or(|f| f <= y) && t.until.is_none_or(|u| y <= u)).collect()),
    }
}

/// A journal on a council's list.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct JournalPoints {
    pub nganh: String,
    pub issn: Vec<String>,
    pub ten: String,
    #[serde(default)]
    pub stt: u32,
    pub tiers: Vec<Tier>,
}

impl JournalPoints {
    pub fn validate(&self) -> Result<(), String> {
        if nganh_by_key(&self.nganh).is_none() {
            return Err(format!("nganh không có: {}", self.nganh));
        }
        if self.issn.is_empty() || self.issn.len() > 8 || self.issn.iter().any(|i| normalize_issn(i).as_deref() != Some(i.as_str())) {
            return Err(format!("{}: ISSN không hợp lệ", self.ten));
        }
        if self.ten.chars().count() > 300 || self.tiers.is_empty() || self.tiers.len() > 12 {
            return Err(format!("{}: tên hoặc mức điểm không hợp lệ", self.ten));
        }
        if self.tiers.iter().any(|t| !(0.0..=4.0).contains(&t.max) || t.from.is_some_and(|y| !(1950..=2100).contains(&y)) || t.until.is_some_and(|y| !(1950..=2100).contains(&y))) {
            return Err(format!("{}: mức điểm ngoài khoảng", self.ten));
        }
        Ok(())
    }
}

/// A Scopus journal with its SJR quartile by year (1 = Q1).
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct SjrJournal {
    pub issn: Vec<String>,
    pub title: String,
    /// `[year, quartile]`, any order.
    pub quartiles: Vec<(i32, u8)>,
}

impl SjrJournal {
    pub fn validate(&self) -> Result<(), String> {
        if self.issn.is_empty() || self.issn.len() > 4 || self.issn.iter().any(|i| normalize_issn(i).as_deref() != Some(i.as_str())) {
            return Err(format!("{}: ISSN không hợp lệ", self.title));
        }
        if self.title.chars().count() > 400 || self.quartiles.is_empty() || self.quartiles.len() > 60 {
            return Err(format!("{}: thiếu hạng", self.title));
        }
        if self.quartiles.iter().any(|(y, q)| !(1990..=2100).contains(y) || !(1..=4).contains(q)) {
            return Err(format!("{}: hạng ngoài khoảng", self.title));
        }
        Ok(())
    }

    /// The quartile for a paper of `year`: that year's, else the closest earlier
    /// year, else the closest later one (None year: the newest). Returns the year used.
    pub fn quartile_for(&self, year: Option<i32>) -> Option<(Quartile, i32)> {
        let to_q = |q: u8| match q {
            1 => Quartile::Q1,
            2 => Quartile::Q2,
            3 => Quartile::Q3,
            _ => Quartile::Q4,
        };
        let newest = self.quartiles.iter().max_by_key(|(y, _)| *y);
        let found = match year {
            None => newest,
            Some(want) => self.quartiles.iter().filter(|(y, _)| *y <= want).max_by_key(|(y, _)| *y)
                .or_else(|| self.quartiles.iter().filter(|(y, _)| *y > want).min_by_key(|(y, _)| *y)),
        };
        found.map(|(y, q)| (to_q(*q), *y))
    }
}

/// Where a paper's points come from.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PointsSource {
    /// Typed by the owner.
    Manual,
    /// The journal is on the council's list.
    Hdgsnn,
    /// Estimated from the Scopus quartile and the field's scale.
    Estimate,
    /// Nothing known.
    #[default]
    None,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub struct AutoPoints {
    pub points: f64,
    pub source: PointsSource,
    pub note: String,
}

fn num(x: f64) -> String {
    let s = format!("{x}");
    s.replace('.', ",")
}

/// Points of a paper the owner gave none for. `listed` = the journal's row on the
/// field's list; `quartile` = (quartile, year it is from, typed by the owner?).
pub fn resolve(field: Option<&Nganh>, year: Option<i32>, listed: Option<&JournalPoints>, quartile: Option<(Quartile, Option<i32>, bool)>) -> AutoPoints {
    if let (Some(n), Some(row)) = (field, listed) {
        return match max_for_year(&row.tiers, year) {
            Some(max) => AutoPoints {
                points: max,
                source: PointsSource::Hdgsnn,
                note: if max > 0.0 {
                    format!("{HDGSNN_LIST}, ngành {}: «{}» tối đa {} điểm", n.label, row.ten, num(max))
                } else {
                    format!("{HDGSNN_LIST}, ngành {}: «{}» không được tính điểm cho năm này", n.label, row.ten)
                },
            },
            None => AutoPoints {
                points: 0.0,
                source: PointsSource::Hdgsnn,
                note: format!("{HDGSNN_LIST}, ngành {}: «{}» chưa được tính điểm vào năm {}", n.label, row.ten, year.map(|y| y.to_string()).unwrap_or_default()),
            },
        };
    }
    if let Some((q, qyear, typed)) = quartile {
        let scale = field.map(|n| n.intl).unwrap_or(DEFAULT_INTL);
        let i = match q {
            Quartile::Q1 => 0,
            Quartile::Q2 => 1,
            Quartile::Q3 => 2,
            Quartile::Q4 => 3,
        };
        let from = if typed { "hạng bạn nhập".to_string() } else { format!("SJR {}", qyear.map(|y| y.to_string()).unwrap_or_default()) };
        let scale_of = field.map(|n| format!("khung ngành {}", n.label)).unwrap_or_else(|| "khung phổ biến (chưa chọn ngành)".into());
        return AutoPoints {
            points: scale[i],
            source: PointsSource::Estimate,
            note: format!("Ước tính: Scopus {q:?} ({from}) → tối đa {} điểm theo {scale_of}; hội đồng xét theo SCIE/IF nên có thể khác", num(scale[i])),
        };
    }
    AutoPoints { points: 0.0, source: PointsSource::None, note: "Chưa tìm thấy tạp chí trong danh mục HĐGSNN hay Scopus; nhập điểm tay nếu có.".into() }
}

/// Whether two person names are the same person, ignoring diacritics, case,
/// punctuation and word order ("Nguyễn Văn An" = "An Nguyen Van" = "Van An Nguyen").
pub fn same_person(a: &str, b: &str) -> bool {
    let words = |s: &str| {
        let mut w: Vec<String> = fold_text(s).split(' ').filter(|x| !x.is_empty()).map(String::from).collect();
        w.sort();
        w
    };
    let (x, y) = (words(a), words(b));
    !x.is_empty() && x.len() >= 2 && x == y
}
