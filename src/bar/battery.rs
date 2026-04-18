use std::io::ErrorKind;

use iced::{
    Element, Subscription,
    alignment::Horizontal,
    time::{self, seconds},
    widget::{column, space, text},
};

use crate::{bar::BAR_WIDTH, icon};

#[derive(Debug)]
pub struct Battery(Option<(BatteryStatus, u8)>);

#[derive(Clone, Debug)]
pub struct BatteryMessage(BatteryStatus, u8);

#[derive(Debug, Clone, Copy)]
pub enum BatteryStatus {
    Charging,
    Discharging,
}

impl Battery {
    pub fn new() -> Self {
        Self(Self::fetch_battery())
    }

    fn fetch_battery() -> Option<(BatteryStatus, u8)> {
        let status = match std::fs::read_to_string("/sys/class/power_supply/BAT0/status") {
            Ok(x) => match x.trim_end() {
                "Charging" | "Full" => BatteryStatus::Charging,
                _ => BatteryStatus::Discharging,
            },
            Err(x) if x.kind() == ErrorKind::NotFound => return None,
            Err(x) => panic!("{x}"),
        };
        let capacity = std::fs::read_to_string("/sys/class/power_supply/BAT0/capacity")
            .unwrap()
            .trim_end()
            .parse()
            .unwrap();
        Some((status, capacity))
    }

    pub fn update(&mut self, message: BatteryMessage) {
        self.0 = Some((message.0, message.1));
    }

    pub fn view(&self, width: f32) -> Element<'_, BatteryMessage> {
        let Some(battery) = self.0 else {
            return space().height(0).into();
        };

        let size = 18.0 / BAR_WIDTH * width;
        let icon = match battery {
            (BatteryStatus::Charging, _) => icon::battery_charging().size(size).line_height(1.0),
            (BatteryStatus::Discharging, 0..14) => icon::battery_1().size(size).line_height(1.0),
            (BatteryStatus::Discharging, 14..40) => icon::battery_2().size(size).line_height(1.0),
            (BatteryStatus::Discharging, 40..66) => icon::battery_3().size(size).line_height(1.0),
            (BatteryStatus::Discharging, 66..) => icon::battery_4().size(size).line_height(1.0),
        };

        let text = text!("{}%", battery.1)
            .size(10.0 / BAR_WIDTH * width)
            .line_height(1.0);

        column![icon, text].align_x(Horizontal::Center).into()
    }

    pub fn subscription(&self) -> Subscription<BatteryMessage> {
        if self.0.is_none() {
            return Subscription::none();
        }

        async fn tick() -> BatteryMessage {
            let status = match tokio::fs::read_to_string("/sys/class/power_supply/BAT0/status")
                .await
                .unwrap()
                .trim_end()
            {
                "Charging" | "Full" => BatteryStatus::Charging,
                _ => BatteryStatus::Discharging,
            };
            let capacity = tokio::fs::read_to_string("/sys/class/power_supply/BAT0/capacity")
                .await
                .unwrap()
                .trim_end()
                .parse()
                .unwrap();
            BatteryMessage(status, capacity)
        }
        time::repeat(tick, seconds(1))
    }
}
