slint::include_modules!();

pub fn start_app() -> Result<(), slint::PlatformError> {
    let ui = AppWindow::new()?;

    // ui.on_request_increase_value({
    //     let ui_handle = ui.as_weak();
    //     move || {
    //         let ui = ui_handle.unwrap();
    //         ui.set_counter(ui.get_counter() + 1);
    //     }
    // });

    ui.run()
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
