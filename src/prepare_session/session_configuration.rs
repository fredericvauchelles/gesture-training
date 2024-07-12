use std::path::PathBuf;

#[derive(Default, Clone, Debug)]
pub struct ImageSelection {
    pub folder_path: PathBuf,
}

#[derive(Clone, Debug)]
pub enum ImageTime {
    FixedTime {
        seconds: u16
    },
    NoLimit,
}
impl Default for ImageTime {
    fn default() -> Self {
        Self::FixedTime { seconds: 30 }
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