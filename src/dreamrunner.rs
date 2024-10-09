use chrono::Local;
use log::info;
use rand::Rng;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex, RwLock};
use std::thread;
use std::time::SystemTime;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::EventLoopProxy;
use winit::monitor::MonitorHandle;
use winit::window::Window;
use winit::window::{Fullscreen, WindowAttributes, WindowId};

use wgpu;

use crate::app_settings::SETTINGS;
use crate::dreams::*;
use crate::user_event::UserLoopEvent;

/// Creates windows using winit and displays dreams according to settings.
pub struct DreamRunner<'dr> {
    dream: Arc<RwLock<dyn Dream>>,
    wgpu_data: HashMap<WindowId, Arc<Mutex<WgpuData<'dr>>>>,
    primary_window_id: Option<WindowId>,
    pub event_loop_proxy: EventLoopProxy<UserLoopEvent>,
    window_finished_rendering: HashSet<WindowId>,
    redraw_thread: Option<thread::JoinHandle<()>>,
}

impl<'dr> DreamRunner<'dr> {
    pub fn new(event_loop_proxy: EventLoopProxy<UserLoopEvent>) -> Self {
        let dream = Self::select_active_dream();
        //let fps_measurement = HashMap::new();
        let wgpu_data = HashMap::new();
        let window_finished_rendering = HashSet::new();
        Self {
            dream,
            //fps_measurement,
            wgpu_data,
            primary_window_id: None,
            event_loop_proxy,
            window_finished_rendering,
            redraw_thread: None,
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

    fn redraw(&mut self, window_id: WindowId) {
        let wgpu_data = self.wgpu_data.get(&window_id);
        if wgpu_data.is_none() {
            return;
        }

        let mut wgpu_data = wgpu_data.unwrap().lock().unwrap();
        wgpu_data.update();
        match wgpu_data.render() {
            Ok(_) => {}
            // Reconfigure the surface if lost
            Err(wgpu::SurfaceError::Lost) => {
                let new_size = wgpu_data.size.clone();
                wgpu_data.resize(new_size);
            }
            // The system is out of memory, we should probably quit
            Err(wgpu::SurfaceError::OutOfMemory) => {
                // event_loop.exit();
            }
            // All other errors (Outdated, Timeout) should be resolved by the next frame
            Err(e) => eprintln!("{:?}", e),
        }
    }
}

impl<'dr> ApplicationHandler<UserLoopEvent> for DreamRunner<'dr> {
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
                .with_fullscreen(Some(Fullscreen::Borderless(Some(display))))
                .with_visible(true)
        }

        // Create primary window
        let attr = build_window_attributes(primary_monitor.clone());
        let pr_window = event_loop.create_window(attr).unwrap();
        let pr_win_id = pr_window.id();
        self.primary_window_id = Some(pr_win_id);

        //self.windows.insert(pr_win_id, aw);
        let mut created_windows = Vec::new();
        created_windows.push(pr_window);

        //Create secondary windows

        if SETTINGS.read().unwrap().attempt_multiscreen {
            for display in event_loop.available_monitors() {
                if display == primary_monitor {
                    continue;
                }
                let attr = build_window_attributes(display);
                let window = event_loop.create_window(attr).unwrap();
                created_windows.push(window);
            }
        }

        // Initialize GPU on windows
        for window in created_windows.into_iter() {
            let window_id = &window.id();
            window.request_redraw();
            let wgpu_data = pollster::block_on(WgpuData::<'dr>::new(window));
            let agpu = Arc::new(Mutex::new(wgpu_data));

            info!("Initialized GPU on window {:?}", &window_id);
            self.wgpu_data.insert(window_id.clone(), agpu);
        }
        self.event_loop_proxy
            .send_event(UserLoopEvent::WindowFinishedRendering(pr_win_id))
            .unwrap();

        // self.redraw_thread = Some(thread::spawn(move || {
        //     for wgpu_rr in arcs {
        //         let mut wgpu = wgpu_rr.lock().unwrap();
        //         wgpu.update();
        //         wgpu.render();
        //     }
        // }));
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

                /*self.event_loop_proxy
                .send_event(UserLoopEvent::WindowFinishedRendering(
                    window_id,
                ))
                .unwrap();*/

                //self.window_finished_rendering.insert(window_id);

                //self.event_loop_proxy.send_event(UserLoopEvent::WindowFinishedRendering(window_id)).unwrap();

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
                //self.wgpu_data.get(&window_id).unwrap().lock().unwrap().window().request_redraw();
                // for wgpu_data in self.wgpu_data.values() {
                //     wgpu_data.lock().unwrap().window().request_redraw();
                // }
                let ids: Vec<WindowId> =
                    self.wgpu_data.keys().cloned().collect();
                for id in ids {
                    self.redraw(id);
                }
            }
        }
    }

    fn about_to_wait(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
    ) {
        // for wgpu_data in self.wgpu_data.values() {
        //    let mut window = wgpu_data.lock().unwrap();
        //    window.window().request_redraw();
        // }
        self.event_loop_proxy
            .send_event(UserLoopEvent::WindowFinishedRendering(
                WindowId::dummy(),
            ))
            .unwrap();
    }
}

struct WgpuData<'window> {
    surface: wgpu::Surface<'window>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,
    // The window must be declared after the surface so
    // it gets dropped after it as the surface contains
    // unsafe references to the window's resources.
    window: Arc<Window>,
}

impl<'window> WgpuData<'window> {
    // Creating some of the wgpu types requires async code
    async fn new(window: Window) -> WgpuData<'window> {
        let window_size = window.inner_size();
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::DX12,
            ..Default::default()
        });

        let win = Arc::new(window);

        let surface = instance.create_surface(win.clone()).unwrap();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                    label: None,
                    memory_hints: Default::default(),
                },
                None, // Trace path
            )
            .await
            .unwrap();

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: window_size.width,
            height: window_size.height,
            present_mode: wgpu::PresentMode::AutoVsync,
            alpha_mode: wgpu::CompositeAlphaMode::Opaque,
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);
        let mut r = WgpuData {
            surface,
            device,
            queue,
            config,
            size: window_size,
            window: win,
        };
        //r.resize(r.window.inner_size());
        r
    }

    pub fn window(&self) -> &Window {
        &self.window
    }

    pub fn window_id(&self) -> WindowId {
        self.window.id()
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    fn input(&mut self, event: &WindowEvent) -> bool {
        false
    }

    fn update(&mut self) {}

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view =
            output.texture.create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self.device.create_command_encoder(
            &wgpu::CommandEncoderDescriptor { label: Some("Render Encoder") },
        );
        {
            let _render_pass =
                encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Render Pass"),
                    color_attachments: &[Some(
                        wgpu::RenderPassColorAttachment {
                            view: &view,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear(wgpu::Color {
                                    r: 0.1,
                                    g: get_milliseconds_since_midnight(),
                                    b: 0.3,
                                    a: 1.0,
                                }),
                                store: wgpu::StoreOp::Store,
                            },
                        },
                    )],
                    depth_stencil_attachment: None,
                    occlusion_query_set: None,
                    timestamp_writes: None,
                });
        }

        // submit will accept anything that implements IntoIter
        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}

fn get_milliseconds_since_midnight() -> f64 {
    let now = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap();
    (now.as_millis() % 3000) as f64 / 3000.0
}
