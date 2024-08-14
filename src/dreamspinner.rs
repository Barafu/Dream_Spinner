use std::collections::VecDeque;
use std::time::Duration;

use display_info::DisplayInfo;

use crate::app_settings::Settings;
use crate::dreams::*;

const RENDER_MEASURE_SIZE: usize = 100;

#[allow(dead_code)]
#[derive(PartialEq, Debug)]
enum ViewportMode {
    Immediate,
    Deferred,
}

pub struct DreamSpinner {
    first_frame: bool,
    #[allow(dead_code)]
    settings: Settings,
    zoo: Zoo,
    primary_display: DisplayInfo,
    secondary_displays: Vec<DisplayInfo>,
    viewport_mode: ViewportMode,
    render_durations: VecDeque<Duration>,
}

impl DreamSpinner {
    /// Called once before the first frame.
    pub fn new(_cc: &eframe::CreationContext<'_>, settings: Settings) -> Self {
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
        //displays.extend_from_within(..displays.len());
        // List secondary monitors for creating additional windows.
        let secondary_displays =
            match settings.read().unwrap().attempt_multiscreen {
                true => displays,
                false => Vec::new(),
            };

        let zoo = build_zoo(settings.clone());

        Self {
            settings,
            first_frame: true,
            zoo,
            primary_display,
            secondary_displays,
            viewport_mode: ViewportMode::Immediate,
            render_durations: VecDeque::with_capacity(RENDER_MEASURE_SIZE),
        }
    }
}

impl eframe::App for DreamSpinner {
    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let timer = std::time::Instant::now();
        let active_dream = self.zoo[1].clone();
        active_dream.write().unwrap().prepare();
        // Get information on all displays

        if self.first_frame {
            self.first_frame = false;

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

                let thread_dream_arc = active_dream.clone();

                match self.viewport_mode {
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
                                    request_updates(ui);
                                });
                            },
                        );
                    }
                }
            }
            // Paint primary window
            let (avg, worst) = analyze_render_durations(&self.render_durations);
            ui.label(format!("Average: {:.4}  Worst: {:.4}", avg, worst));
            active_dream.read().unwrap().dream_egui(ui);
            DreamSpinner::set_input(ui);
            match self.viewport_mode {
                ViewportMode::Immediate => ctx.request_repaint(),
                ViewportMode::Deferred => request_updates(ui),
            }

            // Log render time
            self.render_durations.push_back(timer.elapsed());
            if self.render_durations.len() > RENDER_MEASURE_SIZE {
                self.render_durations.pop_front();
            }
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
fn request_updates(ui: &mut egui::Ui) {
    let mut ids: Vec<egui::ViewportId> = Vec::new();
    ui.ctx().input(|i| {
        ids = i.raw.viewports.keys().cloned().collect();
    });
    for id in ids {
        ui.ctx().request_repaint_of(id);
    }
}

fn analyze_render_durations(dur: &VecDeque<Duration>) -> (f64, f64) {
    if dur.len() < RENDER_MEASURE_SIZE {
        return (0.0, 0.0);
    }
    let sum: Duration = dur.iter().sum();
    let avg = sum.as_secs_f64() / dur.len() as f64;
    let worst = dur.iter().max().unwrap_or(&Duration::ZERO).as_secs_f64();
    (1.0 / avg, 1.0 / worst)
}
