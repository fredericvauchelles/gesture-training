use uuid::Uuid;

#[derive(Debug, Clone, Copy)]
pub enum ImageSourceModification {
    Added(Uuid),
    Modified(Uuid),
    Deleted(Uuid),
}

impl ImageSourceModification {
    pub(crate) fn id(&self) -> Uuid {
        match self {
            ImageSourceModification::Added(id) => *id,
            ImageSourceModification::Modified(id) => *id,
            ImageSourceModification::Deleted(id) => *id,
        }
    }
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