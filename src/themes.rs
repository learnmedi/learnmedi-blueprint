//! Rules for the theme catalog. A theme is a look (colours, fonts, shape) plus
//! optional starter pages built from named slots. The catalog itself lives in
//! the engine's database; this module only says what a valid entry is, builds
//! starter pages, turns a live page into a starter ("save as theme") and moves
//! a page's own content into another theme's layout (port).

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};

use crate::blocks::{self, Block, Field, Kind, apply_style_patch, block_type, is_asset_ref, new_block};
use crate::sections::{self, Section};
use crate::tabs::{COLLECTION_TEMPLATES, TabKind, validate_tab_blocks};
use crate::theme::{self, Theme};

/// Themes are grouped by how much they move, from the most to the least.
/// Old stored values (core, showcase, cv, community) still read.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Category {
    /// Every effect, and the unusual ones (motion "burst").
    #[serde(alias = "showcase")]
    Burst,
    /// Full motion (motion "rich").
    #[serde(alias = "community")]
    Lively,
    /// Things fade in as they come into view (motion "calm").
    #[default]
    #[serde(alias = "core")]
    Gentle,
    /// No motion (motion "none").
    #[serde(alias = "cv")]
    Still,
}

impl Category {
    pub const ALL: [Category; 4] = [Category::Burst, Category::Lively, Category::Gentle, Category::Still];

    pub fn label(self) -> &'static str {
        match self {
            Category::Burst => "Bùng nổ",
            Category::Lively => "Sống động",
            Category::Gentle => "Nhẹ nhàng",
            Category::Still => "Tĩnh lặng",
        }
    }

    /// The motion level a theme of this group uses.
    pub fn motion(self) -> &'static str {
        match self {
            Category::Burst => "burst",
            Category::Lively => "rich",
            Category::Gentle => "calm",
            Category::Still => "none",
        }
    }
}

/// The places every theme is built from. Porting moves a site's content from a
/// slot in one theme to the same slot in another.
pub const SLOTS: &[(&str, &str)] = &[
    ("opener", "Mở đầu"),
    ("about", "Giới thiệu"),
    ("story", "Câu chuyện"),
    ("work", "Công trình nổi bật"),
    ("posts", "Bài viết"),
    ("products", "Sản phẩm"),
    ("community", "Cộng đồng"),
    ("contact", "Liên hệ"),
];

pub fn is_slot(key: &str) -> bool {
    SLOTS.iter().any(|(k, _)| *k == key)
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct StarterBlock {
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub slot: String,
    #[serde(rename = "type")]
    pub kind: String,
    #[serde(default)]
    pub props: Value,
    #[serde(default)]
    pub style: Value,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct StarterSection {
    /// Index of the block the section starts at.
    pub start: usize,
    #[serde(flatten)]
    pub settings: Map<String, Value>,
}

fn yes() -> bool {
    true
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct StarterPage {
    /// "" = the home page.
    #[serde(default)]
    pub slug: String,
    pub title: String,
    #[serde(default)]
    pub kind: TabKind,
    /// Collection template key (`research`, `projects`…) for a collection tab.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub collection: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub link_url: String,
    #[serde(default = "yes")]
    pub show_in_nav: bool,
    #[serde(default)]
    pub blocks: Vec<StarterBlock>,
    #[serde(default)]
    pub sections: Vec<StarterSection>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ThemeEntry {
    pub key: String,
    pub label: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub category: Category,
    #[serde(default)]
    pub featured: bool,
    #[serde(default)]
    pub order: i32,
    pub look: Theme,
    /// Starter pages, home first. Empty = only a look.
    #[serde(default)]
    pub pages: Vec<StarterPage>,
    /// A real site that shows the theme finished (a demo customer).
    #[serde(default)]
    pub demo_site_id: String,
    /// Filled in by the engine from `demo_site_id`.
    #[serde(default)]
    pub demo_url: String,
}

/// A starter page turned into real blocks and sections.
#[derive(Clone, Debug)]
pub struct BuiltPage {
    pub slug: String,
    pub title: String,
    pub kind: TabKind,
    pub collection: String,
    pub link_url: String,
    pub show_in_nav: bool,
    pub blocks: Vec<Block>,
    /// Slot of each block ("" = none), same order as `blocks`.
    pub slots: Vec<String>,
    pub sections: Vec<Section>,
}

fn fill_name(v: &mut Value, name: &str) {
    match v {
        Value::String(s) if s.contains("{{name}}") => *s = s.replace("{{name}}", name),
        Value::Array(a) => a.iter_mut().for_each(|x| fill_name(x, name)),
        Value::Object(o) => o.values_mut().for_each(|x| fill_name(x, name)),
        _ => {}
    }
}

impl StarterPage {
    /// Real blocks for this page, `{{name}}` replaced by the site name. A starter
    /// may not reference uploaded files: it is shared by every site.
    pub fn build(&self, name: &str) -> Result<BuiltPage, String> {
        let page = if self.slug.is_empty() { "trang chủ".to_string() } else { format!("/{}", self.slug) };
        let mut blocks = Vec::new();
        let mut slots = Vec::new();
        for (i, spec) in self.blocks.iter().enumerate() {
            let mut props = spec.props.clone();
            fill_name(&mut props, name);
            let (mut block, mut assets) = new_block(&spec.kind, &props).map_err(|e| format!("{page}, khối {}: {e}", i + 1))?;
            if !spec.style.is_null() {
                assets.extend(apply_style_patch(&mut block.style, &spec.style).map_err(|e| format!("{page}, khối {}: {e}", i + 1))?);
            }
            if !assets.is_empty() {
                return Err(format!("{page}, khối {}: mẫu không được tham chiếu tệp đã tải lên", i + 1));
            }
            if !spec.slot.is_empty() && !is_slot(&spec.slot) {
                return Err(format!("{page}, khối {}: slot không tồn tại: {}", i + 1, spec.slot));
            }
            blocks.push(block);
            slots.push(spec.slot.clone());
        }
        validate_tab_blocks(self.kind, &blocks).map_err(|e| format!("{page}: {e}"))?;
        let mut seen = std::collections::HashSet::new();
        for s in slots.iter().filter(|s| !s.is_empty()) {
            if !seen.insert(s.as_str()) {
                return Err(format!("{page}: slot {s} xuất hiện hai lần"));
            }
        }
        let mut built_sections = Vec::new();
        for s in &self.sections {
            let start = blocks.get(s.start).ok_or_else(|| format!("{page}: section bắt đầu ở khối không tồn tại"))?;
            let mut spec = s.settings.clone();
            spec.insert("start_block_id".into(), json!(start.id));
            let (section, assets) = sections::from_spec(&Value::Object(spec)).map_err(|e| format!("{page}: {e}"))?;
            if !assets.is_empty() {
                return Err(format!("{page}: mẫu không được tham chiếu tệp đã tải lên"));
            }
            built_sections.push(section);
        }
        Ok(BuiltPage {
            slug: self.slug.clone(),
            title: self.title.replace("{{name}}", name),
            kind: self.kind,
            collection: self.collection.clone(),
            link_url: self.link_url.clone(),
            show_in_nav: self.show_in_nav,
            blocks,
            slots,
            sections: built_sections,
        })
    }
}

fn valid_key(key: &str) -> bool {
    (2..=40).contains(&key.len()) && key.bytes().all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
        && !key.starts_with('-') && !key.ends_with('-')
}

/// Everything a catalog entry must satisfy before the engine stores it.
pub fn validate(entry: &ThemeEntry) -> Result<(), String> {
    if !valid_key(&entry.key) {
        return Err("key: 2–40 ký tự a-z, 0-9 và gạch nối".into());
    }
    if entry.label.trim().is_empty() || entry.label.chars().count() > 60 {
        return Err("Tên giao diện 1–60 ký tự".into());
    }
    if entry.description.chars().count() > 300 {
        return Err("Mô tả tối đa 300 ký tự".into());
    }
    theme::validate(&entry.look)?;
    if entry.pages.len() > 20 {
        return Err("Tối đa 20 trang mẫu".into());
    }
    if entry.pages.first().is_some_and(|p| !p.slug.is_empty()) {
        return Err("Trang mẫu đầu tiên phải là trang chủ (slug rỗng)".into());
    }
    let mut slugs = std::collections::HashSet::new();
    for p in &entry.pages {
        if !slugs.insert(p.slug.as_str()) {
            return Err(format!("Trang mẫu trùng đường dẫn: /{}", p.slug));
        }
        if !p.slug.is_empty() && crate::util::slugify(&p.slug) != p.slug {
            return Err(format!("Đường dẫn không hợp lệ: {}", p.slug));
        }
        if p.title.trim().is_empty() || p.title.chars().count() > 80 {
            return Err(format!("/{}: tên trang 1–80 ký tự", p.slug));
        }
        match p.kind {
            TabKind::Link if !p.link_url.starts_with('/') && !p.link_url.starts_with("https://") => {
                return Err(format!("/{}: tab liên kết cần đường dẫn / hoặc https://", p.slug));
            }
            TabKind::Collection if !COLLECTION_TEMPLATES.iter().any(|c| c.key == p.collection) => {
                return Err(format!("/{}: mẫu danh mục không tồn tại: {}", p.slug, p.collection));
            }
            _ => {}
        }
        p.build("Tên của bạn")?;
    }
    Ok(())
}

/// What a theme picker shows: no block data.
pub fn summary(entry: &ThemeEntry) -> Value {
    json!({
        "key": entry.key, "label": entry.label, "description": entry.description,
        "category": entry.category, "category_label": entry.category.label(),
        "featured": entry.featured, "order": entry.order,
        "palette": entry.look.colors, "font_heading": entry.look.font_heading, "font_body": entry.look.font_body,
        "header_style": entry.look.header_style,
        "pages": entry.pages.iter().map(|p| json!({"slug": p.slug, "title": p.title, "kind": p.kind})).collect::<Vec<_>>(),
        "slots": entry.pages.first().map(|p| p.blocks.iter().filter(|b| !b.slot.is_empty()).map(|b| b.slot.clone()).collect::<Vec<_>>()).unwrap_or_default(),
        "demo_url": entry.demo_url,
    })
}

pub fn categories_json() -> Value {
    json!(Category::ALL.iter().map(|c| json!({"key": c, "label": c.label()})).collect::<Vec<_>>())
}

pub fn slots_json() -> Value {
    json!(SLOTS.iter().map(|(k, l)| json!({"key": k, "label": l})).collect::<Vec<_>>())
}

// ───────────────────────────── Slots of a live page ─────────────────────────

fn links_to(b: &Block, path: &str) -> bool {
    b.props.to_string().contains(&format!("\"{path}"))
}

/// The slot each block of a live page fills, guessed from block types and
/// order (live blocks carry no slot). "" = no slot.
pub fn infer_slots(blocks: &[Block]) -> Vec<String> {
    let mut out = vec![String::new(); blocks.len()];
    let mut taken = std::collections::HashSet::new();
    let mut put = |out: &mut Vec<String>, i: usize, slot: &'static str| {
        if out[i].is_empty() && taken.insert(slot) {
            out[i] = slot.into();
        }
    };
    let visible: Vec<usize> = (0..blocks.len()).filter(|&i| !blocks[i].hidden).collect();
    if let Some(&first) = visible.first() {
        if matches!(blocks[first].kind.as_str(), "hero" | "heading") {
            put(&mut out, first, "opener");
        }
    }
    for &i in &visible {
        let b = &blocks[i];
        match b.kind.as_str() {
            "post_list" => put(&mut out, i, "posts"),
            "product_list" | "course_list" => put(&mut out, i, "products"),
            "collection_list" | "cards" | "rows" | "features" | "timeline" | "stats" => put(&mut out, i, "work"),
            "contact" | "subscribe" => put(&mut out, i, "contact"),
            "cta" | "heading" | "media_text" if links_to(b, "/community") => put(&mut out, i, "community"),
            _ => {}
        }
    }
    for &i in &visible {
        if matches!(blocks[i].kind.as_str(), "media_text" | "text" | "quote") {
            put(&mut out, i, "about");
            put(&mut out, i, "story");
        }
    }
    if let Some(&last) = visible.iter().rev().find(|&&i| blocks[i].kind == "cta") {
        put(&mut out, last, "contact");
    }
    out
}

// ───────────────────────────── Save a page as a starter ─────────────────────

/// A field's value with the owner's own words and files taken out: layout
/// choices stay, texts become the field's placeholder, files are dropped,
/// external links are dropped (internal ones stay), lists keep their length.
fn strip_value(field: &Field, v: &Value) -> Value {
    match &field.kind {
        Kind::Text { max, .. } | Kind::Markdown { max } => {
            if v.as_str().is_none_or(|s| s.trim().is_empty()) {
                json!("")
            } else {
                let text = field.default.as_str().filter(|d| !d.is_empty()).unwrap_or(&field.label);
                json!(text.chars().take(*max).collect::<String>())
            }
        }
        Kind::Url => match v.as_str() {
            Some(s) if s.starts_with('/') || s.starts_with('#') => json!(s),
            // An outside link is dropped; a required one falls back to its default ("#").
            _ => json!(field.default.as_str().filter(|d| !d.is_empty()).unwrap_or(if field.required { "#" } else { "" })),
        },
        Kind::Image | Kind::Video => json!(""),
        Kind::List { item, .. } => json!(v.as_array().map(|items| items.iter().map(|it| {
            let mut o = Map::new();
            for f in item {
                o.insert(f.key.clone(), strip_value(f, it.get(&f.key).unwrap_or(&Value::Null)));
            }
            Value::Object(o)
        }).collect::<Vec<_>>()).unwrap_or_default()),
        _ => v.clone(),
    }
}

/// A live block as a starter block, without the owner's words or files.
pub fn strip_block(b: &Block, slot: &str) -> StarterBlock {
    let mut props = Map::new();
    if let Some(bt) = block_type(&b.kind) {
        for f in &bt.fields {
            let v = b.props.get(&f.key).cloned().unwrap_or(Value::Null);
            let mut stripped = strip_value(f, &v);
            if slot == "opener" && f.key == "title" {
                stripped = json!("{{name}}");
            }
            props.insert(f.key.clone(), stripped);
        }
    }
    let mut style = serde_json::to_value(&b.style).unwrap_or(json!({}));
    if let Some(o) = style.as_object_mut() {
        if o.get("background_image").and_then(Value::as_str).is_some_and(|s| is_asset_ref(s) || !s.is_empty()) {
            o.insert("background_image".into(), json!(""));
        }
    }
    StarterBlock { slot: slot.into(), kind: b.kind.clone(), props: Value::Object(props), style }
}

/// A live page as a starter page (for "save this site as a theme").
pub fn strip_page(slug: &str, title: &str, kind: TabKind, collection: &str, link_url: &str, show_in_nav: bool, blocks: &[Block], sections: &[Section]) -> StarterPage {
    let visible: Vec<&Block> = blocks.iter().filter(|b| !b.hidden).collect();
    let owned: Vec<Block> = visible.iter().map(|b| (*b).clone()).collect();
    let slots = infer_slots(&owned);
    let starter_blocks: Vec<StarterBlock> = owned.iter().zip(&slots).map(|(b, s)| strip_block(b, s)).collect();
    let starter_sections = sections.iter().filter_map(|s| {
        let start = owned.iter().position(|b| b.id == s.start_block_id)?;
        let mut style = serde_json::to_value(&s.style).ok()?;
        if let Some(o) = style.as_object_mut() {
            o.insert("background_image".into(), json!(""));
        }
        let mut settings = Map::new();
        settings.insert("layout".into(), json!(s.layout));
        settings.insert("height".into(), json!(s.height));
        settings.insert("style".into(), style);
        Some(StarterSection { start, settings })
    }).collect();
    StarterPage {
        slug: slug.into(),
        title: title.into(),
        kind,
        collection: collection.into(),
        link_url: link_url.into(),
        show_in_nav,
        blocks: starter_blocks,
        sections: starter_sections,
    }
}

// ───────────────────────────── Port ─────────────────────────────────────────

#[derive(Clone, Debug, Default, Serialize)]
pub struct PagePlan {
    /// Slots whose content moves into the new layout: (slot, from block type, to block type).
    pub moved: Vec<(String, String, String)>,
    /// Slots of the new layout the page has nothing for (placeholder text stays).
    pub empty: Vec<String>,
    /// Blocks with no place in the new layout, kept at the end of the page: (type, title).
    pub kept_at_end: Vec<(String, String)>,
}

fn is_layout(field: &Field) -> bool {
    matches!(field.kind, Kind::Select { .. } | Kind::Number { .. } | Kind::Bool)
}

fn is_content(field: &Field) -> bool {
    matches!(field.kind, Kind::Text { .. } | Kind::Markdown { .. } | Kind::Image | Kind::Url | Kind::Video | Kind::List { .. })
}

fn block_title(b: &Block) -> String {
    ["title", "eyebrow", "body"].iter().find_map(|k| b.props.get(*k).and_then(Value::as_str).filter(|s| !s.is_empty()))
        .map(|s| s.chars().take(60).collect()).unwrap_or_default()
}

/// One block of the new layout, filled with the content of `old`.
fn fill(target: &Block, old: &Block) -> Block {
    let Some(bt) = block_type(&target.kind) else { return target.clone() };
    let mut out = target.clone();
    if target.kind == old.kind {
        // Same block type: keep every word, list and image; take the layout choices of the theme.
        let mut props = old.props.clone();
        if let (Some(p), Some(t)) = (props.as_object_mut(), target.props.as_object()) {
            for f in bt.fields.iter().filter(|f| is_layout(f)) {
                if let Some(v) = t.get(&f.key) {
                    p.insert(f.key.clone(), v.clone());
                }
            }
        }
        out.props = props;
        out.id = old.id.clone();
    } else if let (Some(p), Some(src)) = (out.props.as_object_mut(), old.props.as_object()) {
        // Another block type: carry over the content fields both types share.
        let old_type = block_type(&old.kind);
        for f in bt.fields.iter().filter(|f| is_content(f)) {
            let same_kind = old_type.and_then(|ot| ot.fields.iter().find(|of| of.key == f.key))
                .is_some_and(|of| std::mem::discriminant(&of.kind) == std::mem::discriminant(&f.kind));
            if let (true, Some(v)) = (same_kind, src.get(&f.key)) {
                p.insert(f.key.clone(), v.clone());
            }
        }
        if let Ok((props, _)) = blocks::validate_props(&target.kind, &Value::Object(p.clone())) {
            out.props = props;
        } else {
            out.props = target.props.clone();
        }
        out.id = old.id.clone();
    }
    if out.style.background_image.is_empty() && !old.style.background_image.is_empty() && target.kind == old.kind {
        out.style.background_image = old.style.background_image.clone();
    }
    out
}

/// Moves a page's own content into the layout of `starter`: each slot's block
/// takes the content of the page's block in the same slot; blocks with no
/// place are kept at the end, so nothing is lost.
pub fn port_page(current: &[Block], starter: &BuiltPage) -> (Vec<Block>, Vec<Section>, PagePlan) {
    let slots = infer_slots(current);
    let mut used = vec![false; current.len()];
    let mut plan = PagePlan::default();
    let mut out = Vec::new();
    let mut id_map = std::collections::HashMap::new();
    for (target, slot) in starter.blocks.iter().zip(&starter.slots) {
        let found = (!slot.is_empty()).then(|| slots.iter().enumerate().position(|(i, s)| s == slot && !used[i])).flatten();
        let block = match found {
            Some(i) => {
                used[i] = true;
                plan.moved.push((slot.clone(), current[i].kind.clone(), target.kind.clone()));
                fill(target, &current[i])
            }
            None => {
                if !slot.is_empty() {
                    plan.empty.push(slot.clone());
                }
                target.clone()
            }
        };
        id_map.insert(target.id.clone(), block.id.clone());
        out.push(block);
    }
    for (i, b) in current.iter().enumerate() {
        if !used[i] && !b.hidden {
            plan.kept_at_end.push((b.kind.clone(), block_title(b)));
            out.push(b.clone());
        }
    }
    let sections = starter.sections.iter().filter_map(|s| {
        let mut s = s.clone();
        s.start_block_id = id_map.get(&s.start_block_id)?.clone();
        Some(s)
    }).collect();
    (out, sections, plan)
}
