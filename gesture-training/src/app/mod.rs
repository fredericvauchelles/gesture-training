use std::cell::RefCell;
use std::rc::Rc;

use crate::app::app_ui::AppUi;
use crate::app::source_folder::AppSourceFolder;

mod app_impl;
mod app_ui;
mod backend;
mod image_source;
pub mod source_folder;

pub struct App {
    source_folder: AppSourceFolder,
}

impl App {
    pub fn new() -> Self {
        Self {
            source_folder: AppSourceFolder::new(),
        }
    }

    pub fn run() -> Result<(), slint::PlatformError> {
        let app_ui = AppUi::new()?;
        let app_backend = Rc::new(RefCell::new(backend::AppBackend::new()));
        let app = Rc::new(App::new());
        App::bind(&app, &app_ui, &app_backend)?;
        app_ui.run()
    }

    fn source_folder(&self) -> &AppSourceFolder {
        &self.source_folder
    }
}
