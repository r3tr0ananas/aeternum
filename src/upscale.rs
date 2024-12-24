use core::fmt;
use std::{env, fmt::{Display, Formatter}, io::{BufRead, BufReader}, path::PathBuf, process::Stdio, sync::{Arc, Mutex}, thread, time::Duration};
use which::which;
use strum_macros::EnumIter;
use std::process::Command;

use crate::{error::Error, image::Image, notifier::NotifierAPI};

#[derive(Debug, EnumIter, PartialEq, Clone, Copy)]
pub enum Models {
    RealEsrganX4Plus,
    RealEsrganX4PlusAnime
}

impl Display for Models {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Models::RealEsrganX4Plus => write!(f, "Real-Esrgan"),
            Models::RealEsrganX4PlusAnime => write!(f, "Real-Esrgan Anime")
        }
    }
}


impl Models {
    pub fn file_name(&self) -> String {
        match self {
            Models::RealEsrganX4Plus => "realesrgan-x4plus".to_string(),
            Models::RealEsrganX4PlusAnime => "realesrgan-x4plus-anime".to_string()
        }
    }
}

#[derive(Clone)]
pub struct UpscaleOptions {
    pub scale: i32,
    pub model: Models,
}

pub struct Upscale {
    pub options: UpscaleOptions,
    pub upscaling: bool,

    cli: PathBuf,
    upscaling_arc: Arc<Mutex<bool>>
}

impl Default for UpscaleOptions {
    fn default() -> Self {
        Self {
            scale: 4,
            model: Models::RealEsrganX4Plus,
        }
    }
}

impl Upscale {
    pub fn new() -> Result<Self, Error> {
        let tool_name = "upscayl-bin";
        let executable_path = env::current_exe().expect("Failed to get the current executable's path");
        let tool_path = executable_path.with_file_name(tool_name);

        if tool_path.exists() {
            return Ok(Self {
                options: UpscaleOptions::default(),
                upscaling: false,

                cli: tool_path,
                upscaling_arc: Arc::new(false.into())
            })
        }

        match which(tool_name) {
            Ok(path) => {
                Ok(Self {
                    options: UpscaleOptions::default(),
                    upscaling: false,

                    cli: path,
                    upscaling_arc: Arc::new(false.into())
                })
            },
            Err(err) => {
                let err_mesage = err.to_string();

                Err(Error::UpscaylNotInPath(Some(err_mesage)))
            }
        }
    }

    pub fn update(&mut self) {
        if let Ok(value) = self.upscaling_arc.try_lock() {
            self.upscaling = value.clone();
        }
    }
    
    fn reset(&mut self) {
        self.upscaling = false;
        self.upscaling_arc = Arc::new(false.into());
    }

    pub fn upscale(&mut self, image: Image, notifier: &mut NotifierAPI) {
        self.reset();

        let path = &image.path;

        let out = path.with_file_name(
            format!(
                "{}_{}_x{}.{}", 
                path.file_stem().unwrap().to_string_lossy(), 
                self.options.model.file_name(), 
                self.options.scale,
                path.extension().unwrap().to_string_lossy()
            )
        );

        let path = path.clone();
        let cli = self.cli.clone();
        let upscaling_arc = self.upscaling_arc.clone();
        let mut notifier_arc = notifier.clone();
        let options = self.options.clone();

        let upscale_stuff = move || {
            notifier_arc.set_loading(Some("Initializing command...".into()));

            let mut upscale_command = Command::new(cli.to_string_lossy().to_string());

            let cmd = upscale_command
                .args([
                    "-i",
                    path.to_str().unwrap(),
                    "-o",
                    out.to_str().unwrap(),
                    "-n",
                    &options.model.file_name(),
                    "-s",
                    &options.scale.to_string()
                ])
                .stderr(Stdio::piped()) // why do you output to stderr :woe: ~ Ananas
                .spawn();

            match cmd {
                Ok(mut child) => {
                    if let Some(stderr) = child.stderr.take() {
                        let reader = BufReader::new(stderr);
                
                        for line in reader.lines() {
                            match line {
                                Ok(output) => {
                                    if output.as_bytes()[0].is_ascii_digit() {
                                        notifier_arc.set_loading(Some(format!("Processing: {}", output)));
                                    }
                                },
                                Err(e) => {
                                    let error = Error::FailedToUpscaleImage(Some(e.to_string()), "Failed to read output".to_string());
                                    notifier_arc.toasts.lock().unwrap()
                                        .toast_and_log(error.into(), egui_notify::ToastLevel::Error)
                                        .duration(Some(Duration::from_secs(10)));
                                },
                            }
                        }
                    }

                    let _ = child.wait_with_output();
                },
                Err(error) => {
                    let error = Error::FailedToUpscaleImage(Some(error.to_string()), "Failed to spawn child process.".to_string());

                    notifier_arc.toasts.lock().unwrap()
                        .toast_and_log(error.into(), egui_notify::ToastLevel::Error)
                        .duration(Some(Duration::from_secs(10)));
                }
            }

            let mut upscaled = upscaling_arc.lock().unwrap();

            *upscaled = true;
            notifier_arc.unset_loading();
        };

        thread::spawn(upscale_stuff);
    }
}