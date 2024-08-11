use uuid::Uuid;

pub use modifications::{AppBackendModifications, ImageSourceModification, SessionModification};
use crate::app::image_source::folder::ImageSourceBackend;
use crate::app::session::SessionBackend;
use crate::sg;

use super::image_source::{ImageSource, ImageSourceTrait};

mod modifications;

pub struct AppBackend {
    image_sources: ImageSourceBackend,
    session: SessionBackend,
}

impl AppBackend {
    pub fn new() -> Self {
        Self {
            image_sources: ImageSourceBackend::new(),
            session: SessionBackend::default(),
        }
    }
    pub fn image_sources(&self) -> &ImageSourceBackend {
        &self.image_sources
    }
    pub fn image_sources_mut(&mut self) -> &mut ImageSourceBackend {
        &mut self.image_sources
    }

    pub fn session(&self) -> &SessionBackend {
        &self.session
    }

    pub fn add_image_source_to_session(&mut self, uuid: Uuid) -> AppBackendModifications {
        if self.image_sources.get_image_source(uuid).is_some()
            && self.session.add_image_source(uuid)
        {
            SessionModification::AddedImageSource(uuid).into()
        } else {
            AppBackendModifications::default()
        }
    }

    pub fn remove_image_source_from_session(&mut self, uuid: Uuid) -> AppBackendModifications {
        if self.session.remove_image_source(uuid) {
            SessionModification::RemovedImageSource(uuid).into()
        } else {
            AppBackendModifications::default()
        }
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
                enabled: self.session().is_image_source_used(image_source.id()),
                status: image_source.check().status().into(),
            })
    }
    
    pub fn used_image_source<'a>(&'a self) -> impl IntoIterator<Item=ImageSource> + 'a {
        self.session().image_source_used().into_iter().filter_map(|uuid| self.image_sources().get_image_source(*uuid)).cloned()
    }
}
