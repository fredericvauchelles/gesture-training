use std::cmp::min;
use std::io;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use iced::{Alignment, Command, Element, Length, Padding};
use iced::widget::{button, image, Image, row, Space, text};
use rand::random;

use crate::app::{AppWorkflow, Message, State, Workflow};
use crate::col;
use crate::prepare_session::{ImageTime, SessionConfiguration};
use crate::prepare_session::{StatePreparedSession, WorkflowPrepareSession};

#[derive(Debug, Clone)]
pub struct ImageInMemory {
    bytes: Option<Vec<u8>>,
    path: PathBuf,
}

impl ImageInMemory {
    async fn get_or_load_image(&mut self) -> Option<&[u8]> {
        if self.bytes.is_some() {
            return Some(&self.bytes.as_ref().unwrap());
        }

        match async_fs::read(&self.path).await {
            Ok(bytes) => {
                self.bytes = Some(bytes);
                Some(&self.bytes.as_ref().unwrap())
            }
            Err(error) => {
                eprintln!("{}", error);
                None
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct WorkflowRunSession {
    image_index: usize,
    is_running: bool,
    remaining_time: Duration,
    duration: Duration,
    last_tick: Instant,
    images: Vec<ImageInMemory>,
}

impl AppWorkflow for WorkflowRunSession {
    type WorkflowMessage = MessageRunSession;
    type AppMessage = Message;
    fn update(state: &mut State, message: Self::WorkflowMessage) -> Command<Self::AppMessage> {
        match message {
            MessageRunSession::Initialize(workflow) => {
                state.current_workflow = Workflow::RunSession(workflow);
                Command::perform(async {}, |_| Message::RunSession(MessageRunSession::Play))
            }
            MessageRunSession::Stop => {
                state.current_workflow =
                    Workflow::PrepareSession(WorkflowPrepareSession::default());
                Command::none()
            }
            _ => {
                if let Workflow::RunSession(run_session) = &mut state.current_workflow {
                    match message {
                        MessageRunSession::Pause => {
                            run_session.is_running = false;
                            Command::none()
                        }
                        MessageRunSession::Play => {
                            run_session.is_running = false;
                            Command::none()
                        }
                        MessageRunSession::ShowImage(image_index, bytes) => {
                            run_session.image_index = image_index;
                            run_session.is_running = true;
                            run_session.remaining_time = run_session.duration;
                            run_session.last_tick = Instant::now();

                            let mut image = &mut run_session.images[run_session.image_index];

                            if image.bytes.is_none() {
                                if let Some(bytes) = bytes {
                                    image.bytes = Some(bytes);
                                    Command::none()
                                } else {
                                    run_session.is_running = false;
                                    Command::perform(
                                        async_fs::read(image.path.to_path_buf()),
                                        move |bytes| {
                                            if let Ok(bytes) = bytes {
                                                Message::RunSession(MessageRunSession::ShowImage(
                                                    image_index,
                                                    Some(bytes),
                                                ))
                                            } else {
                                                Message::None
                                            }
                                        },
                                    )
                                }
                            } else {
                                Command::none()
                            }
                        }
                        MessageRunSession::NextImage => {
                            let next_index =
                                min(run_session.image_index + 1, run_session.images.len() - 1);
                            Command::perform(async {}, move |_| {
                                Message::RunSession(MessageRunSession::ShowImage(next_index, None))
                            })
                        }
                        MessageRunSession::PreviousImage => {
                            let next_index = if run_session.image_index > 0 {
                                run_session.image_index - 1
                            } else {
                                0
                            };
                            Command::perform(async {}, move |_| {
                                Message::RunSession(MessageRunSession::ShowImage(next_index, None))
                            })
                        }

                        MessageRunSession::Stop => {
                            unreachable!()
                        }
                        MessageRunSession::Initialize(_) => {
                            unreachable!()
                        }
                    }
                } else {
                    unreachable!()
                }
            }
        }
    }
    fn view(&self, _state: &State) -> Element<Self::AppMessage> {
        let image = if self.image_index < self.images.len() {
            match self.images[self.image_index].bytes.clone() {
                None => image(""),
                Some(bytes) => image(image::Handle::from_memory(bytes)),
            }
        } else {
            image("")
        };

        let button_back =
            button(Image::<image::Handle>::new("resources/icons-skip-to-start-90.png").width(30))
                .on_press(Message::RunSession(MessageRunSession::PreviousImage));
        let button_stop =
            button(Image::<image::Handle>::new("resources/icons-stop-90.png").width(30))
                .on_press(Message::RunSession(MessageRunSession::Stop));
        let button_playpause = if self.is_running {
            button(Image::<image::Handle>::new("resources/icons-pause-90.png").width(30))
                .on_press(Message::RunSession(MessageRunSession::Pause))
        } else {
            button(Image::<image::Handle>::new("resources/icons-play-90.png").width(30))
                .on_press(Message::RunSession(MessageRunSession::Play))
        };
        let button_next =
            button(Image::<image::Handle>::new("resources/icons-end-90.png").width(30))
                .on_press(Message::RunSession(MessageRunSession::NextImage));
        let text_timeremaining =
            text(format!("{}s", self.remaining_time.as_secs())).width(Length::Fixed(50.0));
        let space = Space::new(Length::Fill, Length::Shrink);
        let space2 = Space::new(Length::Fill, Length::Shrink);
        let row_actionbar = row!(
            space,
            button_back,
            button_stop,
            button_playpause,
            button_next,
            space2,
            text_timeremaining
        )
        .width(Length::Fill)
        .spacing(5)
        .align_items(Alignment::Center)
        .padding(Padding::from([0, 10, 10, 10 + 50]));

        col!(image, row_actionbar)
            .align_items(Alignment::Center)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    fn tick(&mut self, instant: Instant) -> Command<Self::AppMessage> {
        let elapsed = instant - self.last_tick;
        self.last_tick = instant;
        if !self.is_running {
            return Command::none();
        }

        if elapsed < self.remaining_time {
            self.remaining_time -= elapsed;
            Command::none()
        } else {
            Command::perform(async {}, |_| {
                Message::RunSession(MessageRunSession::NextImage)
            })
        }
    }
}

impl WorkflowRunSession {
    pub async fn new(
        session_configuration: SessionConfiguration,
        session_prepared: StatePreparedSession,
    ) -> io::Result<Self> {
        let remaining_time = match session_configuration.image_time {
            ImageTime::FixedTime(duration) => duration,
            ImageTime::NoLimit => Duration::from_secs(3600),
        };

        // predetermine all images for the session
        let images = (0..session_configuration.image_count)
            .into_iter()
            .map(|_| random::<usize>() % session_prepared.valid_images.len())
            .map(|index| session_prepared.valid_images[index].clone())
            .map(|path| ImageInMemory { path, bytes: None })
            .collect::<Vec<_>>();

        Ok(Self {
            is_running: true,
            remaining_time,
            duration: remaining_time,
            last_tick: Instant::now(),
            image_index: 0,
            images,
        })
    }
}

#[derive(Debug, Clone)]
pub enum MessageRunSession {
    Initialize(WorkflowRunSession),
    Pause,
    Play,
    Stop,
    NextImage,
    PreviousImage,
    ShowImage(usize, Option<Vec<u8>>),
}

impl Into<Message> for MessageRunSession {
    fn into(self) -> Message {
        Message::RunSession(self)
    }
}
