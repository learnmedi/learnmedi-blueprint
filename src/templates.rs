pub const TEMPLATES: &[(&str, &str)] = &[
    ("profile", "Hồ sơ giáo viên / nhà nghiên cứu (giống tranhoangyen.com)"),
    ("tutor", "Gia sư / trung tâm nhỏ — tập trung khoá học"),
    ("blank", "Trang trắng"),
];
use serde::Deserialize;
use serde_json::{Value, json};


#[derive(Deserialize)]
pub struct PageTemplate {
    pub key: String,
    pub label: String,
    pub description: String,
    pub blocks: Vec<Value>,
    pub sections: Vec<TemplateSection>,
}

#[derive(Deserialize)]
pub struct TemplateSection {
    pub start: usize,
    #[serde(flatten)]
    pub settings: serde_json::Map<String, Value>,
}

static PAGE_TEMPLATES: std::sync::LazyLock<Vec<PageTemplate>> = std::sync::LazyLock::new(|| {
    serde_yaml_ng::from_str(include_str!("../ui/page-templates.yaml")).expect("valid bundled page templates")
});

pub fn page_template(key: &str) -> Option<&'static PageTemplate> {
    PAGE_TEMPLATES.iter().find(|template| template.key == key)
}

pub fn page_templates_json() -> Value {
    json!(PAGE_TEMPLATES.iter().map(|template| json!({
        "key": template.key, "label": template.label, "description": template.description,
    })).collect::<Vec<_>>())
}
