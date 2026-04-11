use chrono::{DateTime, Local};
use iced::{
    Background, Border, Element, Length, Radius, Size, Subscription, Task, Theme,
    alignment::Horizontal,
    event::{
        self, PlatformSpecific,
        wayland::{self, OutputEvent},
    },
    padding,
    platform_specific::shell::commands::layer_surface,
    runtime::platform_specific::wayland::layer_surface::{
        IcedMargin, IcedOutput, SctkLayerSurfaceSettings,
    },
    widget::{Container, column, container::Style, responsive, space, text},
    window::Id,
};
use smithay_client_toolkit::{
    reexports::client::{Proxy, protocol::wl_output::WlOutput},
    shell::wlr_layer::Anchor,
};

use crate::bar::{
    audio::{Audio, AudioMessage},
    battery::{Battery, BatteryMessage},
    wifi::{WiFi, WiFiMessage},
    workspace_overview::{WorkspaceOverview, WorkspaceOverviewMessage},
};

mod audio;
mod battery;
mod wifi;
mod workspace_overview;

const BAR_WIDTH: f32 = 32.0;

#[derive(Debug)]
pub struct Bar {
    outputs: Vec<Output>,
    now: DateTime<Local>,
    workspace_overview: WorkspaceOverview,
    wifi: WiFi,
    audio: Audio,
    battery: Battery,
}

#[derive(Debug)]
struct Output {
    id: u32,
    surface_id: Id,
}

#[derive(Clone, Debug)]
pub enum BarMessage {
    OutputCreated(WlOutput),
    OutputRemoved(u32),
    TimeTick(DateTime<Local>),
    WorkspaceOverview(WorkspaceOverviewMessage),
    WiFi(WiFiMessage),
    Audio(AudioMessage),
    Battery(BatteryMessage),
}

impl Bar {
    pub fn new(now: DateTime<Local>) -> Self {
        Self {
            outputs: Vec::new(),
            now,
            workspace_overview: WorkspaceOverview::new(),
            wifi: WiFi::new(),
            audio: Audio::new(),
            battery: Battery::new(),
        }
    }

    pub fn update(&mut self, message: BarMessage) -> Task<BarMessage> {
        match message {
            BarMessage::OutputCreated(x) => return self.create_output(x),
            BarMessage::OutputRemoved(x) => return self.remove_output(x),
            BarMessage::TimeTick(x) => self.now = x,
            BarMessage::WorkspaceOverview(x) => {
                return self
                    .workspace_overview
                    .update(x)
                    .map(BarMessage::WorkspaceOverview);
            }
            BarMessage::WiFi(x) => self.wifi.update(x),
            BarMessage::Audio(x) => self.audio.update(x),
            BarMessage::Battery(x) => self.battery.update(x),
        }
        Task::none()
    }

    fn create_output(&mut self, output: WlOutput) -> Task<BarMessage> {
        let surface_id = Id::unique();

        self.outputs.push(Output {
            id: output.id().protocol_id(),
            surface_id,
        });

        layer_surface::get_layer_surface(SctkLayerSurfaceSettings {
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
            size: Some((Some(BAR_WIDTH as u32), None)),
            exclusive_zone: BAR_WIDTH as i32,
            ..Default::default()
        })
    }

    fn remove_output(&mut self, id: u32) -> Task<BarMessage> {
        let i = self.outputs.iter().position(|x| x.id == id).unwrap();
        let output = self.outputs.swap_remove(i);

        layer_surface::destroy_layer_surface(output.surface_id)
    }

    pub fn view(&self, surface_id: Id) -> Option<Element<'_, BarMessage>> {
        let output_id = self.outputs.iter().find(|x| x.surface_id == surface_id)?.id;

        let view = responsive(move |Size { width, .. }| {
            Container::new(
                column![
                    text(self.now.format("%H\n%M").to_string())
                        .size(14.0 / BAR_WIDTH * width)
                        .height(Length::Fill),
                    self.workspace_overview
                        .view(output_id, width)
                        .map(BarMessage::WorkspaceOverview),
                    column![
                        space().height(Length::Fill),
                        self.wifi.view(width).map(BarMessage::WiFi),
                        self.audio.view(width).map(BarMessage::Audio),
                        self.battery.view(width).map(BarMessage::Battery),
                        text(self.now.format("%d\n%m").to_string()).size(14.0 / BAR_WIDTH * width)
                    ]
                    .spacing(8.0 / BAR_WIDTH * width)
                    .height(Length::Fill)
                    .align_x(Horizontal::Center)
                ]
                .width(Length::Fill)
                .align_x(Horizontal::Center),
            )
            .padding(padding::vertical(8.0 / BAR_WIDTH * width))
            .style(move |theme: &Theme| Style {
                background: Some(Background::Color(theme.palette().background)),
                border: Border {
                    color: theme.extended_palette().background.strong.color,
                    width: 1.0,
                    radius: Radius::new(12.0 / BAR_WIDTH * width),
                },
                ..Default::default()
            })
            .into()
        });
        Some(view.into())
    }

    pub fn subscription(&self) -> Subscription<BarMessage> {
        Subscription::batch([
            self.output_subscription(),
            self.workspace_overview
                .subscription()
                .map(BarMessage::WorkspaceOverview),
            self.wifi.subscription().map(BarMessage::WiFi),
            self.audio.subscription().map(BarMessage::Audio),
            self.battery.subscription().map(BarMessage::Battery),
        ])
    }

    fn output_subscription(&self) -> Subscription<BarMessage> {
        event::listen_with(|event, _, _| match event {
            iced::Event::PlatformSpecific(PlatformSpecific::Wayland(wayland::Event::Output(
                event,
                output,
            ))) => match event {
                OutputEvent::Created(_) => Some(BarMessage::OutputCreated(output)),
                OutputEvent::InfoUpdate(_) => None,
                OutputEvent::Removed => Some(BarMessage::OutputRemoved(output.id().protocol_id())),
            },
            _ => None,
        })
    }
}
