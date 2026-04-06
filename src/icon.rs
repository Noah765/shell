// Generated automatically by iced_lucide at build time.
// Do not edit manually.
// 407827eed8963e4ff0fedb48cae57ce133a15df27f65518fe8a784f402b31a48
use iced::Font;
use iced::widget::{Text, text};

pub const FONT: &[u8] = include_bytes!("../fonts/lucide.ttf");

/// All icons as `(name, codepoint_str)` pairs.
/// Use this to populate an icon-picker widget.
#[allow(dead_code)]
pub const ALL_ICONS: &[(&str, &str)] = &[
    ("battery_1", "\u{E053}"),
    ("battery_2", "\u{E056}"),
    ("battery_3", "\u{E057}"),
    ("battery_4", "\u{E055}"),
    ("battery_charging", "\u{E054}"),
    ("wifi_1", "\u{E5F9}"),
    ("wifi_2", "\u{E5F8}"),
    ("wifi_3", "\u{E5F7}"),
    ("wifi_4", "\u{E1AE}"),
    ("wifi_off", "\u{E1AF}"),
];

pub fn battery_1<'a>() -> Text<'a> {
    icon("\u{E053}")
}

pub fn battery_2<'a>() -> Text<'a> {
    icon("\u{E056}")
}

pub fn battery_3<'a>() -> Text<'a> {
    icon("\u{E057}")
}

pub fn battery_4<'a>() -> Text<'a> {
    icon("\u{E055}")
}

pub fn battery_charging<'a>() -> Text<'a> {
    icon("\u{E054}")
}

pub fn wifi_1<'a>() -> Text<'a> {
    icon("\u{E5F9}")
}

pub fn wifi_2<'a>() -> Text<'a> {
    icon("\u{E5F8}")
}

pub fn wifi_3<'a>() -> Text<'a> {
    icon("\u{E5F7}")
}

pub fn wifi_4<'a>() -> Text<'a> {
    icon("\u{E1AE}")
}

pub fn wifi_off<'a>() -> Text<'a> {
    icon("\u{E1AF}")
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
