// https://github.com/cloudy-org/roseate/blob/main/src/error.rs
use std::{fmt::{self, Display, Formatter}, path::PathBuf};

type AE = Option<String>;

#[derive(Debug, Clone)]
pub enum Error {
    FileNotFound(AE, PathBuf, String),
    NoFileSelected(AE),
    FailedToUpscaleImage(AE, String),
    UpscaylNotInPath(AE),
    FailedToInitImage(AE, PathBuf, String),
    ImageFormatNotSupported(AE, String),
}

impl Error {
    pub fn message(&self) -> String {
        format!("{}", self)
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Error::FileNotFound(_, path, detail) => {
                let message = format!(
                    "The file path given '{}' does not exist! {}",
                    path.to_string_lossy(),
                    detail
                );

                write!(f, "{}", message)
            },
            Error::NoFileSelected(_) => write!(
                f, "No file was selected in the file dialogue!"
            ),
            Error::FailedToUpscaleImage(_, reason) => write!(
                f,
                "Failed to upscale the image. \
                \n\nReason: {}",
                reason
            ),
            Error::FailedToInitImage(_, path, reason) => write!(
                f,
                "Failed to initialize the image ({})! Reason: {}",
                path.file_name().unwrap().to_string_lossy(),
                reason
            ),
            Error::UpscaylNotInPath(..) => write!(
                f, "upscayl-bin isn't in your path. Install it: https://github.com/upscayl/upscayl-ncnn"
            ),
            Error::ImageFormatNotSupported(_, image_format) => write!(
                f, "The image format '{}' is not supported!", image_format
            ),
        }
    }
}