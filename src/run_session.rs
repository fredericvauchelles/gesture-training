use std::cmp::max;
use std::path::PathBuf;

use iced::{Alignment, Command, Element};
use iced::widget::{button, image, row, text};

use crate::app::{AppWorkflow, Message, State, Workflow};
use crate::prepare_session::session_configuration::{ImageTime, SessionConfiguration};
use crate::prepare_session::session_preparation::{StatePreparedSession, WorkflowPrepareSession};

#[derive(Default, Clone, Debug)]
pub struct WorkflowRunSession {
    current_image_index: u8,
    is_running: bool,
    remaining_seconds: u16,
    image_paths: Vec<PathBuf>,
}

impl AppWorkflow for WorkflowRunSession {
    type AppMessage = Message;
    type WorkflowMessage = MessageRunSession;
    fn view(&self, _state: &State) -> Element<Self::AppMessage> {
        let text_title = text("Gesture Training");

        let image = image("");

        let button_back = button("<").on_press(Message::RunSession(MessageRunSession::PreviousImage));
        let button_stop = button("Stop").on_press(Message::RunSession(MessageRunSession::Stop));
        let button_playpause = if self.is_running {
            button("Play").on_press(Message::RunSession(MessageRunSession::Pause))
        } else {
            button("Pause").on_press(Message::RunSession(MessageRunSession::Resume))
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
                MessageRunSession::Resume => {
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
            is_running: true,
            remaining_seconds,
            image_paths: session_prepared.valid_images.clone(),
        }
    }
}


#[derive(Debug, Clone)]
pub enum MessageRunSession {
    Pause,
    Resume,
    Stop,
    NextImage,
    PreviousImage,
}

impl Into<Message> for MessageRunSession {
    fn into(self) -> Message {
        Message::RunSession(self)
    }
}
