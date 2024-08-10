use std::cell::RefCell;
use std::rc::Rc;
use std::str::FromStr;

pub use slint::ComponentHandle;
use slint::SharedString;
use uuid::Uuid;

use crate::app::{App, AppUi};
use crate::app::app_ui::WeakAppUi;
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
                .and_then(|mut backend| {
                    backend.add_or_update_image_source_from_edit_folder(&data)
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
            .unwrap_or_else(sg::EditSourceFolderData::default)
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

        // {
        //     let backend = backend.clone();
        //     ui.ui
        //         .global::<sg::ImageSourceNative>()
        //         .on_delete_source_id(move |id| {
        //             if let Err(error) = Uuid::from_str(&id)
        //                 .map_err(anyhow::Error::from)
        //                 .and_then(|uuid| {
        //                     backend
        //                         .try_borrow_mut()
        //                         .map(|backend| (backend, uuid))
        //                         .map_err(anyhow::Error::from)
        //                 })
        //                 .and_then(|(mut backend, uuid)| {
        //                     App::remove_image_source(&mut backend, &uuid)
        //                 })
        //             {
        //                 eprintln!("{}", error);
        //             }
        //         });
        // }

        Ok(())
    }
}
