use std::rc::{Rc, Weak};

use slint::{ComponentHandle, Model, VecModel};

use crate::app::backend::{AppBackend, AppBackendModifications, ImageSourceModification};
use crate::sg;

#[derive(Clone)]
struct AppUiBackend {
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

    pub(crate) fn backend(&self) -> &AppUiBackend {
        &self.backend
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
        for image_source_diff in modifications.image_sources() {
            match image_source_diff {
                ImageSourceModification::Added(uuid) => {
                    let image_source = backend.get_image_source(uuid).expect("");
                    let image_source_selector_entry = image_source.into();
                    self.backend
                        .image_source_selector_entries
                        .push(image_source_selector_entry)
                }
                ImageSourceModification::Modified(uuid) => {
                    let uuid_str = uuid.to_string();
                    if let Some(position) = self
                        .backend
                        .image_source_selector_entries
                        .iter()
                        .position(|item| &item.id == &uuid_str)
                    {
                        let image_source = backend.get_image_source(uuid).expect("");

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
        }
    }
}
