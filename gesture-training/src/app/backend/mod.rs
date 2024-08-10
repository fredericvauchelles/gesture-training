use uuid::Uuid;

pub use image_sources::ImageSourceBackend;
pub use modifications::{AppBackendModifications, ImageSourceModification};

use crate::sg;

use super::image_source::ImageSourceTrait;

mod modifications;

mod image_sources {
    use std::collections::HashMap;
    use std::path::PathBuf;
    use std::str::FromStr;

    use uuid::Uuid;

    use crate::app::backend::{AppBackendModifications, ImageSourceModification};
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
}

pub struct AppBackend {
    image_sources: ImageSourceBackend,
}

impl AppBackend {
    pub fn new() -> Self {
        Self {
            image_sources: ImageSourceBackend::new(),
        }
    }
    pub fn image_sources(&self) -> &ImageSourceBackend {
        &self.image_sources
    }
    pub fn image_sources_mut(&mut self) -> &mut ImageSourceBackend {
        &mut self.image_sources
    }

    pub fn new_image_source_selector_entry_data(
        &self,
        uuid: Uuid,
    ) -> Option<sg::ImageSourceSelectorEntryData> {
        self.image_sources()
            .get_image_source(uuid)
            .map(|image_source| sg::ImageSourceSelectorEntryData {
                id: image_source.id().to_string().into(),
                image_count: image_source.check().image_count() as i32,
                name: image_source.name().into(),
                enabled: false,
                status: image_source.check().status().into(),
            })
    }
}
