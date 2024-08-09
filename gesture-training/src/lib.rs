use slint_includes::*;

use crate::app_data::AppData;

mod app_data;
mod slint_includes;

pub fn start_app() -> Result<(), slint::PlatformError> {
    let app_window = AppWindow::new()?;

    let app_data = AppData::new();
    AppData::bind_window(&app_data, &app_window);

    app_window.run()
}

#[allow(dead_code)]
fn main() -> Result<(), slint::PlatformError> {
    start_app()
}

#[cfg(target_os = "android")]
#[no_mangle]
fn android_main(app: slint::android::AndroidApp) {
    slint::android::init(app).unwrap();
    start_app().unwrap()
}
