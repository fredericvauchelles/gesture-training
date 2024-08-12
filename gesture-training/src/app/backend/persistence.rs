use serde::{Serialize, Deserialize};
use std::path::PathBuf;

use crate::app::image_source::ImageSource;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppPersistentState {
    pub image_sources: Vec<ImageSource>,
}

pub struct AppPersistence {}

impl AppPersistence {
    pub fn load_state() -> anyhow::Result<Option<AppPersistentState>> {
        let path = Self::state_file();

        if path.exists() && path.is_file() {
            let content = std::fs::read_to_string(&path)?;
            let state: AppPersistentState = serde_yaml::from_str(&content)?;
            Ok(Some(state))
        }
        else {
            Ok(None)
        }
    }

    pub fn save_state<'a>(image_sources: impl IntoIterator<Item=&'a ImageSource>) -> anyhow::Result<()> {
        let path = Self::state_file();

        let state = AppPersistentState {
            image_sources: image_sources.into_iter().cloned().collect(),
        };

        let serialized = serde_yaml::to_string(&state)?;

        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(&parent)?;
        }

        std::fs::write(&path, &serialized)?;

        Ok(())
    }

    fn state_file() -> PathBuf {
        let mut path = dirs::data_local_dir().unwrap_or_else(|| "~/.local/share".into());
        path.push("GestureTraining");
        path.push("state.yml");
        path
    }
}
