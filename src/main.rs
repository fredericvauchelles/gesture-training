use iced::{Settings, Application};

mod prototype;

fn main() -> iced::Result {
    let settings = Settings::with_flags(());
    prototype::Counter::run(settings)
}
