pub fn start_app() {
    slint::slint! {
        export component MainWindow inherits Window {
            Text { text: "Hello World"; }
        }
    }
    MainWindow::new().unwrap().run().unwrap();
}

#[allow(dead_code)]
#[cfg(not(target_os = "android"))]
fn main() {
    start_app()
}

#[cfg(target_os = "android")]
#[no_mangle]
fn android_main(app: slint::android::AndroidApp) {
    slint::android::init(app).unwrap();
    start_app()
}
