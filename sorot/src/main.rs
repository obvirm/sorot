use sorot_gpu::wgpu_backend::WgpuApp;

fn main() {
    pollster::block_on(WgpuApp::run());
}
