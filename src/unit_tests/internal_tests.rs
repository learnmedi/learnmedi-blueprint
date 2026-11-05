use crate::internal::*;

#[test]
fn domains_normalise() {
    assert_eq!(normalize_domain("https://www.Co-Lan.vn/abc?x").as_deref(), Some("www.co-lan.vn"));
    assert_eq!(normalize_domain("colan.vn:443").as_deref(), Some("colan.vn"));
    assert!(normalize_domain("localhost").is_none());
    assert!(normalize_domain("1.2.3.4").is_none());
    assert!(normalize_domain("bad_domain.com").is_none());
    assert!(normalize_domain("-x.com").is_none());
}

#[test]
fn challenge_is_stable_and_case_insensitive() {
    assert_eq!(domain_challenge("k", "A.com"), domain_challenge("k", "a.com"));
    assert_ne!(domain_challenge("k", "a.com"), domain_challenge("k2", "a.com"));
}

#[test]
fn slugs() {
    assert!(validate_slug("co-lan").is_ok());
    assert!(validate_slug("ab").is_err());
    assert!(validate_slug("admin").is_err());
    assert!(validate_slug("Co-Lan").is_err());
    assert!(validate_slug("-x-").is_err());
}
