use std::cell::RefCell;

use std::rc::Rc;
use std::str::FromStr;
use std::time::Duration;

pub use slint::ComponentHandle;
use slint::SharedString;
use uuid::Uuid;

use crate::app::app_ui::WeakAppUi;
use crate::app::backend::{AppBackendModifications, ImageSourceModification};
use crate::app::image_source::{ImageSource, ImageSourceTrait};
use crate::app::log::Log;
use crate::app::session::AppSessionConfiguration;
use crate::app::{App, AppUi};
use crate::sg;

type RcBackend = Rc<RefCell<super::backend::AppBackend>>;

/// Use this structure inside callbacks to perform logic
#[derive(Clone)]
struct AppCallback {
    app: Rc<RefCell<App>>,
    ui: WeakAppUi,
    backend: RcBackend,
}

impl AppCallback {
    fn new(app: &Rc<RefCell<App>>, ui: &WeakAppUi, backend: &RcBackend) -> Self {
        Self {
            app: app.clone(),
            ui: ui.clone(),
            backend: backend.clone(),
        }
    }

    fn handle_error<V>(&self, result: anyhow::Result<V>) -> Option<V> {
        match self.app.try_borrow() {
            Ok(app) => app.handle_error(result),
            Err(error) => {
                Log::handle_error(&error);
                None
            }
        }
    }

    fn trigger_image_source_check_from_modifications(&self, modifications: &AppBackendModifications) {
        let changed_ids = modifications
            .image_sources()
            .iter()
            .filter(|source| !matches!(source, ImageSourceModification::Deleted(_)))
            .map(|source| source.id());
        self.trigger_image_source_check(changed_ids);
    }

    fn trigger_image_source_check(&self, image_source_ids: impl IntoIterator<Item=Uuid>) {
        fn trigger_image_source_check_for_uuid(
            callback: &AppCallback,
            uuid: Uuid,
        ) -> anyhow::Result<()> {
            let backend = callback.backend.borrow();
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
                        let mut backend = callback.backend.borrow_mut();
                        let image_source = backend
                            .image_sources_mut()
                            .get_image_source_mut(image_source.id())
                            .ok_or(anyhow::anyhow!(""))?;
                        image_source.set_check(check);
                        ImageSourceModification::Modified(image_source.id()).into()
                    };

                    {
                        let mut ui = callback.ui.upgrade().ok_or(anyhow::anyhow!(""))?;
                        let backend = callback.backend.borrow();
                        ui.update_with_backend_modifications(&backend, &modifications);
                    }

                    Ok(())
                }

                let error = execute(image_source, &callback_clone).await;
                callback_clone.handle_error(error);
            })?;
            Ok(())
        }

        for uuid in image_source_ids.into_iter() {
            let error = trigger_image_source_check_for_uuid(self, uuid);
            self.handle_error(error);
        }
    }

    fn add_or_save_folder_source(&self, data: sg::EditSourceFolderData) {
        fn execute(this: &AppCallback, data: sg::EditSourceFolderData) -> anyhow::Result<()> {
            // Save in backend
            let modifications = {
                let mut backend = this.backend.borrow_mut();
                let app = this.app.borrow();

                // enable-path-picking
                #[cfg(any(target_os = "linux", target_os = "windows", target_os = "macos"))]
                let path = app.source_folder.edited_path().cloned();
                #[cfg(target_os = "android")]
                let path = Some(std::path::PathBuf::from(data.path.as_str()));

                let modifications = backend
                    .image_sources_mut()
                    .add_or_update_image_source_from_edit_folder(&data, path)?;

                backend.save_to_persistence()?;

                modifications
            };

            // propagate change to the ui
            if let Some(mut ui) = this.ui.upgrade() {
                let backend = this.backend.borrow();
                ui.update_with_backend_modifications(&backend, &modifications);
            }

            // Trigger a check of the image source
            this.trigger_image_source_check_from_modifications(&modifications);

            Ok(())
        }
        let error = execute(self, data);
        self.handle_error(error);
    }

    fn get_folder_source_data_from_id(&self, id: SharedString) -> sg::EditSourceFolderData {
        let result = Uuid::from_str(&id)
            .map_err(anyhow::Error::from)
            .and_then(|uuid| {
                self.backend
                    .borrow()
                    .image_sources()
                    .get_image_source(uuid)
                    .cloned()
                    .ok_or(anyhow::anyhow!(""))
            });

        self.handle_error(result)
            .and_then(|v| v.try_into().ok())
            .unwrap_or_default()
    }

    fn on_request_asked_path(&self) -> i32 {
        #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
        {
            use rfd::AsyncFileDialog;

            let app = self.app.borrow();
            let id = app.source_folder().next_request_ask_path_id() as i32;
            let ui = self.ui.clone();
            let app_clone = self.app.clone();
            let future = async move {
                if let Some(selection) = AsyncFileDialog::new().pick_folder().await {
                    let mut app_clone_ref = app_clone.borrow_mut();
                    // try to store the selected path
                    app_clone_ref
                        .source_folder_mut()
                        .set_edited_path(selection.path());

                    // update the ui
                    ui.upgrade()
                        .unwrap()
                        .ui()
                        .invoke_dispatch_edit_source_folder_request_asked_path_completed(
                            id,
                            selection.path().to_string_lossy().to_string().into(),
                        );
                }
            };

            let error = slint::spawn_local(future).map_err(anyhow::Error::from);
            self.handle_error(error);

            id
        }

        #[cfg(target_os = "android")]
        {
            unreachable!()
        }
    }

    pub(crate) fn on_delete_source_id(&self, id: SharedString) {
        fn execute(this: &AppCallback, id: SharedString) -> anyhow::Result<()> {
            let mut backend = this.backend.borrow_mut();
            let uuid = Uuid::from_str(&id)?;

            if let Some(image_source) = backend.image_sources_mut().remove_image_source(uuid) {
                let diff = ImageSourceModification::Deleted(image_source.id()).into();
                let mut ui = this.ui.upgrade().ok_or(anyhow::anyhow!(""))?;
                ui.update_with_backend_modifications(&backend, &diff);
            }

            Ok(())
        }

        let error = execute(self, id);
        self.handle_error(error);
    }

    fn on_set_image_source_used(&self, id: SharedString, is_used: bool) {
        fn execute(callback: &AppCallback, id: SharedString, is_used: bool) -> anyhow::Result<()> {
            let uuid = Uuid::from_str(&id)?;

            let modification = {
                let mut backend = callback.backend.borrow_mut();
                if is_used {
                    backend.add_image_source_to_session(uuid)
                } else {
                    backend.remove_image_source_from_session(uuid)
                }
            };

            if !modification.is_empty() {
                let mut ui = callback.ui.upgrade().ok_or(anyhow::anyhow!(""))?;
                let backend = callback.backend.borrow();
                ui.update_with_backend_modifications(&backend, &modification);
            }

            Ok(())
        }

        let error = execute(self, id, is_used);
        self.handle_error(error);
    }

    fn on_session_start(&self) {
        fn execute(callback: &AppCallback) -> anyhow::Result<()> {
            let prepared_session_data = {
                let ui = callback.ui.upgrade().ok_or(anyhow::anyhow!(""))?;
                ui.ui().get_prepared_session_data()
            };

            let image_sources = {
                let backend_ref = callback.backend.borrow();
                backend_ref
                    .used_image_source()
                    .into_iter()
                    .collect::<Vec<_>>()
            };

            let session_config = AppSessionConfiguration::new(
                Duration::from_secs(prepared_session_data.image_duration as u64),
                prepared_session_data.used_image_count as usize,
                image_sources,
            );

            {
                let callback_clone = callback.clone();
                let callback_clone2 = callback.clone();
                let callback_clone3 = callback.clone();
                let callback_clone4 = callback.clone();
                let mut app_ref = callback.app.borrow_mut();
                app_ref.session.start_session(
                    &session_config,
                    move |time_left| {
                        let ui = callback_clone.ui.upgrade().unwrap();
                        ui.ui().set_session_time_left(time_left.as_secs_f32())
                    },
                    move || {
                        let modifications =  {let mut app = callback_clone2.app.borrow_mut();
                        app.session.go_to_next_image().unwrap() };
                        {
                            let mut ui = callback_clone2.ui.upgrade().unwrap();
                            let backend = callback_clone2.backend.borrow();
                            ui.update_with_backend_modifications(&backend, &modifications);
                        }
                    },
                    move || {
                        let ui = callback_clone3.ui.upgrade().unwrap();
                        ui.ui().set_session_state(sg::SessionWindowState::Loading);
                    },
                    move |image| {
                        let ui = callback_clone4.ui.upgrade().unwrap();
                        ui.ui().invoke_session_show_image(image);
                        ui.ui().set_session_state(sg::SessionWindowState::Running);
                    },
                )?;
            }

            // Update ui with init data
            let ui = callback.ui.upgrade().ok_or(anyhow::anyhow!(""))?;
            let prepared_session_data = ui.ui().get_prepared_session_data();
            ui.ui()
                .set_session_time_left(prepared_session_data.image_duration as f32);

            Ok(())
        }
        let error = execute(&self);
        self.handle_error(error);
    }
}

impl App {
    fn handle_error<V>(&self, value: Result<V, anyhow::Error>) -> Option<V> {
        match value {
            Ok(value) => Some(value),
            Err(error) => {
                Log::handle_error(&error);
                None
            }
        }
    }

    /// Initialize data in backend and app
    pub(super) fn initialize(
        app: &Rc<RefCell<App>>,
        ui: &mut AppUi,
        backend: &RcBackend,
    ) -> Result<(), anyhow::Error> {
        let app_callback = AppCallback::new(app, &ui.as_weak(), backend);

        ui.ui().set_prepared_session_data(sg::PreparedSessionData {
            available_image_count: 0,
            image_duration: 30,
            used_image_count: 5,
            status: sg::StatusIconData {
                r#type: sg::StatusIconType::Unknown,
                error: SharedString::default(),
            },
        });

        #[cfg(any(target_os = "windows", target_os = "linux", target_os = "macos"))]
        ui.ui().set_enable_pick_path(true);
        #[cfg(target_os = "android")]
        ui.ui().set_enable_pick_path(false);

        let modifications = {
            let mut backend_ref = backend.borrow_mut();
            let modifications = backend_ref.update_from_persistence()?;
            ui.update_with_backend_modifications(&mut backend_ref, &modifications);
            modifications
        };
        app_callback.trigger_image_source_check_from_modifications(&modifications);

        Ok(())
    }

    pub(super) fn bind(
        app: &Rc<RefCell<App>>,
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
                    callback.app.borrow_mut().source_folder.clear_edited_path();
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

        {
            ui.ui().global::<sg::TimerNative>().on_seconds_to_string(
                |seconds: i32| -> SharedString {
                    let min = seconds / 60;
                    let sec = seconds % 60;
                    format!("{:02}:{:02}", min, sec).into()
                },
            );
        }

        {
            let callback = app_callback.clone();
            ui.ui()
                .global::<sg::SessionNative>()
                .on_on_session_start(move || callback.on_session_start());
        }

        {
            let callback = app_callback.clone();
            ui.ui()
                .global::<sg::SessionNative>()
                .on_next_image(move || {
                    let modifications = callback
                        .app
                        .borrow_mut()
                        .session
                        .go_to_next_image()
                        .unwrap();

                    let backend = callback.backend.borrow();
                    callback.ui.upgrade().unwrap().update_with_backend_modifications(&backend, &modifications);
                });
        }

        {
            let callback = app_callback.clone();
            ui.ui()
                .global::<sg::SessionNative>()
                .on_previous_image(move || {
                    callback
                        .app
                        .borrow_mut()
                        .session
                        .go_to_previous_image()
                        .unwrap()
                });
        }

        {
            let callback = app_callback.clone();
            ui.ui()
                .global::<sg::SessionNative>()
                .on_on_image_displayed(move || {
                    callback.app.borrow().session.reset_time_left().unwrap();
                });
        }

        {
            let callback = app_callback.clone();
            ui.ui()
                .global::<sg::SessionNative>()
                .on_on_play_mode_changed(move |is_playing| {
                    callback.app.borrow().session.set_is_playing(is_playing);
                });
        }

        Ok(())
    }
}
