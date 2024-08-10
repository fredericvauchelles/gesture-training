use std::cell::RefCell;
use std::path::PathBuf;
use std::sync::atomic::{AtomicIsize, Ordering};

pub struct AppSourceFolder {
    request_ask_path_id: AtomicIsize,
    currently_edited_path: RefCell<Option<PathBuf>>
}

impl AppSourceFolder {
    pub fn new() -> Self {
        Self {
            request_ask_path_id: AtomicIsize::new(0),
            currently_edited_path: RefCell::new(None)
        }
    }

    pub fn next_request_ask_path_id(&self) -> isize {
        self.request_ask_path_id.fetch_add(1, Ordering::AcqRel)
    }

    pub fn set_edited_path(&self, path: impl Into<PathBuf>) -> anyhow::Result<()> {
        *(self.currently_edited_path.try_borrow_mut()?) = Some(path.into());
        Ok(())
    }

    pub fn clear_edited_path(&self) -> anyhow::Result<()> {
        *(self.currently_edited_path.try_borrow_mut()?) = None;
        Ok(())
    }

    pub fn edited_path(&self) -> anyhow::Result<Option<PathBuf>> {
        self.currently_edited_path.try_borrow().map(|path| path.clone()).map_err(anyhow::Error::from)
    }
}