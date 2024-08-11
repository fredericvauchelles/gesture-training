use slint::Timer;

pub struct AppSession {
    pub session_timer: Timer,
}

impl AppSession {
    pub fn new() -> Self {
        Self {
            session_timer: Timer::default(),
        }
    }
}
