use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

use display_info::DisplayInfo;
use rand::Rng;

use crate::app_settings::{ViewportMode, SETTINGS};
use crate::dreams::*;

const RENDER_MEASURE_SIZE: usize = 10;

struct FPSMeasureData {
    avg: f32,
    worst: f32,
    render_timestamps: Vec<Instant>,
}

impl std::fmt::Display for FPSMeasureData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Avg: {:.2}, Worst: {:.2}", self.avg, self.worst)
    }
}

impl FPSMeasureData {
    fn new() -> Self {
        Self {
            avg: -1.0,
            worst: -1.0,
            render_timestamps: Vec::with_capacity(RENDER_MEASURE_SIZE),
        }
    }

    fn record_timestamp(&mut self) {
        self.render_timestamps.push(Instant::now());
        if self.render_timestamps.len() == RENDER_MEASURE_SIZE {
            let mut durations: Vec<Duration> =
                Vec::with_capacity(RENDER_MEASURE_SIZE);
            for t in self.render_timestamps.windows(2) {
                durations.push(t[1] - t[0]);
            }
            let sum: Duration =
                self.render_timestamps.last().unwrap().duration_since(
                    self.render_timestamps.first().cloned().unwrap(),
                );
            let avg = sum.as_secs_f32() / durations.len() as f32;
            let worst =
                durations.iter().max().unwrap_or(&Duration::ZERO).as_secs_f32();
            self.avg = 1.0 / avg;
            self.worst = 1.0 / worst;
            self.render_timestamps.clear();
        }
    }
}

pub struct DreamSpinner {
    first_frame: bool,
    #[allow(dead_code)]
    dream: Arc<RwLock<dyn Dream>>,
    primary_display: DisplayInfo,
    secondary_displays: Vec<DisplayInfo>,
    fps_measurement: FPSMeasureData,
}

impl DreamSpinner {
    /// Called once before the first frame.
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // Detect the displays.
        let mut displays = DisplayInfo::all().unwrap();
        if displays.is_empty() {
            panic!("Can't find any displays");
        }
        // Find primary display
        let primary_position =
            displays.iter().position(|d| d.is_primary).unwrap();
        let primary_display = displays.swap_remove(primary_position);
        displays.sort_by(|a, b| a.name.partial_cmp(&b.name).unwrap());
        // List secondary monitors for creating additional windows.
        let secondary_displays =
            match SETTINGS.read().unwrap().attempt_multiscreen {
                true => displays,
                false => Vec::new(),
            };

        let dream = Self::select_active_dream();

        Self {
            first_frame: true,
            dream,
            primary_display,
            secondary_displays,
            fps_measurement: FPSMeasureData::new(),
        }
    }

    /// Return Arc with the dream that will be displayed.
    ///
    /// Chooses randomly one of the dreams in selected list. Runs prepare on it.
    fn select_active_dream() -> Arc<RwLock<dyn Dream>> {
        let selected_dreams = SETTINGS.read().unwrap().selected_dreams.clone();
        let mut rng = rand::thread_rng();
        let random_index = rng.gen_range(0..selected_dreams.len());
        let random_id =
            selected_dreams.iter().nth(random_index).unwrap().to_string();
        let dream = build_dream_by_id(&random_id);
        dream.write().unwrap().prepare();
        dream
    }
}

impl eframe::App for DreamSpinner {
    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Get information on all displays

        if self.first_frame {
            self.first_frame = false;

            // These viewport settings need to be set after the primary viewport
            // has been created. Otherwise have DPI problems on Windows.
            ctx.send_viewport_cmd(egui::ViewportCommand::OuterPosition(
                [
                    self.primary_display.x as f32
                        / self.primary_display.scale_factor,
                    self.primary_display.y as f32
                        / self.primary_display.scale_factor,
                ]
                .into(),
            ));
            ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(true));
        }
        egui::CentralPanel::default().show(ctx, |ui| {
            // Create secondary windows
            for (pos, display) in self.secondary_displays.iter().enumerate() {
                let viewport_builder = egui::ViewportBuilder::default()
                    .with_position([
                        display.x as f32 / display.scale_factor,
                        display.y as f32 / display.scale_factor,
                    ])
                    .with_fullscreen(true)
                    .with_taskbar(false)
                    .with_drag_and_drop(false);
                let viewport_id = egui::ViewportId::from_hash_of(display.id * pos as u32);

                let thread_dream_arc = self.dream.clone();

                match SETTINGS.read().unwrap().viewport_mode {
                    ViewportMode::Immediate => {
                        ctx.show_viewport_immediate(
                            viewport_id,
                            viewport_builder,
                            move |ctx, class| {
                                assert!(
                                    class == egui::ViewportClass::Immediate,
                                    "This egui backend doesn't support multiple viewports"
                                );

                                egui::CentralPanel::default().show(ctx, |ui| {
                                    let painter = thread_dream_arc.read().unwrap();
                                    painter.dream_egui(ui);
                                    DreamSpinner::set_input(ui);
                                    // No need to force updates of secondary viewports in immediate mode
                                });
                            },
                        );
                    }
                    ViewportMode::Deferred => {
                        ctx.show_viewport_deferred(
                            viewport_id,
                            viewport_builder,
                            move |ctx, class| {
                                assert!(
                                    class == egui::ViewportClass::Deferred,
                                    "This egui backend doesn't support multiple viewports"
                                );

                                egui::CentralPanel::default().show(ctx, |ui| {
                                    let painter = thread_dream_arc.read().unwrap();
                                    painter.dream_egui(ui);
                                    DreamSpinner::set_input(ui);
                                    request_updates(ui, false, thread_dream_arc.read().unwrap().preferred_update_rate());
                                });
                            },
                        );
                    }
                }
            }
            // Paint primary window
            if SETTINGS.read().unwrap().show_fps {
                // Log render time
                self.fps_measurement.record_timestamp();
                ui.label(self.fps_measurement.to_string());
            }
            self.dream.read().unwrap().dream_egui(ui);
            DreamSpinner::set_input(ui);
            request_updates(ui, true, self.dream.read().unwrap().preferred_update_rate());
        });
    }
}

impl DreamSpinner {
    fn set_input(ui: &mut egui::Ui) {
        let mut need_quit = false;
        ui.input(|input| {
            if input.pointer.any_released() {
                need_quit = true;
            }
        });
        ui.output_mut(|o| o.cursor_icon = egui::CursorIcon::None);
        if need_quit {
            let mut ids: Vec<egui::ViewportId> = Vec::new();
            ui.ctx().input(|i| {
                ids = i.raw.viewports.keys().cloned().collect();
            });
            for id in ids {
                ui.ctx().send_viewport_cmd_to(id, egui::ViewportCommand::Close);
            }
        }
    }
}

// Detects all viewports and requests updates to all of them
fn request_updates(
    ui: &mut egui::Ui,
    is_primary_viewport: bool,
    update_rate: DreamUpdateRate,
) {
    if SETTINGS.read().unwrap().viewport_mode == ViewportMode::Immediate
        && !is_primary_viewport
    {
        // Should not update secondary viewports in immediate mode.
        return;
    }

    match update_rate {
        DreamUpdateRate::Smooth => ui.ctx().request_repaint(),
        DreamUpdateRate::Fixed(n) => ui
            .ctx()
            .request_repaint_after(std::time::Duration::from_secs_f32(n)),
    }
}
