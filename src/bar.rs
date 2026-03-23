use chrono::{DateTime, Local};
use iced::{
    Background, Border, Element, Length, Radius, Task, Theme,
    alignment::{Horizontal, Vertical},
    padding,
    platform_specific::shell::commands::layer_surface,
    runtime::platform_specific::wayland::layer_surface::{
        IcedMargin, IcedOutput, SctkLayerSurfaceSettings,
    },
    widget::{Container, column, container::Style, text},
    window::Id,
};
use smithay_client_toolkit::{
    reexports::client::protocol::wl_output::WlOutput, shell::wlr_layer::Anchor,
};

use crate::shell::Message;

#[derive(Debug)]
pub struct Bar {
    surface_id: Id,
}

impl Bar {
    pub fn new(output: WlOutput) -> (Self, Task<Message>) {
        let surface_id = Id::unique();
        let task = layer_surface::get_layer_surface(SctkLayerSurfaceSettings {
            id: surface_id,
            input_zone: Some(Vec::new()),
            anchor: Anchor::TOP | Anchor::BOTTOM | Anchor::LEFT,
            output: IcedOutput::Output(output),
            namespace: String::from("shell-bar"),
            margin: IcedMargin {
                top: 5,
                right: 0,
                bottom: 5,
                left: 5,
            },
            size: Some((Some(32), None)),
            exclusive_zone: 32,
            ..Default::default()
        });
        (Self { surface_id }, task)
    }

    pub fn view(&self, now: DateTime<Local>) -> Element<'_, Message> {
        Container::new(
            column![
                text(now.format("%H\n%M").to_string()).height(Length::Fill),
                text(now.format("%d\n%m").to_string())
                    .height(Length::Fill)
                    .align_y(Vertical::Bottom),
            ]
            .width(Length::Fill)
            .align_x(Horizontal::Center),
        )
        .padding(padding::vertical(8))
        .style(|theme: &Theme| Style {
            background: Some(Background::Color(theme.palette().background)),
            border: Border {
                color: theme.extended_palette().background.strong.color,
                width: 1.0,
                radius: Radius::new(12),
            },
            ..Default::default()
        })
        .into()
    }

    pub fn surface_id(&self) -> Id {
        self.surface_id
    }

    pub fn destroy(self) -> Task<Message> {
        layer_surface::destroy_layer_surface(self.surface_id)
    }
}
