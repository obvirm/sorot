pub struct RenderTarget {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub width: u32,
    pub height: u32,
}

impl RenderTarget {
    pub fn new(
        device: &wgpu::Device,
        width: u32,
        height: u32,
        format: wgpu::TextureFormat,
        label: &str,
    ) -> Self {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some(label),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        Self {
            texture,
            view,
            width,
            height,
        }
    }

    pub fn sampler(device: &wgpu::Device) -> wgpu::Sampler {
        device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("rt_sampler"),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        })
    }
}

pub struct TexturePool {
    targets: Vec<RenderTarget>,
    sampler: wgpu::Sampler,
}

impl TexturePool {
    pub fn new(device: &wgpu::Device) -> Self {
        Self {
            targets: Vec::new(),
            sampler: RenderTarget::sampler(device),
        }
    }

    pub fn acquire(
        &mut self,
        device: &wgpu::Device,
        width: u32,
        height: u32,
        format: wgpu::TextureFormat,
    ) -> usize {
        let label = format!("rt_{}", self.targets.len());
        let rt = RenderTarget::new(device, width, height, format, &label);
        let id = self.targets.len();
        self.targets.push(rt);
        id
    }

    pub fn get(&self, id: usize) -> &RenderTarget {
        &self.targets[id]
    }

    pub fn sampler(&self) -> &wgpu::Sampler {
        &self.sampler
    }

    pub fn release_all(&mut self) {
        self.targets.clear();
    }
}
