// Generated automatically by iced_lucide at build time.
// Do not edit manually.
// 6dfca0c300c2f30236edb62dd1930e16edd27d51b60aa5551b8f4914eb2de39f
use iced::Font;
use iced::widget::{Text, text};

pub const FONT: &[u8] = include_bytes!("../fonts/lucide.ttf");

/// All icons as `(name, codepoint_str)` pairs.
/// Use this to populate an icon-picker widget.
#[allow(dead_code)]
pub const ALL_ICONS: &[(&str, &str)] = &[
    ("wifi_off", "\u{E1AF}"),
    ("wifi_strong", "\u{E5F7}"),
    ("wifi_strongest", "\u{E1AE}"),
    ("wifi_weak", "\u{E5F8}"),
    ("wifi_weakest", "\u{E5F9}"),
];

pub fn wifi_off<'a>() -> Text<'a> {
    icon("\u{E1AF}")
}

pub fn wifi_strong<'a>() -> Text<'a> {
    icon("\u{E5F7}")
}

pub fn wifi_strongest<'a>() -> Text<'a> {
    icon("\u{E1AE}")
}

pub fn wifi_weak<'a>() -> Text<'a> {
    icon("\u{E5F8}")
}

pub fn wifi_weakest<'a>() -> Text<'a> {
    icon("\u{E5F9}")
}

/// Render any Lucide icon by its codepoint string.
/// Use this together with [`ALL_ICONS`] to display icons dynamically:
/// ```ignore
/// for (name, cp) in ALL_ICONS {
///     button(render(cp)).on_press(Msg::Pick(name.to_string()))
/// }
/// ```
pub fn render(codepoint: &str) -> Text<'_> {
    text(codepoint).font(Font::with_name("lucide"))
}

fn icon(codepoint: &str) -> Text<'_> {
    render(codepoint)
}
