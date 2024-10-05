use winit::window::WindowId;

#[derive(Debug, Copy, Clone)]
pub enum UserLoopEvent {
    WindowFinishedRendering(WindowId),
}
