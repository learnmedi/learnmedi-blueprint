//! Multi-language content: which languages a site may use, the address
//! prefix of each, and how a translated page stays tied to the original.
//!
//! Every language version of a page is its own page; versions share a
//! `group_id` and the same block ids. English is the original once it exists.
//! A translated page keeps one [`BlockSync`] stamp per block id: the
//! fingerprints of the English block and of its own block when the pair was
//! last in step. Comparing them with the current fingerprints gives a
//! [`SyncStatus`] per block (see [`block_status`]).

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::blocks::Block;
use crate::util::sha256_hex;

/// Languages a site may enable: (code, name shown in the switcher).
pub const LANGUAGES: &[(&str, &str)] = &[("vi", "Tiếng Việt"), ("en", "English")];

/// The language every other version is translated from (when the page has it).
pub const ORIGINAL: &str = "en";

pub fn is_supported(lang: &str) -> bool {
    LANGUAGES.iter().any(|(c, _)| *c == lang)
}

pub fn language_name(lang: &str) -> &'static str {
    LANGUAGES.iter().find(|(c, _)| *c == lang).map(|(_, n)| *n).unwrap_or("")
}

/// Address of `path` (e.g. `/`, `/about`, `/blog/x`) in `lang` on a site whose main
/// language is `main`: the main language lives at the root, others under `/<lang>`.
pub fn localized_path(lang: &str, main: &str, path: &str) -> String {
    if lang.is_empty() || lang == main {
        return if path.is_empty() { "/".into() } else { path.to_string() };
    }
    match path {
        "" | "/" => format!("/{lang}"),
        p if p.starts_with('/') => format!("/{lang}{p}"),
        p => format!("/{lang}/{p}"),
    }
}

/// Split a request path into (language, path without the prefix). Paths without a
/// prefix of a supported non-main language belong to `main`.
pub fn split_path<'a>(path: &'a str, main: &'a str) -> (&'a str, &'a str) {
    for (code, _) in LANGUAGES {
        if *code == main {
            continue;
        }
        let Some(rest) = path.strip_prefix('/').and_then(|p| p.strip_prefix(*code)) else { continue };
        if rest.is_empty() {
            return (code, "/");
        }
        if rest.starts_with('/') {
            return (code, rest);
        }
    }
    (main, path)
}

/// Fingerprint of a block's content: type and props (keys sorted), not its style or
/// visibility, so a colour change or hiding a block in one language is not "out of date".
pub fn block_fingerprint(b: &Block) -> String {
    let mut s = String::new();
    s.push_str(&b.kind);
    s.push('\n');
    canonical(&b.props, &mut s);
    sha256_hex(s.as_bytes())[..16].to_string()
}

/// Fingerprint of free text (e.g. a post's title + body).
pub fn text_fingerprint(parts: &[&str]) -> String {
    sha256_hex(parts.join("\u{1f}").as_bytes())[..16].to_string()
}

fn canonical(v: &Value, out: &mut String) {
    match v {
        Value::Object(m) => {
            let mut keys: Vec<&String> = m.keys().collect();
            keys.sort();
            out.push('{');
            for k in keys {
                out.push_str(&serde_json::to_string(k).unwrap_or_default());
                out.push(':');
                canonical(&m[k], out);
                out.push(',');
            }
            out.push('}');
        }
        Value::Array(a) => {
            out.push('[');
            for x in a {
                canonical(x, out);
                out.push(',');
            }
            out.push(']');
        }
        other => out.push_str(&other.to_string()),
    }
}

/// Stamp on a translated block: both fingerprints when the pair was last in step.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BlockSync {
    /// Fingerprint of the original (English) block.
    pub en_hash: String,
    /// Fingerprint of this (translated) block.
    pub own_hash: String,
}

impl BlockSync {
    pub fn of(original: &Block, own: &Block) -> Self {
        BlockSync { en_hash: block_fingerprint(original), own_hash: block_fingerprint(own) }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SyncStatus {
    /// Neither side changed since the pair was last in step.
    InSync,
    /// The original changed: re-translate this block.
    Outdated,
    /// Only the translation changed (edited by hand): carry the change to the original or confirm it.
    EditedHere,
    /// Both changed: a person (or the AI, carefully) decides.
    Conflict,
    /// The original has this block, the translation does not.
    Missing,
    /// Only in the translation, never tied to the original (an extra block for this language).
    OnlyHere,
    /// Was tied to an original block that no longer exists.
    OriginalRemoved,
    /// Both exist but were never stamped (e.g. typed by hand before tracking).
    Unknown,
}

impl SyncStatus {
    /// Needs attention from the teacher or the AI.
    pub fn needs_work(self) -> bool {
        !matches!(self, SyncStatus::InSync | SyncStatus::OnlyHere)
    }
}

/// Status of one block id between the original and a translation.
pub fn block_status(original: Option<&Block>, own: Option<&Block>, stamp: Option<&BlockSync>) -> SyncStatus {
    match (original, own, stamp) {
        (None, None, _) => SyncStatus::InSync,
        (Some(_), None, _) => SyncStatus::Missing,
        (None, Some(_), Some(_)) => SyncStatus::OriginalRemoved,
        (None, Some(_), None) => SyncStatus::OnlyHere,
        (Some(_), Some(_), None) => SyncStatus::Unknown,
        (Some(o), Some(t), Some(s)) => {
            let o_changed = block_fingerprint(o) != s.en_hash;
            let t_changed = block_fingerprint(t) != s.own_hash;
            match (o_changed, t_changed) {
                (false, false) => SyncStatus::InSync,
                (true, false) => SyncStatus::Outdated,
                (false, true) => SyncStatus::EditedHere,
                (true, true) => SyncStatus::Conflict,
            }
        }
    }
}
