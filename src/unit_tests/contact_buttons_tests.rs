use crate::contact_buttons::*;

#[test]
fn phone_numbers_keep_digits_and_a_leading_plus() {
    assert_eq!(normalize_phone("0903 434 340").unwrap(), "0903434340");
    assert_eq!(normalize_phone(" +84 (903) 434-340 ").unwrap(), "+84903434340");
    assert_eq!(normalize_phone("024.2242.2144").unwrap(), "02422422144");
    assert_eq!(normalize_phone("").unwrap(), "");
    assert!(normalize_phone("1234").is_err(), "too short");
    assert!(normalize_phone("0903abc340").is_err());
    assert!(normalize_phone("javascript:alert(1)").is_err());
    assert!(normalize_phone("0903434340\"><script>").is_err());
}

#[test]
fn zalo_takes_a_number_or_a_zalo_me_link() {
    assert_eq!(normalize_zalo("0912 015 746").unwrap(), "0912015746");
    assert_eq!(normalize_zalo("https://zalo.me/0912015746").unwrap(), "0912015746");
    assert_eq!(normalize_zalo("Zalo.me/0912015746/").unwrap(), "0912015746");
    assert_eq!(normalize_zalo("+84 912 015 746").unwrap(), "84912015746");
    assert_eq!(normalize_zalo("  ").unwrap(), "");
    assert!(normalize_zalo("https://zalo.me/g/abcdef").is_err(), "group links are not a number");
    assert!(normalize_zalo("https://evil.example/0912015746").is_err());
    assert!(normalize_zalo("<div class=\"zalo-chat-widget\" data-oaid=\"1\">").is_err(), "no embed code");
}

#[test]
fn messenger_takes_a_page_name_or_link() {
    assert_eq!(normalize_messenger("nhakhoa.hangtre").unwrap(), "nhakhoa.hangtre");
    assert_eq!(normalize_messenger("https://m.me/nhakhoa.hangtre").unwrap(), "nhakhoa.hangtre");
    assert_eq!(normalize_messenger("https://www.facebook.com/NhaKhoaHangTre/?ref=x").unwrap(), "NhaKhoaHangTre");
    assert_eq!(normalize_messenger("100064512345678").unwrap(), "100064512345678");
    assert_eq!(normalize_messenger("").unwrap(), "");
    assert!(normalize_messenger("https://www.facebook.com/groups/abc").is_err());
    assert!(normalize_messenger("a b").is_err());
    assert!(normalize_messenger("x\"onmouseover=\"alert(1)").is_err());
}

#[test]
fn links_and_visibility() {
    let mut c = ContactButtons { enabled: true, zalo: "0912015746".into(), phone: "0903434340".into(), messenger: String::new() };
    assert_eq!(c.zalo_url().as_deref(), Some("https://zalo.me/0912015746"));
    assert_eq!(c.phone_url().as_deref(), Some("tel:0903434340"));
    assert_eq!(c.messenger_url(), None);
    assert!(c.visible());
    c.enabled = false;
    assert!(!c.visible());
    assert!(!ContactButtons { enabled: true, ..Default::default() }.visible(), "on but no number: nothing to show");
    // A site saved before the feature reads as switched off.
    let old: ContactButtons = serde_json::from_value(serde_json::json!({})).unwrap();
    assert_eq!(old, ContactButtons::default());
}

#[test]
fn the_widget_has_its_feature() {
    let owner = crate::features::widget_owner(crate::widgets::CONTACT_BUTTONS).expect("owned");
    assert_eq!(owner.key, crate::plans::CONTACT_BUTTONS);
    assert!(crate::features::tool_names().contains(&"set_contact_buttons"));
    assert!(!crate::features::is_dangerous("set_contact_buttons"));
}
