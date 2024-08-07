#[no_mangle]
#[cfg(target_os = "android")]
fn android_main(app: slint::android::AndroidApp) {
    slint::android::init(app).unwrap();

    // ... rest of your code ...
    slint::slint! {
        export component MainWindow inherits Window {
            Text { text: "Hello World"; }
        }
    }
    MainWindow::new().unwrap().run().unwrap();
}
