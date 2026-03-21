use std::collections::HashMap;

use chrono::{DateTime, Local};
use iced::{
    Element, Event, Point, Rectangle, Size, Subscription, Task,
    event::{
        self, PlatformSpecific,
        wayland::{self, OutputEvent},
    },
    mouse,
    time::{self, seconds},
    window::Id,
};
use smithay_client_toolkit::{
    output::OutputInfo, reexports::client::protocol::wl_output::WlOutput,
};

use crate::background::Background;

#[derive(Debug)]
pub struct Shell {
    backgrounds: HashMap<Id, Background>,
    background_bounds: Rectangle,
    cursor_position: Point,
    now: DateTime<Local>,
}

#[derive(Clone, Debug)]
pub enum Message {
    OutputCreated(WlOutput, Rectangle),
    OutputChanged(WlOutput, Rectangle),
    OutputRemoved(WlOutput),
    CursorMoved(Id, Point),
    TimeTick(DateTime<Local>),
}

impl Shell {
    pub fn new() -> Self {
        Self {
            backgrounds: HashMap::new(),
            background_bounds: Rectangle::new(Point::ORIGIN, Size::ZERO),
            cursor_position: Point::ORIGIN,
            now: Local::now(),
        }
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::OutputCreated(output, bounds) => {
                let surface_id = Id::unique();
                let (background, task) = Background::new(output, bounds, surface_id);
                self.backgrounds.insert(surface_id, background);
                self.update_bounds();
                task
            }
            Message::OutputChanged(output, bounds) => {
                *self
                    .backgrounds
                    .get_mut(&self.get_background_surface_id(output))
                    .unwrap()
                    .bounds_mut() = bounds;
                Task::none()
            }
            Message::OutputRemoved(output) => {
                let surface_id = self.get_background_surface_id(output);
                let background = self.backgrounds.remove(&surface_id).unwrap();
                self.update_bounds();
                background.destroy(surface_id)
            }
            Message::CursorMoved(surface_id, position) => {
                let output_position = self.backgrounds[&surface_id].bounds().position();
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

    fn get_background_surface_id(&self, output: WlOutput) -> Id {
        *self
            .backgrounds
            .iter()
            .find(|x| x.1.on_output(&output))
            .unwrap()
            .0
    }

    fn update_bounds(&mut self) {
        self.background_bounds = self
            .backgrounds
            .values()
            .map(|x| x.bounds())
            .reduce(|acc, x| acc.union(&x))
            .unwrap_or_else(|| Rectangle::new(Point::ORIGIN, Size::ZERO))
    }

    pub fn view(&self, surface_id: Id) -> Element<'_, Message> {
        self.backgrounds[&surface_id].view(self.background_bounds, self.cursor_position, self.now)
    }

    pub fn subscription(&self) -> Subscription<Message> {
        Subscription::batch([
            self.output_subscription(),
            self.mouse_subscription(),
            self.time_subscription(),
        ])
    }

    fn output_subscription(&self) -> Subscription<Message> {
        fn get_output_bounds(info: OutputInfo) -> Rectangle {
            Rectangle {
                x: info.logical_position.unwrap().0 as f32,
                y: info.logical_position.unwrap().1 as f32,
                width: info.logical_size.unwrap().0 as f32,
                height: info.logical_size.unwrap().1 as f32,
            }
        }

        event::listen_with(|event, _, _| match event {
            Event::PlatformSpecific(PlatformSpecific::Wayland(wayland::Event::Output(
                event,
                output,
            ))) => match event {
                OutputEvent::Created(x) => {
                    let bounds = get_output_bounds(x.unwrap());
                    Some(Message::OutputCreated(output, bounds))
                }
                OutputEvent::InfoUpdate(x) => {
                    Some(Message::OutputChanged(output, get_output_bounds(x)))
                }
                OutputEvent::Removed => Some(Message::OutputRemoved(output)),
            },
            _ => None,
        })
    }

    fn mouse_subscription(&self) -> Subscription<Message> {
        event::listen_with(|event, _, surface_id| match event {
            Event::Mouse(mouse::Event::CursorMoved { position }) => {
                Some(Message::CursorMoved(surface_id, position))
            }
            _ => None,
        })
    }

    fn time_subscription(&self) -> Subscription<Message> {
        time::every(seconds(10)).map(|_| Message::TimeTick(Local::now()))
    }
}
