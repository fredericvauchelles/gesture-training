fn main() {
    let config = slint_build::CompilerConfiguration::new().with_style("cosmic-dark".into());

    slint_build::compile_with_config("ui/pages/appwindow.slint", config).unwrap();
    //slint_build::compile("ui/pages/session.slint").unwrap();
}
