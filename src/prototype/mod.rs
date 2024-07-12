use std::cmp::max;
use std::ffi::OsStr;
use std::fs::{DirEntry, FileType};
use std::io;
use std::io::Error;
use std::path::{Path, PathBuf};
use iced::{Alignment, Application, Command, Element, executor, Theme};
use iced::command::channel;
use iced::futures::TryStreamExt;
use iced::theme::Radio;
use iced::widget::{button, text, column, image, text_input, radio, Row, row};
use rfd::AsyncFileDialog;
use crate::prototype::ApplicationWorkflow::PrepareSession;

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

struct ApplicationStateRunSession {
    current_image_index: u8,
    available_images: Vec<PathBuf>,
    is_running: bool,
    remaining_seconds: u16,
}
impl ApplicationStateRunSession {
    fn new(session_configuration: &SessionConfiguration, available_images: Vec<PathBuf>) -> Self {
        let remaining_seconds = match session_configuration.image_time {
            ImageTime::FixedTime { seconds } => seconds,
            ImageTime::NoLimit => 120
        };
        Self {
            current_image_index: 0,
            available_images,
            is_running: true,
            remaining_seconds,
        }
    }
}
enum ApplicationWorkflow {
    PrepareSession,
    RunSession(ApplicationStateRunSession),
}

impl Default for ApplicationWorkflow {
    fn default() -> Self {
        ApplicationWorkflow::PrepareSession
    }
}

#[derive(Default)]
pub struct ApplicationState {
    session_configuration: SessionConfiguration,
    current_workflow: ApplicationWorkflow,
}

#[derive(Debug, Clone)]
enum ApplicationMessagePrepareSession {
    None,
    StartSession,
    InitAndRunSession {
        image_paths: Vec<PathBuf>
    },
    SetImageCount(u8),
    SetImageTime(u16),
    SelectImageFolder,
    SetImageFolder(Option<PathBuf>),
}
impl Into<ApplicationMessage> for ApplicationMessagePrepareSession {
    fn into(self) -> ApplicationMessage {
        ApplicationMessage::PrepareSession(self)
    }
}

#[derive(Debug, Clone)]
enum ApplicationMessageRunSession {
    Pause,
    Resume,
    Stop,
    NextImage,
    PreviousImage,
}

impl Into<ApplicationMessage> for ApplicationMessageRunSession {
    fn into(self) -> ApplicationMessage {
        ApplicationMessage::RunSession(self)
    }
}

#[derive(Debug, Clone)]
pub enum ApplicationMessage {
    PrepareSession(ApplicationMessagePrepareSession),
    RunSession(ApplicationMessageRunSession),
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
            ApplicationMessage::PrepareSession(message) => self.update_prepare_session(message),
            ApplicationMessage::RunSession(message) => self.update_run_session(message),
        }
    }

    fn view(&self) -> Element<ApplicationMessage> {
        match &self.current_workflow {
            ApplicationWorkflow::PrepareSession => self.view_prepare_session(),
            ApplicationWorkflow::RunSession(run_session) => self.view_run_session(&run_session),
        }
    }
}

impl ApplicationState {
    fn view_prepare_session(&self) -> Element<ApplicationMessage> {
        let text_title = text("Gesture Training");

        let text_folder_input = text("Image folder");
        let text_folder_selected = text(self.session_configuration.image_selection.folder_path.to_string_lossy());
        let button_choose_folder = button("Pick").on_press(ApplicationMessage::PrepareSession(ApplicationMessagePrepareSession::SelectImageFolder));
        let row_folder_selector = row!(text_folder_input, text_folder_selected, button_choose_folder);

        let radio_image_counts = Row::with_children((1..5u8).map(|i| {
            let amount = i * 5;
            radio(amount.to_string(), amount, Some(self.session_configuration.image_count), |value| -> ApplicationMessage { ApplicationMessagePrepareSession::SetImageCount(amount).into() })
        }).map(Element::from));
        let radio_image_time = Row::with_children(([30, 60, 90, 120, 240, 600, 1200]).map(|i| {
            let amount = i;
            let selected = match self.session_configuration.image_time {
                ImageTime::FixedTime { seconds } => Some(seconds),
                ImageTime::NoLimit => None
            };
            radio(amount.to_string(), amount, selected, |value| -> ApplicationMessage { ApplicationMessagePrepareSession::SetImageTime(amount).into() })
        }).map(Element::from));
        let button_start = button("Start").on_press(ApplicationMessage::PrepareSession(ApplicationMessagePrepareSession::StartSession));

        column!(text_title, row_folder_selector, radio_image_counts, radio_image_time, button_start)
            .padding(20)
            .align_items(Alignment::Center)
            .into()
    }

    fn view_run_session(&self, run_session: &ApplicationStateRunSession) -> Element<ApplicationMessage> {
        let text_title = text("Gesture Training");

        let image = image("");

        let button_back = button("<").on_press(ApplicationMessage::RunSession(ApplicationMessageRunSession::PreviousImage));
        let button_stop = button("Stop").on_press(ApplicationMessage::RunSession(ApplicationMessageRunSession::Stop));
        let button_playpause = if run_session.is_running {
            button("Play").on_press(ApplicationMessage::RunSession(ApplicationMessageRunSession::Pause))
        } else {
            button("Pause").on_press(ApplicationMessage::RunSession(ApplicationMessageRunSession::Resume))
        };
        let button_next = button(">").on_press(ApplicationMessage::RunSession(ApplicationMessageRunSession::NextImage));
        let text_timeremaining = text(run_session.remaining_seconds.to_string());
        let row_actionbar = row!(button_back, button_stop, button_playpause, button_next, text_timeremaining);

        column!(text_title, image, row_actionbar)
            .padding(20)
            .align_items(Alignment::Center)
            .into()
    }

    fn update_prepare_session(&mut self, message: ApplicationMessagePrepareSession) -> Command<ApplicationMessage> {
        match message {
            ApplicationMessagePrepareSession::None => Command::none(),
            ApplicationMessagePrepareSession::InitAndRunSession { image_paths } => {
                self.current_workflow = ApplicationWorkflow::RunSession(ApplicationStateRunSession::new(&self.session_configuration, image_paths));
                Command::none()
            }
            ApplicationMessagePrepareSession::StartSession => {
                let future = find_image_files_in_directory(self.session_configuration.image_selection.folder_path.to_path_buf());
                Command::perform(future, |results| {
                    match results {
                        Ok(image_paths) => {
                            ApplicationMessage::PrepareSession(ApplicationMessagePrepareSession::InitAndRunSession { image_paths })
                        }
                        Err(_) => ApplicationMessage::PrepareSession(ApplicationMessagePrepareSession::None)
                    }
                })
            }
            ApplicationMessagePrepareSession::SetImageCount(value) => {
                self.session_configuration.image_count = value;
                Command::none()
            }
            ApplicationMessagePrepareSession::SetImageTime(value) => {
                self.session_configuration.image_time = ImageTime::FixedTime { seconds: value };
                Command::none()
            }
            ApplicationMessagePrepareSession::SelectImageFolder => {
                let future = async {
                    AsyncFileDialog::new()
                        .pick_folder()
                        .await
                };
                Command::perform(future, |file_handle| {
                    ApplicationMessage::PrepareSession(ApplicationMessagePrepareSession::SetImageFolder(file_handle.map(|handle| handle.path().to_path_buf())))
                })
            }
            ApplicationMessagePrepareSession::SetImageFolder(option_path) => {
                self.session_configuration.image_selection.folder_path = option_path.unwrap_or(PathBuf::new());
                Command::none()
            }
        }
    }

    fn update_run_session(&mut self, message: ApplicationMessageRunSession) -> Command<ApplicationMessage> {
        if let ApplicationMessageRunSession::Stop = message {
            self.current_workflow = PrepareSession;
            Command::none()
        } else if let ApplicationWorkflow::RunSession(run_session) = &mut self.current_workflow {
            match message {
                ApplicationMessageRunSession::Pause => {
                    run_session.is_running = false;
                    Command::none()
                }
                ApplicationMessageRunSession::Resume => {
                    run_session.is_running = true;
                    Command::none()
                }
                ApplicationMessageRunSession::Stop => { unreachable!() }
                ApplicationMessageRunSession::NextImage => {
                    let image_count = max(1u8, run_session.available_images.len() as u8);
                    run_session.current_image_index = (run_session.current_image_index + 1) % image_count;
                    Command::none()
                }
                ApplicationMessageRunSession::PreviousImage => {
                    let image_count = max(1u8, run_session.available_images.len() as u8);
                    run_session.current_image_index = (run_session.current_image_index + (image_count - 1)) % image_count;
                    Command::none()
                }
            }
        } else {
            unreachable!()
        }
    }
}


const IMAGE_EXTENSIONS: &[&str] = &["jpg", "jpeg", "png", "bmp"];
fn is_image_file(path: &Path) -> bool {
    if let Some(extension) = path.extension().and_then(|ext| ext.to_str()) {
        IMAGE_EXTENSIONS.contains(&extension)
    } else { false }
}
async fn find_image_files_in_directory(path: PathBuf) -> io::Result<Vec<PathBuf>> {
    match async_fs::read_dir(path).await {
        Ok(mut read_dir) => {
            let mut image_paths = Vec::new();
            loop {
                match read_dir.try_next().await {
                    Ok(Some(entry)) => {
                        if is_image_file(&entry.path()) {
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

#[cfg(test)]
mod test {
    use std::path::Path;
    use super::is_image_file;
    #[test]
    fn test_image_extension() {
        assert!(is_image_file(&Path::new(&"test.jpg")));
    }
}