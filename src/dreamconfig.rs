use std::sync::{Arc, RwLock};

use crate::app_settings::{Settings, SettingsRaw};
use crate::dreams::*;
use anyhow::Result;

// What is shown on main window right now
#[derive(Debug, PartialEq)]
enum ActivePanel {
    Generic,
    About,
    Dream(DreamId), // Settings of a dream with given ID
}

pub struct DreamConfigApp {
    settings: Settings,
    active_panel: ActivePanel,
    zoo: Zoo,
}

impl eframe::App for DreamConfigApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // unwrapping settings for easier access
        let a_st = &self.settings.clone();
        let st = a_st.write().unwrap();

        //action flags
        let mut save = false;
        let mut cancel = false;
        egui::TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.with_layout(
                    egui::Layout::right_to_left(egui::Align::Center),
                    |ui| {
                        if ui.button("Close").clicked() {
                            cancel = true;
                        };
                        if ui.button("Save").clicked() {
                            save = true;
                        };
                    },
                );
            });
        });
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.selectable_value(
                        &mut self.active_panel,
                        ActivePanel::Generic,
                        "Settings",
                    );
                    ui.selectable_value(
                        &mut self.active_panel,
                        ActivePanel::About,
                        "About",
                    );
                    for dream in self.zoo.iter() {
                        ui.selectable_value(
                            &mut self.active_panel,
                            ActivePanel::Dream(dream.read().unwrap().id()),
                            dream.read().unwrap().name(),
                        );
                    }
                });
                ui.separator();
                ui.vertical(|ui| match &self.active_panel {
                    ActivePanel::Generic => draw_generic(ui),
                    ActivePanel::About => draw_about(ui),
                    ActivePanel::Dream(id) => {
                        let dream = select_dream_by_id(&self.zoo, id).unwrap();
                        dream.write().unwrap().config_egui(ui);
                    }
                });
            });
        });

        // Lock settings back
        drop(st);

        // Perform deferred actions
        if save {
            self.save();
        }
        if cancel {
            self.cancel(ctx);
        }
    }
}

impl DreamConfigApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Result<Self> {
        // Load settings from file
        let settings =
            Arc::new(RwLock::new(SettingsRaw::read_from_file_default()?));
        let zoo = build_zoo(settings.clone());
        for dream in zoo.iter() {
            dream.write().unwrap().prepare();
        }
        Ok(Self { settings, active_panel: ActivePanel::Generic, zoo })
    }

    fn save(&mut self) {
        for dream in self.zoo.iter() {
            dream.read().unwrap().store();
        }
        self.settings.write().unwrap().write_to_file_default().unwrap();
    }

    fn cancel(&mut self, ctx: &egui::Context) {
        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
    }
}
fn draw_generic(ui: &mut egui::Ui) {
    ui.heading("Dream Spinner");
}

fn draw_about(ui: &mut egui::Ui) {
    ui.vertical(|ui| {
        ui.heading("Dream Spinner");
        ui.separator();
        powered_by_egui_and_eframe(ui);
    });
}

fn powered_by_egui_and_eframe(ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        ui.label("Powered by ");
        ui.hyperlink_to("egui", "https://github.com/emilk/egui");
        ui.label(" and ");
        ui.hyperlink_to(
            "eframe",
            "https://github.com/emilk/egui/tree/master/crates/eframe",
        );
        ui.label(".");
    });
}
