use crate::fortune::*;

fn item(deck: &str, id: &str) -> DeckItem {
    DeckItem {
        deck: deck.into(),
        id: id.into(),
        text: "Trăm năm trong cõi người ta,\nChữ tài chữ mệnh khéo là ghét nhau.".into(),
        original: String::new(),
        author: "Nguyễn Du".into(),
        work: "Truyện Kiều".into(),
        source_url: "https://vi.wikisource.org/wiki/Truy%E1%BB%87n_Ki%E1%BB%81u".into(),
        lang: "vi".into(),
    }
}

fn set() -> FortuneSet {
    FortuneSet { minute: "2026-11-15T07:30".into(), items: vec![item("kieu", "kieu-0001"), item("kinh_dich", "kd-01-1"), item("thich_nhat_hanh", "tnh-001")] }
}

#[test]
fn a_set_passes_and_bad_ones_are_named() {
    assert!(validate_set(&set()).is_ok());
    let mut s = set();
    s.minute = "2026-11-15 07:30".into();
    assert!(validate_set(&s).unwrap_err().contains("minute"));
    let mut s = set();
    s.items.clear();
    assert!(validate_set(&s).is_err());
    let mut s = set();
    s.items[0].deck = "tarot".into();
    assert!(validate_set(&s).unwrap_err().contains("tarot"));
    let mut s = set();
    s.items[1].deck = "kieu".into();
    assert!(validate_set(&s).unwrap_err().contains("lặp"));
    let mut s = set();
    s.items[0].text = "x".repeat(TEXT_MAX_CHARS + 1);
    assert!(validate_set(&s).is_err());
    let mut s = set();
    s.items[0].source_url = "javascript:alert(1)".into();
    assert!(validate_set(&s).unwrap_err().contains("https"));
    let mut s = set();
    s.items[0].source_url = "https://x.vn/\"><script>".into();
    assert!(validate_set(&s).is_err());
    let mut s = set();
    s.items[0].source_url = String::new();
    assert!(validate_set(&s).is_ok(), "a source link is optional");
}

#[test]
fn the_site_deck_is_picked_and_a_missing_one_falls_back() {
    let s = set();
    assert_eq!(s.item_for("kinh_dich").unwrap().id, "kd-01-1");
    let only_kieu = FortuneSet { minute: s.minute.clone(), items: vec![item("kieu", "kieu-0002")] };
    assert_eq!(only_kieu.item_for("thich_nhat_hanh").unwrap().id, "kieu-0002");
}

#[test]
fn settings_default_to_kieu_and_on() {
    let f: Fortune = serde_json::from_str("{}").unwrap();
    assert_eq!(f, Fortune { enabled: true, deck: DEFAULT_DECK.into() });
    assert!(is_deck(DEFAULT_DECK) && is_deck("kinh_dich") && is_deck("thich_nhat_hanh") && is_deck("moi_ngay") && !is_deck("xam"));
    assert_eq!(deck_label("kieu"), Some("Truyện Kiều"));
}

#[test]
fn almanac_days_are_checked() {
    let mut day = AlmanacDay { date: "2026-11-12".into(), lunar_day: 4, lunar_month: 10, lunar_year: 2026, leap: false, year_can_chi: "Bính Ngọ".into(), day_can_chi: "Giáp Tý".into(), star: "Kim Quỹ".into(), star_good: true, truc: "Thành".into(), good_hours: vec!["Tý (23–1h)".into()], types: lunar_day_types(4, true) };
    assert!(validate_day(&day).is_ok());
    assert_eq!(day.lunar_short(false), "4/10");
    day.leap = true;
    assert_eq!(day.lunar_short(false), "4/10 nhuận");
    day.types = vec!["lucky".into()];
    assert!(validate_day(&day).is_err());
    assert_eq!(lunar_day_types(5, true), vec!["nguyet_ky", "hoang_dao"]);
    assert_eq!(lunar_day_types(7, false), vec!["tam_nuong", "hac_dao"]);
}
