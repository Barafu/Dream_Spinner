use crate::app_settings::{ViewportMode, SETTINGS};
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
    zoo: Zoo,
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
                        if ui.button("Save").clicked() {
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
                    ActivePanel::Generic => self.draw_generic(ui),
                    ActivePanel::Select => self.draw_dream_select(ui),
                    ActivePanel::About => self.draw_about(ui),
                    ActivePanel::Dream(id) => {
                        let dream = select_dream_by_id(&self.zoo, id).unwrap();
                        dream.write().unwrap().config_egui(ui);
                    }
                });
            });
        });
    }
}

impl DreamConfigApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Result<Self> {
        // Load settings from file

        let zoo = build_zoo();
        for dream in zoo.iter() {
            dream.write().unwrap().prepare();
        }
        Ok(Self { active_panel: ActivePanel::Generic, zoo })
    }

    fn save(&mut self) {
        for dream in self.zoo.iter() {
            dream.read().unwrap().store();
        }
        SETTINGS.write().unwrap().write_to_file_default().unwrap();
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
            egui::ComboBox::from_label("Presentation mode")
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

        let mut ids = Vec::with_capacity(self.zoo.len());
        let mut names = Vec::with_capacity(self.zoo.len());

        for arcd in self.zoo.iter() {
            let d = arcd.read().unwrap();
            ids.push(d.id());
            names.push(d.name());
        }

        let _active_dreams: Vec<bool> = Vec::with_capacity(ids.len());
        assert_eq!(ids.len(), names.len());

        ui.label("Select dream:");
        let st = &mut settings.selected_dreams;
        for n in 0..ids.len() {
            let mut active = st.contains(&ids[n]);
            ui.checkbox(&mut active, &names[n]);
            if active {
                st.insert(ids[n].clone());
            } else {
                if st.len() > 1 {
                    st.remove(&ids[n]);
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
