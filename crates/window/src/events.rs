use winit::window::WindowId;

use crate::WindowDescriptor;

#[derive(Debug)]
pub struct WindowCreateRequest {
    pub descriptor: WindowDescriptor,
}

#[derive(Debug)]
pub struct WindowCreated {
    pub id: WindowId,
}

#[derive(Debug)]
pub struct WindowCloseRequest {
    pub id: WindowId,
}

#[derive(Debug)]
pub struct WindowClosed {
    pub id: WindowId,
}

#[derive(Debug)]
pub struct WindowResized {
    pub id: WindowId,
    pub width: u32,
    pub height: u32,
}
