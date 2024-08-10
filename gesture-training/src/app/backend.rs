use std::collections::HashMap;
use std::path::PathBuf;
use std::str::FromStr;

use uuid::Uuid;

use crate::app::image_source::folder::ImageSourceFolder;
use crate::sg;

use super::image_source::{ImageSource, ImageSourceTrait};

#[derive(Debug)]
pub enum ImageSourceModification {
    Added(Uuid),
    Modified(Uuid),
    Deleted(Uuid),
}

#[derive(Debug)]
pub struct AppBackendModifications {
    image_sources: Vec<ImageSourceModification>,
}

impl AppBackendModifications {
    pub fn image_sources(&self) -> &[ImageSourceModification] {
        &self.image_sources
    }
}

impl From<ImageSourceModification> for AppBackendModifications {
    fn from(value: ImageSourceModification) -> Self {
        Self {
            image_sources: vec![value],
        }
    }
}

pub struct AppBackend {
    image_sources: HashMap<Uuid, ImageSource>,
}

impl AppBackend {
    pub fn new() -> Self {
        Self {
            image_sources: HashMap::new(),
        }
    }

    pub fn get_image_source(&self, id: &Uuid) -> Option<&ImageSource> {
        self.image_sources.get(id)
    }

    pub fn get_image_source_mut(&mut self, id: &Uuid) -> Option<&mut ImageSource> {
        self.image_sources.get_mut(id)
    }

    pub fn add_image_source(&mut self, image_source: impl Into<ImageSource>) {
        let image_source = image_source.into();
        self.image_sources.insert(*image_source.id(), image_source);
    }

    pub fn remove_image_source(&mut self, uuid: &Uuid) -> Option<ImageSource> {
        self.image_sources.remove(uuid)
    }

    pub fn add_or_update_image_source_from_edit_folder(
        &mut self,
        data: &sg::EditSourceFolderData,
        path: Option<PathBuf>
    ) -> Result<AppBackendModifications, anyhow::Error> {
        let mut data = data.clone();
        let id = Uuid::from_str(&data.id).unwrap_or_else(|_| Uuid::new_v4());
        data.id = id.to_string().into();

        // Update backend
        Ok(match self.get_image_source_mut(&id) {
            // Update image source
            Some(ImageSource::Folder(folder)) => {
                folder.name = data.name.to_string();
                if let Some(path) = path {
                    folder.path = path;
                }
                ImageSourceModification::Modified(id).into()
            }
            None => {
                let image_source = ImageSource::Folder(ImageSourceFolder::new(
                    id,
                    data.name.to_string(),
                    path.unwrap_or_else(|| data.path.to_string().into()),
                ));
                self.add_image_source(image_source.clone());
                ImageSourceModification::Added(*image_source.id()).into()
            }
        })
    }
}
