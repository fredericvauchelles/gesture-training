use std::cell::RefCell;
use std::rc::Rc;

use crate::app::app_ui::AppUi;
use crate::app::image_source::folder::AppSourceFolder;
use crate::app::session::AppSession;

mod app_callback;
mod app_ui;
mod backend;
mod image_source;
mod log;
mod session;
#[cfg(target_os = "android")]
pub mod android_support;

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

    pub fn run() -> anyhow::Result<()> {
        let mut app_ui = AppUi::new()?;
        let app_backend = Rc::new(RefCell::new(backend::AppBackend::new()));
        let app = Rc::new(RefCell::new(App::new()));
        App::initialize(&app, &mut app_ui, &app_backend)?;
        App::bind(&app, &app_ui, &app_backend)?;
        
        Ok(app_ui.run()?)
    }

    fn source_folder(&self) -> &AppSourceFolder {
        &self.source_folder
    }

    fn source_folder_mut(&mut self) -> &mut AppSourceFolder {
        &mut self.source_folder
    }
}
