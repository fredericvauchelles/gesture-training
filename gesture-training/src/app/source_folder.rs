use std::path::PathBuf;
use std::sync::atomic::{AtomicIsize, Ordering};

pub struct AppSourceFolder {
    request_ask_path_id: AtomicIsize,
    currently_edited_path: Option<PathBuf>,
}

impl AppSourceFolder {
    pub fn new() -> Self {
        Self {
            request_ask_path_id: AtomicIsize::new(0),
            currently_edited_path: None,
        }
    }

    pub fn next_request_ask_path_id(&self) -> isize {
        self.request_ask_path_id.fetch_add(1, Ordering::AcqRel)
    }

    pub fn set_edited_path(&mut self, path: impl Into<PathBuf>) {
        self.currently_edited_path = Some(path.into());
    }

    pub fn clear_edited_path(&mut self) {
        self.currently_edited_path = None;
    }

    pub fn edited_path(&self) -> Option<&PathBuf> {
        self.currently_edited_path.as_ref()
    }
}
