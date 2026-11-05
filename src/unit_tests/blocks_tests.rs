use crate::blocks::*;
use serde_json::json;

#[test]
fn every_type_validates_its_defaults() {
    for bt in REGISTRY {
        let (p, _) = validate_props(bt.key, &default_props(bt)).unwrap_or_else(|e| panic!("{}: {e}", bt.key));
        assert!(p.is_object());
    }
}

#[test]
fn unknown_type_and_field_rejected() {
    assert!(validate_props("nope", &json!({})).is_err());
    let err = validate_props("text", &json!({"body": "x", "colour": "red"})).unwrap_err();
    assert!(err.contains("colour"), "{err}");
}

#[test]
fn defaults_fill_missing_fields() {
    let (p, _) = validate_props("hero", &json!({"title": "Hi"})).unwrap();
    assert_eq!(p["layout"], "split");
    assert_eq!(p["buttons"], json!([]));
}

#[test]
fn javascript_urls_are_rejected() {
    let r = validate_props("buttons", &json!({"buttons": [{"label": "x", "url": "javascript:alert(1)"}]}));
    assert!(r.is_err());
    let r = validate_props("buttons", &json!({"buttons": [{"label": "x", "url": "//evil.com"}]}));
    assert!(r.is_err());
    assert!(validate_props("buttons", &json!({"buttons": [{"label": "x", "url": "/about"}]})).is_ok());
}

#[test]
fn asset_refs_are_collected() {
    let (_, assets) = validate_props("gallery", &json!({"images": [{"image": "abc123"}, {"image": "https://x.com/a.jpg"}]})).unwrap();
    assert_eq!(assets, vec!["abc123".to_string()]);
}

#[test]
fn image_must_be_asset_or_https() {
    assert!(validate_props("image", &json!({"image": "http://x.com/a.jpg"})).is_err());
    assert!(validate_props("image", &json!({"image": "file:///etc/passwd"})).is_err());
}

#[test]
fn numbers_are_clamped_and_coerced() {
    let (p, _) = validate_props("features", &json!({"items": [], "columns": "9"})).unwrap();
    assert_eq!(p["columns"], 4);
}

#[test]
fn text_too_long_is_error() {
    let long = "a".repeat(500);
    assert!(validate_props("heading", &json!({"title": long})).is_err());
}

#[test]
fn style_patch_validates() {
    let mut st = BlockStyle::default();
    apply_style_patch(&mut st, &json!({"background": "xanh", "padding": "xl", "animation": "zoom-in"})).unwrap();
    assert_eq!(st.background, "#2563eb");
    assert_eq!(st.padding, "xl");
    assert!(apply_style_patch(&mut st, &json!({"padding": "huge"})).is_err());
    assert!(apply_style_patch(&mut st, &json!({"background": "url(javascript:1)"})).is_err());
    assert!(apply_style_patch(&mut st, &json!({"z": "1"})).is_err());
}

#[test]
fn merge_replaces_top_level_keys() {
    let m = merge_props(&json!({"a": 1, "b": [1, 2]}), &json!({"b": [3]}));
    assert_eq!(m, json!({"a": 1, "b": [3]}));
}

#[test]
fn prompt_catalog_mentions_every_type() {
    let c = catalog_for_prompt();
    for k in type_keys() {
        assert!(c.contains(&format!("- {k} ")), "{k}");
    }
}

