use winit::window::WindowId;

use crate::WindowDescriptor;

pub struct WindowCreateRequest {
    pub descriptor: WindowDescriptor,
}

pub struct WindowCreated {
    pub id: WindowId,
}

pub struct WindowCloseRequest {
    pub id: WindowId,
}

pub struct WindowClosed {
    pub id: WindowId,
}

pub struct WindowResized {
    pub id: WindowId,
    pub width: u32,
    pub height: u32,
}
