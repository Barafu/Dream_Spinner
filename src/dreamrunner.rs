use log::info;
use rand::Rng;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Instant, SystemTime, UNIX_EPOCH};
use winit::application::ApplicationHandler;
use winit::dpi::{PhysicalPosition, PhysicalSize, Size};
use winit::event::WindowEvent;
use winit::event_loop::EventLoopProxy;
use winit::monitor::MonitorHandle;
use winit::raw_window_handle::{HasRawWindowHandle, HasWindowHandle};
use winit::window::Window;
use winit::window::{Fullscreen, WindowAttributes, WindowId};

use rayon::prelude::*;

use pixels::wgpu::Color;
use pixels::{Error, Pixels, SurfaceTexture};

use crate::app_settings::SETTINGS;
use crate::dreams::*;
use crate::fps_measure::FPSMeasureData;
use crate::user_event::UserLoopEvent;

/// Creates windows using winit and displays dreams according to settings.
pub struct DreamRunner {
    dream: Arc<RwLock<dyn Dream>>,
    fps_measurement: FPSMeasureData,
    windows: HashMap<WindowId, Window>,
    pixels: HashMap<WindowId, Pixels>,
    primary_window_id: Option<WindowId>,
    pub event_loop_proxy: EventLoopProxy<UserLoopEvent>,
    counter: usize,
}

impl DreamRunner {
    pub fn new(event_loop_proxy: EventLoopProxy<UserLoopEvent>) -> Self {
        let dream = Self::select_active_dream();
        let fps_measurement = FPSMeasureData::new();
        let windows = HashMap::new();
        let pixels = HashMap::new();
        Self {
            dream,
            fps_measurement,
            windows,
            pixels,
            primary_window_id: None,
            event_loop_proxy,
            counter: 100,
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
        dream.write().unwrap().prepare_dream();
        dream
    }
}

impl ApplicationHandler<UserLoopEvent> for DreamRunner {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        info!("Winit resumed");
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
                .with_visible(false)
        }

        // Create primary window
        let attr = build_window_attributes(primary_monitor.clone());
        let pr_window = event_loop.create_window(attr).unwrap();
        let pr_win_id = pr_window.id();
        self.primary_window_id = Some(pr_win_id);
        self.windows.insert(pr_win_id, pr_window);

        // Create secondary windows
        if SETTINGS.read().unwrap().attempt_multiscreen {
            for display in event_loop.available_monitors() {
                if display == primary_monitor {
                    continue;
                }
                let attr = build_window_attributes(display);
                let window = event_loop.create_window(attr).unwrap();
                self.windows.insert(window.id(), window);
            }
        }

        // Initialize GPU on windows
        for window in self.windows.values() {
            let window_size = window.inner_size();
            let surface_texture = SurfaceTexture::new(
                window_size.width,
                window_size.height,
                &window,
            );
            let pixels = Pixels::new(
                window_size.width,
                window_size.height,
                surface_texture,
            )
            .unwrap();
            let window_id = window.id();
            self.pixels.insert(window_id, pixels);
            info!("Initialized GPU on window {:?}", window_id);
            window.set_visible(true);
        }
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        window_id: winit::window::WindowId,
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
                let pixels = self.pixels.get_mut(&window_id).unwrap();
                let frame = pixels.frame_mut();
                //frame.fill(255);
                let now = SystemTime::now();
                let since_epoch = now
                    .duration_since(UNIX_EPOCH)
                    .expect("Time went backwards");
                let in_seconds = since_epoch.as_secs();
                let time_factor = (in_seconds % 6) as u8;
                frame.par_chunks_exact_mut(4).for_each(|pixel| {
                    pixel[0] = 0x00; // R
                    pixel[1] = time_factor * 40; // G
                    pixel[2] = 0x00; // B
                    pixel[3] = 0xff; // A
                });
                if let Err(err) = pixels.render() {
                    eprintln!("pixels.render error {}", err);
                    event_loop.exit();
                    return;
                }

                /*for (dst, &src) in pixels
                    .frame_mut()
                    .chunks_exact_mut(4)
                    .zip(shapes.frame().iter())
                {
                    dst[0] = (src >> 16) as u8;
                    dst[1] = (src >> 8) as u8;
                    dst[2] = src as u8;
                    dst[3] = (src >> 24) as u8;
                }*/

                //shapes.draw(now.elapsed().as_secs_f32());
                if window_id == self.primary_window_id.unwrap() {
                    self.fps_measurement.record_timestamp();
                    if self.fps_measurement.is_changed() {
                        info!("FPS: {}", self.fps_measurement);
                    }
                }

                self.event_loop_proxy
                    .send_event(UserLoopEvent::WindowFinishedRendering(
                        window_id,
                    ))
                    .unwrap();
            }
            WindowEvent::MouseInput { device_id: _, state: _, button: _ } => {
                // Exit on any mouse click
                event_loop.exit();
            }
            _ => (),
        }
    }

    fn user_event(
        &mut self,
        _event_loop: &winit::event_loop::ActiveEventLoop,
        event: UserLoopEvent,
    ) {
        match event {
            UserLoopEvent::WindowFinishedRendering(window_id) => {
                if self.counter > 0 {
                    self.counter -= 1;
                    self.windows.values_mut().for_each(|window| {
                        window.request_redraw();
                    });
                    return;
                }

                self.windows.get(&window_id).unwrap().request_redraw();
            }
        }
    }

    // fn about_to_wait(
    //     &mut self,
    //     event_loop: &winit::event_loop::ActiveEventLoop,
    // ) {
    //     for window in self.windows.values_mut() {
    //         window.request_redraw();
    //     }
    // }
}
