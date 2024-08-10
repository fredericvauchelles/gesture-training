use std::sync::atomic::{AtomicIsize, Ordering};

pub struct AppSourceFolder {
    request_ask_path_id: AtomicIsize,
}

impl AppSourceFolder {
    pub fn new() -> Self {
        Self {
            request_ask_path_id: AtomicIsize::new(0)
        }
    }

    pub fn next_request_ask_path_id(&self) -> isize {
        self.request_ask_path_id.fetch_add(1, Ordering::AcqRel)
    }
}