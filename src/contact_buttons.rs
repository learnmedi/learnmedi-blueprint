//! Contact buttons: round Zalo / phone / Messenger buttons fixed in the corner of
//! every public page (widget `contact_buttons`, feature `site.contact_buttons`).
//!
//! The owner gives a phone number, a Zalo number and/or a Facebook page; the site
//! builds the links itself (`zalo.me/…`, `tel:…`, `m.me/…`). No HTML or script from
//! the owner ever reaches the page, so a chat widget that needs a third-party
//! script (Zalo OA chat) is not part of this.

use serde::{Deserialize, Serialize};

/// The owner's settings (`SiteContent.contact_buttons`). Empty value = no button.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct ContactButtons {
    #[serde(default)]
    pub enabled: bool,
    /// Digits of the Zalo phone number (`0912015746`, or `84912015746`).
    #[serde(default)]
    pub zalo: String,
    /// Phone number for `tel:` (digits, optional leading `+`).
    #[serde(default)]
    pub phone: String,
    /// Facebook page username or numeric id, for `https://m.me/<it>`.
    #[serde(default)]
    pub messenger: String,
}

impl ContactButtons {
    pub fn zalo_url(&self) -> Option<String> {
        (!self.zalo.is_empty()).then(|| format!("https://zalo.me/{}", self.zalo))
    }

    pub fn phone_url(&self) -> Option<String> {
        (!self.phone.is_empty()).then(|| format!("tel:{}", self.phone))
    }

    pub fn messenger_url(&self) -> Option<String> {
        (!self.messenger.is_empty()).then(|| format!("https://m.me/{}", self.messenger))
    }

    /// Switched on and at least one button to show.
    pub fn visible(&self) -> bool {
        self.enabled && !(self.zalo.is_empty() && self.phone.is_empty() && self.messenger.is_empty())
    }
}

fn digits_of(raw: &str) -> (bool, String) {
    let t = raw.trim();
    let plus = t.starts_with('+');
    (plus, t.chars().filter(char::is_ascii_digit).collect())
}

/// Only digits and the usual separators (space . - ( )), with an optional leading +.
fn looks_like_number(raw: &str) -> bool {
    let t = raw.trim().trim_start_matches('+');
    !t.is_empty() && t.chars().all(|c| c.is_ascii_digit() || matches!(c, ' ' | '.' | '-' | '(' | ')'))
}

/// A phone number for `tel:`: `0903 434 340` → `0903434340`, `+84 903…` → `+84903…`.
/// Empty input clears the button.
pub fn normalize_phone(raw: &str) -> Result<String, String> {
    if raw.trim().is_empty() {
        return Ok(String::new());
    }
    if !looks_like_number(raw) {
        return Err(format!("Số điện thoại '{}' chỉ được có chữ số (và dấu +, cách, chấm, gạch).", raw.trim()));
    }
    let (plus, d) = digits_of(raw);
    if !(8..=15).contains(&d.len()) {
        return Err(format!("Số điện thoại '{}' phải có 8–15 chữ số.", raw.trim()));
    }
    Ok(if plus { format!("+{d}") } else { d })
}

/// The Zalo number: a phone number or a `zalo.me/<số>` link → the digits.
/// `+84 912 015 746` → `84912015746`. Empty input clears the button.
pub fn normalize_zalo(raw: &str) -> Result<String, String> {
    let t = raw.trim();
    if t.is_empty() {
        return Ok(String::new());
    }
    let lower = t.to_ascii_lowercase();
    let rest = ["https://zalo.me/", "http://zalo.me/", "zalo.me/"].iter().find_map(|p| lower.strip_prefix(p).map(|_| &t[p.len()..]));
    let candidate = rest.map(|r| r.trim_end_matches('/')).unwrap_or(t);
    if !looks_like_number(candidate) {
        return Err(format!("Zalo '{t}': nhập số điện thoại Zalo (vd 0912 015 746) hoặc link zalo.me/<số>."));
    }
    let (_, d) = digits_of(candidate);
    if !(8..=15).contains(&d.len()) {
        return Err(format!("Số Zalo '{t}' phải có 8–15 chữ số."));
    }
    Ok(d)
}

/// The Facebook page for Messenger: a username, a numeric id, or an `m.me/…`,
/// `facebook.com/…` link → the username. Empty input clears the button.
pub fn normalize_messenger(raw: &str) -> Result<String, String> {
    let t = raw.trim();
    if t.is_empty() {
        return Ok(String::new());
    }
    let lower = t.to_ascii_lowercase();
    let mut name = t;
    for p in ["https://", "http://"] {
        if lower.starts_with(p) {
            name = &t[p.len()..];
            break;
        }
    }
    let lname = name.to_ascii_lowercase();
    for host in ["www.facebook.com/", "facebook.com/", "m.facebook.com/", "fb.com/", "m.me/"] {
        if lname.starts_with(host) {
            name = &name[host.len()..];
            break;
        }
    }
    let name = name.split(['?', '#']).next().unwrap_or("").trim_end_matches('/');
    let ok = (2..=100).contains(&name.len()) && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '.');
    if !ok || name.contains('/') {
        return Err(format!("Messenger '{t}': nhập tên trang Facebook (vd ten.trang) hoặc link m.me/ten.trang."));
    }
    Ok(name.to_string())
}
