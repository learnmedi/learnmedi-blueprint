use crate::theme::*;
use serde_json::json;

#[test]
fn patch_colors_and_fonts() {
    let mut t = Theme::default();
    let ch = apply_patch(&mut t, &json!({"colors": {"primary": "#123456"}, "font_heading": "inter"})).unwrap();
    assert_eq!(t.colors.primary, "#123456");
    assert_eq!(t.font_heading, "Inter");
    assert_eq!(ch.len(), 2);
}

#[test]
fn patch_rejects_bad_values() {
    let mut t = Theme::default();
    assert!(apply_patch(&mut t, &json!({"colors": {"primary": "red; background:url(x)"}})).is_err());
    assert!(apply_patch(&mut t, &json!({"font_body": "Comic Sans"})).is_err());
    assert!(apply_patch(&mut t, &json!({"radius": "huge"})).is_err());
    assert!(apply_patch(&mut t, &json!({"wat": 1})).is_err());
}

#[test]
fn named_colours_work() {
    let mut t = Theme::default();
    apply_patch(&mut t, &json!({"colors": {"accent": "xanh lá"}})).unwrap();
    assert_eq!(t.colors.accent, "#16a34a");
}

#[test]
fn a_preset_comes_from_the_catalog_lookup() {
    let ocean = Theme { font_body: "Inter".into(), ..Theme::default() };
    let mut t = Theme::default();
    apply_patch_with(&mut t, &json!({"preset": "ocean"}), &|k| (k == "ocean").then(|| ocean.clone())).unwrap();
    assert_eq!(t.font_body, "Inter");
    assert!(apply_patch(&mut t, &json!({"preset": "ocean"})).is_err(), "no catalog, no preset");
}

#[test]
fn validate_accepts_the_default_and_rejects_bad_fields() {
    assert!(validate(&Theme::default()).is_ok());
    assert!(validate(&Theme { font_heading: "Comic Sans".into(), ..Theme::default() }).is_err());
    assert!(validate(&Theme { base_font_size: 40, ..Theme::default() }).is_err());
}

#[test]
fn effects_follow_the_motion_level() {
    let mut t = Theme { motion: "burst".into(), ..Theme::default() };
    apply_patch(&mut t, &json!({"fx": ["iris", "words", "iris"]})).unwrap();
    assert_eq!(t.fx, vec!["iris", "words"], "no duplicates, order kept");
    assert!(validate(&t).is_ok());
    assert!(apply_patch(&mut t, &json!({"fx": ["nope"]})).is_err());
    assert!(apply_patch(&mut t, &json!({"fx": ["a", "b", "c", "d", "e", "f", "g", "h"]})).is_err());
    // Lowering the motion drops the effects it no longer allows.
    apply_patch(&mut t, &json!({"motion": "rich"})).unwrap();
    assert_eq!(t.fx, vec!["words"]);
    assert!(apply_patch(&mut t, &json!({"fx": ["iris"]})).is_err(), "iris needs burst");
    apply_patch(&mut t, &json!({"motion": "burst", "fx": ["iris"]})).unwrap();
    assert_eq!(t.fx, vec!["iris"]);
    let mut bad = t.clone();
    bad.fx = vec!["iris".into()];
    bad.motion = "calm".into();
    assert!(validate(&bad).is_err());
}

#[test]
fn looks_without_effects_keep_the_old_shared_set() {
    let rich = Theme { motion: "rich".into(), ..Theme::default() };
    assert!(effective_fx(&rich).contains(&"words") && !effective_fx(&rich).contains(&"curtain"));
    let burst = Theme { motion: "burst".into(), ..Theme::default() };
    assert!(effective_fx(&burst).contains(&"curtain"));
    let own = Theme { motion: "burst".into(), fx: vec!["blinds".into()], ..Theme::default() };
    assert_eq!(effective_fx(&own), vec!["blinds"]);
    assert!(effective_fx(&Theme::default()).is_empty());
}

#[test]
fn every_effect_key_is_unique_and_every_menu_style_is_known() {
    let mut keys = std::collections::HashSet::new();
    for (k, min, _) in FX {
        assert!(keys.insert(*k), "duplicate fx {k}");
        assert!(MOTIONS.contains(min));
    }
    for o in OPENERS {
        assert!(FX.iter().any(|(k, _, _)| k == o), "opener {o} is an effect");
    }
    let mut t = Theme::default();
    for h in HEADER_STYLES {
        apply_patch(&mut t, &json!({"header_style": h})).unwrap();
    }
}
