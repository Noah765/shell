use clap::Parser;
use iced::{
    Color, Font, Theme,
    font::{Family, Weight},
    theme::{Palette, Style},
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
    let mut cli = Cli::parse();
    cli.background_color.a = cli.bar_opacity;

    let palette = Palette {
        background: cli.background_color,
        text: cli.text_color,
        primary: cli.primary_color,
        success: cli.green,
        warning: cli.yellow,
        danger: cli.red,
    };

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
        .theme(Theme::custom("Custom", palette))
        .run()
}
