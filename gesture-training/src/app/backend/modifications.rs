use uuid::Uuid;
use crate::sg;

/// Describe modifications that occurred
///
/// Send this to the AppUi to update the UI accordingly
#[derive(Debug, Default)]
pub struct AppBackendModifications {
    image_sources: Vec<ImageSourceModification>,
    session: Vec<SessionModification>,
}
impl AppBackendModifications {
    pub fn image_sources(&self) -> &[ImageSourceModification] {
        &self.image_sources
    }
    pub fn session(&self) -> &[SessionModification] {
        &self.session
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.image_sources.is_empty() && self.session.is_empty()
    }
}


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

impl From<ImageSourceModification> for AppBackendModifications {
    fn from(value: ImageSourceModification) -> Self {
        Self {
            image_sources: vec![value],
            session: Vec::new(),
        }
    }
}

impl<T: IntoIterator<Item=ImageSourceModification>> From<T> for AppBackendModifications {
    fn from(value: T) -> Self {
        Self {
            image_sources: value.into_iter().collect(),
            session: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum SessionModification {
    AddedImageSource(Uuid),
    RemovedImageSource(Uuid),
    State(sg::SessionWindowState)
}

impl From<SessionModification> for AppBackendModifications {
    fn from(value: SessionModification) -> Self {
        Self {
            image_sources: Vec::new(),
            session: vec![value],
        }
    }
}