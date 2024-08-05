use iced::{Application, Settings};

pub mod prepare_session;
pub mod run_session;
pub mod app;

fn main() -> iced::Result {
    let settings = Settings::with_flags(());
    app::State::run(settings)
}
