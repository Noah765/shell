use iced::{
    Color, Font,
    font::{Family, Weight},
    theme::Style,
};

use crate::{cli::Cli, shell::Shell};

mod background;
mod bar;
mod calculator;
mod cli;
mod debug;
mod icon;
mod shell;

fn main() -> iced::Result {
    let mut cli = Cli::build();
    cli.background_color.a = cli.bar_opacity;

    iced::daemon(move || Shell::new(cli.clone()), Shell::update, Shell::view)
        .title("shell")
        .subscription(Shell::subscription)
        .font(icon::FONT)
        .default_font(Font {
            family: Family::Monospace,
            weight: Weight::Medium,
            ..Default::default()
        })
        .style(|_, theme| Style {
            background_color: Color::TRANSPARENT,
            text_color: theme.palette().text,
            icon_color: theme.palette().text,
        })
        .theme(Shell::theme)
        .run()
}
