use std::path::{Path, PathBuf};

use chrono::{DateTime, Local};
use iced::{
    ContentFit, Element, Font, Size, Subscription, Task,
    core::image::Handle,
    event::{
        self, PlatformSpecific,
        wayland::{self, OutputEvent},
    },
    font::{Family, Weight},
    platform_specific::shell::commands::layer_surface,
    runtime::platform_specific::wayland::layer_surface::{IcedOutput, SctkLayerSurfaceSettings},
    widget::{center, image, responsive, space, stack, text},
    window::Id,
};
use rand::RngExt;
use smithay_client_toolkit::{
    reexports::client::{Proxy, protocol::wl_output::WlOutput},
    shell::wlr_layer::{Anchor, Layer},
};

#[derive(Debug)]
pub struct Background {
    wallpaper: PathBuf,
    outputs: Vec<Output>,
    now: DateTime<Local>,
}

#[derive(Debug)]
struct Output {
    id: u32,
    surface_id: Id,
}

#[derive(Clone, Debug)]
pub enum BackgroundMessage {
    OutputCreated(WlOutput),
    OutputRemoved(u32),
    TimeTick(DateTime<Local>),
}

impl Background {
    pub fn new(wallpapers: &Path, now: DateTime<Local>) -> Self {
        Self {
            wallpaper: Self::choose_wallpaper(wallpapers),
            outputs: Vec::new(),
            now,
        }
    }

    fn choose_wallpaper(wallpapers: &Path) -> PathBuf {
        let metadata = wallpapers
            .metadata()
            .expect("wallpapers should be a valid path");
        if metadata.is_file() {
            return PathBuf::from(wallpapers);
        } else if !metadata.is_dir() {
            panic!("wallpapers should either be a file or a directory");
        }

        let wallpapers: Vec<_> = wallpapers.read_dir().unwrap().map(|x| x.unwrap()).collect();
        wallpapers[rand::rng().random_range(0..wallpapers.len())].path()
    }

    pub fn update(&mut self, message: BackgroundMessage) -> Task<BackgroundMessage> {
        match message {
            BackgroundMessage::OutputCreated(x) => self.create_output(x),
            BackgroundMessage::OutputRemoved(x) => self.remove_output(x),
            BackgroundMessage::TimeTick(x) => {
                self.now = x;
                Task::none()
            }
        }
    }

    fn create_output(&mut self, output: WlOutput) -> Task<BackgroundMessage> {
        let surface_id = Id::unique();

        self.outputs.push(Output {
            id: output.id().protocol_id(),
            surface_id,
        });

        layer_surface::get_layer_surface(SctkLayerSurfaceSettings {
            id: surface_id,
            layer: Layer::Background,
            anchor: Anchor::all(),
            output: IcedOutput::Output(output),
            namespace: String::from("shell-background"),
            exclusive_zone: -1,
            ..Default::default()
        })
    }

    fn remove_output(&mut self, id: u32) -> Task<BackgroundMessage> {
        let i = self.outputs.iter().position(|x| x.id == id).unwrap();
        let output = self.outputs.swap_remove(i);

        layer_surface::destroy_layer_surface(output.surface_id)
    }

    pub fn view(&self, surface_id: Id) -> Option<Element<'_, BackgroundMessage>> {
        if self.outputs.iter().all(|x| x.surface_id != surface_id) {
            return None;
        }

        let view = stack![
            image(Handle::from_path(&self.wallpaper)).content_fit(ContentFit::Cover),
            self.view_clock(self.now),
        ];
        Some(view.into())
    }

    fn view_clock(&self, now: DateTime<Local>) -> Element<'_, BackgroundMessage> {
        responsive(move |Size { width, .. }| {
            if width == 0.0 {
                return space().into();
            }

            center(
                text(now.format("%R").to_string())
                    .size(width / 15.0)
                    .font(Font {
                        family: Family::Monospace,
                        weight: Weight::Bold,
                        ..Default::default()
                    }),
            )
            .into()
        })
        .into()
    }

    pub fn subscription(&self) -> Subscription<BackgroundMessage> {
        event::listen_with(|event, _, _| match event {
            iced::Event::PlatformSpecific(PlatformSpecific::Wayland(wayland::Event::Output(
                event,
                output,
            ))) => match event {
                OutputEvent::Created(_) => Some(BackgroundMessage::OutputCreated(output)),
                OutputEvent::InfoUpdate(_) => None,
                OutputEvent::Removed => {
                    Some(BackgroundMessage::OutputRemoved(output.id().protocol_id()))
                }
            },
            _ => None,
        })
    }
}
