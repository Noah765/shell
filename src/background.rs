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
use smithay_client_toolkit::reexports::client::{Proxy, protocol::wl_output::WlOutput};

use crate::shell::Message;

#[derive(Debug)]
pub struct Background {
    output_id: u32,
    bounds: Rectangle,
}

impl Background {
    pub fn new(output: WlOutput, bounds: Rectangle, surface_id: Id) -> (Self, Task<Message>) {
        let background = Self {
            output_id: output.id().protocol_id(),
            bounds,
        };

        let task = layer_surface::get_layer_surface(SctkLayerSurfaceSettings {
            id: surface_id,
            layer: Layer::Background,
            anchor: Anchor::all(),
            output: IcedOutput::Output(output),
            namespace: String::from("shell"),
            ..Default::default()
        });

        (background, task)
    }

    pub fn view(
        &self,
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
                global_bounds,
                cursor_position,
            ),
            self.view_clock(now),
            self.view_layer(
                MIDDLEGROUND.clone(),
                MIDDLEGROUND_SCALE,
                global_bounds,
                cursor_position,
            ),
            self.view_layer(
                FOREGROUND.clone(),
                FOREGROUND_SCALE,
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
        global_bounds: Rectangle,
        cursor_position: Point,
    ) -> Element<'_, Message> {
        float(image(handle).content_fit(ContentFit::Cover))
            .scale(scale)
            .translate(move |_, _| {
                let overflow_x = self.bounds.width * ((scale - 1.0) / 2.0);
                let overflow_y = self.bounds.height * ((scale - 1.0) / 2.0);

                Vector {
                    x: overflow_x * (1.0 - 2.0 * (cursor_position.x / global_bounds.width)),
                    y: overflow_y * (1.0 - 2.0 * (cursor_position.y / global_bounds.height)),
                }
            })
            .into()
    }

    fn view_clock(&self, now: DateTime<Local>) -> Element<'_, Message> {
        center(
            text(now.format("%R").to_string())
                .size(self.bounds.width / 15.0)
                .font(Font {
                    family: Family::Monospace,
                    weight: Weight::Bold,
                    ..Default::default()
                }),
        )
        .into()
    }

    pub fn on_output(&self, output: &WlOutput) -> bool {
        self.output_id == output.id().protocol_id()
    }

    pub fn bounds(&self) -> Rectangle {
        self.bounds
    }

    pub fn bounds_mut(&mut self) -> &mut Rectangle {
        &mut self.bounds
    }

    pub fn destroy(self, surface_id: Id) -> Task<Message> {
        layer_surface::destroy_layer_surface(surface_id)
    }
}
