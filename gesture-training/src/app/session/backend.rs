use std::collections::HashSet;
use uuid::Uuid;

#[derive(Default)]
pub struct SessionBackend {
    used_sources: HashSet<Uuid>
}

impl SessionBackend {
    pub fn is_image_source_used(&self, uuid: Uuid) -> bool {
        self.used_sources.contains(&uuid)
    }
    
    pub fn image_source_used(&self) -> impl IntoIterator<Item = &Uuid> {
        self.used_sources.iter()
    }

    pub(crate) fn add_image_source(&mut self, uuid: Uuid) -> bool {
        self.used_sources.insert(uuid)
    }

    pub(crate) fn remove_image_source(&mut self, uuid: Uuid) -> bool {
        self.used_sources.remove(&uuid)
    }
}

