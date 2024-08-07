use display_info::DisplayInfo;
use std::sync::{Arc, RwLock};

use crate::app_settings::Settings;
use crate::dreams::*;

pub struct DreamSpinner {
    first_frame: bool,
    settings: Arc<RwLock<Settings>>,
    zoo: Vec<Arc<RwLock<dyn Dream>>>,
    secondary_displays: Vec<DisplayInfo>,
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
            secondary_displays: Vec::new(),
        }
    }
}

impl eframe::App for DreamSpinner {
    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let active_dream = self.zoo[1].clone();
        if self.first_frame {
            self.first_frame = false;

            // Get information on all displays
            let mut displays = DisplayInfo::all().unwrap();
            if displays.len() == 0 {
                panic!("Can't find any displays");
            }

            // Find primary display, move primary window to it.
            let primary_position = displays.iter().position(|d| d.is_primary).unwrap();
            let primary_display = displays.swap_remove(primary_position);
            ctx.send_viewport_cmd(egui::ViewportCommand::OuterPosition(
                [primary_display.x as f32, primary_display.y as f32].into(),
            ));
            ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(true));

            // List secondary monitors for creating additional windows. 
            if self.settings.read().unwrap().attempt_multiscreen {
                self.secondary_displays = displays;
            }
        }
        egui::CentralPanel::default().show(ctx, |ui| {
            // Create secondary windows
            for display in self.secondary_displays.iter().cloned() {
                let viewport_builder = egui::ViewportBuilder::default()
                    .with_taskbar(false)
                    .with_drag_and_drop(false);
                let viewport_id = egui::ViewportId::from_hash_of(display.id);

                let thread_dream_arc = active_dream.clone();

                ctx.show_viewport_deferred(viewport_id, viewport_builder, move |ctx, class| {
                    assert!(
                        class == egui::ViewportClass::Deferred,
                        "This egui backend doesn't support multiple viewports"
                    );

                    egui::CentralPanel::default().show(ctx, |ui| {
                        let mut painter = thread_dream_arc.write().unwrap();
                        painter.dream_egui(ui);
                    });
                });
            }

            active_dream.write().unwrap().dream_egui(ui);
            self.set_input(ui);
            for display in self.secondary_displays.iter() {
                let viewport_id = egui::ViewportId::from_hash_of(display.id);
                ctx.send_viewport_cmd_to(
                    viewport_id,
                    egui::ViewportCommand::OuterPosition(
                        dbg!([display.x as f32, display.y as f32]).into(),
                    ),
                );
                ctx.send_viewport_cmd_to(viewport_id, egui::ViewportCommand::Fullscreen(true));
            }
        });
    }
}

impl DreamSpinner {
    fn set_input(&self, ui: &mut egui::Ui) {
        ui.input(|input| {
            if input.pointer.any_released() {
                std::process::exit(0);
            }
        });
        ui.output_mut(|o| o.cursor_icon = egui::CursorIcon::None);
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
