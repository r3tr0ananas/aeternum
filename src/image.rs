use std::path::PathBuf;

use crate::Error;

#[derive(Clone)]
pub struct Image {
    pub path: PathBuf,
}

impl Image {
    pub fn from_path(path: PathBuf) -> Result<Self, Error> {
        if let Some(extension) = path.extension() {
            let allowed_extensions = vec!["png", "jpeg", "jpg", "webp"];
            let extension_string = extension.to_string_lossy().to_string();
    
            if !allowed_extensions.iter().any(|e| extension_string.contains(e)) {
                Err(Error::ImageFormatNotSupported(None, extension_string.clone()))
            } else {
                Ok(Self {
                    path,
                })
            }
        } else {
            Err(Error::ImageFormatNotSupported(None, "".to_string()))
        }
    }
}