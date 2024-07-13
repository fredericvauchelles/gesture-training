use std::cmp::{max, min};
use std::io;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use iced::{Alignment, Command, Element};
use iced::widget::{button, image, row, text};
use rand::random;

use crate::app::{AppWorkflow, Message, State, Workflow};
use crate::prepare_session::{ImageTime, SessionConfiguration};
use crate::prepare_session::{StatePreparedSession, WorkflowPrepareSession};

#[derive(Clone, Debug)]
enum ImageIndex {
    BackwardHistoryIndex(usize),
    PathIndex(usize),
}
impl Default for ImageIndex {
    fn default() -> Self {
        Self::PathIndex(0)
    }
}

#[derive(Clone, Debug)]
pub struct WorkflowRunSession {
    image_path_history: Vec<usize>,
    next_image_index: ImageIndex,
    loaded_image_bytes: Option<Vec<u8>>,
    loaded_image_path_index: Option<usize>,
    is_running: bool,
    remaining_time: Duration,
    last_tick: Instant,
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
        let text_timeremaining = text((self.remaining_time.as_secs()).to_string());
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

                    let index_to_load = match run_session.next_image_index {
                        ImageIndex::BackwardHistoryIndex(index) => {
                            if run_session.image_path_history.is_empty() {
                                None
                            } else {
                                let index = min(run_session.image_path_history.len() - 1, index);
                                Some(run_session.image_path_history[run_session.image_path_history.len() - 1 - index])
                            }
                        }
                        ImageIndex::PathIndex(index) => Some(index)
                    };

                    if run_session.loaded_image_path_index == index_to_load && run_session.loaded_image_bytes.is_some() {
                        // image is already loaded
                        run_session.is_running = true;
                        Command::none()
                    } else if let Some(index) = index_to_load {
                        Command::perform(
                            Self::load_image_at(index, run_session.image_paths.clone()),
                            move |bytes| {
                                match bytes {
                                    Ok(Some(bytes)) => Message::RunSession(MessageRunSession::ShowImage(index, bytes)),
                                    Ok(None) => Message::None,
                                    Err(error) => {
                                        eprintln!("{}", error);
                                        Message::None
                                    }
                                }
                            },
                        )
                    } else {
                        Command::none()
                    }
                }
                MessageRunSession::ShowImage(image_path_index, bytes) => {
                    run_session.loaded_image_bytes = Some(bytes);
                    run_session.loaded_image_path_index = Some(image_path_index);
                    run_session.is_running = true;
                    run_session.remaining_time = match state.session_configuration.image_time {
                        ImageTime::FixedTime(duration) => duration,
                        ImageTime::NoLimit => Duration::from_secs(3600)
                    };
                    run_session.last_tick = Instant::now();
                    Command::none()
                }
                MessageRunSession::Stop => { unreachable!() }
                MessageRunSession::NextImage => {
                    match run_session.next_image_index {
                        ImageIndex::BackwardHistoryIndex(backward_index) if backward_index > 0 => {
                            run_session.next_image_index = ImageIndex::BackwardHistoryIndex(backward_index - 1);
                        }
                        _ => {
                            let next_index = {
                                let image_count = max(1usize, run_session.image_paths.len());
                                let mut index = 0usize;
                                for _ in 0..50 {
                                    index = random::<usize>() % image_count;
                                    if !run_session.image_path_history.contains(&index) {
                                        break;
                                    }
                                }
                                index
                            };
                            if let Some(index) = run_session.loaded_image_path_index {
                                run_session.image_path_history.push(index);
                            }
                            run_session.next_image_index = ImageIndex::PathIndex(next_index);
                        }
                    }

                    Command::perform(async {}, |_| Message::RunSession(MessageRunSession::Play))
                }
                MessageRunSession::PreviousImage => {
                    match run_session.next_image_index {
                        ImageIndex::BackwardHistoryIndex(backward_index) => {
                            let new_index = min(backward_index + 1, run_session.image_path_history.len() - 1);
                            run_session.next_image_index = ImageIndex::BackwardHistoryIndex(new_index);
                        }
                        ImageIndex::PathIndex(_) => {
                            run_session.next_image_index = ImageIndex::BackwardHistoryIndex(0);
                        }
                    }

                    Command::perform(async {}, |_| Message::RunSession(MessageRunSession::Play))
                }
            }
        } else {
            unreachable!()
        }
    }

    fn tick(&mut self, instant: Instant) -> Command<Self::AppMessage> {
        let elapsed = instant - self.last_tick;
        self.last_tick = instant;
        if elapsed < self.remaining_time {
            self.remaining_time -= elapsed;
            Command::none()
        } else {
            Command::perform(async {}, |_| Message::RunSession(MessageRunSession::NextImage))
        }
    }
}

impl WorkflowRunSession {
    pub fn new(session_configuration: &SessionConfiguration, session_prepared: &StatePreparedSession) -> Self {
        let remaining_time = match session_configuration.image_time {
            ImageTime::FixedTime(duration) => duration,
            ImageTime::NoLimit => Duration::from_secs(3600)
        };
        Self {
            image_path_history: Vec::new(),
            next_image_index: ImageIndex::PathIndex(0),
            loaded_image_bytes: None,
            loaded_image_path_index: None,
            is_running: true,
            remaining_time,
            last_tick: Instant::now(),
            image_paths: session_prepared.valid_images.clone(),
        }
    }

    async fn load_image_at(index: usize, paths: Vec<PathBuf>) -> io::Result<Option<Vec<u8>>> {
        if paths.is_empty() {
            return Ok(None);
        }
        let safe_index = index % (paths.len() as usize);
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
    ShowImage(usize, Vec<u8>),
}

impl Into<Message> for MessageRunSession {
    fn into(self) -> Message {
        Message::RunSession(self)
    }
}
