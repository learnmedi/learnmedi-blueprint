use crate::blocks::{Block, BlockStyle};
use crate::i18n::*;
use serde_json::json;

fn block(id: &str, text: &str) -> Block {
    Block { id: id.into(), kind: "heading".into(), props: json!({"text": text, "level": 2}), style: BlockStyle::default(), hidden: false }
}

#[test]
fn paths_get_a_prefix_outside_the_main_language() {
    assert_eq!(localized_path("vi", "vi", "/"), "/");
    assert_eq!(localized_path("vi", "vi", "/about"), "/about");
    assert_eq!(localized_path("en", "vi", "/"), "/en");
    assert_eq!(localized_path("en", "vi", "/about"), "/en/about");
    assert_eq!(localized_path("vi", "en", "/blog/x"), "/vi/blog/x");
    assert_eq!(split_path("/en", "vi"), ("en", "/"));
    assert_eq!(split_path("/en/about", "vi"), ("en", "/about"));
    assert_eq!(split_path("/english", "vi"), ("vi", "/english"));
    assert_eq!(split_path("/about", "vi"), ("vi", "/about"));
    assert_eq!(split_path("/en/about", "en"), ("en", "/en/about"));
    assert!(is_supported("en") && is_supported("vi") && !is_supported("fr"));
}

#[test]
fn fingerprint_ignores_key_order_style_and_visibility() {
    let a = block("b1", "Hello");
    let mut b = a.clone();
    b.props = json!({"level": 2, "text": "Hello"});
    b.hidden = true;
    b.style.background = "dark".into();
    assert_eq!(block_fingerprint(&a), block_fingerprint(&b));
    assert_ne!(block_fingerprint(&a), block_fingerprint(&block("b1", "Hello!")));
}

#[test]
fn status_tells_which_side_changed() {
    let en = block("b1", "Hello");
    let vi = block("b1", "Xin chào");
    let stamp = BlockSync::of(&en, &vi);
    assert_eq!(block_status(Some(&en), Some(&vi), Some(&stamp)), SyncStatus::InSync);
    let en2 = block("b1", "Hello there");
    let vi2 = block("b1", "Chào bạn");
    assert_eq!(block_status(Some(&en2), Some(&vi), Some(&stamp)), SyncStatus::Outdated);
    assert_eq!(block_status(Some(&en), Some(&vi2), Some(&stamp)), SyncStatus::EditedHere);
    assert_eq!(block_status(Some(&en2), Some(&vi2), Some(&stamp)), SyncStatus::Conflict);
    assert_eq!(block_status(Some(&en), None, None), SyncStatus::Missing);
    assert_eq!(block_status(None, Some(&vi), None), SyncStatus::OnlyHere);
    assert_eq!(block_status(None, Some(&vi), Some(&stamp)), SyncStatus::OriginalRemoved);
    assert_eq!(block_status(Some(&en), Some(&vi), None), SyncStatus::Unknown);
    assert!(!SyncStatus::InSync.needs_work() && SyncStatus::Outdated.needs_work());
}
