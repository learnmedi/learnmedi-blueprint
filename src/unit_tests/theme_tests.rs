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
