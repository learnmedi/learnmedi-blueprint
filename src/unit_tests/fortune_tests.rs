use crate::fortune::*;

fn q(text: &str, kind: &str) -> Quote {
    let rank = ["thuong", "trung", "ha"][text.len() % 3];
    Quote { text: text.into(), kind: kind.into(), rank: rank.into() }
}

/// A valid full set: the minimum of each type, the rest `normal`.
fn full_set() -> Vec<Quote> {
    let mut out = Vec::new();
    for (key, _, _, min) in DAY_TYPES {
        for i in 0..*min {
            out.push(q(&format!("Câu {key} số {i}"), key));
        }
    }
    let mut i = 0;
    while out.len() < QUOTES_TOTAL {
        out.push(q(&format!("Câu thêm {i}"), "normal"));
        i += 1;
    }
    out
}

#[test]
fn a_full_set_passes_and_bad_ones_are_named() {
    assert_eq!(validate_quotes(&full_set(), &["Trần Hoàng Yến"]).unwrap().len(), QUOTES_TOTAL);
    let mut short = full_set();
    short.pop();
    assert!(validate_quotes(&short, &[]).unwrap_err().contains("đúng 120"));
    let mut bad = full_set();
    bad[0] = q("Đi chơi đi", "lucky");
    assert!(validate_quotes(&bad, &[]).unwrap_err().starts_with("Câu 1"));
    let mut unranked = full_set();
    unranked[2].rank = String::new();
    assert!(validate_quotes(&unranked, &[]).unwrap_err().contains("rank"));
    let mut one_rank = full_set();
    one_rank.iter_mut().for_each(|x| x.rank = "trung".into());
    assert!(validate_quotes(&one_rank, &[]).unwrap_err().contains("hạng"));
    let mut link = full_set();
    link[3] = q("Xem https://x.vn", "normal");
    assert!(validate_quotes(&link, &[]).is_err());
}

#[test]
fn text_is_only_the_reading() {
    for bad in ["Quẻ số 12: Trời yên biển lặng.", "Quẻ 12. Trời yên biển lặng.", "Quẻ Thượng. Trời yên biển lặng.", "Stick no. 3: calm seas.", "Trời yên biển lặng; tin vui sắp tới."] {
        let mut set = full_set();
        set[0].text = bad.into();
        assert!(validate_quotes(&set, &[]).unwrap_err().starts_with("Câu 1"), "{bad}");
    }
    let mut ok = full_set();
    ok[0].text = "Mưa dầm thấm lâu. Chuyện học hành ít hôm nữa sẽ thấy rõ kết quả, quẻ này khuyên cứ làm tiếp.".into();
    assert!(validate_quotes(&ok, &[]).is_ok());
    let with_n: Quote = serde_json::from_str(r#"{"n":12,"text":"a","type":"normal","rank":"ha"}"#).unwrap();
    assert_eq!(with_n.text, "a");
    let b = brief("", "vi");
    assert!(b.get("what_a_slip_is").is_none() && b["bad_examples"][0]["why"].is_string());
    assert!(DAY_TYPES.iter().all(|t| !t.2.contains(';')));
}

#[test]
fn quotes_never_speak_as_the_owner() {
    let mut first = full_set();
    first[5] = q("Hôm nay tôi chúc bạn vui", "normal");
    assert!(validate_quotes(&first, &[]).unwrap_err().contains("ngôi thứ nhất"));
    let mut named = full_set();
    named[5] = q("Như cô Hoàng Yến hay nói, hãy kiên nhẫn", "normal");
    assert!(validate_quotes(&named, &["Trần Hoàng Yến", "Hoàng Yến"]).unwrap_err().contains("tên"));
}

#[test]
fn every_day_gets_a_quote_of_its_main_type() {
    let quotes = validate_quotes(&full_set(), &[]).unwrap();
    // Day 5 is Nguyệt Kỵ even when the star is good: never a hoàng đạo quote.
    let day5 = lunar_day_types(5, true);
    assert_eq!(main_type(&day5), "nguyet_ky");
    for v in ["a", "b", "c", "d", "e"] {
        assert_eq!(pick(&quotes, &day5, "s", "2026-11-23", v).unwrap().kind, "nguyet_ky");
    }
    assert_eq!(main_type(&lunar_day_types(7, true)), "tam_nuong");
    assert_eq!(main_type(&lunar_day_types(9, false)), "hac_dao");
    assert_eq!(main_type(&lunar_day_types(9, true)), "hoang_dao");
    // Every lunar day, good or bad star, finds a quote.
    for d in 1..=30 {
        for good in [true, false] {
            assert!(pick(&quotes, &lunar_day_types(d, good), "s", "x", "v").is_some());
        }
    }
    // Same visitor, same day: same quote.
    let t = lunar_day_types(9, true);
    assert_eq!(pick(&quotes, &t, "s", "2026-11-12", "u"), pick(&quotes, &t, "s", "2026-11-12", "u"));
    // Only normal quotes: still one, on any day.
    let normal: Vec<Quote> = quotes.iter().filter(|x| x.kind == "normal").cloned().collect();
    assert_eq!(pick(&normal, &day5, "s", "x", "v").unwrap().kind, "normal");
}

#[test]
fn days_bios_and_switches_are_checked() {
    let mut day = AlmanacDay { date: "2026-11-12".into(), lunar_day: 4, lunar_month: 10, lunar_year: 2026, leap: false, year_can_chi: "Bính Ngọ".into(), day_can_chi: "Giáp Tý".into(), star: "Kim Quỹ".into(), star_good: true, truc: "Thành".into(), good_hours: vec!["Tý (23–1h)".into()], types: lunar_day_types(4, true) };
    assert!(validate_day(&day).is_ok());
    day.types = vec!["lucky".into()];
    assert!(validate_day(&day).is_err());
    assert!(validate_bio(&"chữ ".repeat(150)).is_ok());
    assert!(validate_bio(&"chữ ".repeat(151)).is_err());
    let mut p = Personality::default();
    assert!(p.fortune_enabled, "on by default");
    assert!(serde_json::from_str::<Personality>("{}").unwrap().fortune_enabled, "old sites: on");
    assert!(!p.envelope_on("vi"), "no quotes = no envelope");
    p.quotes.insert("vi".into(), full_set());
    p.quotes_bio_hash = bio_hash(&p.bio);
    assert!(p.envelope_on("vi") && !p.quotes_stale());
    p.bio = "đổi".into();
    assert!(p.quotes_stale());
    // Sets saved before fortune sticks (old hash, no rank) read as stale.
    p.quotes_bio_hash = format!("{:016x}", 0u64);
    assert!(p.quotes_stale());
    assert_eq!(serde_json::from_str::<Quote>(r#"{"text":"a","type":"normal"}"#).unwrap().rank, "");
}
