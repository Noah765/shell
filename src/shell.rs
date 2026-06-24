use chrono::{DateTime, Local};
use iced::{
    Element, Subscription, Task,
    time::{self, seconds},
    widget::space,
    window::Id,
};

use crate::{
    background::{Background, BackgroundMessage},
    bar::{Bar, BarMessage},
    calculator::{Calculator, CalculatorMessage},
    cli::Cli,
};

#[derive(Debug)]
pub struct Shell {
    cli: Cli,
    background: Option<Background>,
    bar: Bar,
    calculator: Calculator,
}

#[derive(Clone, Debug)]
pub enum Message {
    TimeTick(DateTime<Local>),
    Background(BackgroundMessage),
    Bar(BarMessage),
    Calculator(CalculatorMessage),
}

impl Shell {
    pub fn new(cli: Cli) -> Self {
        let now = Local::now();

        Self {
            background: cli.wallpapers.as_ref().map(|x| Background::new(x, now)),
            cli,
            bar: Bar::new(now),
            calculator: Calculator::new(),
        }
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::TimeTick(x) => Task::batch([
                Task::done(Message::Background(BackgroundMessage::TimeTick(x))),
                Task::done(Message::Bar(BarMessage::TimeTick(x))),
            ]),
            Message::Background(x) => match &mut self.background {
                None => Task::none(),
                Some(background) => background.update(x).map(Message::Background),
            },
            Message::Bar(x) => self.bar.update(x).map(Message::Bar),
            Message::Calculator(x) => self.calculator.update(x).map(Message::Calculator),
        }
    }

    pub fn view(&self, surface_id: Id) -> Element<'_, Message> {
        self.background
            .as_ref()
            .and_then(|x| x.view(surface_id).map(|x| x.map(Message::Background)))
            .or_else(|| self.bar.view(surface_id).map(|x| x.map(Message::Bar)))
            .or_else(|| {
                self.calculator
                    .view(&self.cli, surface_id)
                    .map(|x| x.map(Message::Calculator))
            })
            .unwrap_or_else(|| space().into())
    }

    pub fn subscription(&self) -> Subscription<Message> {
        Subscription::batch([
            time::every(seconds(10)).map(|_| Message::TimeTick(Local::now())),
            match &self.background {
                None => Subscription::none(),
                Some(x) => x.subscription().map(Message::Background),
            },
            self.bar.subscription().map(Message::Bar),
            self.calculator.subscription().map(Message::Calculator),
        ])
    }
}
