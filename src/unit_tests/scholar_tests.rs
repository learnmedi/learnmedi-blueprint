use crate::scholar::*;
use serde_json::json;

#[test]
fn points_split_like_decision_37() {
    assert_eq!(personal_points(2.0, 1, AuthorRole::Equal), 2.0);
    assert_eq!(personal_points(2.0, 4, AuthorRole::Sole), 2.0);
    // Main author of 4: 2/3 + (4/3)/4 = 1.0
    assert_eq!(personal_points(2.0, 4, AuthorRole::Main), 1.0);
    // Co-author of 4: (4/3)/4 = 0.33
    assert_eq!(personal_points(2.0, 4, AuthorRole::Co), 0.33);
    assert_eq!(personal_points(2.0, 4, AuthorRole::Equal), 0.5);
    assert_eq!(personal_points(-1.0, 1, AuthorRole::Sole), 0.0);
    assert_eq!(count_authors("A. Nguyen, B. Tran; C. Le and D. Pham"), 4);
    assert_eq!(count_authors(""), 1);
}

#[test]
fn dois_and_orcids_are_normalized_and_checked() {
    assert_eq!(normalize_doi("https://doi.org/10.1038/NATURE12373").as_deref(), Some("10.1038/nature12373"));
    assert_eq!(normalize_doi("doi:10.1000/xyz123").as_deref(), Some("10.1000/xyz123"));
    assert_eq!(normalize_doi("10.1000"), None);
    assert_eq!(normalize_doi("11.1000/x"), None);
    assert_eq!(normalize_doi("10.1000/a b"), None);
    assert_eq!(normalize_orcid("https://orcid.org/0000-0002-1825-0097").as_deref(), Some("0000-0002-1825-0097"));
    assert_eq!(normalize_orcid("0000-0002-1694-233X").as_deref(), Some("0000-0002-1694-233X"));
    assert_eq!(normalize_orcid("0000-0002-1825-0098"), None);
    assert_eq!(normalize_orcid("1234"), None);
}

#[test]
fn h_index_and_summary() {
    assert_eq!(h_index(&[10, 8, 5, 4, 3]), 4);
    assert_eq!(h_index(&[0, 0]), 0);
    assert_eq!(h_index(&[]), 0);
    let papers = vec![
        PaperPoints { quartile: Some(Quartile::Q1), citations: 10, points: 2.0, authors: 1, role: AuthorRole::Sole, personal: 2.0, ..Default::default() },
        PaperPoints { quartile: Some(Quartile::Q3), citations: 1, personal: 0.5, ..Default::default() },
    ];
    let s = summarize(&papers, 2, Target::Pgs);
    assert_eq!((s.publications, s.q1_q2, s.citations, s.h_index), (2, 1, 11, 1));
    assert_eq!(s.points_total, 2.5);
    assert_eq!(s.progress_percent, 25.0);
    assert!(!s.enough);
    assert_eq!(Target::Gs.min_points(), 20.0);
}

#[test]
fn crossref_openalex_and_orcid_records_are_read() {
    let cr = json!({"message": {"DOI": "10.1/ABC", "title": ["A study"], "author": [{"given": "An", "family": "Nguyen"}, {"name": "Consortium X"}],
        "container-title": ["Journal of Tests"], "issued": {"date-parts": [[2023, 5]]}, "is-referenced-by-count": 7}});
    let w = parse_crossref(&cr).unwrap();
    assert_eq!(w.doi, "10.1/abc");
    assert_eq!(w.authors, vec!["An Nguyen", "Consortium X"]);
    assert_eq!((w.venue.as_str(), w.year, w.citations), ("Journal of Tests", Some(2023), Some(7)));
    assert_eq!(w.url, "https://doi.org/10.1/abc");
    assert!(parse_crossref(&json!({"message": {"DOI": "10.1/x", "title": []}})).is_none());
    assert_eq!(parse_openalex_citations(&json!({"cited_by_count": 12})), Some(12));
    let orcid = json!({"group": [
        {"external-ids": {"external-id": [{"external-id-type": "doi", "external-id-value": "10.1/A"}]}},
        {"external-ids": {"external-id": [{"external-id-type": "eid", "external-id-value": "2-s2.0-1"}, {"external-id-type": "doi", "external-id-value": "https://doi.org/10.1/a"}]}},
        {"external-ids": {"external-id": [{"external-id-type": "doi", "external-id-value": "10.2/b"}]}}
    ]});
    assert_eq!(parse_orcid_dois(&orcid), vec!["10.1/a", "10.2/b"]);
}

#[test]
fn settings_are_validated_and_badge_needs_enough_points() {
    let s = ScholarSettings { orcid: "0000-0002-1825-009".into(), ..Default::default() };
    assert!(s.validated().is_err());
    let s = ScholarSettings { orcid: " https://orcid.org/0000-0002-1825-0097 ".into(), field: "Kinh tế".into(), public_badge: true, ..Default::default() }.validated().unwrap();
    assert_eq!(s.orcid, "0000-0002-1825-0097");
    let mut sum = summarize(&[], 0, Target::Pgs);
    assert!(s.badge(&sum, false).is_none());
    sum.enough = true;
    assert_eq!(s.badge(&sum, false).unwrap(), "Đủ điểm công trình chuẩn PGS – ngành Kinh tế (tự đánh giá)");
    let old: ScholarSettings = serde_json::from_value(json!({})).unwrap();
    assert_eq!(old.target, Target::Pgs);
}
