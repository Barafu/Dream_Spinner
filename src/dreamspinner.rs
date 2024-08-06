use std::sync::{Arc, RwLock};

use crate::dreams::*;
use crate::app_settings::Settings;

pub struct DreamSpinner {
    first_frame: bool,
    settings: Arc<RwLock<Settings>>,
    zoo: Vec<Box<dyn Dream>>,
}

impl DreamSpinner {
    /// Called once before the first frame.
    pub fn new(_cc: &eframe::CreationContext<'_>, settings: Arc<RwLock<Settings>>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        // if let Some(storage) = cc.storage {
        //     return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        // }

        let zoo = build_zoo(settings.clone());

        Self {
            settings,
            first_frame: true,
            zoo,
        }
    }
}

impl eframe::App for DreamSpinner {   

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.first_frame {
            use screen_info::DisplayInfo;
            self.first_frame = false;

            // Get information on all displays
            let mut displays = DisplayInfo::all().unwrap();
            if displays.len() == 0 {
                panic!("Can't find any displays");
            }

            // Find primary display
            let primary_position = displays.iter().position(|d| d.is_primary).unwrap();
            let primary_display = displays.swap_remove(primary_position);
            ctx.send_viewport_cmd(egui::ViewportCommand::OuterPosition(
                [primary_display.x as f32, primary_display.y as f32].into(),
            ));
            ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(true));
        }
        egui::CentralPanel::default().show(ctx, |ui| {
            
            self.zoo[0].dream_egui(ui);
            ui.input(|input| {
                if input.pointer.any_released() {
                    std::process::exit(0);
                }
            });
            ui.output_mut(|o| o.cursor_icon = egui::CursorIcon::None);
        });
    }
}

/*fn powered_by_egui_and_eframe(ui: &mut egui::Ui) {
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
}*/
