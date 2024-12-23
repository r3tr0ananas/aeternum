use core::fmt;
use std::fmt::{Display, Formatter};
use which::which;
use strum_macros::EnumIter;
use std::process::Command;

use crate::{error::Error, image::Image};

#[derive(Debug, EnumIter, PartialEq, Clone, Copy)]
pub enum Models {
    RealEsrganX4Plus,
    RealEsrnetX4Plus,
    RealEsrganX4PlusAnime
}

impl Display for Models {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Models::RealEsrganX4Plus => write!(f, "realesrgan-x4plus"),
            Models::RealEsrnetX4Plus => write!(f, "realesrnet-x4plus"),
            Models::RealEsrganX4PlusAnime => write!(f, "realesrgan-x4plus-anime")
        }
    }
}

pub struct UpscaleOptions {
    pub scale: i32,
    pub model: Models
}

pub struct Upscale {
    pub options: UpscaleOptions
}

impl Default for UpscaleOptions {
    fn default() -> Self {
        Self {
            scale: 4,
            model: Models::RealEsrganX4Plus
        }
    }
}

impl Upscale {
    pub fn new() -> Result<Self, Error> {
        match which("realesrgan-ncnn-vulkan") {
            Ok(_) => {
                Ok(Self {
                    options: UpscaleOptions::default()
                })
            },
            Err(err) => {
                let err_mesage = err.to_string();

                Err(Error::RealEsrganNotInPath(Some(err_mesage)))
            }
        }
    }

    pub fn upscale(&self, image: Image) -> Result<(), Error> {
        let path = &image.path;

        let out = if let Some(stem) = path.file_stem() {
            path.with_file_name(
                format!(
                    "{}_{}_x{}.{}", 
                    stem.to_string_lossy(), 
                    self.options.model, 
                    self.options.scale,
                    path.extension().unwrap().to_string_lossy()
                )
            )
        } else {
            return Err(Error::FailedToUpscaleImage(None, "Failed to modify output path".to_string()))
        };

        let mut upscale_command = Command::new("realesrgan-ncnn-vulkan");

        upscale_command
            .args([
                "-i",
                path.to_str().unwrap(),
                "-o",
                out.to_str().unwrap(),
                "-n",
                &self.options.model.to_string(),
                "-s",
                &self.options.scale.to_string()
            ]);

        match upscale_command.status() {
            Ok(_) => Ok(()),
            Err(error) => {
                Err(Error::FailedToUpscaleImage(Some(error.to_string()), "Failed to spawn realesrgan-ncnn-vulkan".to_string()))
            }
        }
    }
}