// Generated automatically by iced_lucide at build time.
// Do not edit manually.
// b2dfd05fe1801955827e8c3842b328e3cc7c3722052de9a4d062a16dfede13f9
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
    ("volume_1", "\u{E1AC}"),
    ("volume_2", "\u{E1A9}"),
    ("volume_3", "\u{E1AA}"),
    ("volume_4", "\u{E1AB}"),
    ("volume_mute", "\u{E626}"),
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

pub fn volume_1<'a>() -> Text<'a> {
    icon("\u{E1AC}")
}

pub fn volume_2<'a>() -> Text<'a> {
    icon("\u{E1A9}")
}

pub fn volume_3<'a>() -> Text<'a> {
    icon("\u{E1AA}")
}

pub fn volume_4<'a>() -> Text<'a> {
    icon("\u{E1AB}")
}

pub fn volume_mute<'a>() -> Text<'a> {
    icon("\u{E626}")
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
