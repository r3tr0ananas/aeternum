use cirrus_theming::v1::{Colour, Theme};
use eframe::egui::{self, Align, Color32, Context, CursorIcon, Frame, Layout, Margin, Rect, Shadow, Slider, Stroke, Style, TextStyle, Vec2};
use egui_notify::ToastLevel;
use strum::IntoEnumIterator;
use std::time::Duration;

use crate::{files, notifier::NotifierAPI, upscale::{Models, Upscale}, windows::about::AboutWindow, Image};

pub struct Aeternum<'a> {
    theme: Theme,
    image: Option<Image>,
    about_box: AboutWindow<'a>,
    notifier: NotifierAPI,
    upscale: Upscale
}

impl<'a> Aeternum<'a> {
    pub fn new(image: Option<Image>, theme: Theme, notifier: NotifierAPI, upscale: Upscale) -> Self {
        let about_box = AboutWindow::new();

        Self {
            image,
            theme,
            notifier,
            about_box,
            upscale
        }
    }

    fn set_app_style(&self, ctx: &Context) {
        let mut custom_style = Style {
            override_text_style: Some(TextStyle::Monospace),
            ..Default::default()
        };

        custom_style.visuals.panel_fill = Color32::from_hex(
            &self.theme.primary_colour.hex_code
        ).unwrap();

        // Window styling.
        custom_style.visuals.window_highlight_topmost = false;

        custom_style.visuals.window_fill = Color32::from_hex(
            &self.theme.secondary_colour.hex_code
        ).unwrap();
        custom_style.visuals.window_stroke = Stroke::new(
            1.0,
            Color32::from_hex(&self.theme.third_colour.hex_code).unwrap()
        );
        custom_style.visuals.window_shadow = Shadow::NONE;

        custom_style.visuals.widgets.inactive.bg_fill =
            Color32::from_hex(
                &self.theme.primary_colour.hex_code
            ).unwrap();

        // Text styling.
        custom_style.visuals.override_text_color = Some(
            Color32::from_hex(
                match self.theme.is_dark {
                    true => "#b5b5b5",
                    false => "#3b3b3b"
                }
            ).unwrap()
        );

        ctx.set_style(custom_style);
    }

    fn draw_dotted_line(&self, ui: &egui::Painter, pos: &[egui::Pos2]) {
        ui.add(
            egui::Shape::dashed_line(
                pos, 
                Stroke {
                    width: 2.0,
                    color: Color32::from_hex(
                        &self.theme.accent_colour.as_ref()
                            .unwrap_or(&Colour {hex_code: "e05f78".into()}).hex_code
                    ).unwrap()
                },
                10.0, 
                10.0
            )
        );
    }
}

impl eframe::App for Aeternum<'_> {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        self.set_app_style(ctx);

        self.about_box.handle_input(ctx);
        self.upscale.update();

        egui::CentralPanel::default().show(ctx, |ui| {
            let window_rect = ctx.input(|i: &egui::InputState| i.screen_rect());

            self.notifier.update(ctx);
            self.about_box.update(ctx);

            if self.image.is_none() {
                // Collect dropped files.
                ctx.input(|i| {
                    let dropped_files = &i.raw.dropped_files;

                    if !dropped_files.is_empty() {
                        let path = dropped_files.first().unwrap()
                            .path
                            .as_ref()
                            .unwrap();

                        let image = match Image::from_path(path.clone()) {
                            Ok(value) => value,
                            Err(error) => {
                                self.notifier.toasts.lock().unwrap().toast_and_log(
                                    error.into(), ToastLevel::Error
                                );
                                return;
                            }
                        };

                        self.image = Some(image.clone());
                    }
                });

                ui.centered_and_justified(|ui| {
                    let image_width: f32 = 145.0;
                    let file_is_hovering = !ctx.input(|i| i.raw.hovered_files.is_empty());

                    let mut aeter_rect = Rect::NOTHING;

                    egui::Frame::default()
                        .outer_margin(
                            Margin::symmetric(
                                (window_rect.width() / 2.0) - image_width / 2.0, 
                                (window_rect.height() / 2.0) - image_width / 2.0
                            )
                        )
                        .show(ui, |ui| {
                            let aeter_response = ui.add(
                                egui::Image::new(files::get_aeternum_image())
                                    .max_width(image_width)
                                    .sense(egui::Sense::click())
                            );

                            aeter_rect = aeter_response.rect;

                            if file_is_hovering {
                                ui.label("You're about to drop a file.");
                            }

                            aeter_response.clone().on_hover_cursor(CursorIcon::PointingHand);

                            if aeter_response.clicked() {
                                let image_result = files::select_image();

                                match image_result {
                                    Ok(image) => {
                                        self.image = Some(image.clone());
                                    },
                                    Err(error) => {
                                        self.notifier.toasts.lock().unwrap()
                                            .toast_and_log(error.into(), ToastLevel::Error)
                                            .duration(Some(Duration::from_secs(5)));
                                    },
                                }
                            }
                        }
                    );

                    if file_is_hovering {
                        let rect = aeter_rect.expand2(
                            Vec2::new(150.0, 100.0)
                        );
                        let painter = ui.painter();

                        let top_right = rect.right_top();
                        let top_left = rect.left_top();
                        let bottom_right = rect.right_bottom();
                        let bottom_left = rect.left_bottom();

                        self.draw_dotted_line(painter, &[top_left, top_right]);
                        self.draw_dotted_line(painter, &[top_right, bottom_right]);
                        self.draw_dotted_line(painter, &[bottom_right, bottom_left]);
                        self.draw_dotted_line(painter, &[bottom_left, top_left]);
                    }
                });

                return;
            }

            let image = self.image.clone().unwrap();

            egui::Grid::new("upscale_options")
                .spacing(Vec2::new(24.0, 20.0))
                .show(ui, |ui| {
                    let image_path = format!("file://{}", image.path.to_string_lossy());

                    ui.add(egui::Image::from_uri(image_path));
                    ui.end_row();

                    ui.label("Model");

                    egui::ComboBox::from_label("Select a model")
                        .selected_text(format!("{}", &self.upscale.options.model.to_string()))
                        .show_ui(ui, |ui| {
                            for model in Models::iter() {
                                ui.selectable_value(&mut self.upscale.options.model, model, model.to_string());
                            }
                        });
                    ui.end_row();
                    
                    ui.label("Scale");
                    ui.add(
                        Slider::new(&mut self.upscale.options.scale, 1..=16)
                    );

                    ui.end_row();

                    let upscale_button = ui.add_enabled(!self.upscale.upscaling, egui::Button::new("Upscale!"));

                    if upscale_button.clicked() {
                        self.upscale.upscale(image, &mut self.notifier);
                    }
                });
        });

        egui::TopBottomPanel::bottom("status_bar")
        .show_separator_line(false)
        .frame(
            Frame::none()
                .outer_margin(Margin {left: 10.0, bottom: 7.0, ..Default::default()})
        ).show(ctx, |ui| {
            if let Ok(loading_status) = self.notifier.loading_status.try_read() {
                if let Some(loading) = loading_status.as_ref() {
                    ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
                        ui.add(
                            egui::Spinner::new()
                                .color(Color32::from_hex("#e05f78").unwrap()) // NOTE: This should be the default accent colour.
                                .size(20.0)
                        );

                        if let Some(message) = &loading.message {
                            ui.label(message);
                        }
                    });
                }
            }
        }
    );
    }
}