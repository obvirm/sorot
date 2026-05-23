pub struct RenderPipeline {
    pub raw: wgpu::RenderPipeline,
    pub bind_group_layout: wgpu::BindGroupLayout,
}

pub struct PipelineBuilder<'a> {
    device: &'a wgpu::Device,
    shader: Option<wgpu::ShaderModule>,
    color_format: wgpu::TextureFormat,
    depth_format: Option<wgpu::TextureFormat>,
    label: &'a str,
}

impl<'a> PipelineBuilder<'a> {
    pub fn new(device: &'a wgpu::Device) -> Self {
        Self {
            device,
            shader: None,
            color_format: wgpu::TextureFormat::Bgra8UnormSrgb,
            depth_format: None,
            label: "",
        }
    }

    pub fn shader(mut self, source: &str) -> Self {
        let module = self
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("Shader"),
                source: wgpu::ShaderSource::Wgsl(source.into()),
            });
        self.shader = Some(module);
        self
    }

    pub fn color_format(mut self, format: wgpu::TextureFormat) -> Self {
        self.color_format = format;
        self
    }

    pub fn depth_format(mut self, format: wgpu::TextureFormat) -> Self {
        self.depth_format = Some(format);
        self
    }

    pub fn label(mut self, label: &'a str) -> Self {
        self.label = label;
        self
    }

    pub fn build(self) -> RenderPipeline {
        let shader = self.shader.expect("Shader module must be set");

        let bgl = self
            .device
            .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Bind Group Layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        let layout = self
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Pipeline Layout"),
                bind_group_layouts: &[Some(&bgl)],
                immediate_size: 0,
            });

        let pipeline = self
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some(self.label),
                layout: Some(&layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: Some("vs_main"),
                    compilation_options: Default::default(),
                    buffers: &[],
                },
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleStrip,
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: None,
                    unclipped_depth: false,
                    polygon_mode: wgpu::PolygonMode::Fill,
                    conservative: false,
                },
                depth_stencil: self.depth_format.map(|format| wgpu::DepthStencilState {
                    format,
                    depth_write_enabled: Some(true),
                    depth_compare: Some(wgpu::CompareFunction::Less),
                    stencil: wgpu::StencilState::default(),
                    bias: wgpu::DepthBiasState::default(),
                }),
                multisample: wgpu::MultisampleState {
                    count: 1,
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: Some("fs_main"),
                    compilation_options: Default::default(),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: self.color_format,
                        blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                multiview_mask: None,
                cache: None,
            });

        RenderPipeline {
            raw: pipeline,
            bind_group_layout: bgl,
        }
    }
}
