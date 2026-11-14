use crate::decks::*;
use crate::plans::{PlanConfig, Tier};
use serde_json::json;
use std::collections::BTreeMap;

fn deck() -> Deck {
    parse(r#"
meta: {title: Hội thảo, start_time: "09:00", duration_min: 15}
segments: [{id: s1, name: Mở đầu, min: 15}]
data:
  compare: [{id: c1, topic: Kinh phí, status: Chưa có, dec: d1}]
slides:
  - {kind: hero, title: "Kế hoạch thư viện số", stats: [{value: "3", label: mục tiêu}]}
  - {id: overview, kind: matrix, dataset: compare, group_key: status, label_key: topic, table_slide: tbl}
  - {id: tbl, kind: table, dataset: compare, columns: [{key: topic, label: Nội dung}, {key: dec, label: QĐ, format: goto}]}
  - {id: d1, kind: decision, seg: s1, code: D1, refs: c1, refs_dataset: compare, options: [{key: A, text: Giữ}, {key: B, text: Bỏ}], hint: "B"}
  - {kind: summary, title: Kết luận}
"#).unwrap()
}

#[test]
fn every_kind_example_validates() {
    for k in SLIDE_KINDS.iter() {
        let mut s = slide_from_value(&k.example).unwrap_or_else(|e| panic!("{}: {e}", k.key));
        assert_eq!(s.kind, k.key);
        validate_slide(&mut s, &k.key).unwrap_or_else(|e| panic!("{}: {e}", k.key));
    }
}

#[test]
fn a_deck_validates_fills_ids_and_lints_clean() {
    let (d, assets) = validate_deck(&deck()).unwrap();
    assert!(assets.is_empty());
    assert_eq!(d.slides[0].id, "ke_hoach_thu_vien_so");
    assert_eq!(d.slides[4].id, "ket_luan");
    assert_eq!(d.slides[3].props["options"][0]["note"], "", "defaults are filled");
    let l = lint(&d);
    assert!(l.iter().all(|x| x.level != "error"), "{l:?}");
    // Round trip through the stored YAML.
    let back = parse(&to_yaml(&d)).unwrap();
    assert_eq!(validate_deck(&back).unwrap().0, d);
}

#[test]
fn bad_input_is_named() {
    let mut d = deck();
    d.slides[1].props.insert("nope".into(), json!(1));
    assert!(validate_deck(&d).unwrap_err().contains("nope"));
    let mut d = deck();
    d.slides[2].id = "overview".into();
    assert!(validate_deck(&d).unwrap_err().contains("trùng"));
    let mut d = deck();
    d.slides[0].kind = "html".into();
    assert!(validate_deck(&d).unwrap_err().contains("kind"));
    let mut d = deck();
    d.slides[3].seg = "s9".into();
    assert!(validate_deck(&d).is_err());
    assert!(parse("meta: {title: x, script: y}").is_err(), "unknown meta keys are refused");
}

#[test]
fn lint_finds_broken_links_and_missing_data() {
    let mut d = deck();
    d.slides[1].props.insert("table_slide".into(), json!("nowhere"));
    d.slides[2].props.insert("dataset".into(), json!("missing"));
    let (d, _) = validate_deck(&d).unwrap();
    let l = lint(&d);
    assert!(l.iter().any(|x| x.level == "error" && x.message.contains("nowhere")));
    assert!(l.iter().any(|x| x.level == "error" && x.message.contains("missing")));
}

#[test]
fn theme_is_checked_for_contrast_fonts_and_presets() {
    let mut t = DeckTheme::default();
    t.ink = "#eeeeee".into();
    assert!(validate_theme(&t).unwrap_err().contains("khó đọc"));
    let mut t = DeckTheme::default();
    t.heading_font = "Comic Sans".into();
    assert!(validate_theme(&t).is_err());
    let mut t = DeckTheme::default();
    t.primary = "red; background:url(x)".into();
    assert!(validate_theme(&t).is_err());
    let mut t = DeckTheme::default();
    t.preset = "midnight".into();
    t.heading_font = "lora".into();
    let t = validate_theme(&t).unwrap();
    assert_eq!(t.heading_font, "Lora");
    for p in PRESETS {
        let mut t = DeckTheme::default();
        t.preset = p.key.into();
        validate_theme(&t).unwrap_or_else(|e| panic!("{}: {e}", p.key));
    }
}

#[test]
fn patch_keeps_other_fields_and_drops_fields_of_an_old_kind() {
    let (d, _) = validate_deck(&deck()).unwrap();
    let s = patch_slide(&d.slides[3], &json!({"title": "Mới", "hint": null})).unwrap();
    assert_eq!(s.title, "Mới");
    assert_eq!(s.props["code"], "D1");
    assert!(!s.props.contains_key("hint"));
    let s = patch_slide(&d.slides[3], &json!({"kind": "bullets", "items": [{"text": "a"}]})).unwrap();
    assert!(!s.props.contains_key("options"));
}

#[test]
fn channel_switches_default_and_resolve() {
    let cfg = PlanConfig::default();
    let decks = crate::plans::feature_def(crate::plans::DECKS).unwrap();
    let s = cfg.setting(decks);
    assert!(!s.ai && s.share, "decks: sidebar AI off, outside AI on by default");
    let mut cfg = PlanConfig::default();
    cfg.tools.insert("delete_deck".into(), crate::plans::ToolSwitch { ai: true, share: false });
    cfg.tools.insert("nope".into(), crate::plans::ToolSwitch { ai: false, share: false });
    cfg.tools.insert("get_deck".into(), crate::plans::ToolSwitch { ai: true, share: true });
    let cfg = cfg.normalized().unwrap();
    assert_eq!(cfg.share_blocked_tools(), vec!["delete_deck".to_string()]);
    assert!(cfg.ai_blocked_tools().is_empty());
    assert_eq!(cfg.tools.len(), 1);
    let all = cfg.resolve(Tier::Premium, &BTreeMap::new());
    let ai = cfg.resolve_ai(Tier::Premium, &BTreeMap::new());
    let share = cfg.resolve_share(Tier::Premium, &BTreeMap::new());
    let k = crate::plans::DECKS.to_string();
    assert!(all.contains(&k) && !ai.contains(&k) && share.contains(&k));
    let old: crate::plans::FeatureSetting = serde_json::from_str(r#"{"enabled":true,"tier":"basic","ai":true}"#).unwrap();
    assert!(old.share);
}
