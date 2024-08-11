use std::collections::HashSet;
use std::rc::{Rc, Weak};

use slint::{ComponentHandle, Model, VecModel};

use crate::app::backend::{
    AppBackend, AppBackendModifications, ImageSourceModification, SessionModification,
};
use crate::app::image_source::{
    ImageSource, ImageSourceCheck, ImageSourceStatus, ImageSourceTrait,
};
use crate::sg;

#[derive(Clone)]
pub(crate) struct AppUiBackend {
    image_source_selector_entries: Rc<VecModel<sg::ImageSourceSelectorEntryData>>,
}

impl AppUiBackend {
    fn new() -> Self {
        Self {
            image_source_selector_entries: Rc::new(VecModel::default()),
        }
    }
}

#[derive(Clone)]
pub struct WeakAppUi {
    ui: slint::Weak<sg::AppWindow>,
    ui_backend: Weak<AppUiBackend>,
}

impl WeakAppUi {
    pub fn upgrade(&self) -> Option<AppUi> {
        self.ui
            .upgrade()
            .and_then(|ui| self.ui_backend.upgrade().map(|ui_backend| (ui, ui_backend)))
            .map(|(ui, ui_backend)| AppUi {
                ui,
                backend: ui_backend,
            })
    }
}

pub struct AppUi {
    ui: sg::AppWindow,
    backend: Rc<AppUiBackend>,
}

impl AppUi {
    pub fn as_weak(&self) -> WeakAppUi {
        WeakAppUi {
            ui: self.ui.as_weak(),
            ui_backend: Rc::downgrade(&self.backend),
        }
    }

    pub(crate) fn ui(&self) -> &sg::AppWindow {
        &self.ui
    }

    pub fn new() -> Result<Self, slint::PlatformError> {
        let ui = sg::AppWindow::new()?;
        let ui_backend = Rc::new(AppUiBackend::new());

        ui.set_image_source_selector_datas(ui_backend.image_source_selector_entries.clone().into());

        Ok(Self {
            ui,
            backend: ui_backend,
        })
    }

    pub(crate) fn run(&self) -> Result<(), slint::PlatformError> {
        self.ui.run()
    }

    pub fn update_with_backend_modifications(
        &mut self,
        backend: &AppBackend,
        modifications: &AppBackendModifications,
    ) {
        // image source selector entry update
        {
            let edits = modifications
                .image_sources()
                .iter()
                .filter_map(|modif| {
                    if let ImageSourceModification::Modified(uuid) = modif {
                        Some(uuid)
                    } else {
                        None
                    }
                })
                .chain(modifications.session().iter().map(|modif| match modif {
                    SessionModification::AddedImageSource(uuid) => uuid,
                    SessionModification::RemovedImageSource(uuid) => uuid,
                }));

            for uuid in edits {
                let uuid_str = uuid.to_string();
                if let Some(position) = self
                    .backend
                    .image_source_selector_entries
                    .iter()
                    .position(|item| item.id == uuid_str)
                {
                    let image_source = backend.image_sources().get_image_source(*uuid).expect("");

                    let mut model = self
                        .backend
                        .image_source_selector_entries
                        .row_data(position)
                        .unwrap();
                    image_source.update_image_source_selector_entry(&mut model);
                    self.backend
                        .image_source_selector_entries
                        .set_row_data(position, model);
                }
            }
        }

        // image source selector entry add
        {
            let adds = modifications.image_sources().iter().filter_map(|modif| {
                if let ImageSourceModification::Added(uuid) = modif {
                    Some(uuid)
                } else {
                    None
                }
            });

            for uuid in adds {
                let image_source_selector_entry = backend
                    .new_image_source_selector_entry_data(*uuid)
                    .expect("");
                self.backend
                    .image_source_selector_entries
                    .push(image_source_selector_entry)
            }
        }

        // image source selector entry deletes
        {
            let deletes = modifications.image_sources().iter().filter_map(|modif| {
                if let ImageSourceModification::Deleted(uuid) = modif {
                    Some(uuid)
                } else {
                    None
                }
            });

            for uuid in deletes {
                let uuid_str = uuid.to_string();
                if let Some(position) = self
                    .backend
                    .image_source_selector_entries
                    .iter()
                    .position(|entry| entry.id == uuid_str)
                {
                    self.backend.image_source_selector_entries.remove(position);
                }
            }
        }

        // Prepared session data
        {
            let update = !modifications.session().is_empty() || {
                let impacted_image_sources = modifications
                    .image_sources()
                    .iter()
                    .map(|modification| modification.id())
                    .collect::<HashSet<_>>();
                backend
                    .session()
                    .image_source_used()
                    .into_iter()
                    .any(|uuid| impacted_image_sources.contains(uuid))
            };

            if update {
                let status = backend
                    .session()
                    .image_source_used()
                    .into_iter()
                    .filter_map(|uuid| {
                        backend
                            .image_sources()
                            .get_image_source(*uuid)
                            .map(ImageSource::check)
                    })
                    .fold(ImageSourceCheck::default(), |acc, value| {
                        match (acc.status(), value.status()) {
                            (ImageSourceStatus::Unknown, _) => value.clone(),
                            (
                                ImageSourceStatus::Error(old_error),
                                ImageSourceStatus::Error(new_error),
                            ) => ImageSourceCheck::new(
                                acc.image_count(),
                                ImageSourceStatus::Error(old_error.clone() + new_error),
                            ),
                            (_, ImageSourceStatus::Error(new_error)) => ImageSourceCheck::new(
                                acc.image_count(),
                                ImageSourceStatus::Error(new_error.clone()),
                            ),
                            (ImageSourceStatus::Error(old_error), _) => ImageSourceCheck::new(
                                acc.image_count(),
                                ImageSourceStatus::Error(old_error.clone()),
                            ),
                            (ImageSourceStatus::Valid, ImageSourceStatus::Valid) => {
                                ImageSourceCheck::new(
                                    acc.image_count() + value.image_count(),
                                    ImageSourceStatus::Valid,
                                )
                            }
                            (ImageSourceStatus::Valid, _) => {
                                ImageSourceCheck::new(acc.image_count(), ImageSourceStatus::Valid)
                            }
                        }
                    });

                self.ui.set_prepared_session_data(sg::PreparedSessionData {
                    available_image_count: status.image_count() as i32,
                    status: status.status().into(),
                    image_duration: 30,
                })
            }
        }
    }
}
