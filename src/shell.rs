use std::{
    io::ErrorKind,
    path::{Path, PathBuf},
};

use chrono::{DateTime, Local};
use hyprland::{
    data::{Monitor, Monitors},
    event_listener::{self, EventStream},
    shared::{HyprData, HyprDataActive},
};
use iced::{
    Element, Point, Rectangle, Size, Subscription, Task,
    event::{
        self, PlatformSpecific,
        wayland::{self, OutputEvent},
    },
    mouse, stream,
    time::{self, seconds},
    window::Id,
};
use smithay_client_toolkit::{
    output::OutputInfo,
    reexports::client::{Proxy, backend::ObjectId, protocol::wl_output::WlOutput},
};

use crate::{background::Background, bar::Bar, wifi, workspace::Workspace};

#[derive(Debug)]
pub struct Shell {
    outputs: Vec<Output>,
    workspaces: [Workspace; 9],
    background_bounds: Rectangle,
    wallpaper_background: PathBuf,
    wallpaper_middle_ground: PathBuf,
    wallpaper_foreground: PathBuf,
    active_monitor: String,
    wifi_strength: Option<u8>,
    battery: Option<(BatteryStatus, u8)>,
    cursor_position: Point,
    now: DateTime<Local>,
}

#[derive(Debug)]
struct Output {
    id: ObjectId,
    monitor: String,
    workspace: usize,
    bounds: Rectangle,
    background: Background,
    bar: Bar,
}

#[derive(Clone, Debug)]
pub enum Message {
    OutputCreated {
        output: WlOutput,
        monitor: String,
        workspace: Option<usize>,
        bounds: Rectangle,
    },
    OutputChanged {
        output: WlOutput,
        monitor: String,
        bounds: Rectangle,
    },
    OutputRemoved(WlOutput),
    ActiveMonitorChanged(String),
    ActiveWorkspaceChanged(usize),
    WorkspaceMoved {
        monitor: String,
        workspace: usize,
    },
    WorkspaceChanged(Option<Box<[Workspace; 9]>>),
    WifiStrengthChanged(Option<u8>),
    BatteryTick(BatteryStatus, u8),
    CursorMoved {
        surface_id: Id,
        position: Point,
    },
    TimeTick(DateTime<Local>),
}

#[derive(Debug, Clone, Copy)]
pub enum BatteryStatus {
    Charging,
    Discharging,
}

impl Shell {
    pub fn new(
        wallpaper_background: &Path,
        wallpaper_middle_ground: &Path,
        wallpaper_foreground: &Path,
    ) -> Self {
        Self {
            outputs: Vec::new(),
            workspaces: Workspace::fetch(),
            background_bounds: Rectangle::new(Point::ORIGIN, Size::ZERO),
            wallpaper_background: PathBuf::from(wallpaper_background),
            wallpaper_middle_ground: PathBuf::from(wallpaper_middle_ground),
            wallpaper_foreground: PathBuf::from(wallpaper_foreground),
            active_monitor: Self::fetch_active_monitor(),
            wifi_strength: None,
            battery: Self::fetch_battery(),
            cursor_position: Point::ORIGIN,
            now: Local::now(),
        }
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::OutputCreated {
                output,
                monitor,
                workspace: None,
                bounds,
            } => Task::future(async move {
                Message::OutputCreated {
                    output,
                    workspace: Some(Self::fetch_monitor_workspace(&monitor).await),
                    monitor,
                    bounds,
                }
            }),
            Message::OutputCreated {
                output,
                monitor,
                workspace: Some(workspace),
                bounds,
            } => {
                let (background, background_task) = Background::new(output.clone());
                let (bar, bar_task) = Bar::new(output.clone());

                self.outputs.push(Output {
                    id: output.id(),
                    monitor,
                    workspace,
                    bounds,
                    background,
                    bar,
                });
                self.update_background_bounds();

                Task::batch([background_task, bar_task])
            }
            Message::OutputChanged {
                output,
                monitor,
                bounds,
            } => {
                let output = self
                    .outputs
                    .iter_mut()
                    .find(|x| x.id == output.id())
                    .unwrap();
                output.monitor = monitor;
                output.bounds = bounds;

                self.update_background_bounds();

                Task::none()
            }
            Message::OutputRemoved(output) => {
                let output_index = self
                    .outputs
                    .iter()
                    .position(|x| x.id == output.id())
                    .unwrap();
                let output = self.outputs.swap_remove(output_index);

                self.update_background_bounds();

                Task::batch([output.background.destroy(), output.bar.destroy()])
            }
            Message::ActiveMonitorChanged(x) => {
                self.active_monitor = x;
                Task::none()
            }
            Message::ActiveWorkspaceChanged(x) => {
                self.outputs
                    .iter_mut()
                    .find(|x| x.monitor == self.active_monitor)
                    .unwrap()
                    .workspace = x;
                Task::none()
            }
            Message::WorkspaceMoved { monitor, workspace } => {
                self.outputs
                    .iter_mut()
                    .find(|x| x.monitor == monitor)
                    .unwrap()
                    .workspace = workspace;
                Task::none()
            }
            Message::WorkspaceChanged(None) => Task::future(async {
                Message::WorkspaceChanged(Some(Box::new(Workspace::fetch_async().await)))
            }),
            Message::WorkspaceChanged(Some(workspaces)) => {
                self.workspaces = *workspaces;
                Task::none()
            }
            Message::WifiStrengthChanged(x) => {
                self.wifi_strength = x;
                Task::none()
            }
            Message::BatteryTick(status, capacity) => {
                self.battery = Some((status, capacity));
                Task::none()
            }
            Message::CursorMoved {
                surface_id,
                position,
            } => {
                let output_position = self
                    .outputs
                    .iter()
                    .find(|x| x.background.surface_id() == surface_id)
                    .unwrap()
                    .bounds
                    .position();
                self.cursor_position = Point {
                    x: output_position.x + position.x,
                    y: output_position.y + position.y,
                };
                Task::none()
            }
            Message::TimeTick(now) => {
                self.now = now;
                Task::none()
            }
        }
    }

    fn update_background_bounds(&mut self) {
        self.background_bounds = self
            .outputs
            .iter()
            .map(|x| x.bounds)
            .reduce(|acc, x| acc.union(&x))
            .unwrap_or_else(|| Rectangle::new(Point::ORIGIN, Size::ZERO))
    }

    pub fn view(&self, surface_id: Id) -> Element<'_, Message> {
        for x in &self.outputs {
            if x.background.surface_id() == surface_id {
                return x.background.view(
                    self.background_bounds,
                    &self.wallpaper_background,
                    &self.wallpaper_middle_ground,
                    &self.wallpaper_foreground,
                    self.cursor_position,
                    self.now,
                );
            } else if x.bar.surface_id() == surface_id {
                return x.bar.view(
                    x.workspace,
                    &self.workspaces,
                    self.wifi_strength,
                    self.battery,
                    self.now,
                );
            }
        }
        unreachable!();
    }

    pub fn subscription(&self) -> Subscription<Message> {
        Subscription::batch([
            self.output_subscription(),
            self.hyprland_subscription(),
            self.wifi_subscription(),
            self.battery_subscription(),
            self.mouse_subscription(),
            self.time_subscription(),
        ])
    }

    fn output_subscription(&self) -> Subscription<Message> {
        fn get_output_bounds(info: &OutputInfo) -> Rectangle {
            Rectangle {
                x: info.logical_position.unwrap().0 as f32,
                y: info.logical_position.unwrap().1 as f32,
                width: info.logical_size.unwrap().0 as f32,
                height: info.logical_size.unwrap().1 as f32,
            }
        }

        event::listen_with(|event, _, _| match event {
            iced::Event::PlatformSpecific(PlatformSpecific::Wayland(wayland::Event::Output(
                event,
                output,
            ))) => match event {
                OutputEvent::Created(x) => Some(Message::OutputCreated {
                    output,
                    bounds: get_output_bounds(x.as_ref().unwrap()),
                    monitor: x.unwrap().name.unwrap(),
                    workspace: None,
                }),
                OutputEvent::InfoUpdate(x) => Some(Message::OutputChanged {
                    output,
                    bounds: get_output_bounds(&x),
                    monitor: x.name.unwrap(),
                }),
                OutputEvent::Removed => Some(Message::OutputRemoved(output)),
            },
            _ => None,
        })
    }

    fn hyprland_subscription(&self) -> Subscription<Message> {
        Subscription::run(EventStream::new).filter_map(|event| match event.unwrap() {
            event_listener::Event::ActiveMonitorChanged(data) => {
                Some(Message::ActiveMonitorChanged(data.monitor_name))
            }
            event_listener::Event::WorkspaceChanged(data) => {
                Some(Message::ActiveWorkspaceChanged(data.id as usize - 1))
            }
            event_listener::Event::WorkspaceMoved(data) => Some(Message::WorkspaceMoved {
                monitor: data.monitor,
                workspace: data.id as usize - 1,
            }),
            event_listener::Event::ActiveWindowChanged(_)
            | event_listener::Event::WindowMoved(_)
            | event_listener::Event::FullscreenStateChanged(_)
            | event_listener::Event::FloatStateChanged(_) => Some(Message::WorkspaceChanged(None)),
            _ => None,
        })
    }

    fn wifi_subscription(&self) -> Subscription<Message> {
        Subscription::run(|| stream::channel(64, wifi::wifi))
    }

    fn battery_subscription(&self) -> Subscription<Message> {
        if self.battery.is_none() {
            return Subscription::none();
        }

        async fn tick() -> Message {
            let status = match tokio::fs::read_to_string("/sys/class/power_supply/BAT0/status")
                .await
                .unwrap()
                .trim_end()
            {
                "Charging" | "Full" => BatteryStatus::Charging,
                _ => BatteryStatus::Discharging,
            };
            let capacity = tokio::fs::read_to_string("/sys/class/power_supply/BAT0/capacity")
                .await
                .unwrap()
                .trim_end()
                .parse()
                .unwrap();
            Message::BatteryTick(status, capacity)
        }
        time::repeat(tick, seconds(1))
    }

    fn mouse_subscription(&self) -> Subscription<Message> {
        event::listen_with(|event, _, surface_id| match event {
            iced::Event::Mouse(mouse::Event::CursorMoved { position }) => {
                Some(Message::CursorMoved {
                    surface_id,
                    position,
                })
            }
            _ => None,
        })
    }

    fn time_subscription(&self) -> Subscription<Message> {
        time::every(seconds(10)).map(|_| Message::TimeTick(Local::now()))
    }

    fn fetch_active_monitor() -> String {
        Monitor::get_active().unwrap().name
    }

    async fn fetch_monitor_workspace(monitor: &str) -> usize {
        Monitors::get_async()
            .await
            .unwrap()
            .into_iter()
            .find_map(|x| (x.name == monitor).then_some(x.active_workspace.id as usize - 1))
            .unwrap()
    }

    fn fetch_battery() -> Option<(BatteryStatus, u8)> {
        let status = match std::fs::read_to_string("/sys/class/power_supply/BAT0/status") {
            Ok(x) => match x.trim_end() {
                "Charging" | "Full" => BatteryStatus::Charging,
                _ => BatteryStatus::Discharging,
            },
            Err(x) if x.kind() == ErrorKind::NotFound => return None,
            Err(x) => panic!("{x}"),
        };
        let capacity = std::fs::read_to_string("/sys/class/power_supply/BAT0/capacity")
            .unwrap()
            .trim_end()
            .parse()
            .unwrap();
        Some((status, capacity))
    }
}
