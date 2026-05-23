use std::sync::Arc;
use winit::window::Window;

/// Abstract GPU backend trait — per AGENTS.md section 6.
pub trait GpuBackend {
    fn window(&self) -> &Arc<Window>;
    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>);
    fn render(&mut self) -> Result<(), wgpu::SurfaceError>;
}
