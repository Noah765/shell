use iced::{Theme, color, theme::Palette};

use crate::shell::Shell;

mod background;
mod shell;

fn main() -> iced::Result {
    iced::daemon(Shell::new, Shell::update, Shell::view)
        .title("shell")
        .subscription(Shell::subscription)
        .theme(Theme::custom(
            "Everforest",
            Palette {
                background: color!(0x2d353b),
                text: color!(0xd3c6aa),
                primary: color!(0x7fbbb3),
                success: color!(0xa7c080),
                warning: color!(0xdbbc7f),
                danger: color!(0xe67e80),
            },
        ))
        .run()
}
