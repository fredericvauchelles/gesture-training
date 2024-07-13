use std::io;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use iced::{Alignment, Command, Element, Length, Padding};
use iced::alignment::Horizontal;
use iced::Alignment::Start;
use iced::futures::TryStreamExt;
use iced::widget::{button, container, radio, row, Row, Space, text};
use rfd::AsyncFileDialog;

use crate::app::{AppWorkflow, Message, State, Workflow};
use crate::col;
use crate::run_session::{MessageRunSession, WorkflowRunSession};

#[derive(Default, Clone, Debug)]
pub struct ImageSelection {
    pub folder_path: PathBuf,
}

#[derive(Clone, Debug)]
pub enum ImageTime {
    FixedTime(Duration),
    NoLimit,
}
impl Default for ImageTime {
    fn default() -> Self {
        Self::FixedTime(Duration::from_secs(30))
    }
}

#[derive(Clone, Debug)]
pub struct SessionConfiguration {
    pub image_selection: ImageSelection,
    pub image_count: u8,
    pub image_time: ImageTime,
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


#[derive(Default, Debug, Clone)]
pub struct StatePreparedSession {
    pub valid_images: Vec<PathBuf>,
}

#[derive(Default, Debug, Clone)]
pub struct WorkflowPrepareSession {}

impl AppWorkflow for WorkflowPrepareSession {
    type WorkflowMessage = MessagePrepareSession;
    type AppMessage = Message;
    fn update(state: &mut State, message: Self::WorkflowMessage) -> Command<Self::AppMessage> {
        match message {
            MessagePrepareSession::None => Command::none(),
            MessagePrepareSession::StartSession => {
                state.current_workflow = Workflow::RunSession(WorkflowRunSession::new(&state.session_configuration, &state.session_prepared));
                let future = async {};
                Command::perform(future, |_| Message::RunSession(MessageRunSession::Play))
            }
            MessagePrepareSession::SetImageCount(value) => {
                state.session_configuration.image_count = value;
                Command::none()
            }
            MessagePrepareSession::SetImageTime(value) => {
                state.session_configuration.image_time = ImageTime::FixedTime(Duration::from_secs(value as u64));
                Command::none()
            }
            MessagePrepareSession::SelectImageFolder => {
                let future = async {
                    AsyncFileDialog::new()
                        .pick_folder()
                        .await
                };
                Command::perform(future, |file_handle| {
                    Message::PrepareSession(MessagePrepareSession::SetImageFolder(file_handle.map(|handle| handle.path().to_path_buf())))
                })
            }
            MessagePrepareSession::SetImageFolder(option_path) => {
                state.session_configuration.image_selection.folder_path = option_path.unwrap_or(PathBuf::new());

                Command::perform(Self::compute_new_prepared_session(state.session_configuration.clone()), |prepared_session| {
                    match prepared_session {
                        Ok(prepared_session) => { Message::PrepareSession(MessagePrepareSession::SetPreparedSession(prepared_session)) }
                        Err(error) => {
                            eprintln!("{}", error);
                            Message::None
                        }
                    }
                })
            }
            MessagePrepareSession::SetPreparedSession(new_prepared_session) => {
                state.session_prepared = new_prepared_session;
                Command::none()
            }
        }
    }
    fn view(&self, state: &State) -> Element<Self::AppMessage> {
        let text_title = text("Gesture Training")
            .width(Length::Fill)
            .horizontal_alignment(Horizontal::Center)
            .size(50);
        let space = Space::new(Length::Fill, Length::Fill);

        let text_folder_input_label = text("Image folder")
            .width(Length::Fixed(150.0));
        let text_folder_selected = text(state.session_configuration.image_selection.folder_path.to_string_lossy())
            .width(Length::Fill);
        let button_choose_folder = button("Pick")
            .on_press(Message::PrepareSession(MessagePrepareSession::SelectImageFolder));
        let text_image_count = text(format!("({} images)", state.session_prepared.valid_images.len()));
        let row_folder_selector = row!(text_folder_input_label, text_folder_selected, button_choose_folder, text_image_count)
            .align_items(Start)
            .spacing(5)
            .width(Length::Fill);

        let text_image_counts_label = text("Image Counts")
            .width(Length::Fixed(150.0));
        let radio_image_counts_items = (1..5u8).map(|i| {
            let amount = i * 5;
            radio(amount.to_string(), amount, Some(state.session_configuration.image_count), |_value| -> Message { MessagePrepareSession::SetImageCount(amount).into() })
        }).map(Element::from);
        let radio_image_counts = Row::with_children([Element::from(text_image_counts_label)].into_iter().chain(radio_image_counts_items))
            .align_items(Alignment::Start)
            .spacing(5)
            .width(Length::Fill);

        let text_duration_label = text("Duration")
            .width(Length::Fixed(150.0));
        let radio_duration_items = ([5, 30, 60, 90, 120, 240, 600, 1200]).map(|i| {
            let amount = i;
            let selected = match state.session_configuration.image_time {
                ImageTime::FixedTime(duration) => Some(duration.as_secs() as u16),
                ImageTime::NoLimit => None
            };
            radio(amount.to_string(), amount, selected, |_value| -> Message { MessagePrepareSession::SetImageTime(amount).into() })
        }).map(Element::from);
        let radio_duration = Row::with_children([Element::from(text_duration_label)].into_iter().chain(radio_duration_items))
            .align_items(Alignment::Start)
            .spacing(5)
            .width(Length::Fill);

        let button_start = button(text("Start").width(Length::Fill).horizontal_alignment(Horizontal::Center))
            .on_press(Message::PrepareSession(MessagePrepareSession::StartSession))
            .width(Length::Fill);

        let control_panel = container(col!(row_folder_selector, radio_image_counts, radio_duration, button_start)
            .padding(20)
            .spacing(10)
            .align_items(Alignment::Center)
        )
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(Padding::from(50));

        col!(text_title, space,  control_panel)
            .padding(20)
            .into()
    }

    fn tick(&mut self, _instant: Instant) -> Command<Self::AppMessage> {
        Command::none()
    }
}

impl WorkflowPrepareSession {
    const IMAGE_EXTENSIONS: &'static [&'static str] = &["jpg", "jpeg", "png", "bmp"];
    fn is_image_file(path: &Path) -> bool {
        if let Some(extension) = path.extension().and_then(|ext| ext.to_str()) {
            Self::IMAGE_EXTENSIONS.contains(&extension)
        } else { false }
    }
    async fn find_image_files_in_directory(path: &Path) -> io::Result<Vec<PathBuf>> {
        match async_fs::read_dir(path).await {
            Ok(mut read_dir) => {
                let mut image_paths = Vec::new();
                loop {
                    match read_dir.try_next().await {
                        Ok(Some(entry)) => {
                            if Self::is_image_file(&entry.path()) {
                                image_paths.push(entry.path())
                            }
                        }
                        Err(error) => {
                            eprintln!("{}", error);
                            break;
                        }
                        Ok(None) => {
                            break;
                        }
                    }
                }

                Ok(image_paths)
            }
            Err(error) => {
                eprintln!("{}", error);
                Err(error)
            }
        }
    }
    async fn compute_new_prepared_session(session_configuration: SessionConfiguration) -> io::Result<StatePreparedSession> {
        let valid_images = Self::find_image_files_in_directory(&session_configuration.image_selection.folder_path).await?;
        Ok(StatePreparedSession {
            valid_images
        })
    }
}

#[derive(Debug, Clone)]
pub enum MessagePrepareSession {
    None,
    StartSession,
    SetImageCount(u8),
    SetImageTime(u16),
    SelectImageFolder,
    SetImageFolder(Option<PathBuf>),
    SetPreparedSession(StatePreparedSession),
}
impl Into<Message> for MessagePrepareSession {
    fn into(self) -> Message {
        Message::PrepareSession(self)
    }
}


#[cfg(test)]
mod test {
    use std::path::Path;

    use super::WorkflowPrepareSession;

    #[test]
    fn test_image_extension() {
        assert!(WorkflowPrepareSession::is_image_file(&Path::new(&"test.jpg")));
    }
}