use chrono::{Local, Timelike};

use crate::dendraclock::FractalClock;

pub struct DreamSpinner {
    fractal_clock: FractalClock,
    first_frame: bool,
}

impl DreamSpinner {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        // if let Some(storage) = cc.storage {
        //     return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        // }

        Self {
            fractal_clock: FractalClock::default(),
            first_frame: true,
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
            let now = Local::now().time();
            let seconds_from_midnight: f64 =
                now.num_seconds_from_midnight() as f64 + now.nanosecond() as f64 * 1e-9;
            self.fractal_clock.ui(ui, Some(seconds_from_midnight));
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
