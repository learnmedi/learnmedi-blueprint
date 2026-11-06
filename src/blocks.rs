//! The block system.
//!
//! A page is an ordered list of `Block`s. Each block has a `type` from the
//! registry below, `props` validated against that type's field schema, and a
//! `style` (background, colours, spacing, width, alignment, entrance
//! animation). The same registry drives:
//!   - validation of every change (manual editor AND AI tool calls),
//!   - the property panel in the Studio (served as JSON),
//!   - the AI tool docs / system prompt,
//!   - server-side rendering (`render.rs`).
//!
//! Text fields marked "title" accept `*emphasis*`, rendered as an accent-coloured
//! <em> (the editorial look of tranhoangyen.com).

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};

use crate::tabs::{BlockGroup, COLLECTION_TEMPLATES, TAB_KINDS};
use crate::theme::{ANIMATIONS, normalize_color};

// ───────────────────────────── Data ─────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Block {
    pub id: String,
    #[serde(rename = "type")]
    pub kind: String,
    pub props: Value,
    #[serde(default)]
    pub style: BlockStyle,
    #[serde(default)]
    pub hidden: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BlockStyle {
    /// "" (transparent) | theme token (primary, accent, surface, …) | #hex
    #[serde(default)]
    pub background: String,
    /// "" (auto) | theme token | #hex
    #[serde(default)]
    pub text_color: String,
    /// Asset id or https URL of a background image.
    #[serde(default)]
    pub background_image: String,
    /// Dark overlay on background images, 0..80 (%).
    #[serde(default)]
    pub overlay: i64,
    /// none | sm | md | lg | xl
    #[serde(default = "d_md")]
    pub padding: String,
    /// left | center | right
    #[serde(default = "d_left")]
    pub align: String,
    /// narrow | normal | wide | full
    #[serde(default = "d_normal")]
    pub width: String,
    /// "" = theme default, or one of theme::ANIMATIONS
    #[serde(default)]
    pub animation: String,
    /// Optional HTML id so menus can link to `#anchor`.
    #[serde(default)]
    pub anchor: String,
}

fn d_md() -> String {
    "md".into()
}
fn d_left() -> String {
    "left".into()
}
fn d_normal() -> String {
    "normal".into()
}

impl Default for BlockStyle {
    fn default() -> Self {
        BlockStyle {
            background: String::new(),
            text_color: String::new(),
            background_image: String::new(),
            overlay: 0,
            padding: d_md(),
            align: d_left(),
            width: d_normal(),
            animation: String::new(),
            anchor: String::new(),
        }
    }
}

pub const PADDINGS: &[&str] = &["none", "sm", "md", "lg", "xl"];
pub const ALIGNS: &[&str] = &["left", "center", "right"];
pub const WIDTHS: &[&str] = &["narrow", "normal", "wide", "full"];

// ───────────────────────────── Schema ───────────────────────────────────────

#[derive(Debug)]
pub enum Kind {
    /// Single line. `title` = accepts *emphasis*.
    Text { max: usize, title: bool },
    /// Markdown (headings, lists, links, bold…). Raw HTML is escaped.
    Markdown { max: usize },
    Url,
    /// Asset id (uploaded image) or https URL.
    Image,
    /// Asset id (uploaded video) or YouTube/Vimeo/https URL.
    Video,
    Color,
    Select(&'static [&'static str]),
    Number { min: f64, max: f64 },
    Bool,
    Icon,
    List { item: &'static [Field], max: usize },
}

#[derive(Debug)]
pub struct Field {
    pub key: &'static str,
    pub label: &'static str,
    pub kind: Kind,
    pub required: bool,
    /// JSON literal.
    pub default: &'static str,
}

#[derive(Debug)]
pub struct BlockType {
    pub group: BlockGroup,
    pub key: &'static str,
    pub label: &'static str,
    pub description: &'static str,
    pub fields: &'static [Field],
}

const fn f(key: &'static str, label: &'static str, kind: Kind, default: &'static str) -> Field {
    Field { key, label, kind, required: false, default }
}
const fn req(key: &'static str, label: &'static str, kind: Kind, default: &'static str) -> Field {
    Field { key, label, kind, required: true, default }
}
const fn text(max: usize) -> Kind {
    Kind::Text { max, title: false }
}
const fn title(max: usize) -> Kind {
    Kind::Text { max, title: true }
}
const fn md(max: usize) -> Kind {
    Kind::Markdown { max }
}

pub const ICONS: &[&str] = &[
    "book", "research", "spark", "heart", "star", "school", "globe", "chat", "video", "pen", "users", "award", "leaf",
    "check", "calendar", "mail", "phone", "pin", "music", "code", "chart", "lightbulb", "child", "puzzle",
];

pub const BUTTON_FIELDS: &[Field] = &[
    req("label", "Chữ trên nút", text(60), "\"Xem thêm\""),
    req("url", "Liên kết", Kind::Url, "\"#\""),
    f("variant", "Kiểu", Kind::Select(&["primary", "secondary", "ghost"]), "\"primary\""),
    f("new_tab", "Mở tab mới", Kind::Bool, "false"),
];

const IMAGE_ITEM: &[Field] = &[
    req("image", "Ảnh", Kind::Image, "\"\""),
    f("alt", "Mô tả ảnh (cho người khiếm thị/SEO)", text(200), "\"\""),
    f("caption", "Chú thích", text(200), "\"\""),
];

const TAG_ITEM: &[Field] = &[req("text", "Nội dung", text(60), "\"\"")];

const FEATURE_ITEM: &[Field] = &[
    f("icon", "Biểu tượng", Kind::Icon, "\"star\""),
    req("title", "Tiêu đề", title(120), "\"Tiêu đề\""),
    f("body", "Nội dung", md(1200), "\"\""),
    f("link", "Liên kết", Kind::Url, "\"\""),
    f("link_label", "Chữ liên kết", text(60), "\"\""),
];

const CARD_ITEM: &[Field] = &[
    f("image", "Ảnh", Kind::Image, "\"\""),
    f("alt", "Mô tả ảnh", text(200), "\"\""),
    f("kicker", "Dòng nhỏ phía trên", text(80), "\"\""),
    req("title", "Tiêu đề", title(160), "\"Tiêu đề\""),
    f("subtitle", "Mô tả ngắn", md(600), "\"\""),
    f("meta", "Thông tin phụ (giá, định dạng…)", text(120), "\"\""),
    f("link", "Liên kết", Kind::Url, "\"\""),
    f("button_label", "Chữ nút", text(60), "\"\""),
];

const ROW_ITEM: &[Field] = &[
    f("index", "Số thứ tự hiển thị (i., 01…)", text(12), "\"\""),
    req("title", "Tiêu đề", title(160), "\"Tiêu đề\""),
    f("body", "Nội dung", md(1200), "\"\""),
    f("badge", "Nhãn", text(60), "\"\""),
    f("link", "Liên kết", Kind::Url, "\"\""),
];

const STAT_ITEM: &[Field] = &[req("value", "Con số", text(30), "\"100+\""), req("label", "Nhãn", text(120), "\"học sinh\"")];

const TIMELINE_ITEM: &[Field] = &[
    f("date", "Thời gian", text(60), "\"\""),
    req("title", "Tiêu đề", title(200), "\"Sự kiện\""),
    f("body", "Nội dung", md(1500), "\"\""),
    f("badge", "Nhãn (vd: Đã xuất bản)", text(60), "\"\""),
    f("link", "Liên kết", Kind::Url, "\"\""),
];

const FAQ_ITEM: &[Field] = &[req("question", "Câu hỏi", text(300), "\"Câu hỏi?\""), req("answer", "Trả lời", md(3000), "\"\"")];

const LINK_ITEM: &[Field] = &[req("label", "Tên", text(80), "\"\""), req("url", "Liên kết", Kind::Url, "\"\"")];

const QUOTE_ITEM: &[Field] = &[req("text", "Lời trích", md(800), "\"\""), f("author", "Người nói", text(120), "\"\""), f("role", "Vai trò", text(120), "\"\"")];

pub static REGISTRY: &[BlockType] = &[
    BlockType {
        group: BlockGroup::Presentation,
        key: "hero",
        label: "Banner đầu trang",
        description: "Phần mở đầu lớn: dòng nhỏ, tiêu đề lớn, mô tả, các vai trò dạng nhãn, câu trích, nút bấm và 1–2 ảnh. Layout split (chữ bên trái, ảnh bên phải) giống tranhoangyen.com.",
        fields: &[
            f("eyebrow", "Dòng nhỏ phía trên", text(80), "\"\""),
            req("title", "Tiêu đề lớn", title(160), "\"Xin chào, tôi là …\""),
            f("title_line2", "Dòng 2 của tiêu đề (vd tên tách hai dòng)", title(120), "\"\""),
            f("subtitle", "Dòng phụ dưới tiêu đề", text(160), "\"\""),
            f("body", "Đoạn giới thiệu", md(1500), "\"\""),
            f("tags", "Nhãn vai trò", Kind::List { item: TAG_ITEM, max: 8 }, "[]"),
            f("quote", "Câu trích (in nghiêng)", text(300), "\"\""),
            f("buttons", "Nút bấm", Kind::List { item: BUTTON_FIELDS, max: 3 }, "[]"),
            f("image", "Ảnh chính", Kind::Image, "\"\""),
            f("image_alt", "Mô tả ảnh chính", text(200), "\"\""),
            f("image_secondary", "Ảnh phụ (nhỏ, chồng lên)", Kind::Image, "\"\""),
            f("image_secondary_alt", "Mô tả ảnh phụ", text(200), "\"\""),
            f("badges", "Nhãn nổi trên ảnh", Kind::List { item: TAG_ITEM, max: 3 }, "[]"),
            f("layout", "Bố cục", Kind::Select(&["split", "split-reverse", "centered"]), "\"split\""),
        ],
    },
    BlockType {
        group: BlockGroup::Presentation,
        key: "heading",
        label: "Tiêu đề mục",
        description: "Đầu một mục: dòng nhỏ + tiêu đề + đoạn mô tả. Layout 'split' đặt mô tả bên phải tiêu đề.",
        fields: &[
            f("eyebrow", "Dòng nhỏ", text(80), "\"\""),
            req("title", "Tiêu đề", title(200), "\"Tiêu đề mục\""),
            f("body", "Mô tả", md(1500), "\"\""),
            f("level", "Cấp", Kind::Select(&["h1", "h2", "h3"]), "\"h2\""),
            f("layout", "Bố cục", Kind::Select(&["stacked", "split"]), "\"stacked\""),
        ],
    },
    BlockType {
        group: BlockGroup::Presentation,
        key: "text",
        label: "Đoạn văn",
        description: "Văn bản dài định dạng Markdown (tiêu đề ##, danh sách, **đậm**, *nghiêng*, liên kết).",
        fields: &[req("body", "Nội dung (Markdown)", md(20000), "\"Nội dung…\""), f("lead", "Chữ to (đoạn mở đầu)", Kind::Bool, "false")],
    },
    BlockType {
        group: BlockGroup::Presentation,
        key: "image",
        label: "Ảnh",
        description: "Một ảnh, có chú thích và liên kết tuỳ chọn.",
        fields: &[
            req("image", "Ảnh", Kind::Image, "\"\""),
            f("alt", "Mô tả ảnh", text(200), "\"\""),
            f("caption", "Chú thích", text(300), "\"\""),
            f("link", "Liên kết khi bấm", Kind::Url, "\"\""),
            f("size", "Kích thước", Kind::Select(&["small", "medium", "large", "full"]), "\"large\""),
            f("rounded", "Bo góc", Kind::Bool, "true"),
        ],
    },
    BlockType {
        group: BlockGroup::Presentation,
        key: "gallery",
        label: "Bộ sưu tập ảnh",
        description: "Lưới nhiều ảnh.",
        fields: &[
            req("images", "Ảnh", Kind::List { item: IMAGE_ITEM, max: 30 }, "[]"),
            f("columns", "Số cột", Kind::Number { min: 2.0, max: 5.0 }, "3"),
            f("style", "Kiểu", Kind::Select(&["grid", "masonry", "collage"]), "\"grid\""),
        ],
    },
    BlockType {
        group: BlockGroup::Presentation,
        key: "media_text",
        label: "Ảnh + chữ",
        description: "Một câu chuyện: tiêu đề, đoạn văn, câu trích và 1–3 ảnh ghép bên cạnh (trái/phải). Dùng cho 'Về tôi', 'Câu chuyện'.",
        fields: &[
            f("eyebrow", "Dòng nhỏ", text(80), "\"\""),
            req("title", "Tiêu đề", title(200), "\"Câu chuyện của tôi\""),
            f("body", "Nội dung", md(5000), "\"\""),
            f("quote", "Câu trích", text(400), "\"\""),
            f("quote_author", "Ký tên câu trích", text(120), "\"\""),
            f("images", "Ảnh (1–3)", Kind::List { item: IMAGE_ITEM, max: 3 }, "[]"),
            f("image_side", "Ảnh nằm bên", Kind::Select(&["left", "right"]), "\"left\""),
            f("buttons", "Nút", Kind::List { item: BUTTON_FIELDS, max: 3 }, "[]"),
        ],
    },
    BlockType {
        group: BlockGroup::Presentation,
        key: "features",
        label: "Lưới điểm nổi bật",
        description: "2–4 cột, mỗi ô có biểu tượng, tiêu đề, mô tả, liên kết. Có thể kèm tiêu đề mục (eyebrow/title/intro) ngay trong khối. numbered=true hiện số 01, 02… lớn và biểu tượng góc phải, như 'Bốn thế giới, một mục đích'.",
        fields: &[
            f("eyebrow", "Dòng nhỏ của mục", text(80), "\"\""),
            f("title", "Tiêu đề mục", title(200), "\"\""),
            f("intro", "Mô tả mục (bên phải tiêu đề)", md(1000), "\"\""),
            req("items", "Các ô", Kind::List { item: FEATURE_ITEM, max: 12 }, "[]"),
            f("columns", "Số cột", Kind::Number { min: 1.0, max: 4.0 }, "4"),
            f("card", "Hiện dạng thẻ", Kind::Bool, "true"),
            f("numbered", "Đánh số 01, 02…", Kind::Bool, "false"),
        ],
    },
    BlockType {
        group: BlockGroup::Presentation,
        key: "cards",
        label: "Thẻ (sách, sản phẩm, dự án)",
        description: "Lưới thẻ có ảnh bìa, dòng nhỏ, tiêu đề, mô tả, thông tin phụ (giá) và nút. Dùng cho sách, dự án, tài liệu. image_ratio=cover hiện bìa sách nguyên vẹn (không cắt, có bóng). Có thể kèm tiêu đề mục và hàng nút bên dưới.",
        fields: &[
            f("eyebrow", "Dòng nhỏ của mục", text(80), "\"\""),
            f("title", "Tiêu đề mục", title(200), "\"\""),
            f("intro", "Mô tả mục (bên phải tiêu đề)", md(1000), "\"\""),
            req("items", "Các thẻ", Kind::List { item: CARD_ITEM, max: 24 }, "[]"),
            f("columns", "Số cột", Kind::Number { min: 1.0, max: 4.0 }, "3"),
            f("image_ratio", "Tỉ lệ ảnh", Kind::Select(&["portrait", "square", "landscape", "cover"]), "\"portrait\""),
            f("buttons", "Nút bên dưới", Kind::List { item: BUTTON_FIELDS, max: 3 }, "[]"),
        ],
    },
    BlockType {
        group: BlockGroup::Presentation,
        key: "rows",
        label: "Danh sách hàng ngang",
        description: "Danh sách có số thứ tự lớn (i., ii., 01…), tiêu đề, mô tả và nhãn bên phải — như danh sách sản phẩm/dự án. Có thể kèm tiêu đề mục; đặt style.background tối (vd #14223f) để có dải nền xanh đậm.",
        fields: &[
            f("eyebrow", "Dòng nhỏ của mục", text(80), "\"\""),
            f("title", "Tiêu đề mục", title(200), "\"\""),
            f("intro", "Mô tả mục (bên phải tiêu đề)", md(1000), "\"\""),
            req("items", "Các hàng", Kind::List { item: ROW_ITEM, max: 30 }, "[]"),
        ],
    },
    BlockType {
        group: BlockGroup::Presentation,
        key: "buttons",
        label: "Hàng nút",
        description: "Một hoặc nhiều nút bấm.",
        fields: &[req("buttons", "Nút", Kind::List { item: BUTTON_FIELDS, max: 6 }, "[]")],
    },
    BlockType {
        group: BlockGroup::Presentation,
        key: "cta",
        label: "Kêu gọi hành động",
        description: "Dải nổi bật: tiêu đề + mô tả + nút (vd 'Liên hệ hợp tác', 'Đăng ký khoá học').",
        fields: &[
            f("eyebrow", "Dòng nhỏ", text(80), "\"\""),
            req("title", "Tiêu đề", title(200), "\"Cùng bắt đầu nhé\""),
            f("body", "Mô tả", md(1000), "\"\""),
            f("buttons", "Nút", Kind::List { item: BUTTON_FIELDS, max: 3 }, "[]"),
            f("layout", "Bố cục", Kind::Select(&["split", "centered"]), "\"split\""),
        ],
    },
    BlockType {
        group: BlockGroup::Presentation,
        key: "quote",
        label: "Trích dẫn",
        description: "Một hoặc nhiều câu trích dẫn lớn (phương châm, cảm nhận học sinh có thật).",
        fields: &[req("items", "Trích dẫn", Kind::List { item: QUOTE_ITEM, max: 6 }, "[]")],
    },
    BlockType {
        group: BlockGroup::Presentation,
        key: "video",
        label: "Video",
        description: "Video YouTube/Vimeo hoặc video tải lên.",
        fields: &[req("video", "Video (link YouTube/Vimeo hoặc video tải lên)", Kind::Video, "\"\""), f("caption", "Chú thích", text(300), "\"\"")],
    },
    BlockType {
        group: BlockGroup::Presentation,
        key: "stats",
        label: "Con số ấn tượng",
        description: "Hàng các con số + nhãn (vd 15 năm giảng dạy).",
        fields: &[req("items", "Con số", Kind::List { item: STAT_ITEM, max: 6 }, "[]")],
    },
    BlockType {
        group: BlockGroup::Presentation,
        key: "timeline",
        label: "Dòng thời gian / danh sách công trình",
        description: "Các mốc thời gian hoặc danh sách bài báo, công trình (thời gian, tiêu đề, mô tả, nhãn trạng thái).",
        fields: &[
            req("items", "Mốc", Kind::List { item: TIMELINE_ITEM, max: 100 }, "[]"),
            f("filterable", "Cho lọc theo nhãn", Kind::Bool, "false"),
        ],
    },
    BlockType {
        group: BlockGroup::Presentation,
        key: "faq",
        label: "Hỏi đáp",
        description: "Câu hỏi thường gặp (mở/đóng).",
        fields: &[req("items", "Câu hỏi", Kind::List { item: FAQ_ITEM, max: 40 }, "[]")],
    },
    BlockType {
        group: BlockGroup::Presentation,
        key: "contact",
        label: "Liên hệ",
        description: "Thông tin liên hệ: email, điện thoại, địa chỉ, các liên kết mạng xã hội.",
        fields: &[
            f("title", "Tiêu đề", title(160), "\"Liên hệ\""),
            f("body", "Mô tả", md(1500), "\"\""),
            f("email", "Email", text(160), "\"\""),
            f("phone", "Điện thoại", text(40), "\"\""),
            f("address", "Địa chỉ", text(300), "\"\""),
            f("links", "Liên kết", Kind::List { item: LINK_ITEM, max: 12 }, "[]"),
        ],
    },
    BlockType {
        key: "blog_index", group: BlockGroup::Core, label: "Blog",
        description: "Khối chính của tab Blog.",
        fields: &[
            f("layout", "Kiểu", Kind::Select(&["grid","list"]), "\"grid\""),
            f("per_page", "Số bài mỗi trang", Kind::Number { min: 1.0, max: 50.0 }, "12"),
            f("show_tags_filter", "Lọc thẻ", Kind::Bool, "true"),
            f("show_excerpt", "Hiện tóm tắt", Kind::Bool, "true"),
        ],
    },
    BlockType {
        key: "course_index", group: BlockGroup::Core, label: "Khoá học",
        description: "Khối chính của tab Khoá học.",
        fields: &[
            f("layout", "Kiểu", Kind::Select(&["grid","list"]), "\"grid\""),
            f("show_price", "Hiện giá", Kind::Bool, "true"),
        ],
    },
    BlockType {
        key: "collection_index", group: BlockGroup::Core, label: "Danh mục",
        description: "Khối chính của tab Danh mục.",
        fields: &[
            f("layout", "Kiểu", Kind::Select(&["timeline","cards","table"]), "\"timeline\""),
            f("group_by_year", "Nhóm theo năm", Kind::Bool, "true"),
            f("show_filters", "Hiện bộ lọc", Kind::Bool, "true"),
        ],
    },
    BlockType {
        key: "collection_list", group: BlockGroup::Excerpt, label: "Mục danh mục mới",
        description: "Trích các mục mới nhất của một tab Danh mục.",
        fields: &[
            f("title", "Tiêu đề", title(160), "\"Mục mới\""),
            f("page_id", "Tab danh mục (id hoặc slug)", text(100), "\"\""),
            f("limit", "Số mục", Kind::Number { min: 1.0, max: 24.0 }, "3"),
        ],
    },
    BlockType {
        group: BlockGroup::Excerpt,
        key: "post_list",
        label: "Bài viết mới nhất",
        description: "Tự động hiện các bài viết đã xuất bản mới nhất.",
        fields: &[
            f("title", "Tiêu đề", title(160), "\"Bài viết mới\""),
            f("limit", "Số bài", Kind::Number { min: 1.0, max: 24.0 }, "3"),
            f("layout", "Kiểu", Kind::Select(&["grid", "list"]), "\"grid\""),
            f("tag", "Chỉ hiện bài có thẻ", text(60), "\"\""),
        ],
    },
    BlockType {
        group: BlockGroup::Excerpt,
        key: "course_list",
        label: "Khoá học mới nhất",
        description: "Tự động hiện các khoá học đã xuất bản, có giá và nút đăng ký.",
        fields: &[
            f("title", "Tiêu đề", title(160), "\"Khoá học\""),
            f("limit", "Số khoá", Kind::Number { min: 1.0, max: 24.0 }, "6"),
            f("show_price", "Hiện giá", Kind::Bool, "true"),
        ],
    },
    BlockType {
        group: BlockGroup::Presentation,
        key: "subscribe",
        label: "Đăng ký nhận tin",
        description: "Ô nhập email để khách đăng ký nhận bản tin của site (xác nhận qua email).",
        fields: &[
            f("eyebrow", "Dòng nhỏ", text(80), "\"\""),
            req("title", "Tiêu đề", title(200), "\"Nhận bài viết mới qua email\""),
            f("body", "Mô tả", md(1000), "\"Mỗi khi có bài mới, mình sẽ gửi tới hộp thư của bạn. Huỷ bất cứ lúc nào.\""),
            f("ask_name", "Hỏi tên", Kind::Bool, "false"),
            f("placeholder", "Gợi ý ô email", text(80), "\"Email của bạn\""),
            f("button_label", "Chữ trên nút", text(40), "\"Đăng ký\""),
            f("success_message", "Lời cảm ơn", text(300), "\"\""),
            f("layout", "Bố cục", Kind::Select(&["split", "centered"]), "\"split\""),
        ],
    },
    BlockType {
        group: BlockGroup::Presentation,
        key: "divider",
        label: "Đường kẻ",
        description: "Đường phân cách giữa các mục.",
        fields: &[f("variant", "Kiểu", Kind::Select(&["line", "dots", "wave"]), "\"line\"")],
    },
    BlockType {
        group: BlockGroup::Presentation,
        key: "spacer",
        label: "Khoảng trống",
        description: "Khoảng trống dọc.",
        fields: &[f("size", "Độ cao", Kind::Select(&["sm", "md", "lg", "xl"]), "\"md\"")],
    },
];

pub fn block_type(key: &str) -> Option<&'static BlockType> {
    REGISTRY.iter().find(|b| b.key == key)
}

pub fn type_keys() -> Vec<&'static str> {
    REGISTRY.iter().map(|b| b.key).collect()
}

// ───────────────────────────── Validation ───────────────────────────────────

fn parse_default(field: &Field) -> Value {
    serde_json::from_str(field.default).unwrap_or(Value::Null)
}

/// Default props for a block type.
pub fn default_props(bt: &BlockType) -> Value {
    let mut m = Map::new();
    for fd in bt.fields {
        m.insert(fd.key.into(), parse_default(fd));
    }
    Value::Object(m)
}

/// `true` when the value is an uploaded asset id (not a URL).
pub fn is_asset_ref(v: &str) -> bool {
    !v.is_empty() && !v.contains(':') && !v.contains('/') && v.len() <= 40 && v.chars().all(|c| c.is_ascii_alphanumeric())
}

pub fn validate_url(v: &str) -> bool {
    let l = v.trim().to_ascii_lowercase();
    l.is_empty()
        || l.starts_with("https://")
        || l.starts_with("http://")
        || l.starts_with("mailto:")
        || l.starts_with("tel:")
        || (l.starts_with('/') && !l.starts_with("//"))
        || l.starts_with('#')
}

fn validate_field(fd: &Field, v: Option<&Value>, path: &str, assets: &mut Vec<String>) -> Result<Value, String> {
    let v = match v {
        None | Some(Value::Null) => parse_default(fd),
        Some(v) => v.clone(),
    };
    let p = if path.is_empty() { fd.key.to_string() } else { format!("{path}.{}", fd.key) };
    let as_string = |v: &Value| -> Result<String, String> {
        match v {
            Value::String(s) => Ok(s.clone()),
            Value::Number(n) => Ok(n.to_string()),
            Value::Bool(b) => Ok(b.to_string()),
            _ => Err(format!("{p}: phải là chuỗi")),
        }
    };
    let out = match &fd.kind {
        Kind::Text { max, .. } => {
            let s = as_string(&v)?.replace(['\n', '\r'], " ");
            let s = s.trim().to_string();
            if s.chars().count() > *max {
                return Err(format!("{p}: dài quá {max} ký tự"));
            }
            Value::String(s)
        }
        Kind::Markdown { max } => {
            let s = as_string(&v)?;
            if s.chars().count() > *max {
                return Err(format!("{p}: dài quá {max} ký tự"));
            }
            Value::String(s)
        }
        Kind::Url => {
            let s = as_string(&v)?.trim().to_string();
            if !validate_url(&s) {
                return Err(format!("{p}: liên kết không hợp lệ ('{s}'). Dùng https://…, mailto:, tel:, /trang hoặc #muc"));
            }
            Value::String(s)
        }
        Kind::Image | Kind::Video => {
            let s = as_string(&v)?.trim().to_string();
            if is_asset_ref(&s) {
                assets.push(s.clone());
            } else if !s.is_empty() && !s.to_ascii_lowercase().starts_with("https://") {
                return Err(format!("{p}: phải là id ảnh/video đã tải lên hoặc link https://"));
            }
            Value::String(s)
        }
        Kind::Color => {
            let s = as_string(&v)?;
            Value::String(normalize_color(&s).ok_or_else(|| format!("{p}: màu không hợp lệ '{s}'"))?)
        }
        Kind::Select(opts) => {
            let s = as_string(&v)?;
            if !opts.contains(&s.as_str()) {
                return Err(format!("{p}: '{s}' không hợp lệ, chọn một trong: {}", opts.join(", ")));
            }
            Value::String(s)
        }
        Kind::Icon => {
            let s = as_string(&v)?;
            Value::String(if ICONS.contains(&s.as_str()) { s } else { "star".into() })
        }
        Kind::Number { min, max } => {
            let n = match &v {
                Value::Number(n) => n.as_f64().unwrap_or(*min),
                Value::String(s) => s.trim().parse::<f64>().map_err(|_| format!("{p}: phải là số"))?,
                _ => return Err(format!("{p}: phải là số")),
            };
            let n = n.clamp(*min, *max);
            if n.fract() == 0.0 { json!(n as i64) } else { json!(n) }
        }
        Kind::Bool => match &v {
            Value::Bool(b) => Value::Bool(*b),
            Value::String(s) => Value::Bool(matches!(s.as_str(), "true" | "1" | "yes")),
            Value::Number(n) => Value::Bool(n.as_f64().unwrap_or(0.0) != 0.0),
            _ => return Err(format!("{p}: phải là true/false")),
        },
        Kind::List { item, max } => {
            let arr = v.as_array().ok_or_else(|| format!("{p}: phải là danh sách"))?;
            if arr.len() > *max {
                return Err(format!("{p}: tối đa {max} mục"));
            }
            let mut out = Vec::with_capacity(arr.len());
            for (i, it) in arr.iter().enumerate() {
                let obj = it.as_object().ok_or_else(|| format!("{p}[{i}]: phải là object"))?;
                out.push(validate_object(item, obj, &format!("{p}[{i}]"), assets)?);
            }
            Value::Array(out)
        }
    };
    if fd.required {
        let empty = match &out {
            Value::String(s) => s.is_empty(),
            Value::Array(a) => a.is_empty() && !matches!(fd.kind, Kind::List { .. }),
            _ => false,
        };
        if empty && !matches!(fd.kind, Kind::Image | Kind::Video) {
            return Err(format!("{p}: bắt buộc"));
        }
    }
    Ok(out)
}

fn validate_object(fields: &[Field], obj: &Map<String, Value>, path: &str, assets: &mut Vec<String>) -> Result<Value, String> {
    for k in obj.keys() {
        if !fields.iter().any(|f| f.key == k) {
            let allowed: Vec<_> = fields.iter().map(|f| f.key).collect();
            let p = if path.is_empty() { k.clone() } else { format!("{path}.{k}") };
            return Err(format!("{p}: trường không tồn tại. Các trường hợp lệ: {}", allowed.join(", ")));
        }
    }
    let mut out = Map::new();
    for fd in fields {
        out.insert(fd.key.into(), validate_field(fd, obj.get(fd.key), path, assets)?);
    }
    Ok(Value::Object(out))
}

/// Validate full props for a block type. Missing fields get defaults; unknown
/// fields are rejected. Returns (clean props, referenced asset ids).
pub fn validate_props(kind: &str, props: &Value) -> Result<(Value, Vec<String>), String> {
    let bt = block_type(kind).ok_or_else(|| format!("loại block không tồn tại: {kind}. Có: {}", type_keys().join(", ")))?;
    let empty = Map::new();
    let obj = match props {
        Value::Object(o) => o,
        Value::Null => &empty,
        _ => return Err("props phải là object".into()),
    };
    let mut assets = Vec::new();
    let v = validate_object(bt.fields, obj, "", &mut assets)?;
    Ok((v, assets))
}

/// Shallow merge: keys in `patch` replace keys in `base` (lists are replaced
/// wholesale — send the full list to change one item).
pub fn merge_props(base: &Value, patch: &Value) -> Value {
    let mut out = base.as_object().cloned().unwrap_or_default();
    if let Some(p) = patch.as_object() {
        for (k, v) in p {
            out.insert(k.clone(), v.clone());
        }
    }
    Value::Object(out)
}

/// Apply a style patch with validation; returns asset refs found.
pub fn apply_style_patch(style: &mut BlockStyle, patch: &Value) -> Result<Vec<String>, String> {
    let obj = patch.as_object().ok_or("style phải là object")?;
    let mut assets = Vec::new();
    for (k, v) in obj {
        let s = match v {
            Value::String(s) => s.clone(),
            Value::Number(n) => n.to_string(),
            Value::Null => String::new(),
            _ => return Err(format!("style.{k}: giá trị không hợp lệ")),
        };
        match k.as_str() {
            "background" => style.background = normalize_color(&s).ok_or_else(|| format!("style.background: màu không hợp lệ '{s}'"))?,
            "text_color" => style.text_color = normalize_color(&s).ok_or_else(|| format!("style.text_color: màu không hợp lệ '{s}'"))?,
            "background_image" => {
                let s = s.trim().to_string();
                if is_asset_ref(&s) {
                    assets.push(s.clone());
                } else if !s.is_empty() && !s.starts_with("https://") {
                    return Err("style.background_image: phải là id ảnh hoặc https://".into());
                }
                style.background_image = s;
            }
            "overlay" => style.overlay = s.parse::<i64>().map_err(|_| "style.overlay: phải là số 0..80")?.clamp(0, 80),
            "padding" => style.padding = one_of(&s, PADDINGS, "padding")?,
            "align" => style.align = one_of(&s, ALIGNS, "align")?,
            "width" => style.width = one_of(&s, WIDTHS, "width")?,
            "animation" => {
                style.animation = if s.is_empty() { s } else { one_of(&s, ANIMATIONS, "animation")? };
            }
            "anchor" => {
                let a = crate::util::slugify(&s);
                style.anchor = a;
            }
            other => return Err(format!("style.{other}: không tồn tại. Có: background, text_color, background_image, overlay, padding, align, width, animation, anchor")),
        }
    }
    Ok(assets)
}

fn one_of(v: &str, allowed: &[&str], name: &str) -> Result<String, String> {
    if allowed.contains(&v) { Ok(v.to_string()) } else { Err(format!("style.{name}: '{v}' không hợp lệ. Có: {}", allowed.join(", "))) }
}

/// Every asset id referenced by a block (props + style).
pub fn block_asset_refs(b: &Block) -> Vec<String> {
    let mut refs = validate_props(&b.kind, &b.props).map(|(_, a)| a).unwrap_or_default();
    if is_asset_ref(&b.style.background_image) {
        refs.push(b.style.background_image.clone());
    }
    refs
}

pub fn new_block(kind: &str, props: &Value) -> Result<(Block, Vec<String>), String> {
    let (props, assets) = validate_props(kind, props)?;
    Ok((Block { id: crate::util::short_id(), kind: kind.into(), props, style: BlockStyle::default(), hidden: false }, assets))
}

// ───────────────────────────── Schema export ────────────────────────────────

fn kind_json(k: &Kind) -> Value {
    match k {
        Kind::Text { max, title } => json!({"type": "text", "max": max, "emphasis": title}),
        Kind::Markdown { max } => json!({"type": "markdown", "max": max}),
        Kind::Url => json!({"type": "url"}),
        Kind::Image => json!({"type": "image"}),
        Kind::Video => json!({"type": "video"}),
        Kind::Color => json!({"type": "color"}),
        Kind::Select(o) => json!({"type": "select", "options": o}),
        Kind::Number { min, max } => json!({"type": "number", "min": min, "max": max}),
        Kind::Bool => json!({"type": "bool"}),
        Kind::Icon => json!({"type": "icon", "options": ICONS}),
        Kind::List { item, max } => json!({"type": "list", "max": max, "item": fields_json(item)}),
    }
}

fn fields_json(fields: &[Field]) -> Value {
    Value::Array(
        fields
            .iter()
            .map(|f| {
                let mut v = kind_json(&f.kind);
                v["key"] = json!(f.key);
                v["label"] = json!(f.label);
                v["required"] = json!(f.required);
                v["default"] = parse_default(f);
                v
            })
            .collect(),
    )
}

/// Full registry for the Studio property panel.
pub fn registry_json() -> Value {
    json!({
        "types": REGISTRY.iter().map(|b| json!({
            "key": b.key, "group": b.group, "label": b.label, "description": b.description, "fields": fields_json(b.fields)
        })).collect::<Vec<_>>(),
        "style": {
            "padding": PADDINGS, "align": ALIGNS, "width": WIDTHS, "animation": ANIMATIONS,
            "colors": crate::theme::COLOR_TOKENS,
        },
        "icons": ICONS,
        "tab_kinds": TAB_KINDS,
        "collection_templates": COLLECTION_TEMPLATES,
    })
}

fn kind_short(k: &Kind) -> String {
    match k {
        Kind::Text { title: true, .. } => "text(*nhấn*)".into(),
        Kind::Text { .. } => "text".into(),
        Kind::Markdown { .. } => "markdown".into(),
        Kind::Url => "url".into(),
        Kind::Image => "image".into(),
        Kind::Video => "video".into(),
        Kind::Color => "color".into(),
        Kind::Select(o) => o.join("|"),
        Kind::Number { min, max } => format!("number {min}..{max}"),
        Kind::Bool => "bool".into(),
        Kind::Icon => "icon".into(),
        Kind::List { item, max } => format!(
            "list≤{max}[{}]",
            item.iter().map(|f| format!("{}:{}", f.key, kind_short(&f.kind))).collect::<Vec<_>>().join(", ")
        ),
    }
}

/// Compact one-line-per-type catalog for the AI system prompt.
pub fn catalog_for_prompt() -> String {
    let mut s = String::new();
    for b in REGISTRY {
        s.push_str(&format!("- {} ({}): {}\n  props: ", b.key, b.label, b.description));
        s.push_str(
            &b.fields
                .iter()
                .map(|f| format!("{}{}:{}", f.key, if f.required { "*" } else { "" }, kind_short(&f.kind)))
                .collect::<Vec<_>>()
                .join("; "),
        );
        s.push('\n');
    }
    s
}

/// Detailed schema of one type (AI tool `get_block_schema`).
pub fn type_schema_json(key: &str) -> Option<Value> {
    block_type(key).map(|b| json!({"key": b.key, "group": b.group, "label": b.label, "description": b.description, "fields": fields_json(b.fields), "example_props": default_props(b)}))
}
