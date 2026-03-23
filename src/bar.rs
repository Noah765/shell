use chrono::{DateTime, Local};
use iced::{
    Background, Border, Color, Element, Length, Radius, Task, Theme,
    alignment::{Horizontal, Vertical},
    border, padding,
    platform_specific::shell::commands::layer_surface,
    runtime::platform_specific::wayland::layer_surface::{
        IcedMargin, IcedOutput, SctkLayerSurfaceSettings,
    },
    widget::{Container, column, container::Style, row, space, text},
    window::Id,
};
use smithay_client_toolkit::{
    reexports::client::protocol::wl_output::WlOutput, shell::wlr_layer::Anchor,
};

use crate::{
    shell::Message,
    workspace::{WindowGroup, Workspace},
};

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

    pub fn view(
        &self,
        workspace: usize,
        workspaces: &[Workspace; 9],
        now: DateTime<Local>,
    ) -> Element<'_, Message> {
        Container::new(
            column![
                text(now.format("%H\n%M").to_string()).height(Length::Fill),
                self.view_workspaces(workspaces, workspace),
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

    fn view_workspaces(
        &self,
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
                        border: border::rounded(4),
                        ..Style::default()
                    })
                    .into(),
                Workspace { group: None, .. } => Container::new(space())
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .style(move |theme: &Theme| Style {
                        border: Border {
                            color: self.workspace_color(theme, active),
                            width: 1.5,
                            radius: Radius::new(8),
                        },
                        ..Style::default()
                    })
                    .into(),
                Workspace { group: Some(x), .. } => self.view_workspace_window_group(x, active),
            };
            Container::new(content).width(16).height(16).into()
        }))
        .spacing(4)
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
        group: &WindowGroup,
        active: bool,
    ) -> Element<'_, Message> {
        match group {
            WindowGroup::Single => Container::new(space())
                .width(Length::Fill)
                .height(Length::Fill)
                .style(move |theme: &Theme| Style {
                    background: Some(Background::Color(self.workspace_color(theme, active))),
                    border: border::rounded(8),
                    ..Style::default()
                })
                .into(),
            WindowGroup::Horizontal(children) | WindowGroup::Vertical(children) => {
                let children = children
                    .iter()
                    .map(|x| self.view_workspace_window_group(x, active));
                match group {
                    WindowGroup::Horizontal(_) => row(children).spacing(2).into(),
                    _ => column(children).spacing(2).into(),
                }
            }
        }
    }

    pub fn surface_id(&self) -> Id {
        self.surface_id
    }

    pub fn destroy(self) -> Task<Message> {
        layer_surface::destroy_layer_surface(self.surface_id)
    }
}
