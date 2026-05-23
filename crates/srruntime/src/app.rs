use srcore::{Renderer, Shape};
use srgpu::GpuContext;
use srmath::{Color, Mat4};
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop},
    window::{Window, WindowId},
};

pub struct AetherApp {
    renderer: Renderer,
    shapes: Vec<Shape>,
    projection: Mat4,
    width: u32,
    height: u32,
    title: String,
    gpu: Option<GpuContext<'static>>,
    window: Option<Window>,
}

impl AetherApp {
    pub fn new(title: &str, width: u32, height: u32) -> Self {
        Self {
            window: None,
            gpu: None,
            renderer: Renderer::new(Color::rgba(0.1, 0.1, 0.15, 1.0)),
            shapes: Vec::new(),
            projection: Mat4::ortho_rh(0.0, width as f32, height as f32, 0.0, -1.0, 1.0),
            width,
            height,
            title: title.to_string(),
        }
    }

    pub fn add_shape(&mut self, shape: Shape) {
        self.shapes.push(shape);
    }

    pub fn run(mut self) -> Result<(), winit::error::EventLoopError> {
        let event_loop = EventLoop::new()?;
        event_loop.run_app(&mut self)
    }
}

impl ApplicationHandler for AetherApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            let window_attributes = Window::default_attributes()
                .with_title(&self.title)
                .with_inner_size(winit::dpi::LogicalSize::new(self.width, self.height));

            let window = event_loop.create_window(window_attributes).unwrap();
            self.gpu = unsafe { GpuContext::new(&window) };
            if let Some(gpu) = &self.gpu {
                self.renderer.init(gpu);
            }
            self.window = Some(window);
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                self.gpu = None;
                self.window = None;
                event_loop.exit();
            }
            WindowEvent::Resized(physical_size) => {
                if let Some(gpu) = &mut self.gpu {
                    gpu.resize(physical_size.width, physical_size.height);
                    self.width = physical_size.width;
                    self.height = physical_size.height;
                    self.projection = Mat4::ortho_rh(
                        0.0,
                        self.width as f32,
                        self.height as f32,
                        0.0,
                        -1.0,
                        1.0,
                    );
                }
            }
            WindowEvent::RedrawRequested => {
                let should_resize = if let Some(gpu) = &self.gpu {
                    self.renderer.render(gpu, &self.shapes, &self.projection).is_err()
                } else {
                    false
                };
                if should_resize {
                    if let Some(gpu) = &mut self.gpu {
                        gpu.resize(self.width, self.height);
                    }
                }
                if let Some(window) = &self.window {
                    window.request_redraw();
                }
            }
            _ => {}
        }
    }
}
