//! Internal API data types and shared protocol helpers.

use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use crate::models::*;
use crate::util::sha256_hex;

pub const HDR_TS: &str = "x-edm-ts";
pub const HDR_SIG: &str = "x-edm-sig";
pub const MAX_SKEW_SECS: i64 = 300;

pub fn sign(secret: &str, ts: i64, method: &str, path_and_query: &str, body: &[u8]) -> String {
    let msg = format!("{ts}\n{}\n{path_and_query}\n{}", method.to_ascii_uppercase(), sha256_hex(body));
    let mut mac = Hmac::<Sha256>::new_from_slice(secret.as_bytes()).expect("hmac key");
    mac.update(msg.as_bytes());
    hex::encode(mac.finalize().into_bytes())
}

pub fn verify(secret: &str, ts_header: &str, sig_header: &str, method: &str, path_and_query: &str, body: &[u8], now: i64) -> bool {
    let Ok(ts) = ts_header.trim().parse::<i64>() else { return false };
    if (now - ts).abs() > MAX_SKEW_SECS {
        return false;
    }
    let expected = sign(secret, ts, method, path_and_query, body);
    crate::util::secure_eq(&expected, sig_header.trim())
}

// ───────────────────────────── DTOs ─────────────────────────────────────────

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
/// Path the control service fetches on a custom domain to prove it points at us.
pub const DOMAIN_CHALLENGE_PATH: &str = "/.well-known/learnmedi-verify";

/// Deterministic challenge for a domain. The sites service answers it on
/// `DOMAIN_CHALLENGE_PATH`; control recomputes and compares. Both know the
/// shared secret, so no state needs to be exchanged.
pub fn domain_challenge(secret: &str, domain: &str) -> String {
    let mut mac = Hmac::<Sha256>::new_from_slice(secret.as_bytes()).expect("hmac key");
    mac.update(b"domain-verify:");
    mac.update(domain.trim().to_ascii_lowercase().as_bytes());
    hex::encode(&mac.finalize().into_bytes()[..16])
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
