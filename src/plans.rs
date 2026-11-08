//! Product tiers and the feature catalog. The engine stores the platform
//! admin's configuration (a global switch and the lowest tier for each
//! feature, AI credits per tier) and resolves the effective feature set of
//! each site. The site enforces that set on ops, AI tools and routes.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum Tier {
    Basic,
    #[default]
    Standard,
    Premium,
}

impl Tier {
    pub const ALL: [Tier; 3] = [Tier::Basic, Tier::Standard, Tier::Premium];

    pub fn label(self) -> &'static str {
        match self {
            Tier::Basic => "Cơ bản",
            Tier::Standard => "Tiêu chuẩn",
            Tier::Premium => "Cao cấp",
        }
    }
}

pub const WEBSITE: &str = "studio.website";
pub const AI_ASSISTANT: &str = "studio.ai";
pub const CUSTOM_DOMAIN: &str = "studio.custom_domain";
pub const BLOG: &str = "content.blog";
pub const NEWSLETTER: &str = "content.newsletter";
pub const COLLECTIONS: &str = "content.collections";
pub const COMMUNITY: &str = "content.community";
pub const PRODUCTS: &str = "commerce.products";
pub const SELF_PACED: &str = "content.self_paced";
pub const SURVEYS: &str = "research.surveys";
pub const TEACHING: &str = "teaching.classes";

#[derive(Clone, Copy, Debug, Serialize)]
pub struct FeatureDef {
    pub key: &'static str,
    pub label: &'static str,
    pub description: &'static str,
    /// Staff workspace that owns the feature: `studio`, `content` or `teaching`.
    pub workspace: &'static str,
    pub default_tier: Tier,
    pub default_enabled: bool,
    /// Always on in every tier (the admin cannot switch it off or move it).
    pub locked: bool,
}

const fn feature(key: &'static str, label: &'static str, description: &'static str, workspace: &'static str, default_tier: Tier) -> FeatureDef {
    FeatureDef { key, label, description, workspace, default_tier, default_enabled: true, locked: false }
}

pub const CATALOG: &[FeatureDef] = &[
    FeatureDef { locked: true, ..feature(WEBSITE, "Website & giao diện", "Trang, block, giao diện, mẫu trang, màn mở đầu và xuất bản.", "studio", Tier::Basic) },
    feature(AI_ASSISTANT, "Trợ lý AI", "Chỉnh website và viết nội dung bằng AI, trong hạn mức AI credits của gói.", "studio", Tier::Basic),
    feature(CUSTOM_DOMAIN, "Tên miền riêng", "Kết nối và xác minh tên miền của chủ site.", "studio", Tier::Basic),
    feature(BLOG, "Blog, lượt thích & bình luận", "Viết, xuất bản bài và kiểm duyệt bình luận.", "content", Tier::Standard),
    feature(NEWSLETTER, "Bản tin email", "Thu thập người đăng ký và gửi bản tin (trong hạn mức mỗi ngày).", "content", Tier::Standard),
    feature(COLLECTIONS, "Danh mục công trình", "Công bố, dự án, danh sách đọc và các danh mục khác.", "content", Tier::Standard),
    feature(COMMUNITY, "Phục vụ cộng đồng", "Trang /community kể lại những việc chủ site đã làm vì cộng đồng.", "content", Tier::Standard),
    feature(PRODUCTS, "Sản phẩm số & bán hàng", "Tài liệu, bản ghi, gói sản phẩm; thanh toán một lần và tải về có bảo vệ.", "content", Tier::Premium),
    feature(SELF_PACED, "Khoá tự học", "Khoá học và đề tự học, miễn phí hoặc trả phí.", "content", Tier::Premium),
    feature(SURVEYS, "Khảo sát nghiên cứu", "Khảo sát công khai có đồng thuận, ngày đóng và xuất CSV.", "content", Tier::Standard),
    FeatureDef { default_enabled: false, ..feature(TEACHING, "Lớp học & giảng dạy", "Lớp, danh sách học viên, bài tập, chấm điểm, ngân hàng câu hỏi và báo cáo.", "teaching", Tier::Premium) },
];

pub fn feature_def(key: &str) -> Option<&'static FeatureDef> {
    CATALOG.iter().find(|f| f.key == key)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FeatureSetting {
    pub enabled: bool,
    /// Lowest tier that includes the feature; None = in no tier (site overrides only).
    pub tier: Option<Tier>,
}

/// Monthly AI credits for each tier; None = the platform default.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct TierCredits {
    pub basic: Option<i64>,
    pub standard: Option<i64>,
    pub premium: Option<i64>,
}

impl TierCredits {
    pub fn get(&self, tier: Tier) -> Option<i64> {
        match tier {
            Tier::Basic => self.basic,
            Tier::Standard => self.standard,
            Tier::Premium => self.premium,
        }
    }
}

/// The admin's tier board, stored by the engine as one document.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct PlanConfig {
    pub features: BTreeMap<String, FeatureSetting>,
    pub ai_credits: TierCredits,
    /// Tier of a site that has none set.
    pub default_tier: Tier,
}

impl Default for PlanConfig {
    fn default() -> Self {
        Self {
            features: CATALOG.iter().map(|f| (f.key.to_string(), FeatureSetting { enabled: f.default_enabled, tier: Some(f.default_tier) })).collect(),
            ai_credits: TierCredits { basic: Some(1000), standard: None, premium: None },
            // Sites created before tiers keep everything they had.
            default_tier: Tier::Premium,
        }
    }
}

impl PlanConfig {
    /// The saved setting, or the catalog default for a feature added later.
    pub fn setting(&self, def: &FeatureDef) -> FeatureSetting {
        if def.locked {
            return FeatureSetting { enabled: true, tier: Some(Tier::Basic) };
        }
        self.features.get(def.key).copied().unwrap_or(FeatureSetting { enabled: def.default_enabled, tier: Some(def.default_tier) })
    }

    /// Keeps only catalog keys, fixes locked features and rejects negative credits.
    pub fn normalized(mut self) -> Result<Self, String> {
        self.features = CATALOG.iter().map(|f| (f.key.to_string(), self.setting(f))).collect();
        for tier in Tier::ALL {
            if self.ai_credits.get(tier).is_some_and(|v| v < 0) {
                return Err(format!("AI credits của gói {} phải ≥ 0.", tier.label()));
            }
        }
        Ok(self)
    }

    /// Feature keys a site may use. A global off switch wins over everything;
    /// then a site override; then the site's tier.
    pub fn resolve(&self, tier: Tier, overrides: &BTreeMap<String, bool>) -> Vec<String> {
        CATALOG
            .iter()
            .filter(|def| {
                let s = self.setting(def);
                s.enabled && (def.locked || overrides.get(def.key).copied().unwrap_or_else(|| s.tier.is_some_and(|t| t <= tier)))
            })
            .map(|def| def.key.to_string())
            .collect()
    }
}
