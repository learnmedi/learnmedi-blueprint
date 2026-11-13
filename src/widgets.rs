//! Widgets: site-wide elements on every public page, switched on by the owner.
//! Which feature owns a widget is declared in `ui/features/*.yaml`
//! (`surfaces.widgets`); the admin can switch a widget off for every site
//! (`PlanConfig.widgets`).

use serde::Serialize;

#[derive(Clone, Copy, Debug, Serialize)]
pub struct WidgetDef {
    pub key: &'static str,
    pub label: &'static str,
    pub description: &'static str,
    /// Where the owner switches it on.
    pub owner_switch: &'static str,
}

pub const FORTUNE_BUTTON: &str = "fortune_button";
pub const NEWSLETTER_PROMPT: &str = "newsletter_prompt";

pub const WIDGETS: &[WidgetDef] = &[
    WidgetDef {
        key: FORTUNE_BUTTON,
        label: "Nút xin xăm",
        description: "Ống xăm nhỏ ở góc mọi trang công khai; mỗi ngày một quẻ theo tính cách của chủ site.",
        owner_switch: "Cài đặt → Tính cách → Ống xăm mỗi ngày",
    },
    WidgetDef {
        key: NEWSLETTER_PROMPT,
        label: "Lời mời đăng ký bản tin",
        description: "Ô nhỏ góc dưới phải mời người đã đăng nhập (chưa đăng ký) nhận bản tin.",
        owner_switch: "Có khi site dùng Bản tin email",
    },
];

pub fn widget(key: &str) -> Option<&'static WidgetDef> {
    WIDGETS.iter().find(|w| w.key == key)
}
