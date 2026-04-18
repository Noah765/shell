use iced::{
    Border, Element, Radius, color,
    widget::{Container, container::Style},
};

#[allow(dead_code)]
pub trait DebugElement<'a, Message> {
    fn debug(self) -> Element<'a, Message>;
}

impl<'a, Message: 'a, X: Into<Element<'a, Message>>> DebugElement<'a, Message> for X {
    fn debug(self) -> Element<'a, Message> {
        Container::new(self)
            .style(|_| Style {
                border: Border {
                    color: color!(0xff0000),
                    width: 1.0,
                    radius: Radius::new(0),
                },
                ..Default::default()
            })
            .into()
    }
}
