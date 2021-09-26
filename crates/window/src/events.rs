use winit::window::WindowId;

use crate::WindowDescriptor;

// Command
#[derive(Debug, Clone)]
pub struct CreateWindow {
    pub descriptor: WindowDescriptor,
}

// Events
#[derive(Debug, Clone)]
pub struct WindowLaunched {
    pub id: WindowId,
}

#[derive(Debug, Clone)]
pub struct WindowResized {
    pub id: WindowId,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone)]
pub struct WindowCloseRequested {
    pub id: WindowId,
}
