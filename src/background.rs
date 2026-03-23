use std::sync::LazyLock;

use chrono::{DateTime, Local};
use iced::{
    ContentFit, Element, Font, Point, Rectangle, Task, Vector,
    core::image::Handle,
    font::{Family, Weight},
    platform_specific::shell::commands::{
        layer_surface,
        subsurface::{Anchor, Layer},
    },
    runtime::platform_specific::wayland::layer_surface::{IcedOutput, SctkLayerSurfaceSettings},
    widget::{center, float, image, stack, text},
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
        bounds: Rectangle,
        global_bounds: Rectangle,
        cursor_position: Point,
        now: DateTime<Local>,
    ) -> Element<'_, Message> {
        static BACKGROUND: LazyLock<Handle> =
            LazyLock::new(|| Handle::from_bytes(&include_bytes!("../assets/background.png")[..]));
        static MIDDLEGROUND: LazyLock<Handle> =
            LazyLock::new(|| Handle::from_bytes(&include_bytes!("../assets/middleground.png")[..]));
        static FOREGROUND: LazyLock<Handle> =
            LazyLock::new(|| Handle::from_bytes(&include_bytes!("../assets/foreground.png")[..]));

        const BACKGROUND_SCALE: f32 = 1.0;
        const MIDDLEGROUND_SCALE: f32 = 1.01;
        const FOREGROUND_SCALE: f32 = 1.03;

        stack![
            self.view_layer(
                BACKGROUND.clone(),
                BACKGROUND_SCALE,
                bounds,
                global_bounds,
                cursor_position,
            ),
            self.view_clock(bounds, now),
            self.view_layer(
                MIDDLEGROUND.clone(),
                MIDDLEGROUND_SCALE,
                bounds,
                global_bounds,
                cursor_position,
            ),
            self.view_layer(
                FOREGROUND.clone(),
                FOREGROUND_SCALE,
                bounds,
                global_bounds,
                cursor_position,
            ),
        ]
        .into()
    }

    fn view_layer(
        &self,
        handle: Handle,
        scale: f32,
        bounds: Rectangle,
        global_bounds: Rectangle,
        cursor_position: Point,
    ) -> Element<'_, Message> {
        float(image(handle).content_fit(ContentFit::Cover))
            .scale(scale)
            .translate(move |_, _| {
                let overflow_x = bounds.width * ((scale - 1.0) / 2.0);
                let overflow_y = bounds.height * ((scale - 1.0) / 2.0);

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

    fn view_clock(&self, bounds: Rectangle, now: DateTime<Local>) -> Element<'_, Message> {
        center(
            text(now.format("%R").to_string())
                .size(bounds.width / 15.0)
                .font(Font {
                    family: Family::Monospace,
                    weight: Weight::Bold,
                    ..Default::default()
                }),
        )
        .into()
    }

    pub fn surface_id(&self) -> Id {
        self.surface_id
    }

    pub fn destroy(self) -> Task<Message> {
        layer_surface::destroy_layer_surface(self.surface_id)
    }
}
