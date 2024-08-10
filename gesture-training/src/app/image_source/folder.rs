use std::path::PathBuf;

use slint::SharedString;
use uuid::Uuid;

use crate::sg;

use super::{ImageSource, ImageSourceCheck, ImageSourceTrait};

#[derive(Debug, Clone)]
pub struct ImageSourceFolder {
    id: Uuid,
    pub(crate) name: String,
    pub(crate) path: PathBuf,
    check: ImageSourceCheck,
}

impl ImageSourceFolder {
    pub fn new(id: Uuid, name: String, path: PathBuf, check: ImageSourceCheck) -> Self {
        Self { 
            id, 
            name, 
            path,
            check
        }
    }
}

impl ImageSourceTrait for ImageSourceFolder {
    fn id(&self) -> Uuid {
        self.id
    }
    fn name(&self) -> &str {
        &self.name
    }

    fn check(&self) -> &ImageSourceCheck {
        &self.check
    }

    fn set_check(&mut self, check: ImageSourceCheck) {
        self.check = check;
    }

    async fn check_source(&self) -> anyhow::Result<ImageSourceCheck> {
        todo!()
    }
}

impl<'a> From<&'a ImageSourceFolder> for sg::EditSourceFolderData {
    fn from(value: &'a ImageSourceFolder) -> Self {
        Self {
            id: value.id.to_string().into(),
            name: value.name.clone().into(),
            image_count: 0,
            path: value.path.to_string_lossy().to_string().into(),
            status: sg::StatusIconData {
                r#type: sg::StatusIconType::Unknown,
                error: SharedString::default(),
            },
        }
    }
}

impl From<ImageSourceFolder> for sg::EditSourceFolderData {
    fn from(value: ImageSourceFolder) -> Self {
        Self {
            id: value.id.to_string().into(),
            name: value.name.into(),
            image_count: 0,
            path: value.path.to_string_lossy().to_string().into(),
            status: sg::StatusIconData {
                r#type: sg::StatusIconType::Unknown,
                error: SharedString::default(),
            },
        }
    }
}

impl TryFrom<super::ImageSource> for sg::EditSourceFolderData {
    type Error = anyhow::Error;
    fn try_from(value: ImageSource) -> Result<Self, Self::Error> {
        let ImageSource::Folder(folder) = value;
        Ok(folder.into())
    }
}
