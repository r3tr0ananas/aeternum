use std::{env, io::{BufRead, BufReader}, path::PathBuf, process::Stdio, sync::{Arc, Mutex}, thread, time::{Duration, Instant}};
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
    pub compression: i32,
    pub model: Option<Model>,
    pub output: Option<PathBuf>
}

pub struct Upscale {
    pub options: UpscaleOptions,
    pub upscaling: bool,
    pub models: Vec<Model>,

    cli_path: PathBuf,
    upscaling_arc: Arc<Mutex<bool>>
}

impl Default for UpscaleOptions {
    fn default() -> Self {
        Self {
            scale: 4,
            compression: 0,
            model: None,
            output: None
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

                cli_path: tool_path,
                upscaling_arc: Arc::new(false.into())
            })
        }

        match which(tool_name) {
            Ok(path) => {
                Ok(Self {
                    options: UpscaleOptions::default(),
                    upscaling: false,
                    models: Vec::new(),

                    cli_path: path,
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

        let models_folder = PathBuf::from("/usr/lib/upscayl/models"); // NOTE: TEMPORARY SOLUTION!
        // NOTE: Also cross-platform support thrown out the window here.
        // TODO: Figure out a solution for windows support, possibly?

        if !models_folder.exists() && self.models.is_empty() {
            return Err(Error::ModelsFolderNotFound(Some("Folder doesn't exist".to_string()), models_folder))
        }

        self.get_models(models_folder.clone());

        if self.models.is_empty() {
            return Err(Error::NoModels(Some("Vector is empty.".to_string()), models_folder))
        }

        Ok(())
    }

    pub fn update(&mut self) {
        if let Ok(value) = self.upscaling_arc.try_lock() {
            self.upscaling = value.clone();
        }
    }

    pub fn reset_options(&mut self) {
        self.options = UpscaleOptions::default();
    }
    
    fn upscaling_reset(&mut self) {
        self.upscaling = false;
        self.upscaling_arc = Arc::new(false.into());
    }

    pub fn upscale(&mut self, image: Image, notifier: &mut NotifierAPI) {
        self.upscaling_reset();

        let path = &image.path;

        let out = match &self.options.output {
            Some(path) => path.clone(),
            None => image.create_output(
                &self.options.scale, 
                self.options.model.as_ref().unwrap()
            ),
        };

        if out.exists() {
            notifier.toasts.lock().unwrap()
                .toast_and_log("Image already exists!".into(), ToastLevel::Info)
                .duration(Some(Duration::from_secs(10)));

            return;
        }

        let path = path.clone();
        let cli_path = self.cli_path.clone();
        let upscaling_arc = self.upscaling_arc.clone();
        let mut notifier_arc = notifier.clone();
        let options = self.options.clone();

        let mut upscaling = self.upscaling_arc.lock().unwrap();
        *upscaling = true;

        let upscale_stuff = move || {
            let now = Instant::now();

            notifier_arc.set_loading(Some("Initializing command...".into()));

            let mut upscale_command = Command::new(cli_path.to_string_lossy().to_string());

            let model = &options.model.unwrap();

            let cmd = upscale_command
                .args([
                    "-i",
                    &path.to_string_lossy(),
                    "-o",
                    &out.to_string_lossy(),
                    "-m",
                    &model.folder.to_string_lossy(),
                    "-n",
                    &model.name,
                    "-s",
                    &options.scale.to_string(),
                    "-c",
                    &options.compression.to_string()
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
                                _ => {}
                            }
                        }
                    }

                    let status = child.wait_with_output();

                    match status {
                        Ok(status) => {
                            if status.status.success() {
                                let upscale_time = now.elapsed().as_secs();

                                notifier_arc.toasts.lock().unwrap()
                                    .toast_and_log(format!("Successfully upscaled image in {} seconds!", upscale_time).into(), ToastLevel::Success)
                                    .duration(Some(Duration::from_secs(10)));
                            } else {
                                let error = Error::FailedToUpscaleImage(
                                    None, 
                                    "Process returned as not successful.".to_string()
                                );
                                notifier_arc.toasts.lock().unwrap()
                                    .toast_and_log(error.into(), ToastLevel::Error)
                                    .duration(Some(Duration::from_secs(10)));
                            }
                        },
                        Err(error) => {
                            let error = Error::FailedToUpscaleImage(
                                Some(error.to_string()), 
                                "Failed to wait for process.".to_string()
                            );

                            notifier_arc.toasts.lock().unwrap()
                                .toast_and_log(error.into(), ToastLevel::Error)
                                .duration(Some(Duration::from_secs(10)));
                        }
                    }
                },
                Err(error) => {
                    let error = Error::FailedToUpscaleImage(Some(error.to_string()), "Failed to spawn child process.".to_string());

                    notifier_arc.toasts.lock().unwrap()
                        .toast_and_log(error.into(), ToastLevel::Error)
                        .duration(Some(Duration::from_secs(10)));
                }
            }

            notifier_arc.unset_loading();

            let mut upscaling = upscaling_arc.lock().unwrap();
            *upscaling = false;
        };

        thread::spawn(upscale_stuff);
    }

    fn get_models(&mut self, folder_path: PathBuf) {
        let glob_bin = folder_path.join("*.bin");
        let gl = glob_bin.to_string_lossy();

        for entry in glob::glob(&gl).unwrap() {
            match entry {
                Ok(entry_path) => {
                    let param_file = entry_path.with_file_name(
                        format!("{}.param", entry_path.file_stem().unwrap().to_string_lossy())
                    );

                    if param_file.exists() {
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