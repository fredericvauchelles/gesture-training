use std::time::{Duration, Instant};

use iced::{Application, Command, Element, executor, Subscription, Theme, time};

use crate::prepare_session::{MessagePrepareSession, StatePreparedSession, WorkflowPrepareSession};
use crate::prepare_session::SessionConfiguration;
use crate::run_session::{MessageRunSession, WorkflowRunSession};

/// Rexport iced:widget::column!
/// Otherwise, RustRover confuses with another macro with the same name (stdlib-local-copy::column!)
/// during code analysis and anything used inside is not detected as used
#[macro_export]
macro_rules! col {
    () => (
        iced::widget::Column::new()
    );
    ($($x:expr),+ $(,)?) => (
        iced::widget::Column::with_children([$(iced::Element::from($x)),+])
    );
}

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
    Tick(Instant),
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
    fn tick(&mut self, instant: Instant) -> Command<Self::AppMessage>;
}

impl Application for State {
    type Executor = executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, Command<Self::Message>) {
        (Self::default(), Command::none())
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        let tick = match self.current_workflow {
            Workflow::PrepareSession(_) => Subscription::none(),
            Workflow::RunSession(_) => time::every(Duration::from_millis(1000)).map(Message::Tick)
        };

        tick
    }

    fn title(&self) -> String {
        String::from("Gesture Training")
    }

    fn update(&mut self, message: Message) -> Command<Self::Message> {
        match message {
            Message::PrepareSession(message) => WorkflowPrepareSession::update(self, message),
            Message::RunSession(message) => WorkflowRunSession::update(self, message),
            Message::None => Command::none(),
            Message::Tick(instant) => match &mut self.current_workflow {
                Workflow::PrepareSession(workflow) => workflow.tick(instant),
                Workflow::RunSession(workflow) => workflow.tick(instant),
            }
        }
    }

    fn view(&self) -> Element<Message> {
        match &self.current_workflow {
            Workflow::PrepareSession(workflow) => workflow.view(&self),
            Workflow::RunSession(workflow) => workflow.view(&self)
        }
    }
}

