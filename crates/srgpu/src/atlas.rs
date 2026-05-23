pub struct SdfAtlas {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
    #[allow(dead_code)]
    size: u32,
    #[allow(dead_code)]
    cell_size: u32,
    #[allow(dead_code)]
    cols: u32,
    #[allow(dead_code)]
    rows: u32,
    #[allow(dead_code)]
    occupied: Vec<bool>,
}

#[allow(dead_code)]
impl SdfAtlas {
    pub fn new(device: &wgpu::Device, _queue: &wgpu::Queue, size: u32, cell_size: u32) -> Self {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("sdf_atlas"),
            size: wgpu::Extent3d {
                width: size,
                height: size,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::R8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("sdf_sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        let cols = size / cell_size;
        let rows = size / cell_size;

        let occupied = vec![false; (cols * rows) as usize];

        Self {
            texture,
            view,
            sampler,
            size,
            cell_size,
            cols,
            rows,
            occupied,
        }
    }

    pub fn allocate(&mut self) -> Option<(u32, u32)> {
        for i in 0..self.occupied.len() {
            if !self.occupied[i] {
                self.occupied[i] = true;
                let col = i as u32 % self.cols;
                let row = i as u32 / self.cols;
                return Some((col * self.cell_size, row * self.cell_size));
            }
        }
        None
    }

    pub fn upload(
        &self,
        queue: &wgpu::Queue,
        x: u32,
        y: u32,
        width: u32,
        height: u32,
        data: &[u8],
    ) {
        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &self.texture,
                mip_level: 0,
                origin: wgpu::Origin3d { x, y, z: 0 },
                aspect: wgpu::TextureAspect::All,
            },
            data,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(width),
                rows_per_image: Some(height),
            },
            wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );
    }

    pub fn uv_rect(&self, cell_idx: usize) -> (f32, f32, f32, f32) {
        let col = cell_idx as u32 % self.cols;
        let row = cell_idx as u32 / self.cols;
        let u0 = col as f32 / self.cols as f32;
        let v0 = row as f32 / self.rows as f32;
        let u1 = (col + 1) as f32 / self.cols as f32;
        let v1 = (row + 1) as f32 / self.rows as f32;
        (u0, v0, u1, v1)
    }

    pub fn bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("sdf_bgl"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        })
    }

    pub fn create_bind_group(&self, device: &wgpu::Device, layout: &wgpu::BindGroupLayout) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("sdf_bind_group"),
            layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&self.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&self.sampler),
                },
            ],
        })
    }
}
