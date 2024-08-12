use crate::app::App;

mod app;
mod sg;

pub fn start_app() -> anyhow::Result<()> {
    App::run()
}

#[allow(dead_code)]
fn main() -> anyhow::Result<()> {
    start_app()
}

#[cfg(target_os = "android")]
#[no_mangle]
fn android_main(app: slint::android::AndroidApp) {
    slint::android::init(app).unwrap();
    start_app().unwrap()
}
