use display_info::DisplayInfo;
use log::info;
use rand::Rng;
use std::sync::{Arc, RwLock};
use winit::application::ApplicationHandler;
use winit::dpi::{PhysicalPosition, PhysicalSize};
use winit::event::WindowEvent;
use winit::monitor::MonitorHandle;
use winit::window::Window;
use winit::window::{Fullscreen, WindowAttributes};

use glutin::{
    config::ConfigTemplateBuilder,
    context::ContextAttributesBuilder,
    context::PossiblyCurrentContext,
    display::GetGlDisplay,
    prelude::*,
    surface::{SurfaceAttributesBuilder, WindowSurface},
};

use crate::app_settings::SETTINGS;
use crate::dreams::*;
use crate::fps_measure::FPSMeasureData;

/// Creates windows using winit and displays dreams according to settings.
pub struct DreamRunner {
    dream: Arc<RwLock<dyn Dream>>,
    fps_measurement: FPSMeasureData,
    windows: Vec<Window>,
}

impl DreamRunner {
    pub fn new() -> Self {
        let dream = Self::select_active_dream();
        let fps_measurement = FPSMeasureData::new();
        let windows = Vec::new();
        Self { dream, fps_measurement, windows }
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
        dream.write().unwrap().prepare_dream();
        dream
    }
}

impl ApplicationHandler for DreamRunner {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        info!("Monitors information");
        let primary_monitor = event_loop
            .primary_monitor()
            .expect("Could not detect primary monitor");

        // Print monitor info
        /*for monitor in event_loop.available_monitors() {
            let intro = if primary_monitor == monitor {
                "Primary monitor"
            } else {
                "Monitor"
            };

            if let Some(name) = monitor.name() {
                info!("{intro}: {name}");
            } else {
                info!("{intro}: [no name]");
            }

            let PhysicalSize { width, height } = monitor.size();
            info!(
                "  Current mode: {width}x{height}{}",
                if let Some(m_hz) = monitor.refresh_rate_millihertz() {
                    format!(" @ {}.{} Hz", m_hz / 1000, m_hz % 1000)
                } else {
                    String::new()
                }
            );

            let PhysicalPosition { x, y } = monitor.position();
            info!("  Position: {x},{y}");

            info!("  Scale factor: {}", monitor.scale_factor());

            info!("  Available modes (width x height x bit-depth):");
            for mode in monitor.video_modes() {
                let PhysicalSize { width, height } = mode.size();
                let bits = mode.bit_depth();
                let m_hz = mode.refresh_rate_millihertz();
                info!(
                    "    {width}x{height}x{bits} @ {}.{} Hz",
                    m_hz / 1000,
                    m_hz % 1000
                );
            }
        }*/

        /// Builds attributes to create a fullscreen window on a given display.
        fn build_window_attributes(display: MonitorHandle) -> WindowAttributes {
            Window::default_attributes()
                .with_title("Dream Spinner")
                /* .with_inner_size(display.size())*/
                .with_fullscreen(Some(Fullscreen::Borderless(Some(display))))
        }

        // Create primary window
        let attr = build_window_attributes(primary_monitor.clone());
        let pr_window = event_loop.create_window(attr).unwrap();
        self.windows.push(pr_window);

        // Create secondary windows
        if SETTINGS.read().unwrap().attempt_multiscreen {
            for display in event_loop.available_monitors() {
                if display == primary_monitor {
                    continue;
                }
                let attr = build_window_attributes(display);
                let window = event_loop.create_window(attr).unwrap();
                self.windows.push(window);
            }
        }
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                println!("The close button was pressed; stopping");
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                // Redraw the application.
                //
                // It's preferable for applications that do not render continuously to render in
                // this event rather than in AboutToWait, since rendering in here allows
                // the program to gracefully handle redraws requested by the OS.

                // Draw.

                // Queue a RedrawRequested event.
                //
                // You only need to call this if you've determined that you need to redraw in
                // applications which do not always need to. Applications that redraw continuously
                // can render here instead.
                //self.window.as_ref().unwrap().request_redraw();
            }
            WindowEvent::MouseInput { device_id: _, state: _, button: _ } => {
                // Exit on any mouse click
                event_loop.exit();
            }
            _ => (),
        }
    }
}
