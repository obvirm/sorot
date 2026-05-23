use crate::GpuDevice;

pub struct GpuContext<'w> {
    pub device: GpuDevice,
    pub surface: wgpu::Surface<'w>,
    pub config: wgpu::SurfaceConfiguration,
    pub width: u32,
    pub height: u32,
}

impl<'w> GpuContext<'w> {
    pub unsafe fn new(window: &impl raw_window_handle::HasWindowHandle) -> Option<Self> {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle());
        let target = unsafe { wgpu::SurfaceTargetUnsafe::from_window(window) }.ok()?;
        let surface = unsafe { instance.create_surface_unsafe(target) }.ok()?;

        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        }))
        .ok()?;

        let (device_raw, queue) = pollster::block_on(
            adapter.request_device(&wgpu::DeviceDescriptor {
                label: Some("AetherRender GPU"),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                memory_hints: wgpu::MemoryHints::Performance,
                experimental_features: wgpu::ExperimentalFeatures::disabled(),
                trace: wgpu::Trace::Off,
            }),
        )
        .ok()?;

        let caps = surface.get_capabilities(&adapter);
        let format = caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width: 800,
            height: 600,
            present_mode: wgpu::PresentMode::AutoVsync,
            desired_maximum_frame_latency: 2,
            alpha_mode: caps.alpha_modes[0],
            view_formats: vec![],
        };
        surface.configure(&device_raw, &config);

        Some(Self {
            device: GpuDevice {
                adapter,
                device: device_raw,
                queue,
                limits: wgpu::Limits::default(),
            },
            surface,
            config,
            width: 800,
            height: 600,
        })
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if width == 0 || height == 0 {
            return;
        }
        self.width = width;
        self.height = height;
        self.config.width = width;
        self.config.height = height;
        self.surface.configure(&self.device.device, &self.config);
    }

    pub fn acquire_texture(&self) -> Option<wgpu::SurfaceTexture> {
        match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(tex) => Some(tex),
            wgpu::CurrentSurfaceTexture::Suboptimal(tex) => Some(tex),
            wgpu::CurrentSurfaceTexture::Timeout
            | wgpu::CurrentSurfaceTexture::Occluded
            | wgpu::CurrentSurfaceTexture::Outdated
            | wgpu::CurrentSurfaceTexture::Lost
            | wgpu::CurrentSurfaceTexture::Validation => None,
        }
    }
}
