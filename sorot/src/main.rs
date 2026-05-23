mod scene;

use sorot_gpu::wgpu_backend::WgpuApp;

fn main() {
    let meshes = scene::demo_scene();
    pollster::block_on(WgpuApp::run(meshes));
}
