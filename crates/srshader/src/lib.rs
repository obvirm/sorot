pub mod builtin;

use srgpu::GpuDevice;

pub struct ShaderLibrary {
    sdf_test_module: wgpu::ShaderModule,
}

impl ShaderLibrary {
    pub fn new(device: &GpuDevice) -> Self {
        let source = format!(
            "{}\n{}",
            builtin::FULLSCREEN_VERTEX,
            builtin::SDF_TEST_FRAGMENT
        );
        let module = device
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("SDF Test Shader"),
                source: wgpu::ShaderSource::Wgsl(source.into()),
            });

        Self {
            sdf_test_module: module,
        }
    }

    pub fn sdf_test_module(&self) -> &wgpu::ShaderModule {
        &self.sdf_test_module
    }
}
