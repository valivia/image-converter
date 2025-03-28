use egui::Ui;

use crate::structs::settings::{ResizeOptions, Settings};

pub fn resize_input(ui: &mut Ui, settings: &mut Settings) {
    egui::ComboBox::from_label("Resize options")
        .selected_text(match &settings.resize_options {
            ResizeOptions::None => "None",
            ResizeOptions::Largest(_) => "Largest",
            ResizeOptions::Exact(_, _) => "Exact",
            ResizeOptions::Smallest(_) => "Smallest",
        })
        .show_ui(ui, |ui| {
            ui.selectable_value(&mut settings.resize_options, ResizeOptions::None, "None");
            ui.selectable_value(
                &mut settings.resize_options,
                ResizeOptions::Largest(0),
                "Largest",
            );
            ui.selectable_value(
                &mut settings.resize_options,
                ResizeOptions::Exact(0, 0),
                "Exact",
            );
            ui.selectable_value(
                &mut settings.resize_options,
                ResizeOptions::Smallest(0),
                "Smallest",
            );
        });

    match settings.resize_options {
        ResizeOptions::None => {}
        ResizeOptions::Largest(mut size) => {
            ui.label("Resize to largest side");
            ui.add(egui::Slider::new(&mut size, 100..=2000).text("Size"));
            settings.resize_options = ResizeOptions::Largest(size);
        }
        ResizeOptions::Exact(mut width, mut height) => {
            let mut width_string = width.to_string();
            let mut height_string = height.to_string();

            ui.label("Resize to exact size");
            ui.horizontal(|ui| {
                ui.label("Width: ");
                if ui.text_edit_singleline(&mut width_string).changed() {
                    width = width_string.parse().unwrap_or(width);
                }
            });

            ui.horizontal(|ui| {
                ui.label("Height: ");
                if ui.text_edit_singleline(&mut height_string).changed() {
                    height = height_string.parse().unwrap_or(height);
                    println!("Height: {:?}", height);
                }
            });
            settings.resize_options = ResizeOptions::Exact(width, height);
        }
        ResizeOptions::Smallest(mut size) => {
            ui.label("Resize to smallest side");
            ui.add(
                egui::Slider::new(&mut size, 100..=2000)
                    .show_value(true)
                    .text("Size"),
            );
            settings.resize_options = ResizeOptions::Smallest(size);
        }
    }
}
