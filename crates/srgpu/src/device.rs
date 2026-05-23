use wgpu::util::DeviceExt;

pub struct GpuDevice {
    pub adapter: wgpu::Adapter,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub limits: wgpu::Limits,
}

impl GpuDevice {
    pub async fn new() -> Option<Self> {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle());
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: None,
                force_fallback_adapter: false,
            })
            .await
            .ok()?;

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("AetherRender GPU Device"),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                memory_hints: wgpu::MemoryHints::Performance,
                experimental_features: wgpu::ExperimentalFeatures::disabled(),
                trace: wgpu::Trace::Off,
            })
            .await
            .ok()?;

        let limits = device.limits();

        Some(Self {
            adapter,
            device,
            queue,
            limits,
        })
    }

    pub fn new_blocking() -> Option<Self> {
        pollster::block_on(Self::new())
    }

    pub fn create_uniform_buffer<T: bytemuck::Pod>(&self, data: &[T]) -> crate::Buffer {
        let buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Uniform Buffer"),
            contents: bytemuck::cast_slice(data),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        crate::Buffer {
            raw: buffer,
            size: (std::mem::size_of::<T>() * data.len()) as u64,
        }
    }

    pub fn create_vertex_buffer<T: bytemuck::Pod>(&self, data: &[T]) -> crate::Buffer {
        let buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(data),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });
        crate::Buffer {
            raw: buffer,
            size: (std::mem::size_of::<T>() * data.len()) as u64,
        }
    }

    pub fn create_index_buffer(&self, data: &[u32]) -> crate::Buffer {
        let buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(data),
            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
        });
        crate::Buffer {
            raw: buffer,
            size: (std::mem::size_of::<u32>() * data.len()) as u64,
        }
    }

    pub fn write_buffer<T: bytemuck::Pod>(&self, buffer: &crate::Buffer, data: &[T]) {
        self.queue
            .write_buffer(&buffer.raw, 0, bytemuck::cast_slice(data));
    }

    pub fn submit(&self, encoder: wgpu::CommandEncoder) {
        self.queue.submit(std::iter::once(encoder.finish()));
    }
}
