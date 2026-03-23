use std::mem;

use hyprland::{
    data::{Clients, FullscreenMode},
    shared::HyprData,
};
use iced::Rectangle;

#[derive(Clone, Debug)]
pub struct Workspace {
    pub fullscreen: bool,
    pub group: Option<WindowGroup>,
}

#[derive(Clone, Debug, Default)]
pub enum WindowGroup {
    #[default]
    Single,
    Horizontal(Vec<WindowGroup>),
    Vertical(Vec<WindowGroup>),
}

impl Workspace {
    pub fn fetch() -> [Workspace; 9] {
        Self::construct_workspaces(Clients::get().unwrap())
    }

    pub async fn fetch_async() -> [Workspace; 9] {
        Self::construct_workspaces(Clients::get_async().await.unwrap())
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
}
