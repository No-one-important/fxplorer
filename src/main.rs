#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

mod fss;
use fss::Fst;

use eframe::egui;
use simple_home_dir::home_dir;
use std::path::MAIN_SEPARATOR;

fn main() -> Result<(), eframe::Error> {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).
    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(640.0, 480.0)),
        ..Default::default()
    };
    eframe::run_native("Fxplorer", options, Box::new(|_cc| Box::<Fst>::default()))
}

impl Default for Fst {
    fn default() -> Self {
        Self::new(home_dir().unwrap().display().to_string())
    }
}

impl eframe::App for Fst {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let mut cont = true;
        while cont {
            match self.rx.try_recv() {
                Ok(x) => self.sub_items.push(x),
                Err(_) => cont = false,
            }
        }

        // panel to display items
        egui::CentralPanel::default().show(ctx, |ui| {
            // search bar and cwd
            ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                let search_icon = "🔎︎";
                if ui.button(search_icon).clicked() {
                    self.search();
                }

                ui.text_edit_singleline(&mut self.search_term);

                ui.with_layout(egui::Layout::left_to_right(egui::Align::TOP), |ui| {
                    ui.heading(&self.current_path);}
                );
            });

            ui.visuals_mut().button_frame = false;
            if ui.button("..").clicked() {
                self.action("..");
            }

            let items = self.sub_items.clone();
            egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
                for item in items {
                    let i: Vec<&str> = item.split(MAIN_SEPARATOR).collect();

                    if ui.button(i[i.len() - 1]).clicked() {
                        self.action(&item);
                    }
                }
            });
        });
    }
}

/*
struct MyApp {
    name: String,
    age: u32,
}

impl Default for MyApp {
    fn default() -> Self {
        Self {
            name: "Arthur".to_owned(),
            age: 42,
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("My egui Application");
            ui.horizontal(|ui| {
                let name_label = ui.label("Your name: ");
                ui.text_edit_singleline(&mut self.name)
                    .labelled_by(name_label.id);
            });
            ui.add(egui::Slider::new(&mut self.age, 0..=120).text("age"));
            if ui.button("Click each year").clicked() {
                self.age += 1;
            }
            ui.label(format!("Hello '{}', age {}", self.name, self.age));
        });
    }
}
*/
