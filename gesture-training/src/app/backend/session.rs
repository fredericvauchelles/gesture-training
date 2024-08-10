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

    pub(super) fn add_image_source(&mut self, uuid: Uuid) -> bool {
        self.used_sources.insert(uuid)
    }

    pub(super) fn remove_image_source(&mut self, uuid: Uuid) -> bool {
        self.used_sources.remove(&uuid)
    }
}

