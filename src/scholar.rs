//! Scholar profile: pure rules for publications (DOI / ORCID iDs, Crossref,
//! OpenAlex and ORCID records), citation metrics and the HĐGSNN points
//! self-assessment. Network calls happen in the site; this module only parses
//! and computes.
//!
//! HĐGSNN rules used (Quyết định 37/2018/QĐ-TTg, amended by 25/2020/QĐ-TTg):
//! - minimum total of converted points: **PGS 10**, **GS 20**;
//! - a work with several authors: the main author gets 1/3 of its points, the
//!   other 2/3 are split equally among all authors including the main author;
//!   when no main author can be named, the points are split equally.
//! The points of each journal come from the list of the field's council (HĐGS
//! ngành), so the owner enters them per paper. Everything shown is a
//! **self-assessment**, never an official result.

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Extra fields a publication (collection item) may carry for the scholar features.
pub const DOI: &str = "doi";
pub const CITATIONS: &str = "citations";
/// `Q1` | `Q2` | `Q3` | `Q4` (journal rank, entered by the owner or the AI).
pub const QUARTILE: &str = "quartile";
/// Points of the work in the HĐGS ngành list (before splitting among authors).
pub const HDGS_POINTS: &str = "hdgs_points";
/// `sole` | `main` | `co` | `equal` (see `AuthorRole`).
pub const AUTHOR_ROLE: &str = "author_role";
/// Number of authors (for the split); empty = counted from `authors`.
pub const AUTHOR_COUNT: &str = "author_count";
/// `orcid` = found by the ORCID sync and waiting for the owner's review.
pub const REVIEW: &str = "review";
/// Where the metadata came from: `crossref`, `openalex`, `orcid`, `manual`.
pub const SOURCE: &str = "source";
pub const SCHOLAR_KEYS: &[&str] = &[DOI, CITATIONS, QUARTILE, HDGS_POINTS, AUTHOR_ROLE, AUTHOR_COUNT, REVIEW, SOURCE];

pub const PGS_MIN_POINTS: f64 = 10.0;
pub const GS_MIN_POINTS: f64 = 20.0;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Target {
    #[default]
    Pgs,
    Gs,
}

impl Target {
    pub fn min_points(self) -> f64 {
        match self {
            Target::Pgs => PGS_MIN_POINTS,
            Target::Gs => GS_MIN_POINTS,
        }
    }
    pub fn label(self) -> &'static str {
        match self {
            Target::Pgs => "PGS",
            Target::Gs => "GS",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuthorRole {
    /// Only author: all the points.
    Sole,
    /// Main author among several: 1/3 + an equal share of 2/3.
    Main,
    /// Co-author of a work that has a main author: an equal share of 2/3.
    Co,
    /// No main author can be named: an equal share of everything.
    #[default]
    Equal,
}

impl AuthorRole {
    pub fn parse(s: &str) -> Option<Self> {
        Some(match s.trim() {
            "sole" => AuthorRole::Sole,
            "main" => AuthorRole::Main,
            "co" => AuthorRole::Co,
            "equal" | "" => AuthorRole::Equal,
            _ => return None,
        })
    }
}

/// The owner's share of a work's points.
pub fn personal_points(points: f64, authors: u32, role: AuthorRole) -> f64 {
    if points <= 0.0 {
        return 0.0;
    }
    let n = authors.max(1) as f64;
    let share = match role {
        AuthorRole::Sole => points,
        _ if authors <= 1 => points,
        AuthorRole::Main => points / 3.0 + points * 2.0 / 3.0 / n,
        AuthorRole::Co => points * 2.0 / 3.0 / n,
        AuthorRole::Equal => points / n,
    };
    (share * 100.0).round() / 100.0
}

/// Authors listed as "A, B; C and D" → count.
pub fn count_authors(authors: &str) -> u32 {
    let n = authors
        .split([',', ';'])
        .flat_map(|part| part.split(" and "))
        .flat_map(|part| part.split(" và "))
        .filter(|part| !part.trim().is_empty())
        .count();
    n.max(1) as u32
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Quartile {
    Q1,
    Q2,
    Q3,
    Q4,
}

impl Quartile {
    pub fn parse(s: &str) -> Option<Self> {
        Some(match s.trim().to_ascii_uppercase().as_str() {
            "Q1" => Quartile::Q1,
            "Q2" => Quartile::Q2,
            "Q3" => Quartile::Q3,
            "Q4" => Quartile::Q4,
            _ => return None,
        })
    }
}

/// `https://doi.org/10.1/x`, `doi:10.1/X` or `10.1/x` → `10.1/x` (lowercase); None if not a DOI.
pub fn normalize_doi(input: &str) -> Option<String> {
    let mut s = input.trim();
    for prefix in ["https://doi.org/", "http://doi.org/", "https://dx.doi.org/", "http://dx.doi.org/", "doi.org/", "doi:", "DOI:", "DOI "] {
        if let Some(rest) = s.strip_prefix(prefix) {
            s = rest.trim();
        }
    }
    let s = s.to_lowercase();
    let (prefix, suffix) = s.split_once('/')?;
    let ok = prefix.starts_with("10.")
        && prefix.len() > 3
        && prefix[3..].chars().all(|c| c.is_ascii_digit() || c == '.')
        && !suffix.is_empty()
        && suffix.len() <= 300
        && !suffix.chars().any(|c| c.is_whitespace() || c.is_control());
    ok.then_some(s)
}

/// An ORCID iD (with or without `https://orcid.org/`), checked with ISO 7064 11,2.
pub fn normalize_orcid(input: &str) -> Option<String> {
    let s = input.trim().trim_start_matches("https://orcid.org/").trim_start_matches("http://orcid.org/").trim_start_matches("orcid.org/");
    let digits: Vec<char> = s.chars().filter(|c| *c != '-').collect();
    if digits.len() != 16 || !digits[..15].iter().all(|c| c.is_ascii_digit()) {
        return None;
    }
    let mut total = 0u32;
    for c in &digits[..15] {
        total = (total + c.to_digit(10)?) * 2;
    }
    let check = (12 - total % 11) % 11;
    let want = if check == 10 { 'X' } else { char::from_digit(check, 10)? };
    if digits[15].to_ascii_uppercase() != want {
        return None;
    }
    let d: String = digits.iter().collect::<String>().to_ascii_uppercase();
    Some(format!("{}-{}-{}-{}", &d[0..4], &d[4..8], &d[8..12], &d[12..16]))
}

/// h-index of a list of citation counts.
pub fn h_index(citations: &[u64]) -> u64 {
    let mut c = citations.to_vec();
    c.sort_unstable_by(|a, b| b.cmp(a));
    c.iter().enumerate().take_while(|(i, n)| **n >= (*i as u64 + 1)).count() as u64
}

/// Metadata of one work, read from Crossref (or OpenAlex).
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct WorkMeta {
    pub doi: String,
    pub title: String,
    pub authors: Vec<String>,
    pub venue: String,
    pub year: Option<i32>,
    pub url: String,
    pub citations: Option<u64>,
}

fn first_str(v: &Value) -> String {
    match v {
        Value::Array(a) => a.first().and_then(Value::as_str).unwrap_or("").trim().to_string(),
        Value::String(s) => s.trim().to_string(),
        _ => String::new(),
    }
}

/// `GET https://api.crossref.org/works/{doi}` → `message`.
pub fn parse_crossref(body: &Value) -> Option<WorkMeta> {
    let m = body.get("message").unwrap_or(body);
    let doi = normalize_doi(m.get("DOI")?.as_str()?)?;
    let title = first_str(m.get("title").unwrap_or(&Value::Null));
    if title.is_empty() {
        return None;
    }
    let authors = m
        .get("author")
        .and_then(Value::as_array)
        .map(|list| {
            list.iter()
                .filter_map(|a| {
                    let given = a.get("given").and_then(Value::as_str).unwrap_or("").trim();
                    let family = a.get("family").and_then(Value::as_str).unwrap_or("").trim();
                    let name = a.get("name").and_then(Value::as_str).unwrap_or("").trim();
                    let full = if family.is_empty() { name.to_string() } else { format!("{given} {family}").trim().to_string() };
                    (!full.is_empty()).then_some(full)
                })
                .collect()
        })
        .unwrap_or_default();
    let venue = first_str(m.get("container-title").unwrap_or(&Value::Null));
    let year = ["published-print", "published-online", "issued", "created"].iter().find_map(|k| {
        m.get(*k)?.get("date-parts")?.as_array()?.first()?.as_array()?.first()?.as_i64().map(|y| y as i32)
    });
    let citations = m.get("is-referenced-by-count").and_then(Value::as_u64);
    Some(WorkMeta { url: format!("https://doi.org/{doi}"), doi, title, authors, venue, year, citations })
}

/// `GET https://api.openalex.org/works/doi:{doi}` → citation count.
pub fn parse_openalex_citations(body: &Value) -> Option<u64> {
    body.get("cited_by_count").and_then(Value::as_u64)
}

/// `GET https://pub.orcid.org/v3.0/{orcid}/works` (JSON) → DOIs of the works.
pub fn parse_orcid_dois(body: &Value) -> Vec<String> {
    let mut out = Vec::new();
    for group in body.get("group").and_then(Value::as_array).into_iter().flatten() {
        let ids = group.get("external-ids").and_then(|e| e.get("external-id")).and_then(Value::as_array);
        for id in ids.into_iter().flatten() {
            if id.get("external-id-type").and_then(Value::as_str) == Some("doi") {
                if let Some(doi) = id.get("external-id-value").and_then(Value::as_str).and_then(normalize_doi) {
                    if !out.contains(&doi) {
                        out.push(doi);
                    }
                }
            }
        }
    }
    out
}

/// One publication as the calculator sees it.
#[derive(Clone, Debug, Default, Serialize)]
pub struct PaperPoints {
    pub id: String,
    pub title: String,
    pub year: Option<i32>,
    pub quartile: Option<Quartile>,
    pub citations: u64,
    pub points: f64,
    pub authors: u32,
    pub role: AuthorRole,
    pub personal: f64,
}

#[derive(Clone, Debug, Default, Serialize)]
pub struct Summary {
    pub publications: usize,
    pub with_doi: usize,
    pub q1_q2: usize,
    pub citations: u64,
    pub h_index: u64,
    pub points_total: f64,
    pub target: Target,
    pub target_points: f64,
    /// 0..100 (capped).
    pub progress_percent: f64,
    pub enough: bool,
}

pub fn summarize(papers: &[PaperPoints], with_doi: usize, target: Target) -> Summary {
    let total: f64 = papers.iter().map(|p| p.personal).sum();
    let total = (total * 100.0).round() / 100.0;
    let cites: Vec<u64> = papers.iter().map(|p| p.citations).collect();
    let min = target.min_points();
    Summary {
        publications: papers.len(),
        with_doi,
        q1_q2: papers.iter().filter(|p| matches!(p.quartile, Some(Quartile::Q1 | Quartile::Q2))).count(),
        citations: cites.iter().sum(),
        h_index: h_index(&cites),
        points_total: total,
        target,
        target_points: min,
        progress_percent: (total / min * 100.0).min(100.0).max(0.0),
        enough: total >= min,
    }
}

/// The owner's scholar settings (kept in the site content).
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ScholarSettings {
    /// ORCID iD, `0000-0000-0000-0000`.
    pub orcid: String,
    /// Council field (ngành) the points are counted for, e.g. "Kinh tế".
    pub field: String,
    pub target: Target,
    /// Show the self-assessed "enough points" badge on the public site.
    pub public_badge: bool,
    /// Show Q1/Q2 count, citations and h-index on the public site.
    pub public_metrics: bool,
    /// Collection tab that holds the publications ("" = the first `research` tab).
    pub research_page_id: String,
    pub orcid_synced_at: i64,
    pub cv: CvProfile,
}

/// Fields of the scientific CV (NAFOSTED / ministry "Lý lịch khoa học") that are
/// not elsewhere on the site.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct CvProfile {
    pub full_name: String,
    pub birth_year: String,
    pub gender: String,
    pub degree: String,
    pub academic_title: String,
    pub position: String,
    pub institution: String,
    pub address: String,
    pub email: String,
    pub phone: String,
    pub research_areas: String,
    pub foreign_languages: String,
    /// Markdown: one line per step ("2010–2014: Tiến sĩ, Đại học …").
    pub education_md: String,
    pub employment_md: String,
}

impl ScholarSettings {
    /// Keeps only valid values; errors name the field.
    pub fn validated(mut self) -> Result<Self, String> {
        if !self.orcid.trim().is_empty() {
            self.orcid = normalize_orcid(&self.orcid).ok_or("ORCID iD không hợp lệ (dạng 0000-0002-1825-0097).")?;
        } else {
            self.orcid.clear();
        }
        let cut = |s: &mut String, n: usize| *s = s.trim().chars().take(n).collect();
        cut(&mut self.field, 80);
        let cv = &mut self.cv;
        for (s, n) in [(&mut cv.full_name, 120), (&mut cv.birth_year, 10), (&mut cv.gender, 20), (&mut cv.degree, 120), (&mut cv.academic_title, 120),
            (&mut cv.position, 200), (&mut cv.institution, 300), (&mut cv.address, 300), (&mut cv.email, 200), (&mut cv.phone, 40),
            (&mut cv.research_areas, 500), (&mut cv.foreign_languages, 200), (&mut cv.education_md, 5000), (&mut cv.employment_md, 5000)] {
            cut(s, n);
        }
        Ok(self)
    }

    /// The public badge text, when the owner switched it on and the total is enough.
    pub fn badge(&self, summary: &Summary, en: bool) -> Option<String> {
        if !self.public_badge || !summary.enough {
            return None;
        }
        let field = self.field.trim();
        Some(match (en, field.is_empty()) {
            (false, false) => format!("Đủ điểm công trình chuẩn {} – ngành {field} (tự đánh giá)", self.target.label()),
            (false, true) => format!("Đủ điểm công trình chuẩn {} (tự đánh giá)", self.target.label()),
            (true, false) => format!("Enough publication points for {} – {field} (self-assessed)", if self.target == Target::Gs { "Full Professor" } else { "Associate Professor" }),
            (true, true) => format!("Enough publication points for {} (self-assessed)", if self.target == Target::Gs { "Full Professor" } else { "Associate Professor" }),
        })
    }
}
