use chrono::{DateTime, Local};
use iced::{
    Background, Border, Color, Element, Length, Radius, Size, Task, Theme,
    alignment::Horizontal,
    border, padding,
    platform_specific::shell::commands::layer_surface,
    runtime::platform_specific::wayland::layer_surface::{
        IcedMargin, IcedOutput, SctkLayerSurfaceSettings,
    },
    widget::{Container, column, container::Style, responsive, row, space, text},
    window::Id,
};
use smithay_client_toolkit::{
    reexports::client::protocol::wl_output::WlOutput, shell::wlr_layer::Anchor,
};

use crate::{
    icon,
    shell::Message,
    workspace::{WindowGroup, Workspace},
};

const WIDTH: f32 = 32.0;

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
            size: Some((Some(WIDTH as u32), None)),
            exclusive_zone: WIDTH as i32,
            ..Default::default()
        });
        (Self { surface_id }, task)
    }

    pub fn view<'a>(
        &'a self,
        workspace: usize,
        workspaces: &'a [Workspace; 9],
        wifi_strength: Option<u8>,
        now: DateTime<Local>,
    ) -> Element<'a, Message> {
        responsive(move |Size { width, .. }| {
            Container::new(
                column![
                    text(now.format("%H\n%M").to_string())
                        .size(14.0 / WIDTH * width)
                        .height(Length::Fill),
                    self.view_workspaces(width, workspaces, workspace),
                    column![
                        space().height(Length::Fill),
                        self.view_wifi(width, wifi_strength),
                        text(now.format("%d\n%m").to_string()).size(14.0 / WIDTH * width)
                    ]
                    .spacing(4.0 / WIDTH * width)
                    .height(Length::Fill)
                    .align_x(Horizontal::Center)
                ]
                .width(Length::Fill)
                .align_x(Horizontal::Center),
            )
            .padding(padding::vertical(8.0 / WIDTH * width))
            .style(move |theme: &Theme| Style {
                background: Some(Background::Color(theme.palette().background)),
                border: Border {
                    color: theme.extended_palette().background.strong.color,
                    width: 1.0,
                    radius: Radius::new(12.0 / WIDTH * width),
                },
                ..Default::default()
            })
            .into()
        })
        .into()
    }

    fn view_workspaces(
        &self,
        width: f32,
        workspaces: &[Workspace; 9],
        workspace: usize,
    ) -> Element<'_, Message> {
        column(workspaces.iter().enumerate().map(|(i, x)| {
            let active = i == workspace;
            let content = match x {
                Workspace {
                    fullscreen: true, ..
                } => Container::new(space())
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .style(move |theme: &Theme| Style {
                        background: Some(Background::Color(self.workspace_color(theme, active))),
                        border: border::rounded(4.0 / WIDTH * width),
                        ..Style::default()
                    })
                    .into(),
                Workspace { group: None, .. } => Container::new(space())
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .style(move |theme: &Theme| Style {
                        border: Border {
                            color: self.workspace_color(theme, active),
                            width: 1.5 / WIDTH * width,
                            radius: Radius::new(width),
                        },
                        ..Style::default()
                    })
                    .into(),
                Workspace { group: Some(x), .. } => {
                    self.view_workspace_window_group(width, x, active)
                }
            };
            Container::new(content)
                .width(16.0 / WIDTH * width)
                .height(16.0 / WIDTH * width)
                .into()
        }))
        .spacing(4.0 / WIDTH * width)
        .into()
    }

    fn workspace_color(&self, theme: &Theme, active: bool) -> Color {
        if active {
            theme.palette().primary
        } else {
            theme.extended_palette().background.strong.color
        }
    }

    fn view_workspace_window_group(
        &self,
        width: f32,
        group: &WindowGroup,
        active: bool,
    ) -> Element<'_, Message> {
        match group {
            WindowGroup::Single => Container::new(space())
                .width(Length::Fill)
                .height(Length::Fill)
                .style(move |theme: &Theme| Style {
                    background: Some(Background::Color(self.workspace_color(theme, active))),
                    border: border::rounded(width),
                    ..Style::default()
                })
                .into(),
            WindowGroup::Horizontal(children) | WindowGroup::Vertical(children) => {
                let children = children
                    .iter()
                    .map(|x| self.view_workspace_window_group(width, x, active));
                match group {
                    WindowGroup::Horizontal(_) => row(children).spacing(2.0 / WIDTH * width).into(),
                    _ => column(children).spacing(2.0 / WIDTH * width).into(),
                }
            }
        }
    }

    fn view_wifi(&self, width: f32, strength: Option<u8>) -> Element<'_, Message> {
        let size = 16.0 / WIDTH * width;

        match strength {
            None => icon::wifi_off().size(size).into(),
            Some(0..25) => icon::wifi_1().size(size).into(),
            Some(25..50) => icon::wifi_2().size(size).into(),
            Some(50..75) => icon::wifi_3().size(size).into(),
            Some(75..) => icon::wifi_4().size(size).into(),
        }
    }

    pub fn surface_id(&self) -> Id {
        self.surface_id
    }

    pub fn destroy(self) -> Task<Message> {
        layer_surface::destroy_layer_surface(self.surface_id)
    }
}
