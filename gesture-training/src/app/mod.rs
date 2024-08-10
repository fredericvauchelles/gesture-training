use std::cell::RefCell;
use std::rc::Rc;

use crate::app::app_ui::AppUi;

mod app_impl;
mod app_ui;
mod backend;
mod image_source;

pub struct App {}

impl App {
    pub fn new() -> Self {
        Self {}
    }

    pub fn run() -> Result<(), slint::PlatformError> {
        let app_ui = AppUi::new()?;
        let app_backend = Rc::new(RefCell::new(backend::AppBackend::new()));
        let app = Rc::new(App::new());
        App::bind(&app, &app_ui, &app_backend)?;
        app_ui.run()
    }
}
