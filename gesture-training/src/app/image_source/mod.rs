pub mod folder;

use slint::SharedString;
use uuid::Uuid;

use folder::ImageSourceFolder;

use crate::sg;

pub trait ImageSourceTrait {
    fn id(&self) -> &Uuid;
    fn name(&self) -> &str;
}

#[derive(Debug, Clone)]
pub enum ImageSource {
    Folder(ImageSourceFolder),
}

impl<'a> From<&'a ImageSource> for sg::ImageSourceSelectorEntryData {
    fn from(value: &'a ImageSource) -> Self {
        Self {
            id: value.id().to_string().into(),
            name: value.name().into(),
            image_count: 0,
            status: sg::StatusIconData {
                r#type: sg::StatusIconType::Unknown,
                error: SharedString::default(),
            },
            enabled: false,
        }
    }
}

impl ImageSourceTrait for ImageSource {
    fn id(&self) -> &Uuid {
        match self {
            ImageSource::Folder(value) => value.id(),
        }
    }

    fn name(&self) -> &str {
        match self {
            ImageSource::Folder(value) => value.name(),
        }
    }
}

impl ImageSource {
    pub(crate) fn update_image_source_selector_entry(
        &self,
        target: &mut sg::ImageSourceSelectorEntryData,
    ) {
        target.name = self.name().to_string().into();
    }
}