use slint::SharedString;
use uuid::Uuid;

use folder::ImageSourceFolder;

use crate::sg;

pub mod folder;

#[derive(Debug, Clone)]
pub enum ImageSourceStatus {
    Unknown,
    Valid,
    Error(String),
}

impl Default for ImageSourceStatus {
    fn default() -> Self {
        Self::Unknown
    }
}

#[derive(Debug, Clone, Default)]
pub struct ImageSourceCheck {
    image_count: usize,
    status: ImageSourceStatus,
}

impl ImageSourceCheck {
    pub fn image_count(&self) -> usize {
        self.image_count
    }
    pub fn status(&self) -> &ImageSourceStatus {
        &self.status
    }
}

impl From<ImageSourceStatus> for sg::StatusIconData {
    fn from(value: ImageSourceStatus) -> Self {
        match value {
            ImageSourceStatus::Unknown => Self {
                r#type: sg::StatusIconType::Unknown,
                error: SharedString::default(),
            },
            ImageSourceStatus::Valid => Self {
                r#type: sg::StatusIconType::Valid,
                error: SharedString::default(),
            },
            ImageSourceStatus::Error(error) => Self {
                r#type: sg::StatusIconType::Error,
                error: error.to_string().into(),
            },
        }
    }
}
impl<'a> From<&'a ImageSourceStatus> for sg::StatusIconData {
    fn from(value: &'a ImageSourceStatus) -> Self {
        match value {
            ImageSourceStatus::Unknown => Self {
                r#type: sg::StatusIconType::Unknown,
                error: SharedString::default(),
            },
            ImageSourceStatus::Valid => Self {
                r#type: sg::StatusIconType::Valid,
                error: SharedString::default(),
            },
            ImageSourceStatus::Error(error) => Self {
                r#type: sg::StatusIconType::Error,
                error: error.to_string().into(),
            },
        }
    }
}

pub trait ImageSourceTrait {
    fn id(&self) -> Uuid;
    fn name(&self) -> &str;
    fn check(&self) -> &ImageSourceCheck;
    fn set_check(&mut self, check: ImageSourceCheck);
    async fn check_source(&self) -> ImageSourceCheck;
}

#[derive(Debug, Clone)]
pub enum ImageSource {
    Folder(ImageSourceFolder),
}

impl ImageSourceTrait for ImageSource {
    fn id(&self) -> Uuid {
        match self {
            ImageSource::Folder(value) => value.id(),
        }
    }

    fn name(&self) -> &str {
        match self {
            ImageSource::Folder(value) => value.name(),
        }
    }

    fn check(&self) -> &ImageSourceCheck {
        match self {
            ImageSource::Folder(value) => value.check(),
        }
    }

    fn set_check(&mut self, check: ImageSourceCheck) {
        match self {
            ImageSource::Folder(value) => value.set_check(check),
        }
    }

    async fn check_source(&self) -> ImageSourceCheck {
        match self {
            ImageSource::Folder(value) => value.check_source().await,
        }
    }
}

impl ImageSource {
    pub(crate) fn update_image_source_selector_entry(
        &self,
        target: &mut sg::ImageSourceSelectorEntryData,
    ) {
        target.name = self.name().to_string().into();
        target.image_count = self.check().image_count() as i32;
        target.status = self.check().status().into();
    }
}
