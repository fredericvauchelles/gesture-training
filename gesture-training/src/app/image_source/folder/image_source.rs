use std::io;
use std::path::{Path, PathBuf};

use async_std::prelude::StreamExt;
use slint::{Image, SharedString};
use uuid::Uuid;
use serde::{Serialize, Deserialize};

use crate::app::image_source::{ImageSource, ImageSourceCheck, ImageSourceStatus, ImageSourceTrait};
use crate::app::log::Log;
use crate::sg;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageSourceFolder {
    id: Uuid,
    pub(crate) name: String,
    pub(crate) path: PathBuf,
    #[serde(skip)]
    check: ImageSourceCheck,
}

impl ImageSourceFolder {
    pub fn new(id: Uuid, name: String, path: PathBuf, check: ImageSourceCheck) -> Self {
        Self {
            id,
            name,
            path,
            check,
        }
    }

    const IMAGE_EXTENSIONS: &'static [&'static str] = &["jpg", "jpeg", "png", "bmp"];
    fn is_image_file(path: &async_std::path::Path) -> bool {
        if let Some(extension) = path.extension().and_then(|ext| ext.to_str()) {
            Self::IMAGE_EXTENSIONS.contains(&extension)
        } else {
            false
        }
    }
    async fn find_image_files_in_directory(path: &Path) -> io::Result<Vec<PathBuf>> {
        let mut paths = vec![path.to_path_buf()];
        let mut image_paths = Vec::new();
        while let Some(current_path) = paths.pop() {
            match async_std::fs::read_dir(current_path).await {
                Ok(mut read_dir) => loop {
                    match read_dir.next().await {
                        Some(Ok(entry)) => {
                            if Self::is_image_file(&entry.path()) {
                                image_paths.push(entry.path().into())
                            } else if let Ok(entry_type) = entry.file_type().await {
                                if entry_type.is_dir() {
                                    paths.push(entry.path().into())
                                }
                            }
                        }
                        Some(Err(error)) => {
                            Log::handle_error(&error);
                            break;
                        }
                        None => {
                            break;
                        }
                    }
                },
                Err(error) => {
                    Log::handle_error(&error);
                    return Err(error);
                }
            }
        }
        Ok(image_paths)
    }
}

impl ImageSourceTrait for ImageSourceFolder {
    fn id(&self) -> Uuid {
        self.id
    }
    fn name(&self) -> &str {
        &self.name
    }

    fn check(&self) -> &ImageSourceCheck {
        &self.check
    }

    fn set_check(&mut self, check: ImageSourceCheck) {
        self.check = check;
    }

    async fn check_source(&self) -> ImageSourceCheck {
        Self::find_image_files_in_directory(&self.path)
            .await
            .map(|paths| ImageSourceCheck {
                image_count: paths.len(),
                status: ImageSourceStatus::Valid,
            })
            .unwrap_or_else(|error| ImageSourceCheck {
                image_count: 0,
                status: ImageSourceStatus::Error(error.to_string()),
            })
    }

    async fn load_image(&self, index: usize) -> anyhow::Result<Image> {
        let images = Self::find_image_files_in_directory(&self.path).await?;
        Ok(slint::Image::load_from_path(&images[index])?)
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

impl TryFrom<ImageSource> for sg::EditSourceFolderData {
    type Error = anyhow::Error;
    fn try_from(value: ImageSource) -> Result<Self, Self::Error> {
        let ImageSource::Folder(folder) = value;
        Ok(folder.into())
    }
}
