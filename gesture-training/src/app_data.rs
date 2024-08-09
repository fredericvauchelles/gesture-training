use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;
use std::str::FromStr;
use std::sync::Arc;

use slint::{ModelRc, VecModel};
use uuid::Uuid;

use crate::slint_includes::*;

#[derive(Debug, Clone)]
pub struct FolderImageSource {
    id: uuid::Uuid,
    name: String,
    path: PathBuf,
    image_count: Option<usize>,
    status: StatusData,
}
impl FolderImageSource {
    pub fn new_from(source: FolderSourceData) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: source.name.into(),
            path: source.path.into(),
            image_count: 0usize,
            status: StatusData {
                r#type: StatusType::Unknown,
                error: "".into(),
            },
        }
    }
}
impl From<FolderImageSource> for FolderSourceData {
    fn from(value: FolderImageSource) -> Self {
        Self {
            id: value.id.to_string().into(),
            name: value.name.into(),
            path: value.path.to_string_lossy().to_string().into(),
            image_count: value.image_count.unwrap_or(0) as i32,
            status: value.status,
        }
    }
}

#[derive(Debug, Clone)]
pub enum ImageSource {
    Folder(FolderImageSource),
}

impl ImageSource {
    fn id(&self) -> Uuid {
        match &self {
            ImageSource::Folder(value) => value.id,
        }
    }
}

#[derive(Clone)]
pub struct AppData {
    sources: Vec<ImageSource>,
    app_source_data: Rc<VecModel<SourceData>>,
}

impl AppData {
    pub fn get_image_source_index(&self, id: Uuid) -> Option<usize> {
        self.sources.iter().position(|source| source.id() == id)
    }

    fn get_image_source(&self, id: Uuid) -> Option<&ImageSource> {
        self.get_image_source_index(id)
            .map(|index| &self.sources[index])
    }

    pub fn app_source_datas(&self) -> ModelRc<SourceData> {
        self.app_source_data.clone().into()
    }

    pub fn add_folder_source(&mut self, data: FolderSourceData) {
        let image_source = ImageSource::Folder(FolderImageSource::new_from(data));
        let source = SourceData::from(&image_source);
    }

    pub fn try_remove_image_source_id(&mut self, id: Uuid) -> Result<ImageSource, anyhow::Error> {
        if let Some(index) = self.get_image_source_index(id) {
            self.app_source_data.remove(index);
            Ok(self.sources.remove(index))
        } else {
            Err(anyhow::anyhow!("Invalid index"))
        }
    }
}

impl AppData {
    pub fn new() -> Arc<RefCell<Self>> {
        Arc::new(RefCell::new(Self {
            sources: Vec::new(),
            app_source_data: Rc::new(VecModel::from(Vec::new())),
        }))
    }

    pub fn bind_window(app_data: &Arc<RefCell<AppData>>, app_window: &AppWindow) {
        app_window.set_source_datas(app_data.borrow().app_source_datas());

        // ImageSourceNative.delete-source-at
        {
            let app_data_1 = app_data.clone();
            app_window
                .global::<ImageSourceNative>()
                .on_delete_source_id(move |id| {
                    match Uuid::from_str(&id)
                        .map_err(anyhow::Error::from)
                        .and_then(|id| {
                            app_data_1
                                .try_borrow_mut()
                                .map(|app_data| (id, app_data))
                                .map_err(anyhow::Error::from)
                        })
                        .and_then(|(id, mut app_data)| {
                            app_data
                                .try_remove_image_source_id(id)
                                .map_err(anyhow::Error::from)
                        }) {
                        Ok(_) => {}
                        Err(error) => {
                            eprintln!("{}", error)
                        }
                    }
                });
        }

        // SourceFolderNative.get-folder-source-data-from-index
        {
            let app_data_1 = app_data.clone();
            app_window
                .global::<SourceFolderNative>()
                .on_get_folder_source_data_from_id(move |id| {
                    Uuid::from_str(&id)
                        .map_err(anyhow::Error::from)
                        .and_then(|id| {
                            app_data_1
                                .try_borrow()
                                .map(|app_data| (id, app_data))
                                .map_err(anyhow::Error::from)
                        })
                        .and_then(|(id, mut app_data)| {
                            if let Some(ImageSource::Folder(folder)) = app_data.get_image_source(id)
                            {
                                return Ok(FolderSourceData::from(folder.clone()));
                            }
                            Ok(FolderSourceData::default())
                        })
                        .unwrap_or_else(|error| {
                            eprintln!("{}", error);
                            FolderSourceData::default()
                        })
                });
        }

        // SourceFolderNative.get-folder-source-data-from-index
        {
            let app_data_1 = app_data.clone();
            app_window
                .global::<SourceFolderNative>()
                .on_add_or_save_folder_source(move |folder_source_data| {
                    app_data_1.add_folder_source(folder_source_data)
                });
        }
    }
}
