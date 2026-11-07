//! Shared section settings for the renderer, Studio and the API.
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::blocks::{BlockStyle, apply_style_patch};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Section {
    pub start_block_id: String,
    pub layout: String,
    pub height: String,
    pub style: BlockStyle,
}

#[derive(Serialize, Deserialize)]
struct OptionDef {
    key: String,
    label: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
}

#[derive(Serialize, Deserialize)]
struct Catalog {
    layouts: Vec<OptionDef>,
    heights: Vec<OptionDef>,
}

static CATALOG: std::sync::LazyLock<Catalog> = std::sync::LazyLock::new(|| {
    serde_yaml_ng::from_str(include_str!("../ui/sections.yaml")).expect("valid bundled section catalog")
});

pub fn schema_json() -> Value {
    json!(&*CATALOG)
}

pub fn from_spec(spec: &Value) -> Result<(Section, Vec<String>), String> {
    let spec = spec.as_object().ok_or("section phải là object")?;
    for key in spec.keys() {
        if !matches!(key.as_str(), "start_block_id" | "layout" | "height" | "style") {
            return Err(format!("Thuộc tính section không hỗ trợ: {key}"));
        }
    }
    let start_block_id = spec.get("start_block_id").and_then(Value::as_str)
        .filter(|id| !id.is_empty()).ok_or("Cần start_block_id")?;
    let layout = choice(spec.get("layout"), "stack", &CATALOG.layouts, "layout")?;
    let height = choice(spec.get("height"), "content", &CATALOG.heights, "height")?;
    let mut style = BlockStyle::default();
    let assets = if let Some(patch) = spec.get("style") {
        apply_style_patch(&mut style, patch)?
    } else {
        vec![]
    };
    Ok((Section { start_block_id: start_block_id.into(), layout, height, style }, assets))
}

fn choice(value: Option<&Value>, default: &str, options: &[OptionDef], field: &str) -> Result<String, String> {
    let value = match value {
        Some(value) => value.as_str().ok_or_else(|| format!("{field} phải là chuỗi"))?,
        None => default,
    };
    if !options.iter().any(|option| option.key == value) {
        return Err(format!("{field} không hợp lệ: {value}"));
    }
    Ok(value.into())
}
