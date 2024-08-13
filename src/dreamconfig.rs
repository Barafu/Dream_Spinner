use std::sync::{Arc, RwLock};

use crate::app_settings::SettingsRaw;
use anyhow::Result;
use crate::dreams::*;

// What is shown on main window right now
#[derive(Debug, PartialEq)]
enum ActivePanel {
    Generic,
    About,
    Dream(u32), // Settings of a dream with given ID
}

pub struct DreamConfigApp {
    settings: SettingsRaw,
    active_panel: ActivePanel,
    zoo: Zoo,
}


impl Drop for DreamConfigApp {
    fn drop(&mut self) {
        self.settings.write_to_file_default().unwrap();
    }
}

impl eframe::App for DreamConfigApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.vertical(|ui|{
                    ui.selectable_value(&mut self.active_panel, ActivePanel::Generic, "Settings");
                    ui.selectable_value(&mut self.active_panel, ActivePanel::About, "About");
                    for dream in self.zoo.iter() {
                        ui.selectable_value(&mut self.active_panel, ActivePanel::Dream(dream.read().unwrap().id()), dream.read().unwrap().name());
                    }
                });
                ui.separator();
                ui.vertical(|ui|{});
            });
        });
    }
}

impl DreamConfigApp {
    pub fn new(_cc: &eframe::CreationContext<'_>,) -> Result<Self> {
        let settings = SettingsRaw::read_from_file_default()?;
        let zoo = build_zoo(Arc::new(RwLock::new(settings.clone())));
        Ok(Self {
            settings,
            active_panel: ActivePanel::Generic,
            zoo,
        })
    }    
}