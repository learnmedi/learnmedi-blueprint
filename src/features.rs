//! The feature registry: one YAML file per feature in `ui/features/`.
//!
//! A feature declares its plan metadata (key, label, tier…), where visitors see
//! it (a Page = tab kind with one core block, Blocks, Widgets), its routes, the
//! slugs it reserves, the AI tools it unlocks (with the dangerous ones marked)
//! and its AI prompt text. The core file (`00-core.yaml`, `studio.website`)
//! also owns the general prompt template and the small texts the site uses to
//! fill it.
//!
//! The site reads routes, tools, dangerous tools, reserved slugs and prompt text
//! from here and never keeps its own copy; the engine reads it to show the admin
//! board and the prompt viewer. Adding a feature = one YAML file here + logic in
//! the site.
//!
//! A file with `enabled_by` is a **tool group**, not a plan feature: its tools,
//! routes and surfaces are on when any feature in `enabled_by` is on (course and
//! quiz tools serve both Classes and Self-paced courses).

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::sync::LazyLock;

use crate::plans::Tier;
use crate::tabs::{BlockGroup, TabKind};

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct LocalText {
    #[serde(default)]
    pub vi: String,
    #[serde(default)]
    pub en: String,
}

/// A Page: a tab kind with exactly one core block.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct PageSurface {
    pub tab_kind: TabKind,
    pub core_block: String,
    pub allowed_block_groups: Vec<BlockGroup>,
    #[serde(default)]
    pub default_slug: Option<String>,
    #[serde(default)]
    pub default_title: Option<LocalText>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct Surfaces {
    pub page: Option<PageSurface>,
    /// Block types owned by the feature (its core block and excerpt blocks).
    pub blocks: Vec<String>,
    /// Widget keys (see `widgets::WIDGETS`).
    pub widgets: Vec<String>,
}

/// Route patterns the feature gates. A pattern is a path, optionally prefixed
/// by `WRITE ` (only non-GET requests). It matches the path itself and every
/// path below it; `*` matches one segment; a trailing `$` means exact match.
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct Routes {
    pub public: Vec<String>,
    pub staff: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ToolDef {
    pub name: String,
    /// Destroys data or sends mail: the AI must preview and get confirmation.
    #[serde(default)]
    pub dangerous: bool,
    /// Staff workspaces whose AI assistant may use this tool; empty = the feature's `ai_workspaces`.
    #[serde(default)]
    pub workspaces: Vec<String>,
}

/// One text, or named parts placed by `{prompt:KEY#part}` lines of the core template.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum PromptText {
    One(String),
    Parts(BTreeMap<String, String>),
}

impl PromptText {
    pub fn part(&self, name: &str) -> Option<&str> {
        match self {
            PromptText::One(s) if name == "main" => Some(s),
            PromptText::One(_) => None,
            PromptText::Parts(m) => m.get(name).map(String::as_str),
        }
    }

    pub fn parts(&self) -> Vec<(&str, &str)> {
        match self {
            PromptText::One(s) => vec![("main", s.as_str())],
            PromptText::Parts(m) => m.iter().map(|(k, v)| (k.as_str(), v.as_str())).collect(),
        }
    }
}

fn yes() -> bool {
    true
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct FeatureSpec {
    /// Stored in `PlanConfig` and `site.feature_overrides`: never rename.
    pub key: String,
    pub label: String,
    pub description: String,
    /// Staff workspace that owns the feature: studio | content | teaching | overview | settings.
    pub workspace: String,
    pub default_tier: Tier,
    #[serde(default = "yes")]
    pub default_enabled: bool,
    /// Always on in every tier (the admin cannot switch it off or move it).
    #[serde(default)]
    pub locked: bool,
    /// Non-empty = a tool group, on when any of these features is on.
    #[serde(default)]
    pub enabled_by: Vec<String>,
    /// Staff workspaces (studio | content | teaching) whose AI assistant gets this
    /// feature's tools. Empty = `workspace` when it is one of those, else studio.
    #[serde(default)]
    pub ai_workspaces: Vec<String>,
    #[serde(default)]
    pub surfaces: Surfaces,
    #[serde(default)]
    pub routes: Routes,
    #[serde(default)]
    pub reserved_slugs: Vec<String>,
    #[serde(default)]
    pub tools: Vec<ToolDef>,
    /// Core only: small texts the site uses to fill the prompt.
    #[serde(default)]
    pub texts: BTreeMap<String, String>,
    #[serde(default)]
    pub prompt: Option<PromptText>,
}

impl FeatureSpec {
    pub fn is_group(&self) -> bool {
        !self.enabled_by.is_empty()
    }

    pub fn is_core(&self) -> bool {
        self.key == crate::plans::WEBSITE
    }

    /// Plan features that switch this item on (any one of them).
    pub fn gate(&'static self) -> Vec<&'static str> {
        if self.is_group() { self.enabled_by.iter().map(String::as_str).collect() } else { vec![self.key.as_str()] }
    }
}

const FILES: &[(&str, &str)] = &[
    ("00-core.yaml", include_str!("../ui/features/00-core.yaml")),
    ("01-teaching.yaml", include_str!("../ui/features/01-teaching.yaml")),
    ("02-courses.yaml", include_str!("../ui/features/02-courses.yaml")),
    ("03-products.yaml", include_str!("../ui/features/03-products.yaml")),
    ("04-surveys.yaml", include_str!("../ui/features/04-surveys.yaml")),
    ("05-blog.yaml", include_str!("../ui/features/05-blog.yaml")),
    ("06-community.yaml", include_str!("../ui/features/06-community.yaml")),
    ("07-newsletter.yaml", include_str!("../ui/features/07-newsletter.yaml")),
    ("08-custom_domain.yaml", include_str!("../ui/features/08-custom_domain.yaml")),
    ("09-ai.yaml", include_str!("../ui/features/09-ai.yaml")),
    ("10-collections.yaml", include_str!("../ui/features/10-collections.yaml")),
    ("11-self_paced.yaml", include_str!("../ui/features/11-self_paced.yaml")),
    ("12-assets.yaml", include_str!("../ui/features/12-assets.yaml")),
    ("13-personality.yaml", include_str!("../ui/features/13-personality.yaml")),
    ("14-fortune.yaml", include_str!("../ui/features/14-fortune.yaml")),
    ("15-associates.yaml", include_str!("../ui/features/15-associates.yaml")),
    ("16-scholar.yaml", include_str!("../ui/features/16-scholar.yaml")),
];

/// Every feature in file order (routes are matched in this order).
pub static FEATURES: LazyLock<Vec<FeatureSpec>> = LazyLock::new(|| {
    FILES
        .iter()
        .map(|(name, src)| serde_yaml_ng::from_str(src).unwrap_or_else(|e| panic!("ui/features/{name}: {e}")))
        .collect()
});

pub fn feature(key: &str) -> Option<&'static FeatureSpec> {
    FEATURES.iter().find(|f| f.key == key)
}

pub fn core() -> &'static FeatureSpec {
    feature(crate::plans::WEBSITE).expect("core feature file")
}

/// A text of the core file (`texts:`), "" when missing.
pub fn text(key: &str) -> &'static str {
    core().texts.get(key).map(String::as_str).unwrap_or("")
}

// ───────────────────────────── Tools ─────────────────────────────────────────

pub fn tool_owner(name: &str) -> Option<&'static FeatureSpec> {
    FEATURES.iter().find(|f| f.tools.iter().any(|t| t.name == name))
}

static TOOL_GATES: LazyLock<BTreeMap<&'static str, Vec<&'static str>>> = LazyLock::new(|| {
    FEATURES
        .iter()
        .filter(|f| !f.is_core())
        .flat_map(|f| f.tools.iter().map(move |t| (t.name.as_str(), f.gate())))
        .collect()
});

/// Plan features a tool or op needs (any one of them); None = core website editing
/// (or a tool not in the registry).
pub fn tool_gate(name: &str) -> Option<&'static [&'static str]> {
    TOOL_GATES.get(name).map(Vec::as_slice)
}

static DANGEROUS: LazyLock<Vec<&'static str>> =
    LazyLock::new(|| FEATURES.iter().flat_map(|f| f.tools.iter().filter(|t| t.dangerous).map(|t| t.name.as_str())).collect());

/// Tools that destroy data or send mail to real people.
pub fn dangerous_tools() -> &'static [&'static str] {
    &DANGEROUS
}

pub fn is_dangerous(name: &str) -> bool {
    DANGEROUS.contains(&name)
}

/// Every declared tool name.
pub fn tool_names() -> Vec<&'static str> {
    FEATURES.iter().flat_map(|f| f.tools.iter().map(|t| t.name.as_str())).collect()
}

/// Workspaces that have an AI assistant.
pub const AI_WORKSPACES: &[&str] = &["studio", "content", "teaching"];

impl FeatureSpec {
    /// Workspaces whose assistant gets this feature's tools (see `ai_workspaces`).
    pub fn default_ai_workspaces(&self) -> Vec<&str> {
        if !self.ai_workspaces.is_empty() {
            self.ai_workspaces.iter().map(String::as_str).collect()
        } else if AI_WORKSPACES.contains(&self.workspace.as_str()) {
            vec![self.workspace.as_str()]
        } else {
            vec!["studio"]
        }
    }
}

static TOOL_WORKSPACES: LazyLock<BTreeMap<&'static str, Vec<&'static str>>> = LazyLock::new(|| {
    FEATURES
        .iter()
        .flat_map(|f| {
            f.tools.iter().map(move |t| {
                let ws = if t.workspaces.is_empty() { f.default_ai_workspaces() } else { t.workspaces.iter().map(String::as_str).collect() };
                (t.name.as_str(), ws)
            })
        })
        .collect()
});

/// Workspaces whose AI assistant may use a tool (a tool not in the registry: studio).
pub fn tool_workspaces(name: &str) -> &'static [&'static str] {
    TOOL_WORKSPACES.get(name).map(Vec::as_slice).unwrap_or(&["studio"])
}

// ───────────────────────────── Routes and slugs ──────────────────────────────

static RESERVED: LazyLock<Vec<&'static str>> =
    LazyLock::new(|| FEATURES.iter().flat_map(|f| f.reserved_slugs.iter().map(String::as_str)).collect());

/// Slugs a page may not take: system routes and every feature's own routes.
pub fn reserved_slugs() -> &'static [&'static str] {
    &RESERVED
}

/// Does a route pattern match the request?
pub fn route_matches(pattern: &str, is_get: bool, path: &str) -> bool {
    let (write_only, pattern) = match pattern.strip_prefix("WRITE ") {
        Some(rest) => (true, rest.trim()),
        None => (false, pattern.trim()),
    };
    if write_only && is_get {
        return false;
    }
    let (exact, pattern) = match pattern.strip_suffix('$') {
        Some(p) => (true, p),
        None => (false, pattern),
    };
    let p = path.trim_end_matches('/');
    let want: Vec<&str> = pattern.trim_end_matches('/').split('/').collect();
    let have: Vec<&str> = p.split('/').collect();
    if have.len() < want.len() || (exact && have.len() != want.len()) {
        return false;
    }
    want.iter().zip(&have).all(|(w, h)| *w == "*" && !h.is_empty() || w == h)
}

static ROUTES: LazyLock<Vec<(&'static str, Vec<&'static str>)>> = LazyLock::new(|| {
    FEATURES
        .iter()
        .filter(|f| !f.is_core())
        .flat_map(|f| f.routes.public.iter().chain(&f.routes.staff).map(move |r| (r.as_str(), f.gate())))
        .collect()
});

/// Plan features a route needs (any one of them); None = open to every site.
pub fn route_gate(is_get: bool, path: &str) -> Option<&'static [&'static str]> {
    ROUTES.iter().find(|(pattern, _)| route_matches(pattern, is_get, path)).map(|(_, gate)| gate.as_slice())
}

// ───────────────────────────── Surfaces ──────────────────────────────────────

pub fn block_owner(block_type: &str) -> Option<&'static FeatureSpec> {
    FEATURES.iter().find(|f| f.surfaces.blocks.iter().any(|b| b == block_type))
}

pub fn widget_owner(widget: &str) -> Option<&'static FeatureSpec> {
    FEATURES.iter().find(|f| f.surfaces.widgets.iter().any(|w| w == widget))
}

/// The feature whose Page is this tab kind; None = a core kind (Free, Link).
pub fn tab_kind_owner(kind: TabKind) -> Option<&'static FeatureSpec> {
    FEATURES.iter().find(|f| f.surfaces.page.as_ref().is_some_and(|p| p.tab_kind == kind))
}

// ───────────────────────────── Prompt ────────────────────────────────────────

/// Is the feature available to the AI, given a check for plan feature keys?
/// Core is always on; a tool group is on when any feature that enables it is on.
pub fn ai_on(spec: &'static FeatureSpec, on: &dyn Fn(&str) -> bool) -> bool {
    spec.is_core() || spec.gate().iter().any(|k| on(k))
}

/// Placeholders `{prompt:KEY}` / `{prompt:KEY#part}` / `{prompt:*}` in a template line.
fn expand_line(line: &str, on: &dyn Fn(&'static FeatureSpec) -> bool, used: &[(String, String)]) -> (String, bool) {
    let mut out = String::new();
    let mut rest = line;
    let mut had = false;
    while let Some(at) = rest.find("{prompt:") {
        let Some(end) = rest[at..].find('}') else { break };
        out.push_str(&rest[..at]);
        let token = &rest[at + 8..at + end];
        had = true;
        if token == "*" {
            let mut extra = Vec::new();
            for f in FEATURES.iter().filter(|f| !f.is_core() && on(f)) {
                for (part, text) in f.prompt.as_ref().map(|p| p.parts()).unwrap_or_default() {
                    if !used.iter().any(|(k, p)| k == &f.key && p == part) {
                        extra.push(text.trim_end().to_string());
                    }
                }
            }
            out.push_str(&extra.join("\n"));
        } else {
            let (key, part) = token.split_once('#').unwrap_or((token, "main"));
            if let Some(f) = feature(key).filter(|f| on(f)) {
                if let Some(text) = f.prompt.as_ref().and_then(|p| p.part(part)) {
                    out.push_str(text.trim_end());
                }
            }
        }
        rest = &rest[at + end + 1..];
    }
    out.push_str(rest);
    (out, had)
}

/// Every `{prompt:…}` reference in the core template, as (feature key, part).
pub fn template_refs() -> Vec<(String, String)> {
    let template = core().prompt.as_ref().and_then(|p| p.part("main")).unwrap_or("");
    let mut refs = Vec::new();
    let mut rest = template;
    while let Some(at) = rest.find("{prompt:") {
        let Some(end) = rest[at..].find('}') else { break };
        let token = &rest[at + 8..at + end];
        if token != "*" {
            let (key, part) = token.split_once('#').unwrap_or((token, "main"));
            refs.push((key.to_string(), part.to_string()));
        }
        rest = &rest[at + end + 1..];
    }
    refs
}

/// The core prompt with every AI-enabled feature's text in place. `on` says
/// whether a plan feature key is on for the AI. Runtime placeholders
/// (`{goal_block}`, `{overview}`…) are left for the site (`fill`).
pub fn prompt_for(on: &dyn Fn(&str) -> bool) -> String {
    let is_on = |f: &'static FeatureSpec| ai_on(f, on);
    let used = template_refs();
    let template = core().prompt.as_ref().and_then(|p| p.part("main")).unwrap_or("");
    let mut out = Vec::new();
    for line in template.lines() {
        let (expanded, had) = expand_line(line, &is_on, &used);
        let expanded = expanded.trim_end().to_string();
        if had && expanded.trim().is_empty() {
            continue;
        }
        out.push(expanded);
    }
    out.join("\n")
}

/// Replaces `{name}` placeholders in one pass (values are never re-scanned, so
/// user text that contains `{…}` stays as it is).
pub fn fill(template: &str, vars: &[(&str, &str)]) -> String {
    let mut out = String::with_capacity(template.len());
    let mut rest = template;
    while let Some(at) = rest.find('{') {
        out.push_str(&rest[..at]);
        let tail = &rest[at..];
        let hit = tail.find('}').and_then(|end| {
            let name = &tail[1..end];
            vars.iter().find(|(k, _)| *k == name).map(|(_, v)| (end, *v))
        });
        match hit {
            Some((end, value)) => {
                out.push_str(value);
                rest = &tail[end + 1..];
            }
            None => {
                out.push('{');
                rest = &tail[1..];
            }
        }
    }
    out.push_str(rest);
    out
}

/// "A, B và C".
pub fn join_list(items: &[&str]) -> String {
    match items {
        [] => String::new(),
        [one] => one.to_string(),
        [head @ .., last] => format!("{}{}{last}", head.join(", "), text("list_and")),
    }
}

/// The tab-kinds line of the prompt, for the kinds whose feature is on.
pub fn tabs_for_prompt(on: &dyn Fn(&str) -> bool) -> String {
    let kinds: Vec<_> = crate::tabs::TAB_KINDS
        .iter()
        .filter(|def| tab_kind_owner(def.key).is_none_or(|f| ai_on(f, on)))
        .collect();
    let labels: Vec<&str> = kinds.iter().map(|d| d.label).collect();
    let cores: Vec<&str> = kinds.iter().filter(|d| d.core_block.is_some()).map(|d| d.label).collect();
    fill(text("tabs_line"), &[("kinds", &join_list(&labels)), ("core_kinds", &join_list(&cores))])
}

/// Prompt text and tools of one item, for the admin's prompt viewer.
#[derive(Clone, Debug, Serialize)]
pub struct PromptPart {
    pub key: &'static str,
    pub label: &'static str,
    pub group: bool,
    pub enabled_by: &'static [String],
    /// (part name, text); the core's `main` is the template.
    pub parts: Vec<(&'static str, &'static str)>,
    pub texts: &'static BTreeMap<String, String>,
    pub tools: &'static [ToolDef],
}

pub fn prompt_parts() -> Vec<PromptPart> {
    FEATURES
        .iter()
        .map(|f| PromptPart {
            key: &f.key,
            label: &f.label,
            group: f.is_group(),
            enabled_by: &f.enabled_by,
            parts: f.prompt.as_ref().map(|p| p.parts()).unwrap_or_default(),
            texts: &f.texts,
            tools: &f.tools,
        })
        .collect()
}
