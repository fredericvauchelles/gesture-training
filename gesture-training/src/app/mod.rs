use std::cell::RefCell;
use std::rc::Rc;
use std::sync::atomic::{AtomicIsize, Ordering};
use crate::app::app_ui::AppUi;

mod app_impl;
mod app_ui;
mod backend;
mod image_source;

pub struct App {
    edit_source_folder_request_ask_path: AtomicIsize,
}

impl App {
    pub fn new() -> Self {
        Self {
            edit_source_folder_request_ask_path: AtomicIsize::new(0)
        }
    }

    pub fn run() -> Result<(), slint::PlatformError> {
        let app_ui = AppUi::new()?;
        let app_backend = Rc::new(RefCell::new(backend::AppBackend::new()));
        let app = Rc::new(App::new());
        App::bind(&app, &app_ui, &app_backend)?;
        app_ui.run()
    }
    
    fn next_edit_source_folder_request_ask_path_id(&self) -> isize {
        self.edit_source_folder_request_ask_path.fetch_add(1, Ordering::AcqRel)
    }
}
