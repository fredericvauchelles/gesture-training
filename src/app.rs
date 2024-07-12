use iced::{Application, Command, Element, executor, Theme};

use crate::prepare_session::session_configuration::SessionConfiguration;
use crate::prepare_session::session_preparation::{MessagePrepareSession, StatePreparedSession, WorkflowPrepareSession};
use crate::run_session::{MessageRunSession, WorkflowRunSession};

#[derive(Clone, Debug)]
pub enum Workflow {
    PrepareSession(WorkflowPrepareSession),
    RunSession(WorkflowRunSession),
}

impl Default for Workflow {
    fn default() -> Self {
        Workflow::PrepareSession(WorkflowPrepareSession::default())
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    None,
    PrepareSession(MessagePrepareSession),
    RunSession(MessageRunSession),
}

#[derive(Default, Clone, Debug)]
pub struct State {
    pub session_configuration: SessionConfiguration,
    pub session_prepared: StatePreparedSession,
    pub current_workflow: Workflow,
}

pub trait AppWorkflow {
    type WorkflowMessage;
    type AppMessage;

    fn update(state: &mut State, message: Self::WorkflowMessage) -> Command<Self::AppMessage>;
    fn view(&self, state: &State) -> Element<Self::AppMessage>;
}

impl Application for State {
    type Executor = executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, Command<Self::Message>) {
        (Self::default(), Command::none())
    }

    fn title(&self) -> String {
        String::from("Counter - Iced")
    }

    fn update(&mut self, message: Message) -> Command<Self::Message> {
        let state = self;
        match message {
            Message::PrepareSession(message) => WorkflowPrepareSession::update(state, message),
            Message::RunSession(message) => WorkflowRunSession::update(state, message),
            Message::None => Command::none()
        }
    }

    fn view(&self) -> Element<Message> {
        match &self.current_workflow {
            Workflow::PrepareSession(workflow) => workflow.view(&self),
            Workflow::RunSession(workflow) => workflow.view(&self)
        }
    }
}

