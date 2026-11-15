use crate::journals::*;
use crate::scholar::*;
use serde_json::json;

fn t(max: f64, from: Option<i32>, until: Option<i32>) -> Tier {
    Tier { max, from, until }
}

#[test]
fn fields_are_found_from_what_people_type() {
    assert_eq!(nganh("Kinh tế").map(|n| n.key), Some("kinh_te"));
    assert_eq!(nganh("  kinh te ").map(|n| n.key), Some("kinh_te"));
    assert_eq!(nganh("HĐGS ngành Y học").map(|n| n.key), Some("y_hoc"));
    assert_eq!(nganh("CNTT").map(|n| n.key), Some("cong_nghe_thong_tin"));
    assert_eq!(nganh("Hoá học").map(|n| n.key), Some("hoa_hoc_cong_nghe_thuc_pham"));
    assert_eq!(nganh("Văn học").map(|n| n.key), Some("van_hoc"));
    assert_eq!(nganh("Văn hoá").map(|n| n.key), Some("van_hoa_nghe_thuat_the_duc_the_thao"));
    assert_eq!(nganh("Khoa học giáo dục").map(|n| n.key), Some("khoa_hoc_giao_duc"));
    assert_eq!(nganh("vat_ly").map(|n| n.key), Some("vat_ly"));
    assert!(nganh("Kỹ thuật").is_none());
    assert!(nganh("").is_none());
    assert_eq!(NGANH.len(), 28);
}

#[test]
fn issns_are_checked() {
    assert_eq!(normalize_issn("2615-9198").as_deref(), Some("2615-9198"));
    assert_eq!(normalize_issn("p-1859-378x").as_deref(), Some("1859-378X"));
    assert_eq!(normalize_issn("e-2615-9929").as_deref(), Some("2615-9929"));
    assert_eq!(normalize_issn("1859-3780"), None);
    assert_eq!(normalize_issn("1234"), None);
}

#[test]
fn points_follow_the_publication_year() {
    // Nhi khoa (Y học): 0–0,5 / 0–0,25 từ 2020 / 0–0,5 từ 2021 / 0–0,25 từ 2022 / 0–0,5 từ 2025
    let nhi = [t(0.5, None, None), t(0.25, Some(2020), None), t(0.5, Some(2021), None), t(0.25, Some(2022), None), t(0.5, Some(2025), None)];
    assert_eq!(max_for_year(&nhi, Some(2019)), Some(0.5));
    assert_eq!(max_for_year(&nhi, Some(2020)), Some(0.25));
    assert_eq!(max_for_year(&nhi, Some(2023)), Some(0.25));
    assert_eq!(max_for_year(&nhi, Some(2026)), Some(0.5));
    assert_eq!(max_for_year(&nhi, None), Some(0.5));
    // "0 từ 2025" = no longer counted
    let gone = [t(0.5, None, None), t(0.0, Some(2025), None)];
    assert_eq!(max_for_year(&gone, Some(2025)), Some(0.0));
    // only listed from 2020: an older paper is not covered
    let late = [t(0.5, Some(2020), None), t(0.75, Some(2024), None)];
    assert_eq!(max_for_year(&late, Some(2018)), None);
    // "trước 2020" ends in 2019
    let ended = [t(0.5, None, Some(2019))];
    assert_eq!(max_for_year(&ended, Some(2019)), Some(0.5));
    assert_eq!(max_for_year(&ended, Some(2021)), None);
}

#[test]
fn sjr_quartile_by_year() {
    let j = SjrJournal { issn: vec!["2071-1050".into()], title: "Sustainability".into(), quartiles: vec![(2018, 2), (2019, 1), (2021, 1)] };
    assert!(j.validate().is_ok());
    assert_eq!(j.quartile_for(Some(2018)), Some((Quartile::Q2, 2018)));
    assert_eq!(j.quartile_for(Some(2020)), Some((Quartile::Q1, 2019)));
    assert_eq!(j.quartile_for(Some(2015)), Some((Quartile::Q2, 2018)));
    assert_eq!(j.quartile_for(None), Some((Quartile::Q1, 2021)));
    let bad = SjrJournal { issn: vec!["1234-5678".into()], title: "x".into(), quartiles: vec![(2020, 1)] };
    assert!(bad.validate().is_err());
}

#[test]
fn listed_journal_wins_then_quartile_estimate() {
    let kt = nganh("Kinh tế");
    let row = JournalPoints { nganh: "kinh_te".into(), issn: vec!["2515-964X".into()], ten: "Journal of Asian Business and Economic Studies".into(), stt: 29,
        tiers: vec![t(1.25, Some(2018), None), t(1.5, Some(2022), None), t(2.0, Some(2023), None)] };
    assert!(row.validate().is_ok());
    let a = resolve(kt, Some(2024), Some(&row), Some((Quartile::Q2, Some(2024), false)));
    assert_eq!((a.points, a.source), (2.0, PointsSource::Hdgsnn));
    assert!(a.note.contains("tối đa 2 điểm"));
    let b = resolve(kt, Some(2024), None, Some((Quartile::Q1, Some(2024), false)));
    assert_eq!((b.points, b.source), (2.0, PointsSource::Estimate));
    assert!(b.note.starts_with("Ước tính"));
    let c = resolve(None, Some(2024), None, Some((Quartile::Q1, None, true)));
    assert_eq!(c.points, 3.0);
    let d = resolve(kt, Some(2024), None, None);
    assert_eq!((d.points, d.source), (0.0, PointsSource::None));
    let e = resolve(kt, Some(2016), Some(&row), None);
    assert_eq!((e.points, e.source), (0.0, PointsSource::Hdgsnn));
}

#[test]
fn every_field_scale_is_sane() {
    for n in NGANH {
        assert!(n.intl.iter().all(|p| (0.5..=3.0).contains(p)), "{}", n.key);
        assert!(n.intl.windows(2).all(|w| w[0] >= w[1]), "{}", n.key);
        assert!(!n.aliases.is_empty());
    }
}

#[test]
fn owner_role_from_author_list() {
    assert!(same_person("Nguyễn Văn An", "An Nguyen Van"));
    assert!(same_person("Trần Thị Bình", "Thi Binh Tran"));
    assert!(!same_person("Nguyễn Văn An", "Nguyễn Văn Anh"));
    assert!(!same_person("An", "An"));
    let list: Vec<String> = ["Van An Nguyen", "Binh Tran"].map(String::from).to_vec();
    assert_eq!(auto_role("Nguyễn Văn An", &list), Some(AuthorRole::Main));
    assert_eq!(auto_role("Trần Bình", &list), Some(AuthorRole::Co));
    assert_eq!(auto_role("Lê Cường", &list), None);
    assert_eq!(auto_role("Nguyễn Văn An", &["An Nguyen Van".to_string()]), Some(AuthorRole::Sole));
}

#[test]
fn crossref_issns_are_read() {
    let body = json!({"message": {"DOI": "10.1/x", "title": ["T"], "ISSN": ["2071-1050", "bad"], "issn-type": [{"type": "electronic", "value": "2071-1050"}]}});
    let w = parse_crossref(&body).unwrap();
    assert_eq!(w.issn, vec!["2071-1050".to_string()]);
}

#[test]
fn summary_recommends_one_and_a_half_times_the_minimum() {
    let papers = vec![
        PaperPoints { personal: 9.0, points_source: PointsSource::Manual, ..Default::default() },
        PaperPoints { personal: 4.0, points_source: PointsSource::Estimate, ..Default::default() },
    ];
    let s = summarize(&papers, 2, Target::Pgs);
    assert_eq!((s.points_total, s.enough, s.recommended_points, s.recommended_enough), (13.0, true, 15.0, false));
    assert_eq!((s.estimated_points, s.auto_papers), (4.0, 1));
}
