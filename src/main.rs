use iced::{
    Color, Font, Theme, color,
    font::{Family, Weight},
    theme::{Palette, Style},
};

use crate::shell::Shell;

mod background;
mod bar;
mod shell;

fn main() -> iced::Result {
    iced::daemon(Shell::new, Shell::update, Shell::view)
        .title("shell")
        .subscription(Shell::subscription)
        .default_font(Font {
            family: Family::Monospace,
            weight: Weight::Medium,
            ..Default::default()
        })
        .style(|_, theme: &Theme| Style {
            background_color: Color::TRANSPARENT,
            text_color: theme.palette().text,
            icon_color: theme.palette().text,
        })
        .theme(Theme::custom(
            "Everforest",
            Palette {
                background: color!(0x2d353b, 0.75),
                text: color!(0xd3c6aa),
                primary: color!(0xa7c080),
                success: color!(0xa7c080),
                warning: color!(0xdbbc7f),
                danger: color!(0xe67e80),
            },
        ))
        .run()
}
