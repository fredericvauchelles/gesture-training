use std::cell::RefCell;
use std::sync::Arc;

use slint::ComponentHandle;

use crate::sg;

mod image_source {
    use slint::SharedString;
    use uuid::Uuid;

    use folder::ImageSourceFolder;

    use crate::sg;

    mod folder {
        use std::str::FromStr;

        use uuid::Uuid;

        use crate::sg;

        use super::{ImageSource, ImageSourceTrait};

        #[derive(Debug, Clone)]
        pub struct ImageSourceFolder {
            id: Uuid,
            pub name: String,
        }

        impl ImageSourceTrait for ImageSourceFolder {
            fn id(&self) -> &Uuid {
                &self.id
            }
            fn name(&self) -> &str {
                &self.name
            }
        }

        impl TryFrom<sg::EditSourceFolderData> for ImageSource {
            type Error = anyhow::Error;
            fn try_from(value: sg::EditSourceFolderData) -> Result<Self, Self::Error> {
                Ok(ImageSource::Folder(ImageSourceFolder {
                    id: Uuid::from_str(&value.id)
                        .map_err(anyhow::Error::from)
                        .unwrap(),
                    name: value.name.to_string(),
                }))
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
            self.image_sources
                .insert(image_source.id().clone(), image_source);
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
        let backend = Arc::new(RefCell::new(app_backend::AppBackend::new()));
        Self::bind(&ui, &backend);
        ui.run()
    }
}

mod app_impl {
    use std::cell::RefCell;
    use std::str::FromStr;
    use std::sync::Arc;

    use slint::ComponentHandle;
    use uuid::Uuid;

    use crate::app::App;
    use crate::app::image_source::{ImageSource, ImageSourceTrait};
    use crate::sg;
    use crate::sg::AppWindow;

    type ArcBackend = Arc<RefCell<super::app_backend::AppBackend>>;
    type Backend = super::app_backend::AppBackend;

    enum ImageSourceDiff {
        Added(Uuid),
        Modified(Uuid),
        None,
    }

    impl App {
        pub(super) fn bind(ui: &sg::AppWindow, backend: &ArcBackend) {
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
                                App::add_or_update_image_source_from_edit(
                                    &mut backend,
                                    &data,
                                );
                            })
                        {
                            eprintln!("{}", error);
                        }
                    });
            }
        }

        pub(super) fn add_or_update_image_source_from_edit(
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
                    ImageSourceDiff::Modified(id.clone())
                }
                Some(_) => {
                    eprintln!(
                        "Image Source with id {} is not a Folder image source.",
                        data.id
                    );
                    ImageSourceDiff::None
                }
                None => {
                    let image_source: Result<ImageSource, anyhow::Error> = data.try_into();
                    match image_source {
                        // Add image source
                        Ok(image_source) => {
                            backend.add_image_source(image_source.clone());
                            ImageSourceDiff::Added(image_source.id().clone())
                        }
                        Err(error) => {
                            eprintln!("{}", error);
                            ImageSourceDiff::None
                        }
                    }
                }
            };

            // Notify ui
            match diff {
                ImageSourceDiff::Added(uuid) => {
                    let image_source = backend.get_image_source(&uuid).expect("");
                    let image_source_selector_entry = image_source.into();
                    backend
                        .image_source_selector_datas()
                        .push(image_source_selector_entry)
                }
                ImageSourceDiff::Modified(uuid) => {}
                ImageSourceDiff::None => {}
            }
        }
    }
}
