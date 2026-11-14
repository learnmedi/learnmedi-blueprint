use crate::plans::*;
use std::collections::BTreeMap;

#[test]
fn teaching_is_off_by_default() {
    let cfg = PlanConfig::default();
    let premium = cfg.resolve(Tier::Premium, &BTreeMap::new());
    assert!(!premium.contains(&TEACHING.to_string()));
    assert!(premium.contains(&SURVEYS.to_string()));
}

#[test]
fn tiers_are_cumulative() {
    let cfg = PlanConfig::default();
    let basic = cfg.resolve(Tier::Basic, &BTreeMap::new());
    let standard = cfg.resolve(Tier::Standard, &BTreeMap::new());
    let premium = cfg.resolve(Tier::Premium, &BTreeMap::new());
    assert!(basic.contains(&AI_ASSISTANT.to_string()));
    assert!(!basic.contains(&BLOG.to_string()));
    assert!(basic.iter().all(|k| standard.contains(k)));
    assert!(standard.contains(&SURVEYS.to_string()) && standard.contains(&COMMUNITY.to_string()));
    assert!(!standard.contains(&PRODUCTS.to_string()));
    assert!(standard.iter().all(|k| premium.contains(k)));
    assert!(premium.contains(&PRODUCTS.to_string()));
}

#[test]
fn global_off_beats_site_override() {
    let mut cfg = PlanConfig::default();
    let grant = BTreeMap::from([(TEACHING.to_string(), true), (PRODUCTS.to_string(), true)]);
    assert!(!cfg.resolve(Tier::Basic, &grant).contains(&TEACHING.to_string()));
    assert!(cfg.resolve(Tier::Basic, &grant).contains(&PRODUCTS.to_string()));
    assert!(!cfg.resolve(Tier::Basic, &BTreeMap::new()).contains(&PRODUCTS.to_string()));
    cfg.features.insert(TEACHING.into(), FeatureSetting::new(true, None, true));
    assert!(cfg.resolve(Tier::Basic, &grant).contains(&TEACHING.to_string()));
    assert!(!cfg.resolve(Tier::Premium, &BTreeMap::new()).contains(&TEACHING.to_string()));
}

#[test]
fn site_deny_beats_tier() {
    let cfg = PlanConfig::default();
    let deny = BTreeMap::from([(BLOG.to_string(), false)]);
    assert!(!cfg.resolve(Tier::Premium, &deny).contains(&BLOG.to_string()));
}

#[test]
fn website_cannot_be_switched_off() {
    let mut cfg = PlanConfig::default();
    cfg.features.insert(WEBSITE.into(), FeatureSetting::new(false, None, true));
    cfg.features.insert("unknown".into(), FeatureSetting::new(true, None, true));
    let cfg = cfg.normalized().unwrap();
    assert!(!cfg.features.contains_key("unknown"));
    let deny = BTreeMap::from([(WEBSITE.to_string(), false)]);
    assert!(cfg.resolve(Tier::Basic, &deny).contains(&WEBSITE.to_string()));
}

#[test]
fn negative_credits_are_rejected() {
    let mut cfg = PlanConfig::default();
    cfg.ai_credits.premium = Some(-1);
    assert!(cfg.normalized().is_err());
    assert_eq!(PlanConfig::default().ai_credits.get(Tier::Basic), Some(1000));
}
