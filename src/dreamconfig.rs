use std::collections::BTreeMap;

use crate::app_settings::{ColorScheme, SettingsRaw, ViewportMode, SETTINGS};
use crate::dreams::*;
use anyhow::Result;

// What is shown on main window right now
#[derive(Debug, PartialEq)]
enum ActivePanel {
    Generic,
    Select,
    ColorScheme,
    About,
    Dream(DreamId), // Settings of a dream with given ID
}

/// The EGUI App object that provides selecting and configuring the dreams.
pub struct DreamConfigApp {
    /// What tab is selected currently.
    active_panel: ActivePanel,
    /// A list of all dreams, to call their settings functions from.
    zoo: BTreeMap<String, ADream>,
    /// The state of the settings as they were saved last.
    /// If current state differs from that, the save button will be enabled.
    saved_settings: SettingsRaw,
    /// A list of all color schemes
    color_schemes: BTreeMap<String, ColorScheme>,
    color_schemes_keys: Vec<String>,
}

impl eframe::App for DreamConfigApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // The status bar
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
        // The main contents: a list of tabs on the left and a panel on the right
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    // Common tabs
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
                        ActivePanel::ColorScheme,
                        "Select color",
                    );
                    ui.selectable_value(
                        &mut self.active_panel,
                        ActivePanel::About,
                        "About",
                    );
                    //ui.separator();
                    // TODO: Can't enable separator: breaks UI for some reason

                    // List all dreams as tabs
                    let app_dev_mode =
                        SETTINGS.read().unwrap().allow_dev_dreams;
                    for dream in self.zoo.values() {
                        // If dev mode is off, don't show dev dreams at all. If it is on, show them
                        // but append (dev) to their names.
                        let dream_dev_mode =
                            dream.read().unwrap().in_development();
                        if dream_dev_mode && !app_dev_mode {
                            continue;
                        };
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
                    ActivePanel::ColorScheme => {
                        self.draw_colorscheme_config(ui)
                    }
                    ActivePanel::About => self.draw_about(ui),
                    ActivePanel::Dream(id) => {
                        let mut dr = self.zoo[*id].write().unwrap();
                        dr.config_egui(ui);
                        dr.store();
                    }
                });
            });
        });
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
        let color_schemes: BTreeMap<String, ColorScheme> =
            ColorScheme::read_default_schemes();
        let color_schemes_keys: Vec<String> =
            color_schemes.keys().cloned().collect();
        Ok(Self {
            active_panel: ActivePanel::Generic,
            zoo,
            saved_settings,
            color_schemes,
            color_schemes_keys,
        })
    }

    fn save(&mut self) {
        SETTINGS.write().unwrap().write_to_file_default().unwrap();
        self.saved_settings = SETTINGS.read().unwrap().clone();
    }

    fn cancel(&mut self, ctx: &egui::Context) {
        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
    }

    /// Generic settings panel UI
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
            ui.label("Has no effect if there is only 1 screen");
        }
    }

    /// UI panel to select which dreams to run.
    fn draw_dream_select(&mut self, ui: &mut egui::Ui) {
        // The list of selected dreams must never be completely empty.
        // So the solution is not to save the change if the user
        // unchecks the last dream.
        let settings = &mut SETTINGS.write().unwrap();
        let dev_mode = settings.allow_dev_dreams;

        ui.label("Select dream:");
        let st = &mut settings.selected_dreams;
        for (id, dream) in self.zoo.iter() {
            if !dev_mode && dream.read().unwrap().in_development() {
                continue;
            }
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

    fn draw_colorscheme_config(&mut self, ui: &mut egui::Ui) {
        use egui_extras::{Column, TableBuilder};

        ui.vertical(|ui| {
            ui.heading("Select color scheme");
            let table = TableBuilder::new(ui)
                .striped(false)
                .column(Column::remainder().at_least(40.0).clip(true));
            table.body(|body| {
                let row_height = 28.0;
                let num_rows = self.color_schemes.len();
                body.rows(row_height, num_rows, |mut row| {
                    let row_index = row.index();
                    let mut row_text =
                        self.color_schemes_keys[row_index].clone();
                    let row_scheme = self.color_schemes.get(&row_text).unwrap();
                    let highlight =
                        SETTINGS.read().unwrap().color_scheme == *row_scheme;
                    if highlight {
                        row_text = format!("*** {} ***", row_text);
                    }
                    row.col(|ui| {
                        egui::Frame::none().fill(row_scheme.background).show(
                            ui,
                            |ui| {
                                let r = ui.add_sized(
                                    ui.available_size(),
                                    egui::Label::new(
                                        egui::RichText::new(row_text)
                                            .heading()
                                            .color(row_scheme.foreground),
                                    ),
                                );
                                if r.is_pointer_button_down_on() {
                                    SETTINGS.write().unwrap().color_scheme =
                                        row_scheme.clone();
                                }
                            },
                        );
                    });
                });
            })
        });
    }

    fn draw_about(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            ui.heading("Dream Spinner");
            ui.separator();
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
        });
    }
}
