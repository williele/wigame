pub struct WgpuRenderer {
    pub instance: wgpu::Instance,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub initialized: bool,
}
