use crate::util::*;

#[test]
fn slugify_vietnamese() {
    assert_eq!(slugify("Giới thiệu về Tôi!"), "gioi-thieu-ve-toi");
    assert_eq!(slugify("Đường đến trường"), "duong-den-truong");
    assert_eq!(slugify("  --  "), "");
}

#[test]
fn escape() {
    assert_eq!(esc("<a href=\"x\">'"), "&lt;a href=&quot;x&quot;&gt;&#39;");
}

#[test]
fn url_roundtrip() {
    let s = "a b/c?d=é&";
    assert_eq!(url_decode(&url_encode(s)), s);
    assert_eq!(url_decode("%zz%41+"), "%zzA ");
}

#[test]
fn vnd() {
    assert_eq!(format_vnd(1500000), "1.500.000 ₫");
    assert_eq!(format_vnd(0), "0 ₫");
}

#[test]
fn codes_have_no_lookalikes() {
    let c = random_code(200);
    assert!(!c.contains('0') && !c.contains('O') && !c.contains('I') && !c.contains('1'));
}
