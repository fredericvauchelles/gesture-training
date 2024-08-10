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
use crate::app::image_source::ImageSourceTrait;
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

    fn add_or_save_folder_source(&self, data: sg::EditSourceFolderData) {
        fn execute(this: &AppCallback, data: sg::EditSourceFolderData) -> anyhow::Result<()> {
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
                    backend.add_or_update_image_source_from_edit_folder(&data, path)
                })?;

            if let Some(mut ui) = this.ui.upgrade() {
                let backend = this.backend.try_borrow().map_err(anyhow::Error::from)?;
                ui.update_with_backend_modifications(&backend, &diff);
            }

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
                            .get_image_source(&uuid)
                            .cloned()
                            .ok_or(anyhow::anyhow!(""))
                    }),
            )
            .and_then(|v| v.try_into().ok())
            .unwrap_or_default()
    }

    pub(crate) fn on_delete_source_id(&self, id: SharedString) {
        fn execute(this: &AppCallback, id: SharedString) -> anyhow::Result<()> {
            let mut backend = this.backend.try_borrow_mut()?;
            let uuid = Uuid::from_str(&id)?;

            if let Some(image_source) = backend.remove_image_source(&uuid) {
                let diff = ImageSourceModification::Deleted(image_source.id().clone()).into();
                let mut ui = this.ui.upgrade().ok_or(anyhow::anyhow!(""))?;
                ui.update_with_backend_modifications(&mut backend, &diff);
            }

            Ok(())
        }

        self.app.handle_error(execute(self, id));
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
                .on_request_asked_path(move || {
                    let id = callback.app.source_folder().next_request_ask_path_id() as i32;
                    let ui = callback.ui.clone();
                    let app_clone = callback.app.clone();
                    let future = async move {
                        if let Some(selection) = AsyncFileDialog::new().pick_folder().await {
                            app_clone.source_folder().set_edited_path(selection.path());

                            // update the ui
                            ui.upgrade()
                                .unwrap()
                                .ui()
                                .invoke_edit_source_folder_request_asked_path_completed(
                                    id,
                                    selection.path().to_string_lossy().to_string().into(),
                                );
                        }
                    };
                    callback
                        .app
                        .handle_error(slint::spawn_local(future).map_err(anyhow::Error::from));

                    id
                });
        }

        {
            let callback = app_callback.clone();
            ui.ui()
                .global::<sg::ImageSourceNative>()
                .on_delete_source_id(move |id| callback.on_delete_source_id(id));
        }

        Ok(())
    }
}
