// Generated automatically by iced_lucide at build time.
// Do not edit manually.
// 0fafbce6b5df007d4314c068be79a8ee591aa748d21c2abc71f245fd69152847
use iced::Font;
use iced::widget::{Text, text};

pub const FONT: &[u8] = include_bytes!("../fonts/lucide.ttf");

/// All icons as `(name, codepoint_str)` pairs.
/// Use this to populate an icon-picker widget.
#[allow(dead_code)]
pub const ALL_ICONS: &[(&str, &str)] = &[
    ("wifi_1", "\u{E5F9}"),
    ("wifi_2", "\u{E5F8}"),
    ("wifi_3", "\u{E5F7}"),
    ("wifi_4", "\u{E1AE}"),
    ("wifi_off", "\u{E1AF}"),
];

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
