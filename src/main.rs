use iced::{Sandbox, Settings};

mod prototype;

fn main() -> iced::Result {
    prototype::Counter::run(Settings::default())
}
