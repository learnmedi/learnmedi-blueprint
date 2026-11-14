use crate::blocks::REGISTRY;
use crate::features::*;
use crate::plans::{self, PlanConfig, Tier};
use crate::tabs::{BlockGroup, TAB_KINDS};
use std::collections::{BTreeMap, BTreeSet};

#[test]
fn every_file_parses_and_keys_are_unique() {
    let keys: Vec<&str> = FEATURES.iter().map(|f| f.key.as_str()).collect();
    let set: BTreeSet<_> = keys.iter().collect();
    assert_eq!(set.len(), keys.len(), "duplicate feature key");
    for f in FEATURES.iter() {
        assert!(["studio", "content", "teaching", "overview", "settings", "learn"].contains(&f.workspace.as_str()), "{}: workspace", f.key);
        for k in &f.enabled_by {
            assert!(plans::feature_def(k).is_some(), "{} enabled_by unknown {k}", f.key);
        }
    }
}

#[test]
fn existing_plan_keys_keep_their_strings() {
    for key in [
        plans::WEBSITE, plans::AI_ASSISTANT, plans::CUSTOM_DOMAIN, plans::BLOG, plans::NEWSLETTER, plans::COLLECTIONS,
        plans::COMMUNITY, plans::PRODUCTS, plans::SELF_PACED, plans::SURVEYS, plans::TEACHING,
        plans::ASSETS, plans::PERSONALITY, plans::FORTUNE, plans::ASSOCIATES,
    ] {
        assert!(plans::feature_def(key).is_some(), "{key}");
    }
    // The new keys are always on, so tiers do not change.
    for key in [plans::WEBSITE, plans::ASSETS, plans::PERSONALITY, plans::FORTUNE, plans::ASSOCIATES] {
        assert!(plans::is_locked(key), "{key}");
    }
    assert!(!plans::is_locked(plans::BLOG));
    assert!(plans::feature_def("shared.courses").is_none(), "a tool group is not a plan feature");
}

#[test]
fn surfaces_point_at_real_blocks_widgets_and_tab_kinds() {
    for f in FEATURES.iter() {
        for b in &f.surfaces.blocks {
            assert!(crate::blocks::block_type(b).is_some(), "{}: block {b}", f.key);
        }
        for w in &f.surfaces.widgets {
            assert!(crate::widgets::widget(w).is_some(), "{}: widget {w}", f.key);
        }
        if let Some(page) = &f.surfaces.page {
            let def = page.tab_kind.definition();
            let core = crate::blocks::block_type(&page.core_block).unwrap_or_else(|| panic!("{}: core block", f.key));
            assert_eq!(core.group, BlockGroup::Core, "{}", f.key);
            assert_eq!(def.core_block, Some(page.core_block.as_str()), "{}", f.key);
            assert_eq!(def.allowed_block_groups, page.allowed_block_groups.as_slice(), "{}", f.key);
            assert!(f.surfaces.blocks.contains(&page.core_block), "{}: owns its core block", f.key);
        }
    }
}

#[test]
fn every_block_widget_and_core_tab_kind_has_one_owner() {
    for bt in REGISTRY.iter() {
        let n = FEATURES.iter().filter(|f| f.surfaces.blocks.contains(&bt.key)).count();
        assert_eq!(n, 1, "block {} owned by {n} features", bt.key);
    }
    for w in crate::widgets::WIDGETS {
        assert_eq!(FEATURES.iter().filter(|f| f.surfaces.widgets.iter().any(|x| x == w.key)).count(), 1, "widget {}", w.key);
    }
    for def in TAB_KINDS {
        let n = FEATURES.iter().filter(|f| f.surfaces.page.as_ref().is_some_and(|p| p.tab_kind == def.key)).count();
        assert_eq!(n, usize::from(def.core_block.is_some()), "tab kind {:?}", def.key);
    }
}

#[test]
fn every_tool_is_declared_once() {
    let names = tool_names();
    let set: BTreeSet<_> = names.iter().collect();
    assert_eq!(set.len(), names.len(), "a tool is declared twice");
    for n in &names {
        assert!(n.chars().all(|c| c.is_ascii_lowercase() || c == '_'), "{n}");
    }
}

#[test]
fn dangerous_list_is_unchanged() {
    let mut got: Vec<&str> = dangerous_tools().to_vec();
    got.sort();
    let mut want = vec![
        "delete_post", "delete_page", "delete_tab", "delete_assets", "delete_quiz", "delete_collection_items",
        "delete_community_contribution", "bank_delete_questions", "remove_class_member", "remove_subscribers",
        "delete_newsletter", "send_newsletter", "delete_deck",
    ];
    want.sort();
    assert_eq!(got, want);
}

#[test]
fn tool_gates() {
    assert_eq!(tool_gate("save_product"), Some(&[plans::PRODUCTS][..]));
    assert_eq!(tool_gate("share_quiz"), Some(&[plans::TEACHING][..]));
    assert_eq!(tool_gate("create_quiz"), Some(&[plans::TEACHING, plans::SELF_PACED][..]));
    assert_eq!(tool_gate("set_personality"), Some(&[plans::PERSONALITY][..]));
    assert_eq!(tool_gate("add_block"), None);
    assert_eq!(tool_gate("nope"), None);
}

#[test]
fn reserved_slugs_do_not_repeat_and_cover_page_slugs() {
    let all = reserved_slugs();
    let set: BTreeSet<_> = all.iter().collect();
    assert_eq!(set.len(), all.len(), "a reserved slug is declared twice");
    for slug in ["blog", "courses", "studio", "community", "products", "overview", "api"] {
        assert!(all.contains(&slug), "{slug}");
    }
    // A Page's default slug is reserved by that same feature.
    for f in FEATURES.iter() {
        if let Some(slug) = f.surfaces.page.as_ref().and_then(|p| p.default_slug.as_ref()) {
            assert!(f.reserved_slugs.contains(slug), "{}: {slug}", f.key);
        }
    }
}

#[test]
fn routes_match_like_before() {
    let g = |get: bool, p: &str| route_gate(get, p);
    assert_eq!(g(true, "/teach"), Some(&[plans::TEACHING][..]));
    assert_eq!(g(true, "/teach/x"), None);
    assert_eq!(g(false, "/api/studio/classrooms/c1"), Some(&[plans::TEACHING][..]));
    assert_eq!(g(true, "/api/studio/courses"), Some(&[plans::TEACHING, plans::SELF_PACED][..]));
    assert_eq!(g(true, "/products"), Some(&[plans::PRODUCTS][..]));
    assert_eq!(g(true, "/products/ebook/"), Some(&[plans::PRODUCTS][..]));
    assert_eq!(g(false, "/api/products/p1/checkout"), Some(&[plans::PRODUCTS][..]));
    assert_eq!(g(true, "/api/products/p1/me"), None);
    assert_eq!(g(false, "/api/posts/p1/like"), Some(&[plans::BLOG][..]));
    assert_eq!(g(true, "/api/posts/p1/social"), None);
    assert_eq!(g(true, "/en/community"), Some(&[plans::COMMUNITY][..]));
    assert_eq!(g(true, "/communityx"), None);
    assert_eq!(g(false, "/api/newsletter/subscribe"), Some(&[plans::NEWSLETTER][..]));
    assert_eq!(g(true, "/api/ai/sessions"), Some(&[plans::AI_ASSISTANT][..]));
    assert_eq!(g(true, "/"), None);
}

#[test]
fn prompt_references_every_feature_part_once() {
    let refs = template_refs();
    let set: BTreeSet<_> = refs.iter().collect();
    assert_eq!(set.len(), refs.len(), "a prompt part is placed twice");
    for f in FEATURES.iter().filter(|f| !f.is_core()) {
        for (part, _) in f.prompt.as_ref().map(|p| p.parts()).unwrap_or_default() {
            assert!(refs.iter().any(|(k, p)| k == &f.key && p == part), "{}#{part} not placed in the core template", f.key);
        }
    }
    for (k, p) in &refs {
        let f = feature(k).unwrap_or_else(|| panic!("template names unknown feature {k}"));
        assert!(f.prompt.as_ref().and_then(|t| t.part(p)).is_some(), "{k}#{p} missing");
    }
}

#[test]
fn prompt_drops_lines_of_features_that_are_off() {
    let all = prompt_for(&|_| true);
    assert!(all.contains("get_community_contributions") && all.contains("create_quiz_from_bank"));
    let none = prompt_for(&|k| k == plans::WEBSITE);
    assert!(!none.contains("{prompt:"));
    assert!(!none.contains("get_community_contributions"));
    assert!(!none.contains("create_quiz_from_bank"));
    assert!(none.contains("# Cách bạn làm việc"));
    // Self-paced alone still gets the shared course text.
    let sp = prompt_for(&|k| k == plans::SELF_PACED);
    assert!(sp.contains("Bài kiểm tra: create_quiz") && !sp.contains("create_quiz_from_bank"));
}

#[test]
fn tabs_line_is_generated_from_tab_kinds() {
    let merged = [plans::PRODUCTS, plans::COMMUNITY, plans::SURVEYS, plans::ASSOCIATES];
    assert_eq!(
        tabs_for_prompt(&|k| !merged.contains(&k)),
        "Tab công khai có loại Tự do, Blog, Khoá học, Danh mục và Liên kết. Blog, Khoá học và Danh mục có đúng một khối chính không thể xoá. Dùng list_tabs để tìm tab."
    );
    assert!(!tabs_for_prompt(&|k| k != plans::BLOG).contains("Blog"));
    assert!(tabs_for_prompt(&|_| true).contains("Liên kết, Sản phẩm, Phục vụ cộng đồng, Khảo sát và Cộng sự. Blog,"));
}

#[test]
fn fill_is_single_pass() {
    assert_eq!(fill("a {x} {y} {z}", &[("x", "{y}"), ("y", "2")]), "a {y} 2 {z}");
    assert_eq!(fill("{{total}}", &[]), "{{total}}");
}

#[test]
fn ai_switch_is_kept_and_defaults_on() {
    // Boards stored before the switch existed mean "AI on".
    let old: plans::FeatureSetting = serde_json::from_str(r#"{"enabled":true,"tier":"basic"}"#).unwrap();
    assert!(old.ai);
    let mut cfg = PlanConfig::default();
    cfg.features.get_mut(plans::BLOG).unwrap().ai = false;
    cfg.features.insert(plans::WEBSITE.into(), plans::FeatureSetting::new(false, None, false));
    cfg.features.insert(plans::FORTUNE.into(), plans::FeatureSetting::new(false, None, false));
    let cfg = cfg.normalized().unwrap();
    let all = cfg.resolve(Tier::Premium, &BTreeMap::new());
    let ai = cfg.resolve_ai(Tier::Premium, &BTreeMap::new());
    assert!(all.contains(&plans::BLOG.to_string()) && !ai.contains(&plans::BLOG.to_string()));
    assert!(ai.contains(&plans::WEBSITE.to_string()), "the website stays on for the AI");
    assert!(all.contains(&plans::FORTUNE.to_string()) && !ai.contains(&plans::FORTUNE.to_string()));
}

#[test]
fn block_and_widget_switches() {
    let mut cfg = PlanConfig::default();
    cfg.blocks.insert("slideshow".into(), false);
    cfg.blocks.insert("hero".into(), true);
    cfg.blocks.insert("nope".into(), false);
    cfg.widgets.insert("fortune_button".into(), false);
    let cfg = cfg.normalized().unwrap();
    assert_eq!(cfg.disabled_blocks(), vec!["slideshow".to_string()]);
    assert_eq!(cfg.disabled_widgets(), vec!["fortune_button".to_string()]);
    assert!(cfg.block_on("hero") && !cfg.block_on("slideshow"));
    // Old stored boards have no block/widget maps.
    let old: PlanConfig = serde_json::from_str(r#"{"features":{},"default_tier":"premium"}"#).unwrap();
    assert!(old.disabled_blocks().is_empty());
}

#[test]
fn prompt_parts_cover_every_feature() {
    let parts = prompt_parts();
    assert_eq!(parts.len(), FEATURES.len());
    let core = parts.iter().find(|p| p.key == plans::WEBSITE).unwrap();
    assert!(core.parts.iter().any(|(k, t)| *k == "main" && t.contains("{goal_block}")));
    assert!(!core.texts.is_empty());
}

#[test]
fn ai_workspaces_are_known() {
    for f in FEATURES.iter() {
        for w in f.default_ai_workspaces().into_iter().chain(f.tools.iter().flat_map(|t| t.workspaces.iter().map(String::as_str))) {
            assert!(AI_WORKSPACES.contains(&w), "{}: workspace {w}", f.key);
        }
    }
    assert_eq!(tool_workspaces("create_quiz"), &["teaching", "content"]);
    assert_eq!(tool_workspaces("share_quiz"), &["teaching"]);
    assert_eq!(tool_workspaces("save_product"), &["content"]);
    assert_eq!(tool_workspaces("set_personality"), &["studio"]);
}
