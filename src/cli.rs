use std::path::PathBuf;

use clap::{
    Parser,
    builder::{
        Styles,
        styling::{AnsiColor, Color, Style},
    },
};

const YELLOW: Option<Color> = Some(Color::Ansi(AnsiColor::Yellow));
const GREEN: Option<Color> = Some(Color::Ansi(AnsiColor::Green));
const RED: Option<Color> = Some(Color::Ansi(AnsiColor::Red));

const STYLES: Styles = Styles::styled()
    .header(Style::new().fg_color(YELLOW).bold())
    .usage(Style::new().fg_color(YELLOW).bold())
    .literal(Style::new().fg_color(GREEN).bold())
    .placeholder(Style::new().fg_color(GREEN))
    .valid(Style::new().fg_color(GREEN).bold())
    .invalid(Style::new().fg_color(RED).bold())
    .context(Style::new().fg_color(GREEN));

/// A minimal desktop shell
#[derive(Clone, Debug, Parser)]
#[command(version, styles = STYLES)]
pub struct Cli {
    /// The wallpaper background image
    #[arg(long, value_name = "PATH")]
    pub wallpaper_background: PathBuf,

    /// The wallpaper middle ground image
    #[arg(long, value_name = "PATH")]
    pub wallpaper_middle_ground: PathBuf,

    /// The wallpaper foreground image
    #[arg(long, value_name = "PATH")]
    pub wallpaper_foreground: PathBuf,

    /// The background color
    #[arg(long, value_name = "COLOR")]
    pub background_color: iced::Color,

    /// The text color
    #[arg(long, value_name = "COLOR")]
    pub text_color: iced::Color,

    /// The primary color
    #[arg(long, value_name = "COLOR")]
    pub primary_color: iced::Color,

    /// The bar opacity
    #[arg(long, value_name = "OPACITY")]
    pub bar_opacity: f32,
}
