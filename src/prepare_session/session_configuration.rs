use std::path::PathBuf;
use std::time::Duration;

#[derive(Default, Clone, Debug)]
pub struct ImageSelection {
    pub folder_path: PathBuf,
}

#[derive(Clone, Debug)]
pub enum ImageTime {
    FixedTime(Duration),
    NoLimit,
}
impl Default for ImageTime {
    fn default() -> Self {
        Self::FixedTime(Duration::from_secs(30))
    }
}

#[derive(Clone, Debug)]
pub struct SessionConfiguration {
    pub image_selection: ImageSelection,
    pub image_count: u8,
    pub image_time: ImageTime,
}
impl Default for SessionConfiguration {
    fn default() -> Self {
        Self {
            image_selection: ImageSelection::default(),
            image_count: 5,
            image_time: ImageTime::default(),
        }
    }
}