use std::io;
use std::path::{Path, PathBuf};

use iced::{Alignment, Command, Element};
use iced::futures::TryStreamExt;
use iced::widget::{button, radio, row, Row, text};
use rfd::AsyncFileDialog;

use crate::app::{AppWorkflow, Message, State, Workflow};
use crate::run_session::{MessageRunSession, WorkflowRunSession};

use super::session_configuration::{ImageTime, SessionConfiguration};

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
                state.session_configuration.image_time = ImageTime::FixedTime { seconds: value };
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
        let text_title = text("Gesture Training");

        let text_folder_input = text("Image folder");
        let text_folder_selected = text(state.session_configuration.image_selection.folder_path.to_string_lossy());
        let button_choose_folder = button("Pick").on_press(Message::PrepareSession(MessagePrepareSession::SelectImageFolder));
        let text_image_count = text(format!("({} images", state.session_prepared.valid_images.len()));
        let row_folder_selector = row!(text_folder_input, text_folder_selected, button_choose_folder, text_image_count);

        let radio_image_counts = Row::with_children((1..5u8).map(|i| {
            let amount = i * 5;
            radio(amount.to_string(), amount, Some(state.session_configuration.image_count), |_value| -> Message { MessagePrepareSession::SetImageCount(amount).into() })
        }).map(Element::from));
        let radio_image_time = Row::with_children(([30, 60, 90, 120, 240, 600, 1200]).map(|i| {
            let amount = i;
            let selected = match state.session_configuration.image_time {
                ImageTime::FixedTime { seconds } => Some(seconds),
                ImageTime::NoLimit => None
            };
            radio(amount.to_string(), amount, selected, |_value| -> Message { MessagePrepareSession::SetImageTime(amount).into() })
        }).map(Element::from));
        let button_start = button("Start").on_press(Message::PrepareSession(MessagePrepareSession::StartSession));

        iced::widget::column!(text_title, row_folder_selector, radio_image_counts, radio_image_time, button_start)
            .padding(20)
            .align_items(Alignment::Center)
            .into()
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