//! Product tiers and the feature catalog. The engine stores the platform
//! admin's configuration (a global switch and the lowest tier for each
//! feature, AI credits per tier) and resolves the effective feature set of
//! each site. The site enforces that set on ops, AI tools and routes.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::sync::LazyLock;

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
pub const ASSETS: &str = "site.assets";
pub const PERSONALITY: &str = "site.personality";
pub const FORTUNE: &str = "site.fortune";
pub const ASSOCIATES: &str = "site.associates";
pub const SCHOLAR: &str = "site.scholar";

/// A plan feature as the admin board shows it. Built from `ui/features/*.yaml`
/// (the files without `enabled_by`); see `features`.
#[derive(Clone, Copy, Debug, Serialize)]
pub struct FeatureDef {
    pub key: &'static str,
    pub label: &'static str,
    pub description: &'static str,
    /// Staff workspace that owns the feature: `studio`, `content`, `teaching`, `overview` or `settings`.
    pub workspace: &'static str,
    pub default_tier: Tier,
    pub default_enabled: bool,
    /// Always on in every tier (the admin cannot switch it off or move it).
    pub locked: bool,
}

/// Every plan feature, in registry order.
pub static CATALOG: LazyLock<Vec<FeatureDef>> = LazyLock::new(|| {
    crate::features::FEATURES
        .iter()
        .filter(|f| !f.is_group())
        .map(|f| FeatureDef {
            key: &f.key,
            label: &f.label,
            description: &f.description,
            workspace: &f.workspace,
            default_tier: f.default_tier,
            default_enabled: f.default_enabled,
            locked: f.locked,
        })
        .collect()
});

pub fn feature_def(key: &str) -> Option<&'static FeatureDef> {
    CATALOG.iter().find(|f| f.key == key)
}

/// Locked features are on for every site, even when an older engine leaves them
/// out of the effective list.
pub fn is_locked(key: &str) -> bool {
    feature_def(key).is_some_and(|f| f.locked)
}

fn yes() -> bool {
    true
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FeatureSetting {
    pub enabled: bool,
    /// Lowest tier that includes the feature; None = in no tier (site overrides only).
    pub tier: Option<Tier>,
    /// The AI assistant may use the feature's tools and prompt. Stored boards
    /// from before this switch mean "on".
    #[serde(default = "yes")]
    pub ai: bool,
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
    /// Block types the admin switched off (missing = on).
    pub blocks: BTreeMap<String, bool>,
    /// Widgets the admin switched off (missing = on).
    pub widgets: BTreeMap<String, bool>,
}

impl Default for PlanConfig {
    fn default() -> Self {
        Self {
            features: CATALOG.iter().map(|f| (f.key.to_string(), FeatureSetting { enabled: f.default_enabled, tier: Some(f.default_tier), ai: true })).collect(),
            ai_credits: TierCredits { basic: Some(1000), standard: None, premium: None },
            // Sites created before tiers keep everything they had.
            default_tier: Tier::Premium,
            blocks: BTreeMap::new(),
            widgets: BTreeMap::new(),
        }
    }
}

impl PlanConfig {
    /// The saved setting, or the catalog default for a feature added later.
    pub fn setting(&self, def: &FeatureDef) -> FeatureSetting {
        let saved = self.features.get(def.key).copied();
        if def.locked {
            // The website itself is what the assistant edits: its AI switch stays on.
            let ai = def.key == WEBSITE || saved.is_none_or(|s| s.ai);
            return FeatureSetting { enabled: true, tier: Some(Tier::Basic), ai };
        }
        saved.unwrap_or(FeatureSetting { enabled: def.default_enabled, tier: Some(def.default_tier), ai: true })
    }

    /// Keeps only catalog keys, fixes locked features and rejects negative credits.
    /// Block and widget switches keep only known keys that are off.
    pub fn normalized(mut self) -> Result<Self, String> {
        self.features = CATALOG.iter().map(|f| (f.key.to_string(), self.setting(f))).collect();
        self.blocks.retain(|k, on| !*on && crate::blocks::block_type(k).is_some());
        self.widgets.retain(|k, on| !*on && crate::widgets::widget(k).is_some());
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

    /// The subset of `resolve` the AI assistant may use.
    pub fn resolve_ai(&self, tier: Tier, overrides: &BTreeMap<String, bool>) -> Vec<String> {
        self.resolve(tier, overrides).into_iter().filter(|k| feature_def(k).is_some_and(|d| self.setting(d).ai)).collect()
    }

    pub fn block_on(&self, key: &str) -> bool {
        self.blocks.get(key).copied().unwrap_or(true)
    }

    pub fn widget_on(&self, key: &str) -> bool {
        self.widgets.get(key).copied().unwrap_or(true)
    }

    /// Block types switched off for every site.
    pub fn disabled_blocks(&self) -> Vec<String> {
        self.blocks.iter().filter(|(_, on)| !**on).map(|(k, _)| k.clone()).collect()
    }

    /// Widgets switched off for every site.
    pub fn disabled_widgets(&self) -> Vec<String> {
        self.widgets.iter().filter(|(_, on)| !**on).map(|(k, _)| k.clone()).collect()
    }
}
