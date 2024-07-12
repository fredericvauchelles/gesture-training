use std::path::PathBuf;
use iced::{Alignment, Application, Command, Element, executor, Theme};
use iced::theme::Radio;
use iced::widget::{button, text, column, image, text_input, radio, Row};

#[derive(Default)]
struct ImageSelection {
    folder_path: PathBuf,
}

enum ImageTime {
    FixedTime {
        seconds: u16
    },
    NoLimit,
}
impl Default for ImageTime {
    fn default() -> Self {
        Self::FixedTime { seconds: 30 }
    }
}

struct SessionConfiguration {
    image_selection: ImageSelection,
    image_count: u8,
    image_time: ImageTime,
}
impl Default for SessionConfiguration {
    fn default() -> Self {
        Self {
            image_selection: ImageSelection::default(),
            image_count: 5,
            image_time: ImageTime::default(),
        }
    }
}

#[derive(Default)]
pub struct ApplicationState {
    session_configuration: SessionConfiguration,
}

#[derive(Debug, Clone, Copy)]
pub enum ApplicationMessage {
    None,
    StartSession,
    SetImageCount(u8),
    SetImageTime(u16),
}

impl Application for ApplicationState {
    type Executor = executor::Default;
    type Message = ApplicationMessage;
    type Theme = Theme;
    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, Command<Self::Message>) {
        (Self::default(), Command::none())
    }

    fn title(&self) -> String {
        String::from("Counter - Iced")
    }

    fn update(&mut self, message: ApplicationMessage) -> Command<Self::Message> {
        match message {
            ApplicationMessage::StartSession => {}
            ApplicationMessage::None => {}
            ApplicationMessage::SetImageCount(value) => { self.session_configuration.image_count = value; }
            ApplicationMessage::SetImageTime(value) => { self.session_configuration.image_time = ImageTime::FixedTime { seconds: value } }
        }
        Command::none()
    }

    fn view(&self) -> Element<ApplicationMessage> {
        let text_title = text("Gesture Training");
        let textinput_folder_input = text_input("image folder", "");
        let radio_image_counts = Row::with_children((1..5u8).map(|i| {
            let amount = i * 5;
            radio(amount.to_string(), amount, Some(self.session_configuration.image_count), |value| -> Self::Message { ApplicationMessage::SetImageCount(amount) })
        }).map(Element::from));
        let radio_image_time = Row::with_children(([30, 60, 90, 120, 240, 600, 1200]).map(|i| {
            let amount = i;
            let selected = match self.session_configuration.image_time {
                ImageTime::FixedTime { seconds } => Some(seconds),
                ImageTime::NoLimit => None
            };
            radio(amount.to_string(), amount, selected, |value| -> Self::Message { ApplicationMessage::SetImageTime(amount) })
        }).map(Element::from));
        let button_start = button("Start");

        column![text_title, textinput_folder_input, radio_image_counts, radio_image_time, button_start]
            .padding(20)
            .align_items(Alignment::Center)
            .into()
    }
}
