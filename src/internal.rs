//! Internal API data types and shared protocol helpers.

use serde::{Deserialize, Serialize};
use crate::models::*;
use crate::util::keyed_digest;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolveHostReq {
    pub host: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolveHostResp {
    /// `platform` | `site` | `unknown`
    pub kind: String,
    pub site: Option<SiteRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedeemTicketReq {
    pub ticket: String,
    pub host: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedeemTicketResp {
    pub user: User,
    pub next_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetStaffReq {
    pub staff: Vec<StaffMember>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainReq {
    /// `add` | `remove` | `verify`
    pub action: String,
    pub domain: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainResp {
    pub site: SiteRecord,
    #[serde(default)]
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuotaReq {
    pub site_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuotaResp {
    pub month: String,
    pub used: i64,
    pub limit: i64,
    pub remaining: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageReport {
    pub site_id: String,
    pub user_id: String,
    pub session_id: String,
    pub model: String,
    pub prompt_tokens: i64,
    pub completion_tokens: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateOrderReq {
    pub site_id: String,
    pub course_id: String,
    pub course_title: String,
    pub user_id: String,
    pub user_email: String,
    pub amount_vnd: i64,
}

/// Order + everything the student needs to pay it.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderView {
    pub order: Order,
    pub qr_url: String,
    pub bank_name: String,
    pub bank_account: String,
    pub account_holder: String,
    /// Exact text to put in the transfer memo.
    pub transfer_content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaidOrdersResp {
    pub orders: Vec<Order>,
    /// Pass back as `since` next time.
    pub cursor: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SitePaymentsResp {
    pub orders: Vec<Order>,
    pub ledger: Vec<LedgerEntry>,
    pub balance_vnd: i64,
    pub total_sales_vnd: i64,
    pub total_paid_out_vnd: i64,
    pub fee_percent: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageSummary {
    pub month: String,
    pub used: i64,
    pub limit: i64,
    pub calls: i64,
    pub by_user: Vec<(String, i64)>,
}

/// Path the control service fetches on a custom domain to prove it points at us.
pub const DOMAIN_CHALLENGE_PATH: &str = "/.well-known/learnmedi-verify";

/// Deterministic challenge for a domain. The sites service answers it on
/// `DOMAIN_CHALLENGE_PATH`; control recomputes and compares. Both know the
/// shared secret, so no state needs to be exchanged.
pub fn domain_challenge(secret: &str, domain: &str) -> String {
    let key = keyed_digest(secret.as_bytes(), b"domain-challenge");
    let domain = domain.trim().to_ascii_lowercase();
    hex::encode(&keyed_digest(&key, domain.as_bytes())[..16])
}

/// Normalise user input like "https://www.Co-Lan.vn/abc" → "www.co-lan.vn".
pub fn normalize_domain(input: &str) -> Option<String> {
    let mut d = input.trim().to_ascii_lowercase();
    for p in ["https://", "http://"] {
        if let Some(r) = d.strip_prefix(p) {
            d = r.to_string();
        }
    }
    let d = d.split(['/', '?', '#']).next().unwrap_or("").trim_end_matches('.').to_string();
    let d = crate::util::strip_port(&d).to_string();
    let labels: Vec<&str> = d.split('.').collect();
    if labels.len() < 2 || d.len() > 253 {
        return None;
    }
    for l in &labels {
        if l.is_empty() || l.len() > 63 || l.starts_with('-') || l.ends_with('-') || !l.chars().all(|c| c.is_ascii_alphanumeric() || c == '-') {
            return None;
        }
    }
    if labels.last().map(|t| t.chars().all(|c| c.is_ascii_digit())).unwrap_or(true) {
        return None; // IP addresses are not allowed
    }
    Some(d)
}

/// Subdomain labels nobody may claim.
pub const RESERVED_SLUGS: &[&str] = &[
    "www", "admin", "api", "app", "apps", "mail", "email", "smtp", "studio", "static", "assets", "media", "cdn", "auth", "login", "logout",
    "root", "engine", "learnmedi", "help", "support", "docs", "blog", "status", "internal", "dashboard", "account", "accounts", "billing", "pay",
    "payment", "payments", "dev", "test", "staging", "ns1", "ns2", "ftp", "learn",
];

pub fn validate_slug(slug: &str) -> Result<(), String> {
    if slug.len() < 3 || slug.len() > 40 {
        return Err("Tên miền con dài 3–40 ký tự.".into());
    }
    if !slug.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-') || slug.starts_with('-') || slug.ends_with('-') {
        return Err("Tên miền con chỉ gồm chữ thường không dấu, số và dấu gạch ngang.".into());
    }
    if RESERVED_SLUGS.contains(&slug) {
        return Err("Tên này đã được hệ thống giữ, hãy chọn tên khác.".into());
    }
    Ok(())
}

/// Runtime configuration exposed to site instances.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct SiteSettings {
    pub openai_model: String,
    pub ai_session_max_tokens: i64,
    pub ai_session_max_user_turns: usize,
    pub ai_max_tool_rounds: usize,
    pub max_upload_bytes: i64,
    pub max_video_bytes: i64,
    pub max_student_upload_bytes: i64,
}

impl Default for SiteSettings {
    fn default() -> Self {
        Self {
            openai_model: "gpt-4.1-mini".into(),
            ai_session_max_tokens: 200000,
            ai_session_max_user_turns: 20,
            ai_max_tool_rounds: 8,
            max_upload_bytes: 26214400,
            max_video_bytes: 2147483648,
            max_student_upload_bytes: 20971520,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidateSessionReq {
    pub user_id: String,
    pub version: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MembershipKind {
    Student,
    Monitor,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MembershipUpdate {
    pub user_id: String,
    pub site_id: String,
    pub kind: MembershipKind,
    pub active: bool,
    #[serde(default)]
    pub revision: i64,
}
