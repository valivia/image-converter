use std::fmt::Write;
use std::path::PathBuf;
use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::channel,
        Arc,
    },
    thread,
};

use eframe::egui;

use crate::structs::update::Update;
use crate::util::files::get_files;
use crate::{
    components::resize::resize_input,
    process::convert_images,
    structs::{
        file_type::{EncodingOptions, JpegSettings, WebpSettings},
        settings::{ResizeOptions, Settings},
    },
};

const FORBIDDEN_CHARS: &[char] = &['/', '\\', ':', '*', '?', '"', '<', '>', '|'];
const LOG_LENGTH: usize = 18;

#[derive(PartialEq, Clone, Copy)]
enum Page {
    Home,
    Encoding,
    Export,
    Resize,
    About,
}

pub struct App {
    settings: Settings,

    page: Page,

    // Communication
    stop_flag: Arc<AtomicBool>,
    receiver: Option<std::sync::mpsc::Receiver<Update>>,

    // Messages
    messages: Vec<String>,

    files: Vec<PathBuf>,
    success: Vec<PathBuf>,
    failed: Vec<PathBuf>,
}

impl Default for App {
    fn default() -> Self {
        Self {
            settings: Settings::default(),

            page: Page::Home,

            // Communication
            stop_flag: Arc::new(AtomicBool::new(false)),
            receiver: None,
            messages: Vec::new(),

            files: get_files().unwrap(),
            success: Vec::new(),
            failed: Vec::new(),
        }
    }
}

impl App {
    fn stop_processing(&mut self) {
        self.stop_flag.store(true, Ordering::Relaxed);
    }

    fn handle_completion(&mut self) {
        self.receiver = None;
        self.success.clear();
        self.failed.clear();
        self.stop_flag.store(false, Ordering::Relaxed);
    }

    fn start_processing(&mut self) {
        self.stop_flag.store(false, Ordering::Relaxed);
        let (sender, receiver) = channel::<Update>();
        self.receiver = Some(receiver);

        self.messages.clear();

        let settings = self.settings.clone();
        let files = self.files.clone();
        let stop_flag = Arc::clone(&self.stop_flag);

        thread::spawn(move || {
            convert_images(sender, stop_flag, files, settings);
        });
    }

    fn handle_messages(&mut self) {
        if let Some(receiver) = &self.receiver {
            if let Ok(received) = receiver.try_recv() {
                let received = match received {
                    Update::StartProcessing(path) => {
                        let file_name = path.file_name().unwrap().to_str().unwrap();
                        format!("Processing '{}'", file_name)
                    }
                    Update::FinishedProcessing(path, success, duration) => {
                        let file_name = path.file_name().unwrap().to_str().unwrap();
                        let message = if success {
                            self.success.push(path.clone());
                            format!("Processed '{}'", file_name)
                        } else {
                            self.failed.push(path.clone());
                            format!("Failed to process '{}'", file_name)
                        };
                        format!("{} ({:#?})", message, duration)
                    }
                    Update::Message(msg) => msg,
                    Update::QueueCompleted(duration) => {
                        let message = match self.stop_flag.load(Ordering::Relaxed) {
                            true => "Stopped".to_string(),
                            false => format!("Completed in {:#?}", duration),
                        };
                        self.handle_completion();
                        message.to_string()
                    }
                };

                self.push_message(received);
            }
        }
    }

    fn push_message(&mut self, message: String) {
        self.messages.push(message);

        if self.messages.len() > LOG_LENGTH {
            self.messages.remove(0);
        }
    }

    // Pages
    fn home_page(&mut self, ui: &mut egui::Ui) {
        // Encoding
        let mut summary = String::new();

        write!(
            summary,
            "Your images will be saved as {{name}}{}.{}",
            self.settings.name_extension.as_deref().unwrap_or(""),
            self.settings.encoding_options
        )
        .unwrap();

        // Resize options
        let resize_options = match self.settings.resize_options {
            ResizeOptions::None => "with their original resolution".to_string(),
            ResizeOptions::Largest(size) => {
                format!("and will be resized to {}px on the largest dimension", size)
            }
            ResizeOptions::Smallest(size) => {
                format!(
                    "and will be resized to {}px on the smallest dimension",
                    size
                )
            }
            ResizeOptions::Exact(width, height) => {
                format!("and will be resized to {}px by {}px", width, height)
            }
        };

        write!(summary, ", {}.", resize_options).unwrap();

        ui.heading("Summary");
        ui.label(summary);

        ui.add_space(8.0);

        ui.heading("Logs");
        ui.label(self.messages.join("\n"));
    }

    fn export_page(&mut self, ui: &mut egui::Ui) {
        ui.heading("Export options");
        ui.horizontal(|ui| {
            ui.label("Name extension");
            ui.text_edit_singleline(self.settings.name_extension.get_or_insert_with(String::new));
        });

        // Remove forbidden characters
        if let Some(extension) = self.settings.name_extension.take() {
            let mut cleaned = extension.trim().to_string();

            for c in FORBIDDEN_CHARS {
                cleaned.retain(|x| x != *c);
            }

            self.settings.name_extension = match cleaned.is_empty() {
                true => None,
                false => Some(cleaned),
            };
        }

        // Exif
        // ui.add(egui::Checkbox::new(
        //     &mut self.settings.keep_exif,
        //     "Keep EXIF data",
        // ));
    }

    fn encoding_page(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            ui.heading("Encoding options");
            // Type
            egui::ComboBox::from_label("Choose export type")
                .selected_text(format!("{}", self.settings.encoding_options))
                .show_ui(ui, |ui| {
                    ui.selectable_value(
                        &mut self.settings.encoding_options,
                        EncodingOptions::WebP(WebpSettings::default()),
                        "WebP",
                    );
                    ui.selectable_value(
                        &mut self.settings.encoding_options,
                        EncodingOptions::Avif(Default::default()),
                        "AVIF",
                    );
                    ui.selectable_value(
                        &mut self.settings.encoding_options,
                        EncodingOptions::Jpeg(JpegSettings::default()),
                        "JPEG",
                    );
                });

            match &mut self.settings.encoding_options {
                EncodingOptions::Avif(settings) => {
                    // Lossless
                    // ui.add(egui::Checkbox::new(&mut settings.lossless, "Lossless"));

                    // Quality
                    ui.add(egui::Slider::new(&mut settings.quality, 5..=100).text("Quality"));

                    // Speed
                    ui.add(egui::Slider::new(&mut settings.speed, 1..=10).text("Speed"));
                }
                EncodingOptions::WebP(settings) => {
                    // Lossless
                    ui.add(egui::Checkbox::new(&mut settings.lossless, "Lossless"));

                    // Quality
                    ui.add_enabled(
                        !settings.lossless,
                        egui::Slider::new(&mut settings.quality, 5..=100).text("Quality"),
                    );
                }

                EncodingOptions::Jpeg(settings) => {
                    // Quality
                    ui.add(egui::Slider::new(&mut settings.quality, 5..=100).text("Quality"));
                }
            }
        });
    }

    fn resize_page(&mut self, ui: &mut egui::Ui) {
        ui.heading("Resize options");
        resize_input(ui, &mut self.settings);
    }

    fn about_page(&mut self, ui: &mut egui::Ui) {
        ui.heading("About");
        ui.label("Simple bulk image converter and resizer written in rust.");
        ui.with_layout(egui::Layout::left_to_right(egui::Align::Min), |ui| {
            ui.label("Made with <3 by");
            ui.hyperlink_to("Owlive", "https://owlive.eu/");
        });
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // State
        self.handle_messages();

        let total_processed = self.success.len() + self.failed.len();

        let percentage = if total_processed > 0 {
            total_processed as f32 / self.files.len() as f32
        } else {
            0.0
        };

        // Render
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add_space(4.0);

            // Nav
            ui.horizontal(|ui| {
                for page in &[
                    Page::Home,
                    Page::Resize,
                    Page::Encoding,
                    Page::Export,
                    Page::About,
                ] {
                    let label = match page {
                        Page::Home => "Home",
                        Page::Resize => "Resize",
                        Page::Encoding => "Encoding",
                        Page::Export => "Export",
                        Page::About => "About",
                    };

                    if ui.selectable_label(self.page == *page, label).clicked() {
                        self.page = *page;
                    }
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Max), |ui| {
                    ui.add_space(10.0);
                    if self.receiver.is_none() {
                        // Start button
                        if ui.button("Run").clicked() {
                            self.start_processing();
                        }
                    } else {
                        // Stop button (disabled if stop_flag is set)
                        ui.add_enabled_ui(!self.stop_flag.load(Ordering::Relaxed), |ui| {
                            if ui.button("Stop").clicked() {
                                self.stop_processing();
                            }
                        });
                    }

                    if total_processed > 0 {
                        ui.label(format!("{:.0}%", percentage * 100.0));
                    }
                });
            });

            ui.add_space(8.0);

            ui.add(egui::ProgressBar::new(percentage).desired_height(8.0));

            ui.add_space(8.0);

            // Content
            match self.page {
                Page::Home => self.home_page(ui),
                Page::Encoding => self.encoding_page(ui),
                Page::Resize => self.resize_page(ui),
                Page::Export => self.export_page(ui),
                Page::About => self.about_page(ui),
            }
        });
    }
}
