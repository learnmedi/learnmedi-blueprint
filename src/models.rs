//! Shared types for the engine and teacher sites.

use serde::{Deserialize, Serialize};
pub fn new_id() -> String {
    crate::util::random_token(20).to_ascii_lowercase()
}

pub fn now_ms() -> i64 {
    chrono::Utc::now().timestamp_millis()
}

// ───────────────────────────── Users & sessions ─────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    #[serde(rename = "_id")]
    pub id: String,
    pub email: String,
    pub name: String,
    #[serde(default)]
    pub picture: Option<String>,
    #[serde(default)]
    pub google_sub: Option<String>,
    #[serde(default)]
    pub is_platform_admin: bool,
    #[serde(default)]
    pub disabled: bool,
    pub ts_created: i64,
    #[serde(default)]
    pub ts_last_login: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum SiteRole {
    Owner,
    /// Co-teacher / assistant: edits content and courses, no money/domains.
    Editor,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StaffMember {
    #[serde(default)]
    pub user_id: Option<String>,
    pub email: String,
    pub role: SiteRole,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomDomain {
    pub domain: String,
    pub verify_token: String,
    #[serde(default)]
    pub verified: bool,
    #[serde(default)]
    pub ts_verified: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NavItem {
    pub label: String,
    pub url: String,
}

/// Registry entry for a teacher site. OWNED BY THE CONTROL SERVICE
/// (`control` DB). The sites service reads it through the internal API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SiteRecord {
    #[serde(rename = "_id")]
    pub id: String,
    /// Subdomain label: `<slug>.<ROOT_DOMAIN>`.
    pub slug: String,
    pub name: String,
    pub owner_id: String,
    pub owner_email: String,
    #[serde(default)]
    pub staff: Vec<StaffMember>,
    /// `active` | `suspended`
    pub status: String,
    #[serde(default)]
    pub custom_domains: Vec<CustomDomain>,
    /// Monthly AI token budget; None = platform default.
    #[serde(default)]
    pub ai_monthly_token_limit: Option<i64>,
    /// Starter template used when the sites service first seeds content.
    #[serde(default)]
    pub template: String,
    /// Max newsletter emails per day for this site; None = platform default.
    #[serde(default)]
    pub newsletter_daily_limit: Option<i64>,
    /// Counters pushed by the sites service (for the platform admin).
    #[serde(default)]
    pub stats: SiteStats,
    pub ts_created: i64,
    #[serde(default)]
    pub ts_updated: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SiteStats {
    #[serde(default)]
    pub pages: i64,
    #[serde(default)]
    pub posts: i64,
    #[serde(default)]
    pub courses: i64,
    #[serde(default)]
    pub students: i64,
    #[serde(default)]
    pub classrooms: i64,
    #[serde(default)]
    pub storage_bytes: i64,
    #[serde(default)]
    pub subscribers: i64,
    #[serde(default)]
    pub ts_reported: i64,
}

impl SiteRecord {
    pub fn role_of(&self, user: &User) -> Option<SiteRole> {
        if self.owner_id == user.id {
            return Some(SiteRole::Owner);
        }
        self.staff
            .iter()
            .find(|m| m.user_id.as_deref() == Some(user.id.as_str()) || m.email.eq_ignore_ascii_case(&user.email))
            .map(|m| m.role.clone())
    }
    pub fn verified_domains(&self) -> Vec<String> {
        self.custom_domains.iter().filter(|d| d.verified).map(|d| d.domain.clone()).collect()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Order {
    #[serde(rename = "_id")]
    pub id: String,
    pub site_id: String,
    pub course_id: String,
    pub course_title: String,
    pub user_id: String,
    pub user_email: String,
    pub amount_vnd: i64,
    /// Payment code the student puts in the transfer memo, e.g. `EDMK7F3Q9XA`.
    pub code: String,
    /// `pending` | `paid` | `underpaid` | `expired` | `cancelled`
    pub status: String,
    #[serde(default)]
    pub paid_amount_vnd: i64,
    #[serde(default)]
    pub sepay_tx_ids: Vec<i64>,
    pub ts_created: i64,
    pub ts_expires: i64,
    #[serde(default)]
    pub ts_paid: i64,
}
