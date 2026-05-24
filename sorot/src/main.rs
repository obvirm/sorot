mod scene;

use gpu::wgpu_backend::WgpuApp;

fn main() {
    let frame = scene::build_frame();
    pollster::block_on(WgpuApp::run(frame));
}
