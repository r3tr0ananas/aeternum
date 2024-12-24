use std::{env, io::{BufRead, BufReader}, path::PathBuf, process::Stdio, sync::{Arc, Mutex}, thread, time::Duration};
use egui_notify::ToastLevel;
use which::which;
use std::process::Command;

use crate::{error::Error, image::Image, notifier::NotifierAPI};

#[derive(Debug, Clone, PartialEq)]
pub struct Model {
    path: PathBuf,
    folder: PathBuf,

    pub name: String
}

#[derive(Clone)]
pub struct UpscaleOptions {
    pub scale: i32,
    pub model: Option<Model>,
}

pub struct Upscale {
    pub options: UpscaleOptions,
    pub upscaling: bool,
    pub models: Vec<Model>,

    cli: PathBuf,
    upscaling_arc: Arc<Mutex<bool>>
}

impl Default for UpscaleOptions {
    fn default() -> Self {
        Self {
            scale: 4,
            model: None
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
                models: Vec::new(),

                cli: tool_path,
                upscaling_arc: Arc::new(false.into())
            })
        }

        match which(tool_name) {
            Ok(path) => {
                Ok(Self {
                    options: UpscaleOptions::default(),
                    upscaling: false,
                    models: Vec::new(),

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

    pub fn init(&mut self, custom_path: Option<String>) -> Result<(), Error> {
        match custom_path {
            Some(path) => {
                let path = PathBuf::from(path);

                if path.exists() {
                    self.get_models(path.clone());
                } else {
                    return Err(Error::NoModels(Some("Custom folder doesn't exist.".to_string()), path))
                }
            },
            None => {} 
        }

        let models_folder = self.cli.with_file_name("models");

        if !models_folder.exists() && self.models.is_empty() {
            return Err(Error::ModelsFolderNotFound(Some("Folder doesn't exist".to_string()), models_folder))
        }

        self.get_models(models_folder.clone());

        if self.models.is_empty() {
            return Err(Error::NoModels(Some("Vector is empty.".to_string()), models_folder))
        }

        self.options.model = Some(
            self.models.first().unwrap().clone()
        );

        Ok(())
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
                self.options.model.as_ref().unwrap().name, 
                self.options.scale,
                path.extension().unwrap().to_string_lossy()
            )
        );

        if out.exists() {
            notifier.toasts.lock().unwrap()
                .toast_and_log("Upscaled image already exists.".into(), ToastLevel::Info)
                .duration(Some(Duration::from_secs(10)));

            return;
        }

        let path = path.clone();
        let cli = self.cli.clone();
        let upscaling_arc = self.upscaling_arc.clone();
        let mut notifier_arc = notifier.clone();
        let options = self.options.clone();

        let upscale_stuff = move || {
            notifier_arc.set_loading(Some("Initializing command...".into()));

            let mut upscale_command = Command::new(cli.to_string_lossy().to_string());

            let model = &options.model.unwrap();

            let cmd = upscale_command
                .args([
                    "-i",
                    path.to_str().unwrap(),
                    "-o",
                    out.to_str().unwrap(),
                    "-m",
                    &model.folder.to_string_lossy(),
                    "-n",
                    &model.name,
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
                                    let out_bytes = output.as_bytes();

                                    if !out_bytes.is_empty() && out_bytes[0].is_ascii_digit() {
                                        notifier_arc.set_loading(Some(format!("Processing: {}", output)));
                                    }
                                },
                                Err(error) => {
                                    let error = Error::FailedToUpscaleImage(Some(error.to_string()), "Failed to read output".to_string());
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

    fn get_models(&mut self, folder_path: PathBuf) {
        let gl = format!("{}{}*.bin", folder_path.to_string_lossy(), std::path::MAIN_SEPARATOR_STR);

        for entry in glob::glob(&gl).unwrap() {
            match entry {
                Ok(entry_path) => {
                    let param_file = entry_path.with_file_name(
                        format!("{}.param", entry_path.file_stem().unwrap().to_string_lossy())
                    );

                    if param_file.exists() { // 
                        self.models.push(
                            Model {
                                path: entry_path.clone(),
                                folder: folder_path.clone(),

                                name: entry_path.file_stem().unwrap().to_string_lossy().to_string()
                            }
                        );
                    }
                },
                Err(err) => {
                    panic!("Error while getting models: {}", err);
                }
            }
        }
    }
}