use std::collections::BTreeMap;

use crate::app_settings::{SettingsRaw, ViewportMode, SETTINGS};
use crate::dreams::*;
use anyhow::Result;

// What is shown on main window right now
#[derive(Debug, PartialEq)]
enum ActivePanel {
    Generic,
    Select,
    About,
    Dream(DreamId), // Settings of a dream with given ID
}

pub struct DreamConfigApp {
    active_panel: ActivePanel,
    zoo: BTreeMap<String, ADream>,
    saved_settings: SettingsRaw,
}

impl eframe::App for DreamConfigApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.with_layout(
                    egui::Layout::right_to_left(egui::Align::Center),
                    |ui| {
                        if ui.button("Close").clicked() {
                            self.cancel(ctx);
                        };
                        let have_changes =
                            SETTINGS.read().unwrap().ne(&self.saved_settings);
                        if ui
                            .add_enabled(
                                have_changes,
                                egui::Button::new("Save"),
                            )
                            .clicked()
                        {
                            self.save();
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
                        ActivePanel::Select,
                        "Select Dreams",
                    );
                    ui.selectable_value(
                        &mut self.active_panel,
                        ActivePanel::About,
                        "About",
                    );
                    //ui.separator();
                    // TODO: Can't enable separator: breaks UI for some reason
                    for dream in self.zoo.values() {
                        ui.selectable_value(
                            &mut self.active_panel,
                            ActivePanel::Dream(dream.read().unwrap().id()),
                            dream.read().unwrap().name(),
                        );
                    }
                });
                ui.separator();
                ui.vertical(|ui| match &self.active_panel {
                    ActivePanel::Generic => self.draw_generic(ui),
                    ActivePanel::Select => self.draw_dream_select(ui),
                    ActivePanel::About => self.draw_about(ui),
                    ActivePanel::Dream(id) => {
                        let mut dr = self.zoo[*id].write().unwrap();
                        dr.config_egui(ui);
                        dr.store();
                    }
                });
            });
        });
        for dream in self.zoo.values() {
            dream.read().unwrap().store();
        }
    }
}

impl DreamConfigApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Result<Self> {
        // Load settings from file

        let dream_ids = build_dreams_id_list();
        let zoo: BTreeMap<String, ADream> = dream_ids
            .keys()
            .map(|id| (id.to_string(), build_dream_by_id(id)))
            .collect();
        let saved_settings = SETTINGS.read().unwrap().clone();
        Ok(Self { active_panel: ActivePanel::Generic, zoo, saved_settings })
    }

    fn save(&mut self) {
        SETTINGS.write().unwrap().write_to_file_default().unwrap();
        self.saved_settings = SETTINGS.read().unwrap().clone();
    }

    fn cancel(&mut self, ctx: &egui::Context) {
        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
    }

    fn draw_generic(&mut self, ui: &mut egui::Ui) {
        let settings = &mut SETTINGS.write().unwrap();
        ui.heading("Dream Spinner");
        ui.separator();
        ui.checkbox(&mut settings.show_fps, "Show FPS")
            .on_hover_text("Show FPS of the primary screen");
        ui.checkbox(&mut settings.attempt_multiscreen, "Detect additional screens")
        .on_hover_text("Display dream on all screens. If you don't have a second screen, it will be ignored");
        if settings.attempt_multiscreen {
            egui::ComboBox::from_label("Multidisplay mode")
            .selected_text(format!(
                "{}",
                settings.viewport_mode.to_string()
            ))
            .show_ui(ui, |ui| {
                ui.selectable_value(
                    &mut settings.viewport_mode,
                    ViewportMode::Immediate,
                    ViewportMode::Immediate.to_string(),
                ).on_hover_text("Fill all screens in one draw call. Smooth, but FPS will be limited to the slowest screen");
                ui.selectable_value(
                    &mut settings.viewport_mode,
                    ViewportMode::Deferred,
                    ViewportMode::Deferred.to_string(),
                ).on_hover_text("Fill all screens independently. Better FPS, but may cause problems.");
            });
        }
    }

    /// UI panel to select which dreams to run.
    fn draw_dream_select(&mut self, ui: &mut egui::Ui) {
        // The list of selected dreams must never be completely empty.
        // So the idea is not to save the change if the user
        // unchecks the last dream.
        let settings = &mut SETTINGS.write().unwrap();

        ui.label("Select dream:");
        let st = &mut settings.selected_dreams;
        for (id, dream) in self.zoo.iter() {
            let mut active = st.contains(id);
            ui.checkbox(&mut active, dream.read().unwrap().name());
            if active {
                st.insert(id.to_string());
            } else {
                if st.len() > 1 {
                    st.remove(id);
                }
            }
        }
    }

    fn draw_about(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            ui.heading("Dream Spinner");
            ui.separator();
            self.powered_by_egui_and_eframe(ui);
        });
    }

    fn powered_by_egui_and_eframe(&mut self, ui: &mut egui::Ui) {
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
}
