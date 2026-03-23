use chrono::{DateTime, Local};
use iced::{
    Element, Point, Rectangle, Size, Subscription, Task,
    event::{
        self, PlatformSpecific,
        wayland::{self, OutputEvent},
    },
    mouse,
    time::{self, seconds},
    window::Id,
};
use smithay_client_toolkit::{
    output::OutputInfo,
    reexports::client::{Proxy, backend::ObjectId, protocol::wl_output::WlOutput},
};

use crate::{background::Background, bar::Bar};

#[derive(Debug)]
pub struct Shell {
    outputs: Vec<Output>,
    background_bounds: Rectangle,
    cursor_position: Point,
    now: DateTime<Local>,
}

#[derive(Debug)]
struct Output {
    id: ObjectId,
    bounds: Rectangle,
    background: Background,
    bar: Bar,
}
#[derive(Clone, Debug)]
pub enum Message {
    OutputCreated { output: WlOutput, bounds: Rectangle },
    OutputChanged { output: WlOutput, bounds: Rectangle },
    OutputRemoved(WlOutput),
    CursorMoved { surface_id: Id, position: Point },
    TimeTick(DateTime<Local>),
}

impl Shell {
    pub fn new() -> Self {
        Self {
            outputs: Vec::new(),
            background_bounds: Rectangle::new(Point::ORIGIN, Size::ZERO),
            cursor_position: Point::ORIGIN,
            now: Local::now(),
        }
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::OutputCreated { output, bounds } => {
                let (background, background_task) = Background::new(output.clone());
                let (bar, bar_task) = Bar::new(output.clone());

                self.outputs.push(Output {
                    id: output.id(),
                    bounds,
                    background,
                    bar,
                });
                self.update_background_bounds();

                Task::batch([background_task, bar_task])
            }
            Message::OutputChanged { output, bounds } => {
                let output = self
                    .outputs
                    .iter_mut()
                    .find(|x| x.id == output.id())
                    .unwrap();
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
                    x.bounds,
                    self.background_bounds,
                    self.cursor_position,
                    self.now,
                );
            } else if x.bar.surface_id() == surface_id {
                return x.bar.view(self.now);
            }
        }
        unreachable!();
    }

    pub fn subscription(&self) -> Subscription<Message> {
        Subscription::batch([
            self.output_subscription(),
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
                }),
                OutputEvent::InfoUpdate(x) => Some(Message::OutputChanged {
                    output,
                    bounds: get_output_bounds(&x),
                }),
                OutputEvent::Removed => Some(Message::OutputRemoved(output)),
            },
            _ => None,
        })
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
}
