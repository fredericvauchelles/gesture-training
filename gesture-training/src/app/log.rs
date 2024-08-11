use std::fmt::Display;

pub struct Log {}

impl Log {
    pub fn handle_error<E: Display>(error: E) {
        eprintln!("{}", error);
    }
}
