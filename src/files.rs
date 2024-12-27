use std::path::PathBuf;

use eframe::egui::{self, ImageSource};
use rfd::FileDialog;

use crate::{image::Image, upscale::UpscaleOptions, Error};

pub fn get_aeternum_image<'a>() -> ImageSource<'a> {
    egui::include_image!("../assets/image.png")
}

pub fn select_image() -> Result<Image, Error> {
    let image_path = FileDialog::new()
        .add_filter("images", &["png", "jpeg", "jpg", "webp"])
        .pick_file();

    let image_or_error = match image_path {
        Some(path) => {
            if !path.exists() {
                Err(
                    Error::FileNotFound(
                        None,
                        path,
                        "The file picked in the file selector does not exist!".to_string()
                    )
                )
            } else {
                Image::from_path(path)
            }
        },
        None => Err(Error::NoFileSelected(None))
    };

    image_or_error
}

pub fn save_image(image: &Image, options: &UpscaleOptions) -> Result<PathBuf, Error> {
    let binding = image.create_output(
        &options.scale, 
        options.model.as_ref().unwrap()
    );

    let file_name = binding.file_name().unwrap().to_str().unwrap();

    match FileDialog::new().set_file_name(file_name).save_file() {
        Some(path) => {
            Ok(path)
        },
        None => Err(Error::NoFileSelected(None))
    }
}