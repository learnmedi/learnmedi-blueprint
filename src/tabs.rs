//! Definitions and validation for the tabs that make up a public site.
use serde::{Deserialize, Serialize};

use crate::blocks::{Block, REGISTRY};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TabKind {
    #[default]
    Free,
    Blog,
    Courses,
    Collection,
    Link,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BlockGroup { Presentation, Core, Excerpt }

#[derive(Serialize)]
pub struct TabKindDef {
    pub key: TabKind,
    pub label: &'static str,
    pub core_block: Option<&'static str>,
    pub allowed_block_groups: &'static [BlockGroup],
    pub default_blocks: &'static [&'static str],
}

use BlockGroup::*;
pub const TAB_KINDS: &[TabKindDef] = &[
    TabKindDef { key: TabKind::Free, label: "Tự do", core_block: None, allowed_block_groups: &[Presentation, Excerpt], default_blocks: &["heading"] },
    TabKindDef { key: TabKind::Blog, label: "Blog", core_block: Some("blog_index"), allowed_block_groups: &[Presentation, Core, Excerpt], default_blocks: &["blog_index"] },
    TabKindDef { key: TabKind::Courses, label: "Khoá học", core_block: Some("course_index"), allowed_block_groups: &[Presentation, Core, Excerpt], default_blocks: &["course_index"] },
    TabKindDef { key: TabKind::Collection, label: "Danh mục", core_block: Some("collection_index"), allowed_block_groups: &[Presentation, Core, Excerpt], default_blocks: &["collection_index"] },
    TabKindDef { key: TabKind::Link, label: "Liên kết", core_block: None, allowed_block_groups: &[], default_blocks: &[] },
];

impl TabKind {
    pub fn definition(self) -> &'static TabKindDef {
        TAB_KINDS.iter().find(|def| def.key == self).expect("tab kind defined")
    }
}

pub fn validate_tab_blocks(kind: TabKind, blocks: &[Block]) -> Result<(), String> {
    let def = kind.definition();
    let mut cores = 0;
    for block in blocks {
        let group = REGISTRY.iter().find(|ty| ty.key == block.kind).map(|ty| ty.group)
            .ok_or_else(|| format!("Khối không hợp lệ: {}", block.kind))?;
        if !def.allowed_block_groups.contains(&group) {
            return Err(format!("Tab {} không cho phép khối {}.", def.label, block.kind));
        }
        if group == Core {
            if def.core_block != Some(block.kind.as_str()) || block.hidden {
                return Err("Khối chính phải đúng loại và luôn hiển thị.".into());
            }
            cores += 1;
        }
    }
    if cores != usize::from(def.core_block.is_some()) {
        return Err("Tab cần đúng một khối chính của loại này.".into());
    }
    Ok(())
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CollectionSettings {
    pub template: String,
    pub item_label: String,
    pub fields: Vec<String>,
}

#[derive(Serialize)]
pub struct CollectionTemplate {
    pub key: &'static str,
    pub label: &'static str,
    pub item_label: &'static str,
    pub fields: &'static [&'static str],
}
pub const COLLECTION_TEMPLATES: &[CollectionTemplate] = &[
    CollectionTemplate { key: "research", label: "Nghiên cứu", item_label: "Công trình", fields: &["authors","year","venue","doi","link_url","tags","summary","body_md"] },
    CollectionTemplate { key: "projects", label: "Dự án", item_label: "Dự án", fields: &["date","year","authors","link_url","tags","summary","body_md","cover_asset_id"] },
    CollectionTemplate { key: "achievements", label: "Thành tích", item_label: "Thành tích", fields: &["year","date","venue","link_url","tags","summary","body_md"] },
    CollectionTemplate { key: "documents", label: "Tài liệu", item_label: "Tài liệu", fields: &["year","authors","attachment_asset_id","link_url","tags","summary","body_md"] },
    CollectionTemplate { key: "custom", label: "Tuỳ chỉnh", item_label: "Mục", fields: &["year","link_url","tags","summary","body_md"] },
];

impl CollectionSettings {
    pub fn for_template(key: &str) -> Option<Self> {
        let def = COLLECTION_TEMPLATES.iter().find(|def| def.key == key)?;
        Some(Self {
            template: def.key.into(), item_label: def.item_label.into(),
            fields: def.fields.iter().map(|field| (*field).into()).collect(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::blocks::new_block;
    use serde_json::json;

    #[test]
    fn validates_every_tab_and_rejects_missing_or_duplicate_core() {
        for def in TAB_KINDS {
            let blocks: Vec<_> = def.default_blocks.iter().map(|key| new_block(key, &json!({})).unwrap().0).collect();
            assert!(validate_tab_blocks(def.key, &blocks).is_ok());
            if def.core_block.is_some() {
                assert!(validate_tab_blocks(def.key, &[]).is_err());
                assert!(validate_tab_blocks(def.key, &[blocks[0].clone(), blocks[0].clone()]).is_err());
            }
        }
    }
}
