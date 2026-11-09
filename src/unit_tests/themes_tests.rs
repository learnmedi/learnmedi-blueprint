use crate::blocks::new_block;
use crate::themes::*;
use serde_json::json;

fn entry() -> ThemeEntry {
    serde_json::from_value(json!({
        "key": "test-showcase", "label": "Thử", "category": "showcase",
        "look": crate::theme::Theme::default(),
        "pages": [{"slug": "", "title": "Trang chủ", "blocks": [
            {"slot": "opener", "type": "hero", "props": {"title": "{{name}}", "layout": "centered"}, "style": {"padding": "xl"}},
            {"slot": "about", "type": "media_text", "props": {"title": "Giới thiệu", "image_side": "left"}},
            {"type": "divider"},
            {"slot": "contact", "type": "cta", "props": {"title": "Liên hệ", "layout": "centered"}}
        ], "sections": [{"start": 0, "layout": "stack", "height": "screen"}]}]
    })).unwrap()
}

#[test]
fn a_starter_builds_with_the_site_name_and_validates() {
    let e = entry();
    validate(&e).unwrap();
    let page = e.pages[0].build("Cô Lan").unwrap();
    assert_eq!(page.blocks[0].props["title"], "Cô Lan");
    assert_eq!(page.slots, ["opener", "about", "", "contact"]);
    assert_eq!(page.sections[0].start_block_id, page.blocks[0].id);
}

#[test]
fn bad_entries_are_refused() {
    let mut e = entry();
    e.key = "Có dấu".into();
    assert!(validate(&e).is_err());
    let mut e = entry();
    e.pages[0].blocks[0].slot = "nowhere".into();
    assert!(validate(&e).is_err());
    let mut e = entry();
    e.pages[0].blocks[1].props = json!({"images": [{"image": "abc123def456"}]});
    assert!(validate(&e).is_err(), "a starter may not point at uploaded files");
    let mut e = entry();
    e.pages[0].slug = "gioi-thieu".into();
    assert!(validate(&e).is_err(), "home page first");
}

fn live(kind: &str, props: serde_json::Value) -> crate::blocks::Block {
    new_block(kind, &props).unwrap().0
}

#[test]
fn slots_are_guessed_from_block_types_and_order() {
    let blocks = vec![
        live("hero", json!({"title": "Lan"})),
        live("media_text", json!({"title": "Về tôi"})),
        live("post_list", json!({})),
        live("cta", json!({"title": "Cộng đồng", "buttons": [{"label": "Xem", "url": "/community"}]})),
        live("cta", json!({"title": "Liên hệ"})),
    ];
    assert_eq!(infer_slots(&blocks), ["opener", "about", "posts", "community", "contact"]);
}

#[test]
fn port_keeps_the_owners_content_and_loses_nothing() {
    let current = vec![
        live("hero", json!({"title": "TS. Lan", "subtitle": "Giáo dục", "layout": "split"})),
        live("text", json!({"body": "Tôi dạy văn."})),
        live("gallery", json!({})),
        live("cta", json!({"title": "Viết cho tôi"})),
    ];
    let page = entry().pages[0].build("x").unwrap();
    let (blocks, sections, plan) = port_page(&current, &page);
    assert_eq!(blocks[0].props["title"], "TS. Lan", "words stay");
    assert_eq!(blocks[0].props["layout"], "centered", "layout comes from the theme");
    assert_eq!(blocks[0].style.padding, "xl");
    assert_eq!(blocks[1].kind, "media_text");
    assert_eq!(blocks[1].props["body"], "Tôi dạy văn.", "text carried into another block type");
    assert_eq!(blocks[3].props["title"], "Viết cho tôi");
    assert_eq!(blocks.last().unwrap().kind, "gallery", "a block with no place is kept at the end");
    assert_eq!(plan.kept_at_end.len(), 1);
    assert_eq!(sections[0].start_block_id, blocks[0].id);
}

#[test]
fn a_saved_starter_drops_words_files_and_outside_links() {
    let b = live("hero", json!({"title": "Trần Thị Hoàng Yến", "subtitle": "Tiến sĩ", "layout": "split",
        "image": "abc123def456", "buttons": [{"label": "Sách", "url": "/sach"}, {"label": "Scholar", "url": "https://scholar.google.com"}]}));
    let s = strip_block(&b, "opener");
    assert_eq!(s.props["title"], "{{name}}");
    assert_ne!(s.props["subtitle"], "Tiến sĩ");
    assert_eq!(s.props["layout"], "split");
    assert_eq!(s.props["image"], "");
    assert_eq!(s.props["buttons"][0]["url"], "/sach");
    assert_eq!(s.props["buttons"][1]["url"], "#", "an outside link falls back to the default");
    assert!(!s.props.to_string().contains("Yến"));
}

#[test]
fn a_placeholder_fits_its_field() {
    let b = live("rows", json!({"items": [{"index": "01", "title": "Một dự án"}]}));
    let s = strip_block(&b, "work");
    assert!(crate::blocks::validate_props("rows", &s.props).is_ok(), "{}", s.props);
}
