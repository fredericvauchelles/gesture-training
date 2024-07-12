use std::cmp::max;
use std::io;
use std::path::PathBuf;

use iced::{Alignment, Command, Element};
use iced::widget::{button, image, row, text};

use crate::app::{AppWorkflow, Message, State, Workflow};
use crate::prepare_session::session_configuration::{ImageTime, SessionConfiguration};
use crate::prepare_session::session_preparation::{StatePreparedSession, WorkflowPrepareSession};

#[derive(Default, Clone, Debug)]
pub struct WorkflowRunSession {
    current_image_index: u8,
    loaded_image_bytes: Option<Vec<u8>>,
    loaded_image_index: u8,
    is_running: bool,
    remaining_seconds: u16,
    image_paths: Vec<PathBuf>,
}

impl AppWorkflow for WorkflowRunSession {
    type AppMessage = Message;
    type WorkflowMessage = MessageRunSession;
    fn view(&self, _state: &State) -> Element<Self::AppMessage> {
        let text_title = text("Gesture Training");

        let image = match &self.loaded_image_bytes {
            None => image(""),
            Some(bytes) => image(image::Handle::from_memory(bytes.clone())),
        };

        let button_back = button("<").on_press(Message::RunSession(MessageRunSession::PreviousImage));
        let button_stop = button("Stop").on_press(Message::RunSession(MessageRunSession::Stop));
        let button_playpause = if self.is_running {
            button("Play").on_press(Message::RunSession(MessageRunSession::Pause))
        } else {
            button("Pause").on_press(Message::RunSession(MessageRunSession::Play))
        };
        let button_next = button(">").on_press(Message::RunSession(MessageRunSession::NextImage));
        let text_timeremaining = text(self.remaining_seconds.to_string());
        let row_actionbar = row!(button_back, button_stop, button_playpause, button_next, text_timeremaining);

        iced::widget::column!(text_title, image, row_actionbar)
            .padding(20)
            .align_items(Alignment::Center)
            .into()
    }
    fn update(state: &mut State, message: Self::WorkflowMessage) -> Command<Self::AppMessage> {
        if let MessageRunSession::Stop = message {
            state.current_workflow = Workflow::PrepareSession(WorkflowPrepareSession::default());
            Command::none()
        } else if let Workflow::RunSession(run_session) = &mut state.current_workflow {
            match message {
                MessageRunSession::Pause => {
                    run_session.is_running = false;
                    Command::none()
                }
                MessageRunSession::Play => {
                    run_session.is_running = false;

                    if run_session.loaded_image_index == run_session.current_image_index && run_session.loaded_image_bytes.is_some() {
                        // image is already loaded
                        run_session.is_running = true;
                        Command::none()
                    } else {
                        Command::perform(
                            Self::load_image_at(run_session.current_image_index, run_session.image_paths.clone()),
                            |bytes| {
                                match bytes {
                                    Ok(Some(bytes)) => Message::RunSession(MessageRunSession::ShowImage(bytes)),
                                    Ok(None) => Message::None,
                                    Err(error) => {
                                        eprintln!("{}", error);
                                        Message::None
                                    }
                                }
                            },
                        )
                    }
                }
                MessageRunSession::ShowImage(bytes) => {
                    run_session.loaded_image_bytes = Some(bytes);
                    run_session.is_running = true;
                    Command::none()
                }
                MessageRunSession::Stop => { unreachable!() }
                MessageRunSession::NextImage => {
                    let image_count = max(1u8, run_session.image_paths.len() as u8);
                    run_session.current_image_index = (run_session.current_image_index + 1) % image_count;
                    Command::none()
                }
                MessageRunSession::PreviousImage => {
                    let image_count = max(1u8, run_session.image_paths.len() as u8);
                    run_session.current_image_index = (run_session.current_image_index + (image_count - 1)) % image_count;
                    Command::none()
                }
            }
        } else {
            unreachable!()
        }
    }
}

impl WorkflowRunSession {
    pub fn new(session_configuration: &SessionConfiguration, session_prepared: &StatePreparedSession) -> Self {
        let remaining_seconds = match session_configuration.image_time {
            ImageTime::FixedTime { seconds } => seconds,
            ImageTime::NoLimit => 120
        };
        Self {
            current_image_index: 0,
            loaded_image_bytes: None,
            loaded_image_index: 0,
            is_running: true,
            remaining_seconds,
            image_paths: session_prepared.valid_images.clone(),
        }
    }

    async fn load_image_at(index: u8, paths: Vec<PathBuf>) -> io::Result<Option<Vec<u8>>> {
        if paths.is_empty() {
            return Ok(None);
        }
        let safe_index = index % (paths.len() as u8);
        let path = &paths[safe_index as usize];

        async_fs::read(path).await.map(Some)
    }
}


#[derive(Debug, Clone)]
pub enum MessageRunSession {
    Pause,
    Play,
    Stop,
    NextImage,
    PreviousImage,
    ShowImage(Vec<u8>),
}

impl Into<Message> for MessageRunSession {
    fn into(self) -> Message {
        Message::RunSession(self)
    }
}
