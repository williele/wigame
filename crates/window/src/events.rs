use winit::{
    event::{ElementState, ScanCode, VirtualKeyCode},
    window::WindowId,
};

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

pub struct WindowKeyboardInput {
    pub key_code: Option<VirtualKeyCode>,
    pub scan_code: ScanCode,
    pub state: ElementState,
}
