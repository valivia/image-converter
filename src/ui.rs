use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::channel,
        Arc,
    },
    thread,
};

use eframe::egui;

use crate::{
    components::resize::resize_input,
    process::convert_images,
    structs::{
        file_type::{EncodingOptions, JpegSettings, WebpSettings},
        settings::{self, Settings},
    },
    types::{Message, Progress},
};

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
    receiver: Option<std::sync::mpsc::Receiver<Message>>,

    // Messages
    messages: Vec<String>,

    progress: Option<Progress>,
}

impl Default for App {
    fn default() -> Self {
        Self {
            settings: Settings::default(),

            page: Page::Encoding,

            // Communication
            stop_flag: Arc::new(AtomicBool::new(false)),
            receiver: None,
            messages: Vec::new(),

            progress: None,
        }
    }
}

impl App {
    fn stop_processing(&mut self) {
        self.stop_flag.store(true, Ordering::Relaxed);
    }

    fn handle_completion(&mut self) {
        self.receiver = None;
        self.progress = None;
        self.stop_flag.store(false, Ordering::Relaxed);
    }

    fn start_processing(&mut self) {
        self.stop_flag.store(false, Ordering::Relaxed);
        let (sender, receiver) = channel::<Message>();
        self.receiver = Some(receiver);

        self.messages.clear();

        let settings = self.settings.clone();
        let stop_flag = Arc::clone(&self.stop_flag);

        thread::spawn(move || {
            convert_images(sender, stop_flag, settings);
        });
    }

    fn handle_messages(&mut self) {
        if let Some(receiver) = &self.receiver {
            if let Ok(received) = receiver.try_recv() {
                let received = match received {
                    Message::Warning(msg) => format!("⚠️: {}", msg),
                    Message::Failed(msg) => {
                        self.handle_completion();
                        format!("⛔: {}", msg)
                    }
                    Message::Progress(progress) => {
                        println!("Progress: {}/{}", progress.success, progress.total);
                        self.progress = Some(progress);
                        return;
                    }
                    Message::Message(msg) => msg,
                    Message::Completed => {
                        let message = match self.stop_flag.load(Ordering::Relaxed) {
                            true => "Stopped",
                            false => "Completed",
                        };
                        self.handle_completion();
                        message.to_string()
                    }
                };
                self.messages.push(received);
                if self.messages.len() > 20 {
                    self.messages.remove(0);
                }
            }
        }
    }

    // Pages
    fn home_page(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            ui.label(self.messages.join("\n"));
        });
    }

    fn export_page(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            // Name extension
            ui.horizontal(|ui| {
                ui.label("Name extension");
                ui.text_edit_singleline(
                    self.settings.name_extension.get_or_insert_with(String::new),
                );
            });

            // Exif
            ui.add(egui::Checkbox::new(
                &mut self.settings.keep_exif,
                "Keep EXIF data",
            ));
        });
    }

    fn encoding_page(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
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
                    ui.add(egui::Checkbox::new(&mut settings.lossless, "Lossless"));

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
        resize_input(ui, &mut self.settings);
    }

    fn about_page(&mut self, ui: &mut egui::Ui) {
        ui.label("About page");
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // State
        self.handle_messages();

        let percentage = match &self.progress {
            Some(progress) => (progress.success + progress.failed) as f32 / progress.total as f32,
            None => 1.0,
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

                    if self.progress.is_some() {
                        ui.label(format!("{:.0}%", percentage * 100.0));
                    }
                });

                ui.add_space(10.0); // Optional spacing
            });

            ui.add_space(8.0);

            ui.add(egui::ProgressBar::new(percentage).desired_height(8.0));

            ui.add_space(16.0);

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
