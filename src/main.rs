use clap::Parser;
use iced::{
    Color, Font, Theme, color,
    font::{Family, Weight},
    theme::{Palette, Style},
};

use crate::{cli::Cli, shell::Shell};

mod audio;
mod background;
mod bar;
mod cli;
mod icon;
mod shell;
mod wifi;
mod workspace;

fn main() -> iced::Result {
    let mut cli = Cli::parse();
    cli.background_color.a = cli.bar_opacity;

    iced::daemon(
        move || {
            Shell::new(
                &cli.wallpaper_background,
                &cli.wallpaper_middle_ground,
                &cli.wallpaper_foreground,
            )
        },
        Shell::update,
        Shell::view,
    )
    .title("shell")
    .subscription(Shell::subscription)
    .font(icon::FONT)
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
        "Custom",
        Palette {
            background: cli.background_color,
            text: cli.text_color,
            primary: cli.primary_color,
            success: color!(0xff0000),
            warning: color!(0xff0000),
            danger: color!(0xff0000),
        },
    ))
    .run()
}
