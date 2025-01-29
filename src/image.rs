use std::path::PathBuf;
use eframe::egui;
use imagesize::ImageSize;

use crate::{upscale::UpscaleOptions, Error};

#[derive(Clone)]
pub struct Image {
    pub path: PathBuf,
    pub image_size: ImageSize
}

impl Image {
    pub fn from_path(path: PathBuf) -> Result<Self, Error> {
        if let Some(extension) = path.extension() {
            let allowed_extensions = vec!["png", "jpeg", "jpg", "webp"];
            let extension_string = extension.to_string_lossy().to_string().to_lowercase();

            match allowed_extensions.iter().any(|e| extension_string.contains(e)) {
                true => {
                    let image_size = match imagesize::size(&path) {
                        Ok(size) => size,
                        Err(why) => return Err(
                            Error::FailedToInitImage(
                                Some(why.to_string()), 
                                path.clone(), 
                                "Failed to get image size.".to_string()
                            )
                        )
                    };

                    Ok(Self {
                        path,
                        image_size
                    })
                },
                false => Err(Error::ImageFormatNotSupported(None, extension_string.clone())),
            }
        } else {
            Err(Error::ImageFormatNotSupported(None, "".to_string()))
        }
    }

    pub fn create_output(&self, options: &UpscaleOptions) -> PathBuf {
        let model = &options.model.clone().unwrap();
        let extension = &options.output_ext.to_string().to_lowercase();

        let out = self.path.with_file_name(
            format!(
                "{}_{}_x{}.{}", 
                self.path.file_stem().unwrap().to_string_lossy(), 
                model.name, 
                &options.scale,
                extension
            )
        );

        out
    }
}

pub fn load_icon() -> egui::IconData {
    let (icon_rgba, icon_width, icon_height) = {
        let image_bytes = include_bytes!("../assets/aeternum.ico");
        let image = image::load_from_memory(image_bytes)
            .expect("Failed to open icon path")
            .into_rgba8();
        let (width, height) = image.dimensions();
        let rgba = image.into_raw();
        (rgba, width, height)
    };

    egui::IconData {
        rgba: icon_rgba,
        width: icon_width,
        height: icon_height,
    }
}