use std::path::{Path, PathBuf};

use chrono::{DateTime, Local};
use iced::{
    ContentFit, Element, Font, Point, Rectangle, Size, Subscription, Task, Vector,
    core::{image::Handle, mouse},
    event::{
        self, PlatformSpecific,
        wayland::{self, OutputEvent},
    },
    font::{Family, Weight},
    platform_specific::shell::commands::{
        layer_surface,
        subsurface::{Anchor, Layer},
    },
    runtime::platform_specific::wayland::layer_surface::{IcedOutput, SctkLayerSurfaceSettings},
    widget::{center, float, image, responsive, space, stack, text},
    window::Id,
};
use smithay_client_toolkit::{
    output::OutputInfo,
    reexports::client::{Proxy, protocol::wl_output::WlOutput},
};

#[derive(Debug)]
pub struct Background {
    outputs: Vec<Output>,
    global_bounds: Rectangle,
    wallpaper_background: PathBuf,
    wallpaper_middle_ground: PathBuf,
    wallpaper_foreground: PathBuf,
    cursor_position: Point,
    now: DateTime<Local>,
}

#[derive(Debug)]
struct Output {
    id: u32,
    bounds: Rectangle,
    surface_id: Id,
}

#[derive(Clone, Debug)]
pub enum BackgroundMessage {
    OutputCreated { output: WlOutput, bounds: Rectangle },
    OutputChanged { id: u32, bounds: Rectangle },
    OutputRemoved(u32),
    CursorMoved { surface_id: Id, position: Point },
    TimeTick(DateTime<Local>),
}

impl Background {
    pub fn new(
        wallpaper_background: &Path,
        wallpaper_middle_ground: &Path,
        wallpaper_foreground: &Path,
        now: DateTime<Local>,
    ) -> Self {
        Self {
            outputs: Vec::new(),
            global_bounds: Rectangle::new(Point::ORIGIN, Size::ZERO),
            wallpaper_background: PathBuf::from(wallpaper_background),
            wallpaper_middle_ground: PathBuf::from(wallpaper_middle_ground),
            wallpaper_foreground: PathBuf::from(wallpaper_foreground),
            cursor_position: Point::ORIGIN,
            now,
        }
    }

    pub fn update(&mut self, message: BackgroundMessage) -> Task<BackgroundMessage> {
        match message {
            BackgroundMessage::OutputCreated { output, bounds } => {
                self.create_output(output, bounds)
            }
            BackgroundMessage::OutputChanged { id, bounds } => self.update_output(id, bounds),
            BackgroundMessage::OutputRemoved(x) => self.remove_output(x),
            BackgroundMessage::CursorMoved {
                surface_id,
                position,
            } => self.move_cursor(surface_id, position),
            BackgroundMessage::TimeTick(x) => {
                self.now = x;
                Task::none()
            }
        }
    }

    fn create_output(&mut self, output: WlOutput, bounds: Rectangle) -> Task<BackgroundMessage> {
        let surface_id = Id::unique();

        self.outputs.push(Output {
            id: output.id().protocol_id(),
            bounds,
            surface_id,
        });
        self.update_bounds();

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

    fn update_output(&mut self, id: u32, bounds: Rectangle) -> Task<BackgroundMessage> {
        self.outputs.iter_mut().find(|x| x.id == id).unwrap().bounds = bounds;
        self.update_bounds();
        Task::none()
    }

    fn remove_output(&mut self, id: u32) -> Task<BackgroundMessage> {
        let i = self.outputs.iter().position(|x| x.id == id).unwrap();
        let output = self.outputs.swap_remove(i);
        self.update_bounds();

        layer_surface::destroy_layer_surface(output.surface_id)
    }

    fn update_bounds(&mut self) {
        self.global_bounds = self
            .outputs
            .iter()
            .map(|x| x.bounds)
            .reduce(|acc, x| acc.union(&x))
            .unwrap_or_else(|| Rectangle::new(Point::ORIGIN, Size::ZERO))
    }

    fn move_cursor(&mut self, surface_id: Id, position: Point) -> Task<BackgroundMessage> {
        let output_position = self
            .outputs
            .iter()
            .find(|x| x.surface_id == surface_id)
            .unwrap()
            .bounds
            .position();
        self.cursor_position = Point {
            x: output_position.x + position.x,
            y: output_position.y + position.y,
        };
        Task::none()
    }

    pub fn view(&self, surface_id: Id) -> Option<Element<'_, BackgroundMessage>> {
        if self.outputs.iter().all(|x| x.surface_id != surface_id) {
            return None;
        }

        const BACKGROUND_SCALE: f32 = 1.0;
        const MIDDLE_GROUND_SCALE: f32 = 1.01;
        const FOREGROUND_SCALE: f32 = 1.03;

        let view = stack![
            self.view_layer(
                &self.wallpaper_background,
                BACKGROUND_SCALE,
                self.global_bounds,
                self.cursor_position,
            ),
            self.view_clock(self.now),
            self.view_layer(
                &self.wallpaper_middle_ground,
                MIDDLE_GROUND_SCALE,
                self.global_bounds,
                self.cursor_position,
            ),
            self.view_layer(
                &self.wallpaper_foreground,
                FOREGROUND_SCALE,
                self.global_bounds,
                self.cursor_position,
            ),
        ];
        Some(view.into())
    }

    fn view_layer(
        &self,
        image_path: &Path,
        scale: f32,
        global_bounds: Rectangle,
        cursor_position: Point,
    ) -> Element<'_, BackgroundMessage> {
        float(image(Handle::from_path(image_path)).content_fit(ContentFit::Cover))
            .scale(scale)
            .translate(move |viewport, _| {
                let overflow_x = viewport.width * ((scale - 1.0) / 2.0);
                let overflow_y = viewport.height * ((scale - 1.0) / 2.0);

                let moved_cursor_position = Point::new(
                    cursor_position.x - global_bounds.x,
                    cursor_position.y - global_bounds.y,
                );

                Vector {
                    x: overflow_x * (1.0 - 2.0 * (moved_cursor_position.x / global_bounds.width)),
                    y: overflow_y * (1.0 - 2.0 * (moved_cursor_position.y / global_bounds.height)),
                }
            })
            .into()
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
        Subscription::batch([self.output_subscription(), self.cursor_subscription()])
    }

    fn output_subscription(&self) -> Subscription<BackgroundMessage> {
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
                OutputEvent::Created(x) => Some(BackgroundMessage::OutputCreated {
                    output,
                    bounds: get_output_bounds(x.as_ref().unwrap()),
                }),
                OutputEvent::InfoUpdate(x) => Some(BackgroundMessage::OutputChanged {
                    id: output.id().protocol_id(),
                    bounds: get_output_bounds(&x),
                }),
                OutputEvent::Removed => {
                    Some(BackgroundMessage::OutputRemoved(output.id().protocol_id()))
                }
            },
            _ => None,
        })
    }

    fn cursor_subscription(&self) -> Subscription<BackgroundMessage> {
        event::listen_with(|event, _, surface_id| match event {
            iced::Event::Mouse(mouse::Event::CursorMoved { position }) => {
                Some(BackgroundMessage::CursorMoved {
                    surface_id,
                    position,
                })
            }
            _ => None,
        })
    }
}
