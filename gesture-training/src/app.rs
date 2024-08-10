use std::cell::RefCell;
use std::rc::Rc;

use slint::ComponentHandle;

use crate::sg;

mod image_source {
    use slint::SharedString;
    use uuid::Uuid;

    use folder::ImageSourceFolder;

    use crate::sg;

    pub mod folder {
        use std::path::PathBuf;

        use slint::SharedString;
        use uuid::Uuid;

        use crate::sg;

        use super::ImageSourceTrait;

        #[derive(Debug, Clone)]
        pub struct ImageSourceFolder {
            id: Uuid,
            pub(crate) name: String,
            path: PathBuf,
        }

        impl ImageSourceFolder {
            pub fn new(id: Uuid, name: String, path: PathBuf) -> Self {
                Self { id, name, path }
            }
        }

        impl ImageSourceTrait for ImageSourceFolder {
            fn id(&self) -> &Uuid {
                &self.id
            }
            fn name(&self) -> &str {
                &self.name
            }
        }

        impl<'a> From<&'a ImageSourceFolder> for sg::EditSourceFolderData {
            fn from(value: &'a ImageSourceFolder) -> Self {
                Self {
                    id: value.id.to_string().into(),
                    name: value.name.clone().into(),
                    image_count: 0,
                    path: value.path.to_string_lossy().to_string().into(),
                    status: sg::StatusIconData {
                        r#type: sg::StatusIconType::Unknown,
                        error: SharedString::default(),
                    },
                }
            }
        }

        impl From<ImageSourceFolder> for sg::EditSourceFolderData {
            fn from(value: ImageSourceFolder) -> Self {
                Self {
                    id: value.id.to_string().into(),
                    name: value.name.into(),
                    image_count: 0,
                    path: value.path.to_string_lossy().to_string().into(),
                    status: sg::StatusIconData {
                        r#type: sg::StatusIconType::Unknown,
                        error: SharedString::default(),
                    },
                }
            }
        }
    }

    pub trait ImageSourceTrait {
        fn id(&self) -> &Uuid;
        fn name(&self) -> &str;
    }

    #[derive(Debug, Clone)]
    pub enum ImageSource {
        Folder(ImageSourceFolder),
    }

    impl<'a> From<&'a ImageSource> for sg::ImageSourceSelectorEntryData {
        fn from(value: &'a ImageSource) -> Self {
            Self {
                id: value.id().to_string().into(),
                name: value.name().into(),
                image_count: 0,
                status: sg::StatusIconData {
                    r#type: sg::StatusIconType::Unknown,
                    error: SharedString::default(),
                },
                enabled: false,
            }
        }
    }

    impl ImageSourceTrait for ImageSource {
        fn id(&self) -> &Uuid {
            match self {
                ImageSource::Folder(value) => value.id(),
            }
        }

        fn name(&self) -> &str {
            match self {
                ImageSource::Folder(value) => value.name(),
            }
        }
    }
}

mod app_backend {
    use std::collections::HashMap;
    use std::rc::Rc;

    use slint::VecModel;
    use uuid::Uuid;

    use crate::sg;

    use super::image_source::{ImageSource, ImageSourceTrait};

    pub struct AppBackend {
        image_sources: HashMap<Uuid, ImageSource>,
        image_source_selector_datas: Rc<VecModel<sg::ImageSourceSelectorEntryData>>,
    }

    impl AppBackend {
        pub fn new() -> Self {
            Self {
                image_sources: HashMap::new(),
                image_source_selector_datas: Rc::new(VecModel::default()),
            }
        }

        pub fn get_image_source(&self, id: &Uuid) -> Option<&ImageSource> {
            self.image_sources.get(id)
        }

        pub fn get_image_source_mut(&mut self, id: &Uuid) -> Option<&mut ImageSource> {
            self.image_sources.get_mut(id)
        }

        pub fn add_image_source(&mut self, image_source: impl Into<ImageSource>) {
            let image_source = image_source.into();
            self.image_sources.insert(*image_source.id(), image_source);
        }

        pub fn image_source_selector_datas(
            &self,
        ) -> &Rc<VecModel<sg::ImageSourceSelectorEntryData>> {
            &self.image_source_selector_datas
        }
    }
}

pub struct App {}

impl App {
    pub fn run() -> Result<(), slint::PlatformError> {
        let ui = sg::AppWindow::new()?;
        let backend = Rc::new(RefCell::new(app_backend::AppBackend::new()));
        Self::bind(&ui, &backend);
        ui.run()
    }
}

mod app_impl {
    use std::cell::RefCell;
    use std::rc::Rc;
    use std::str::FromStr;

    use slint::{ComponentHandle, Model};
    use uuid::Uuid;

    use crate::app::App;
    use crate::app::image_source::{ImageSource, ImageSourceTrait};
    use crate::sg;

    use super::image_source::folder::ImageSourceFolder;

    type RcBackend = Rc<RefCell<super::app_backend::AppBackend>>;
    type Backend = super::app_backend::AppBackend;

    enum ImageSourceDiff {
        Added(Uuid),
        Modified(Uuid),
        None,
    }

    impl App {
        pub(super) fn bind(ui: &sg::AppWindow, backend: &RcBackend) {
            {
                ui.set_image_source_selector_datas(
                    backend
                        .borrow()
                        .image_source_selector_datas()
                        .clone()
                        .into(),
                );
            }

            {
                let backend = backend.clone();
                ui.global::<sg::EditSourceFolderNative>()
                    .on_add_or_save_folder_source(move |data| {
                        if let Err(error) = backend
                            .try_borrow_mut()
                            .map_err(anyhow::Error::from)
                            .map(|mut backend| {
                                App::add_or_update_image_source_from_edit(&mut backend, &data);
                            })
                        {
                            eprintln!("{}", error);
                        }
                    });
            }

            {
                let backend = backend.clone();
                ui.global::<sg::EditSourceFolderNative>()
                    .on_get_folder_source_data_from_id(move |id| -> sg::EditSourceFolderData {
                        match Uuid::from_str(&id)
                            .map_err(anyhow::Error::from)
                            .and_then(|uuid| {
                                backend
                                    .try_borrow()
                                    .map_err(anyhow::Error::from)
                                    .map(|backend| (backend, uuid))
                            })
                            .and_then(|(backend, uuid)| {
                                backend
                                    .get_image_source(&uuid)
                                    .cloned()
                                    .ok_or(anyhow::anyhow!(""))
                            }) {
                            Ok(ImageSource::Folder(value)) => value.into(),
                            Err(error) => {
                                eprintln!("{}", error);
                                sg::EditSourceFolderData::default()
                            }
                        }
                    });
            }
        }

        fn update_dataview_from_diff(
            backend: &mut Backend,
            image_source_diffs: &[ImageSourceDiff],
        ) {
            for image_source_diff in image_source_diffs {
                match image_source_diff {
                    ImageSourceDiff::Added(uuid) => {
                        let image_source = backend.get_image_source(uuid).expect("");
                        let image_source_selector_entry = image_source.into();
                        backend
                            .image_source_selector_datas()
                            .push(image_source_selector_entry)
                    }
                    ImageSourceDiff::Modified(uuid) => {
                        let uuid_str = uuid.to_string();
                        if let Some(position) = backend
                            .image_source_selector_datas()
                            .iter()
                            .position(|item| &item.id == &uuid_str)
                        {
                            let image_source = backend.get_image_source(uuid).expect("");

                            let mut model = backend
                                .image_source_selector_datas()
                                .row_data(position)
                                .unwrap();
                            model.name = image_source.name().into();
                            backend
                                .image_source_selector_datas()
                                .set_row_data(position, model);
                        }
                    }
                    ImageSourceDiff::None => {}
                }
            }
        }

        fn add_or_update_image_source_from_edit(
            backend: &mut Backend,
            data: &sg::EditSourceFolderData,
        ) {
            let mut data = data.clone();
            let id = Uuid::from_str(&data.id).unwrap_or_else(|_| Uuid::new_v4());
            data.id = id.to_string().into();

            // Update backend
            let diff = match backend.get_image_source_mut(&id) {
                // Update image source
                Some(ImageSource::Folder(folder)) => {
                    folder.name = data.name.to_string();
                    ImageSourceDiff::Modified(id)
                }
                None => {
                    let image_source = ImageSource::Folder(ImageSourceFolder::new(
                        id,
                        data.name.to_string(),
                        data.path.to_string().into(),
                    ));
                    backend.add_image_source(image_source.clone());
                    ImageSourceDiff::Added(*image_source.id())
                }
            };

            Self::update_dataview_from_diff(backend, &[diff]);
        }
    }
}
