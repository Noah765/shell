use crate::shell::Shell;

mod shell;

fn main() -> iced::Result {
    iced::daemon(Shell::new, Shell::update, Shell::view)
        .title("shell")
        .subscription(Shell::subscription)
        .run()
}
