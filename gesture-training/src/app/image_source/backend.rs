use std::collections::HashMap;
use std::path::PathBuf;
use std::str::FromStr;

use uuid::Uuid;

use crate::app::backend::{AppBackendModifications, ImageSourceModification, AppPersistentState};
use crate::app::image_source::{ImageSource, ImageSourceCheck, ImageSourceTrait};
use crate::app::image_source::folder::ImageSourceFolder;
use crate::sg;

pub struct ImageSourceBackend {
    image_sources: HashMap<Uuid, ImageSource>,
}

impl ImageSourceBackend {
    pub fn new() -> Self {
        Self {
            image_sources: HashMap::new(),
        }
    }

    pub fn update_from_state(&mut self, state: &AppPersistentState) -> anyhow::Result<AppBackendModifications> {
        let deletes = self.image_sources.keys().cloned().map(ImageSourceModification::Deleted).collect::<Vec<_>>();
        
        self.image_sources = state.image_sources.iter().cloned().map(|source| (source.id(), source)).collect();

        let adds = self.image_sources.keys().cloned().map(ImageSourceModification::Added);
        Ok(deletes.into_iter().chain(adds).into())
    }

    pub fn get_image_source(&self, id: Uuid) -> Option<&ImageSource> {
        self.image_sources.get(&id)
    }

    pub fn get_image_source_mut(&mut self, id: Uuid) -> Option<&mut ImageSource> {
        self.image_sources.get_mut(&id)
    }

    pub fn add_image_source(&mut self, image_source: impl Into<ImageSource>) {
        let image_source = image_source.into();
        self.image_sources.insert(image_source.id(), image_source);
    }

    pub fn remove_image_source(&mut self, uuid: Uuid) -> Option<ImageSource> {
        self.image_sources.remove(&uuid)
    }

    pub fn image_sources<'a>(&'a self) -> impl IntoIterator<Item = &'a ImageSource> {
        self.image_sources.values()
    }

    pub fn add_or_update_image_source_from_edit_folder(
        &mut self,
        data: &sg::EditSourceFolderData,
        path: Option<PathBuf>,
    ) -> Result<AppBackendModifications, anyhow::Error> {
        let mut data = data.clone();
        let id = Uuid::from_str(&data.id).unwrap_or_else(|_| Uuid::new_v4());
        data.id = id.to_string().into();

        // Update backend
        Ok(match self.get_image_source_mut(id) {
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
                    ImageSourceCheck::default(),
                ));
                self.add_image_source(image_source.clone());
                ImageSourceModification::Added(image_source.id()).into()
            }
        })
    }
}