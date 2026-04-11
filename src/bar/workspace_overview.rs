use std::mem;

use hyprland::{
    data::{Clients, FullscreenMode, Monitor, Monitors},
    event_listener::{self, EventStream},
    shared::{HyprData, HyprDataActive},
};
use iced::{
    Background, Border, Color, Element, Length, Radius, Rectangle, Subscription, Task, Theme,
    border,
    event::{
        self, PlatformSpecific,
        wayland::{self, OutputEvent},
    },
    widget::{Container, column, container::Style, row, space},
};
use smithay_client_toolkit::reexports::client::Proxy;

use crate::bar::BAR_WIDTH;

#[derive(Debug)]
pub struct WorkspaceOverview {
    outputs: Vec<Output>,
    workspaces: [Workspace; 9],
    active_monitor: String,
}

#[derive(Debug)]
struct Output {
    id: u32,
    monitor: String,
    workspace: usize,
}

#[derive(Clone, Debug)]
pub enum WorkspaceOverviewMessage {
    OutputCreated {
        id: u32,
        monitor: String,
        workspace: Option<usize>,
    },
    OutputChanged {
        id: u32,
        monitor: String,
    },
    OutputRemoved(u32),
    ActiveMonitorChanged(String),
    ActiveWorkspaceChanged(usize),
    WorkspaceMoved {
        monitor: String,
        workspace: usize,
    },
    WorkspaceChanged(Option<Box<[Workspace; 9]>>),
}

#[derive(Clone, Debug)]
pub struct Workspace {
    fullscreen: bool,
    group: Option<WindowGroup>,
}

#[derive(Clone, Debug, Default)]
enum WindowGroup {
    #[default]
    Single,
    Horizontal(Vec<WindowGroup>),
    Vertical(Vec<WindowGroup>),
}

impl WorkspaceOverview {
    pub fn new() -> Self {
        Self {
            outputs: Vec::new(),
            workspaces: Self::construct_workspaces(Clients::get().unwrap()),
            active_monitor: Monitor::get_active().unwrap().name,
        }
    }

    pub fn update(&mut self, message: WorkspaceOverviewMessage) -> Task<WorkspaceOverviewMessage> {
        match message {
            WorkspaceOverviewMessage::OutputCreated {
                id,
                monitor,
                workspace,
            } => return self.create_output(id, monitor, workspace),
            WorkspaceOverviewMessage::OutputChanged { id, monitor } => {
                self.update_output(id, monitor)
            }
            WorkspaceOverviewMessage::OutputRemoved(x) => self.remove_output(x),
            WorkspaceOverviewMessage::ActiveMonitorChanged(x) => self.active_monitor = x,
            WorkspaceOverviewMessage::ActiveWorkspaceChanged(x) => self.change_active_workspace(x),
            WorkspaceOverviewMessage::WorkspaceMoved { monitor, workspace } => {
                self.move_workspace(monitor, workspace)
            }
            WorkspaceOverviewMessage::WorkspaceChanged(x) => return self.update_workspaces(x),
        }
        Task::none()
    }

    fn create_output(
        &mut self,
        id: u32,
        monitor: String,
        workspace: Option<usize>,
    ) -> Task<WorkspaceOverviewMessage> {
        match workspace {
            None => Task::future(async move {
                WorkspaceOverviewMessage::OutputCreated {
                    id,
                    workspace: Some(Self::fetch_monitor_workspace(&monitor).await),
                    monitor,
                }
            }),
            Some(x) => {
                self.outputs.push(Output {
                    id,
                    monitor,
                    workspace: x,
                });
                Task::none()
            }
        }
    }

    async fn fetch_monitor_workspace(monitor: &str) -> usize {
        Monitors::get_async()
            .await
            .unwrap()
            .into_iter()
            .find_map(|x| (x.name == monitor).then_some(x.active_workspace.id as usize - 1))
            .unwrap()
    }

    fn update_output(&mut self, id: u32, monitor: String) {
        self.outputs
            .iter_mut()
            .find(|x| x.id == id)
            .unwrap()
            .monitor = monitor;
    }

    fn remove_output(&mut self, id: u32) {
        let i = self.outputs.iter().position(|x| x.id == id).unwrap();
        self.outputs.swap_remove(i);
    }

    fn change_active_workspace(&mut self, workspace: usize) {
        self.outputs
            .iter_mut()
            .find(|x| x.monitor == self.active_monitor)
            .unwrap()
            .workspace = workspace;
    }

    fn move_workspace(&mut self, monitor: String, workspace: usize) {
        self.outputs
            .iter_mut()
            .find(|x| x.monitor == monitor)
            .unwrap()
            .workspace = workspace;
    }

    fn update_workspaces(
        &mut self,
        workspaces: Option<Box<[Workspace; 9]>>,
    ) -> Task<WorkspaceOverviewMessage> {
        match workspaces {
            None => Task::future(async {
                let workspaces = Self::construct_workspaces(Clients::get_async().await.unwrap());
                WorkspaceOverviewMessage::WorkspaceChanged(Some(Box::new(workspaces)))
            }),
            Some(x) => {
                self.workspaces = *x;
                Task::none()
            }
        }
    }

    fn construct_workspaces(windows: Clients) -> [Workspace; 9] {
        let mut workspaces = [const { (false, Vec::new()) }; 9];
        for window in windows {
            let workspace = window.workspace.id as usize - 1;

            if window.fullscreen != FullscreenMode::None {
                workspaces[workspace].0 = true;
            }

            if window.floating {
                continue;
            }

            let bounds = Rectangle {
                x: window.at.0,
                y: window.at.1,
                width: window.size.0,
                height: window.size.1,
            };
            workspaces[workspace].1.push((bounds, WindowGroup::Single));
        }

        const EMPTY_WORKSPACE: Workspace = Workspace {
            fullscreen: false,
            group: None,
        };
        let mut result = [EMPTY_WORKSPACE; 9];
        for (i, mut workspace) in workspaces.into_iter().enumerate() {
            loop {
                let rows_changed = Self::merge_workspace_rows(&mut workspace.1);
                let columns_changed = Self::merge_workspace_columns(&mut workspace.1);
                if !rows_changed && !columns_changed {
                    break;
                }
            }
            result[i].fullscreen = workspace.0;
            result[i].group = workspace.1.into_iter().next().map(|x| x.1);
        }

        result
    }

    fn merge_workspace_rows(workspace: &mut Vec<(Rectangle<i16>, WindowGroup)>) -> bool {
        if workspace.is_empty() {
            return false;
        }

        let mut changed = false;
        workspace.sort_unstable_by_key(|x| (x.0.y, x.0.x));

        let mut i = 0;
        while i < workspace.len() - 1 {
            if workspace[i].0.y != workspace[i + 1].0.y
                || workspace[i].0.height != workspace[i + 1].0.height
                || workspace.iter().any(|x| {
                    let x_between = x.0.x > workspace[i].0.x && x.0.x < workspace[i + 1].0.x;
                    let y_between = x.0.y + x.0.height >= workspace[i].0.y
                        && x.0.y <= workspace[i].0.y + workspace[i].0.height;
                    x_between && y_between
                })
            {
                i += 1;
                continue;
            }

            let (bounds, group) = workspace.remove(i + 1);
            changed = true;

            workspace[i].0.width = (bounds.x + bounds.width) - workspace[i].0.x;

            match (&mut workspace[i].1, group) {
                (WindowGroup::Horizontal(a), WindowGroup::Horizontal(b)) => a.extend(b),
                (WindowGroup::Horizontal(a), b) => a.push(b),
                (a, WindowGroup::Horizontal(mut b)) => {
                    b.insert(0, mem::take(a));
                    *a = WindowGroup::Horizontal(b);
                }
                (a, b) => *a = WindowGroup::Horizontal(vec![mem::take(a), b]),
            }
        }

        changed
    }

    fn merge_workspace_columns(workspace: &mut Vec<(Rectangle<i16>, WindowGroup)>) -> bool {
        if workspace.is_empty() {
            return false;
        }

        let mut changed = false;
        workspace.sort_unstable_by_key(|x| (x.0.x, x.0.y));

        let mut i = 0;
        while i < workspace.len() - 1 {
            if workspace[i].0.x != workspace[i + 1].0.x
                || workspace[i].0.width != workspace[i + 1].0.width
                || workspace.iter().any(|x| {
                    let x_between = x.0.x + x.0.width >= workspace[i].0.x
                        && x.0.x <= workspace[i].0.x + workspace[i].0.width;
                    let y_between = x.0.y > workspace[i].0.y && x.0.y < workspace[i + 1].0.y;
                    x_between && y_between
                })
            {
                i += 1;
                continue;
            }

            let (bounds, group) = workspace.remove(i + 1);
            changed = true;

            workspace[i].0.height = (bounds.y + bounds.height) - workspace[i].0.y;

            match (&mut workspace[i].1, group) {
                (WindowGroup::Vertical(a), WindowGroup::Vertical(b)) => a.extend(b),
                (WindowGroup::Vertical(a), b) => a.push(b),
                (a, WindowGroup::Vertical(mut b)) => {
                    b.insert(0, mem::take(a));
                    *a = WindowGroup::Vertical(b);
                }
                (a, b) => *a = WindowGroup::Vertical(vec![mem::take(a), b]),
            }
        }

        changed
    }

    pub fn view(&self, output_id: u32, width: f32) -> Element<'_, WorkspaceOverviewMessage> {
        let active_workspace = self
            .outputs
            .iter()
            .find(|x| x.id == output_id)
            .unwrap()
            .workspace;

        column(self.workspaces.iter().enumerate().map(|(i, x)| {
            let active = i == active_workspace;
            let content = match x {
                Workspace {
                    fullscreen: true, ..
                } => Container::new(space())
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .style(move |theme: &Theme| Style {
                        background: Some(Background::Color(self.workspace_color(theme, active))),
                        border: border::rounded(4.0 / BAR_WIDTH * width),
                        ..Style::default()
                    })
                    .into(),
                Workspace { group: None, .. } => Container::new(space())
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .style(move |theme: &Theme| Style {
                        border: Border {
                            color: self.workspace_color(theme, active),
                            width: 1.5 / BAR_WIDTH * width,
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
                .width(16.0 / BAR_WIDTH * width)
                .height(16.0 / BAR_WIDTH * width)
                .into()
        }))
        .spacing(4.0 / BAR_WIDTH * width)
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
    ) -> Element<'_, WorkspaceOverviewMessage> {
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
                    WindowGroup::Horizontal(_) => {
                        row(children).spacing(2.0 / BAR_WIDTH * width).into()
                    }
                    _ => column(children).spacing(2.0 / BAR_WIDTH * width).into(),
                }
            }
        }
    }

    pub fn subscription(&self) -> Subscription<WorkspaceOverviewMessage> {
        Subscription::batch([self.output_subscription(), self.workspace_subscription()])
    }

    fn output_subscription(&self) -> Subscription<WorkspaceOverviewMessage> {
        event::listen_with(|event, _, _| match event {
            iced::Event::PlatformSpecific(PlatformSpecific::Wayland(wayland::Event::Output(
                event,
                output,
            ))) => match event {
                OutputEvent::Created(x) => Some(WorkspaceOverviewMessage::OutputCreated {
                    id: output.id().protocol_id(),
                    monitor: x.unwrap().name.unwrap(),
                    workspace: None,
                }),
                OutputEvent::InfoUpdate(x) => Some(WorkspaceOverviewMessage::OutputChanged {
                    id: output.id().protocol_id(),
                    monitor: x.name.unwrap(),
                }),
                OutputEvent::Removed => Some(WorkspaceOverviewMessage::OutputRemoved(
                    output.id().protocol_id(),
                )),
            },
            _ => None,
        })
    }

    fn workspace_subscription(&self) -> Subscription<WorkspaceOverviewMessage> {
        Subscription::run(EventStream::new).filter_map(|event| match event.unwrap() {
            event_listener::Event::ActiveMonitorChanged(data) => Some(
                WorkspaceOverviewMessage::ActiveMonitorChanged(data.monitor_name),
            ),
            event_listener::Event::WorkspaceChanged(data) => Some(
                WorkspaceOverviewMessage::ActiveWorkspaceChanged(data.id as usize - 1),
            ),
            event_listener::Event::WorkspaceMoved(data) => {
                Some(WorkspaceOverviewMessage::WorkspaceMoved {
                    monitor: data.monitor,
                    workspace: data.id as usize - 1,
                })
            }
            event_listener::Event::ActiveWindowChanged(_)
            | event_listener::Event::WindowMoved(_)
            | event_listener::Event::FullscreenStateChanged(_)
            | event_listener::Event::FloatStateChanged(_) => {
                Some(WorkspaceOverviewMessage::WorkspaceChanged(None))
            }
            _ => None,
        })
    }
}
