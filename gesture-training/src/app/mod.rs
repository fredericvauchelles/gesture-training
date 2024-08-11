use std::cell::RefCell;
use std::rc::Rc;

use crate::app::app_ui::AppUi;
use crate::app::session::AppSession;
use crate::app::source_folder::AppSourceFolder;

mod app_impl;
mod app_ui;
mod backend;
mod image_source;
mod session;
pub mod source_folder;

pub struct App {
    source_folder: AppSourceFolder,
    session: AppSession,
}

impl App {
    pub fn new() -> Self {
        Self {
            source_folder: AppSourceFolder::new(),
            session: AppSession::new(),
        }
    }

    pub fn run() -> Result<(), slint::PlatformError> {
        let app_ui = AppUi::new()?;
        let app_backend = Rc::new(RefCell::new(backend::AppBackend::new()));
        let app = Rc::new(RefCell::new(App::new()));
        App::initialize(&app, &app_ui, &app_backend)?;
        App::bind(&app, &app_ui, &app_backend)?;
        app_ui.run()
    }

    fn source_folder(&self) -> &AppSourceFolder {
        &self.source_folder
    }

    fn source_folder_mut(&mut self) -> &mut AppSourceFolder {
        &mut self.source_folder
    }
}
