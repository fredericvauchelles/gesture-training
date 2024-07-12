use std::path::PathBuf;
use iced::{Alignment, Application, Command, Element, executor, Theme};
use iced::command::channel;
use iced::theme::Radio;
use iced::widget::{button, text, column, image, text_input, radio, Row, row};
use rfd::AsyncFileDialog;

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

#[derive(Debug, Clone)]
pub enum ApplicationMessage {
    None,
    StartSession,
    SetImageCount(u8),
    SetImageTime(u16),
    SelectImageFolder,
    SetImageFolder(Option<PathBuf>),
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
            ApplicationMessage::StartSession => { Command::none() }
            ApplicationMessage::None => { Command::none() }
            ApplicationMessage::SetImageCount(value) => {
                self.session_configuration.image_count = value;
                Command::none()
            }
            ApplicationMessage::SetImageTime(value) => {
                self.session_configuration.image_time = ImageTime::FixedTime { seconds: value };
                Command::none()
            }
            ApplicationMessage::SelectImageFolder => {
                let future = async {
                    AsyncFileDialog::new()
                        .pick_folder()
                        .await
                };
                Command::perform(future, |file_handle| {
                    ApplicationMessage::SetImageFolder(file_handle.map(|handle| handle.path().to_path_buf()))
                })
            }
            ApplicationMessage::SetImageFolder(option_path) => {
                self.session_configuration.image_selection.folder_path = option_path.unwrap_or(PathBuf::new());
                Command::none()
            }
        }
    }

    fn view(&self) -> Element<ApplicationMessage> {
        let text_title = text("Gesture Training");

        let text_folder_input = text("Image folder");
        let text_folder_selected = text(self.session_configuration.image_selection.folder_path.to_string_lossy());
        let button_choose_folder = button("Pick").on_press(ApplicationMessage::SelectImageFolder);
        let row_folder_selector = row!(text_folder_input, text_folder_selected, button_choose_folder);

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
        let button_start = button("Start").on_press(ApplicationMessage::StartSession);

        column!(text_title, row_folder_selector, radio_image_counts, radio_image_time, button_start)
            .padding(20)
            .align_items(Alignment::Center)
            .into()
    }
}
