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
    /// Wallpaper directory or image
    #[arg(long, value_name = "PATH")]
    pub wallpapers: Option<PathBuf>,

    /// Background color
    #[arg(long, value_name = "COLOR", default_value = "#2d353b")]
    pub background_color: iced::Color,

    /// Text color
    #[arg(long, value_name = "COLOR", default_value = "#d3c6aa")]
    pub text_color: iced::Color,

    /// Primary color
    #[arg(long, value_name = "COLOR", default_value = "#a7c080")]
    pub primary_color: iced::Color,

    /// Red color
    #[arg(long, value_name = "COLOR", default_value = "#e67e80")]
    pub red: iced::Color,

    /// Green color
    #[arg(long, value_name = "COLOR", default_value = "#a7c080")]
    pub green: iced::Color,

    /// Yellow color
    #[arg(long, value_name = "COLOR", default_value = "#dbbc7f")]
    pub yellow: iced::Color,

    /// Blue color
    #[arg(long, value_name = "COLOR", default_value = "#7fbbb3")]
    pub blue: iced::Color,

    /// Magenta color
    #[arg(long, value_name = "COLOR", default_value = "#d699b6")]
    pub magenta: iced::Color,

    /// Cyan color
    #[arg(long, value_name = "COLOR", default_value = "#83c092")]
    pub cyan: iced::Color,

    /// Bar opacity
    #[arg(long, value_name = "OPACITY", default_value_t = 0.75)]
    pub bar_opacity: f32,
}
