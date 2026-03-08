use std::collections::HashMap;

use iced::{
    Event, Subscription, Task,
    event::{
        self, PlatformSpecific,
        wayland::{self, OutputEvent},
    },
    platform_specific::shell::commands::{
        layer_surface,
        subsurface::{Anchor, Layer},
    },
    runtime::platform_specific::wayland::layer_surface::{IcedOutput, SctkLayerSurfaceSettings},
    widget::{Container, container},
    window::Id,
};
use wayland_client::{Proxy, backend::ObjectId, protocol::wl_output::WlOutput};

pub struct Shell {
    backgrounds: HashMap<ObjectId, Id>,
}

#[derive(Clone, Debug)]
pub enum Message {
    Output(OutputEvent, WlOutput),
}

impl Shell {
    pub fn new() -> Self {
        Self {
            backgrounds: HashMap::new(),
        }
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Output(event, output) => match event {
                OutputEvent::Created(_) => {
                    let id = Id::unique();
                    self.backgrounds.insert(output.id(), id);
                    layer_surface::get_layer_surface(SctkLayerSurfaceSettings {
                        id,
                        layer: Layer::Background,
                        input_zone: Some(Vec::new()),
                        anchor: Anchor::all(),
                        output: IcedOutput::Output(output),
                        namespace: String::from("shell"),
                        ..Default::default()
                    })
                }
                OutputEvent::Removed => {
                    let id = self.backgrounds.remove(&output.id()).unwrap();
                    layer_surface::destroy_layer_surface(id)
                }
                OutputEvent::InfoUpdate(_) => Task::none(),
            },
        }
    }

    pub fn view(&self, _: Id) -> Container<'_, Message> {
        container("Hello, world!")
    }

    pub fn subscription(&self) -> Subscription<Message> {
        event::listen_with(|event, _, _| match event {
            Event::PlatformSpecific(PlatformSpecific::Wayland(wayland::Event::Output(
                event,
                output,
            ))) => Some(Message::Output(event, output)),
            _ => None,
        })
    }
}
