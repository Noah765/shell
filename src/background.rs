use std::path::Path;

use chrono::{DateTime, Local};
use iced::{
    ContentFit, Element, Font, Point, Rectangle, Size, Task, Vector,
    core::image::Handle,
    font::{Family, Weight},
    platform_specific::shell::commands::{
        layer_surface,
        subsurface::{Anchor, Layer},
    },
    runtime::platform_specific::wayland::layer_surface::{IcedOutput, SctkLayerSurfaceSettings},
    widget::{center, float, image, responsive, space, stack, text},
    window::Id,
};
use smithay_client_toolkit::reexports::client::protocol::wl_output::WlOutput;

use crate::shell::Message;

#[derive(Debug)]
pub struct Background {
    surface_id: Id,
}

impl Background {
    pub fn new(output: WlOutput) -> (Self, Task<Message>) {
        let surface_id = Id::unique();
        let task = layer_surface::get_layer_surface(SctkLayerSurfaceSettings {
            id: surface_id,
            layer: Layer::Background,
            anchor: Anchor::all(),
            output: IcedOutput::Output(output),
            namespace: String::from("shell-background"),
            exclusive_zone: -1,
            ..Default::default()
        });
        (Self { surface_id }, task)
    }

    pub fn view(
        &self,
        global_bounds: Rectangle,
        background: &Path,
        middle_ground: &Path,
        foreground: &Path,
        cursor_position: Point,
        now: DateTime<Local>,
    ) -> Element<'_, Message> {
        const BACKGROUND_SCALE: f32 = 1.0;
        const MIDDLE_GROUND_SCALE: f32 = 1.01;
        const FOREGROUND_SCALE: f32 = 1.03;

        stack![
            self.view_layer(background, BACKGROUND_SCALE, global_bounds, cursor_position,),
            self.view_clock(now),
            self.view_layer(
                middle_ground,
                MIDDLE_GROUND_SCALE,
                global_bounds,
                cursor_position,
            ),
            self.view_layer(foreground, FOREGROUND_SCALE, global_bounds, cursor_position,),
        ]
        .into()
    }

    fn view_layer(
        &self,
        image_path: &Path,
        scale: f32,
        global_bounds: Rectangle,
        cursor_position: Point,
    ) -> Element<'_, Message> {
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

    fn view_clock(&self, now: DateTime<Local>) -> Element<'_, Message> {
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

    pub fn surface_id(&self) -> Id {
        self.surface_id
    }

    pub fn destroy(self) -> Task<Message> {
        layer_surface::destroy_layer_surface(self.surface_id)
    }
}
