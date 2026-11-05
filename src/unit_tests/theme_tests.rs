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
fn presets_exist() {
    for p in PRESETS {
        assert!(is_hex_color(&(p.theme)().colors.primary));
    }
    let mut t = Theme::default();
    apply_patch(&mut t, &json!({"preset": "ocean"})).unwrap();
    assert_eq!(t.font_body, "Inter");
}
