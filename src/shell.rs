use std::path::Path;

use chrono::{DateTime, Local};
use iced::{
    Element, Subscription, Task,
    time::{self, seconds},
    window::Id,
};

use crate::{
    background::{Background, BackgroundMessage},
    bar::{Bar, BarMessage},
};

#[derive(Debug)]
pub struct Shell {
    background: Background,
    bar: Bar,
}

#[derive(Clone, Debug)]
pub enum Message {
    TimeTick(DateTime<Local>),
    Background(BackgroundMessage),
    Bar(BarMessage),
}

impl Shell {
    pub fn new(
        wallpaper_background: &Path,
        wallpaper_middle_ground: &Path,
        wallpaper_foreground: &Path,
    ) -> Self {
        let now = Local::now();

        Self {
            background: Background::new(
                wallpaper_background,
                wallpaper_middle_ground,
                wallpaper_foreground,
                now,
            ),
            bar: Bar::new(now),
        }
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::TimeTick(x) => Task::batch([
                Task::done(Message::Background(BackgroundMessage::TimeTick(x))),
                Task::done(Message::Bar(BarMessage::TimeTick(x))),
            ]),
            Message::Background(x) => self.background.update(x).map(Message::Background),
            Message::Bar(x) => self.bar.update(x).map(Message::Bar),
        }
    }

    pub fn view(&self, surface_id: Id) -> Element<'_, Message> {
        self.background
            .view(surface_id)
            .map(|x| x.map(Message::Background))
            .or_else(|| self.bar.view(surface_id).map(|x| x.map(Message::Bar)))
            .unwrap()
    }

    pub fn subscription(&self) -> Subscription<Message> {
        Subscription::batch([
            time::every(seconds(10)).map(|_| Message::TimeTick(Local::now())),
            self.background.subscription().map(Message::Background),
            self.bar.subscription().map(Message::Bar),
        ])
    }
}
