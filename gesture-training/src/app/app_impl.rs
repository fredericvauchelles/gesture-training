use std::cell::RefCell;
use std::rc::Rc;
use std::str::FromStr;

use rfd::AsyncFileDialog;
pub use slint::ComponentHandle;
use slint::SharedString;
use uuid::Uuid;

use crate::app::{App, AppUi};
use crate::app::app_ui::WeakAppUi;
use crate::app::backend::ImageSourceModification;
use crate::app::image_source::{ImageSource, ImageSourceTrait};
use crate::sg;

type RcBackend = Rc<RefCell<super::backend::AppBackend>>;

#[derive(Clone)]
struct AppCallback {
    app: Rc<App>,
    ui: WeakAppUi,
    backend: RcBackend,
}

impl AppCallback {
    fn new(app: &Rc<App>, ui: &WeakAppUi, backend: &RcBackend) -> Self {
        Self {
            app: app.clone(),
            ui: ui.clone(),
            backend: backend.clone(),
        }
    }

    fn trigger_image_source_check(&self, image_source_ids: impl IntoIterator<Item = Uuid>) {
        fn trigger_image_source_check_for_uuid(
            callback: &AppCallback,
            uuid: Uuid,
        ) -> anyhow::Result<()> {
            let backend = callback.backend.try_borrow()?;
            let image_source = backend
                .image_sources()
                .get_image_source(uuid)
                .ok_or(anyhow::anyhow!(""))?
                .clone();

            let callback_clone = callback.clone();
            slint::spawn_local(async move {
                async fn execute(
                    image_source: ImageSource,
                    callback: &AppCallback,
                ) -> anyhow::Result<()> {
                    let modifications = {
                        let check = image_source.check_source().await;
                        let mut backend = callback.backend.try_borrow_mut()?;
                        let image_source = backend
                            .image_sources_mut()
                            .get_image_source_mut(image_source.id())
                            .ok_or(anyhow::anyhow!(""))?;
                        image_source.set_check(check);
                        ImageSourceModification::Modified(image_source.id()).into()
                    };

                    {
                        let mut ui = callback.ui.upgrade().ok_or(anyhow::anyhow!(""))?;
                        let backend = callback.backend.try_borrow()?;
                        ui.update_with_backend_modifications(&backend, &modifications);
                    }

                    Ok(())
                }

                callback_clone
                    .app
                    .handle_error(execute(image_source, &callback_clone).await);
            })?;
            Ok(())
        }

        for uuid in image_source_ids.into_iter() {
            self.app
                .handle_error(trigger_image_source_check_for_uuid(self, uuid));
        }
    }

    fn add_or_save_folder_source(&self, data: sg::EditSourceFolderData) {
        fn execute(this: &AppCallback, data: sg::EditSourceFolderData) -> anyhow::Result<()> {
            // Save in backend
            let diff = this
                .backend
                .try_borrow_mut()
                .map_err(anyhow::Error::from)
                .and_then(|backend| {
                    this.app
                        .source_folder
                        .edited_path()
                        .map(|path| (backend, path))
                })
                .and_then(|(mut backend, path)| {
                    backend
                        .image_sources_mut()
                        .add_or_update_image_source_from_edit_folder(&data, path)
                })?;

            // propagate change to the ui
            if let Some(mut ui) = this.ui.upgrade() {
                let backend = this.backend.try_borrow().map_err(anyhow::Error::from)?;
                ui.update_with_backend_modifications(&backend, &diff);
            }

            // Trigger a check of the image source
            let changed_ids = diff
                .image_sources()
                .iter()
                .filter(|source| !matches!(source, ImageSourceModification::Deleted(_)))
                .map(|source| source.id());
            this.trigger_image_source_check(changed_ids);

            Ok(())
        }
        self.app.handle_error(execute(self, data));
    }

    fn get_folder_source_data_from_id(&self, id: SharedString) -> sg::EditSourceFolderData {
        self.app
            .handle_error(
                Uuid::from_str(&id)
                    .map_err(anyhow::Error::from)
                    .and_then(|uuid| {
                        self.backend
                            .try_borrow()
                            .map_err(anyhow::Error::from)
                            .map(|backend| (backend, uuid))
                    })
                    .and_then(|(backend, uuid)| {
                        backend
                            .image_sources()
                            .get_image_source(uuid)
                            .cloned()
                            .ok_or(anyhow::anyhow!(""))
                    }),
            )
            .and_then(|v| v.try_into().ok())
            .unwrap_or_default()
    }

    fn on_request_asked_path(&self) -> i32 {
        let id = self.app.source_folder().next_request_ask_path_id() as i32;
        let ui = self.ui.clone();
        let app_clone = self.app.clone();
        let future = async move {
            if let Some(selection) = AsyncFileDialog::new().pick_folder().await {
                // try to store the selected path
                if let Err(error) = app_clone.source_folder().set_edited_path(selection.path()) {
                    app_clone.handle_error::<()>(Err(error));
                } else {
                    // update the ui
                    ui.upgrade()
                        .unwrap()
                        .ui()
                        .invoke_dispatch_edit_source_folder_request_asked_path_completed(
                            id,
                            selection.path().to_string_lossy().to_string().into(),
                        );
                }
            }
        };
        self.app
            .handle_error(slint::spawn_local(future).map_err(anyhow::Error::from));

        id
    }

    pub(crate) fn on_delete_source_id(&self, id: SharedString) {
        fn execute(this: &AppCallback, id: SharedString) -> anyhow::Result<()> {
            let mut backend = this.backend.try_borrow_mut()?;
            let uuid = Uuid::from_str(&id)?;

            if let Some(image_source) = backend.image_sources_mut().remove_image_source(uuid) {
                let diff = ImageSourceModification::Deleted(image_source.id()).into();
                let mut ui = this.ui.upgrade().ok_or(anyhow::anyhow!(""))?;
                ui.update_with_backend_modifications(&backend, &diff);
            }

            Ok(())
        }

        self.app.handle_error(execute(self, id));
    }

    fn on_set_image_source_used(&self, id: SharedString, is_used: bool) {
        fn execute(callback: &AppCallback, id: SharedString, is_used: bool) -> anyhow::Result<()> {
            let uuid = Uuid::from_str(&id)?;

            let modification = {
                let mut backend = callback.backend.try_borrow_mut()?;
                if is_used {
                    backend.add_image_source_to_session(uuid)
                } else {
                    backend.remove_image_source_from_session(uuid)
                }
            };

            if !modification.is_empty() {
                let mut ui = callback.ui.upgrade().ok_or(anyhow::anyhow!(""))?;
                let backend = callback.backend.try_borrow()?;
                ui.update_with_backend_modifications(&backend, &modification);
            }

            Ok(())
        }

        self.app.handle_error(execute(self, id, is_used));
    }
}

impl App {
    fn handle_error<V>(&self, value: Result<V, anyhow::Error>) -> Option<V> {
        match value {
            Ok(value) => Some(value),
            Err(error) => {
                eprintln!("{}", error);
                None
            }
        }
    }

    /// Initialize data in backend and app
    pub(super) fn initialize(
        _app: &Rc<App>,
        ui: &AppUi,
        _backend: &RcBackend,
    ) -> Result<(), slint::PlatformError> {
        ui.ui().set_prepared_session_data(sg::PreparedSessionData {
            available_image_count: 0,
            image_duration: 30,
            status: sg::StatusIconData {
                r#type: sg::StatusIconType::Unknown,
                error: SharedString::default(),
            },
        });

        Ok(())
    }

    pub(super) fn bind(
        app: &Rc<App>,
        ui: &AppUi,
        backend: &RcBackend,
    ) -> Result<(), slint::PlatformError> {
        let app_callback = AppCallback::new(app, &ui.as_weak(), backend);

        {
            let callback = app_callback.clone();
            ui.ui()
                .global::<sg::EditSourceFolderNative>()
                .on_add_or_save_folder_source(move |data| callback.add_or_save_folder_source(data));
        }

        {
            let callback = app_callback.clone();
            ui.ui()
                .global::<sg::EditSourceFolderNative>()
                .on_get_folder_source_data_from_id(move |id| -> sg::EditSourceFolderData {
                    callback.get_folder_source_data_from_id(id)
                });
        }

        {
            let callback = app_callback.clone();
            ui.ui()
                .global::<sg::EditSourceFolderNative>()
                .on_request_asked_path(move || callback.on_request_asked_path());
        }

        {
            let callback = app_callback.clone();
            ui.ui()
                .global::<sg::EditSourceFolderNative>()
                .on_clear_source_folder_editor(move || {
                    callback
                        .app
                        .handle_error(callback.app.source_folder.clear_edited_path());
                });
        }

        {
            let callback = app_callback.clone();
            ui.ui()
                .global::<sg::ImageSourceNative>()
                .on_delete_source_id(move |id| callback.on_delete_source_id(id));
        }

        {
            let callback = app_callback.clone();
            ui.ui()
                .global::<sg::ImageSourceSelectorNative>()
                .on_set_image_source_used(move |id, is_used| {
                    callback.on_set_image_source_used(id, is_used)
                });
        }

        Ok(())
    }
}
