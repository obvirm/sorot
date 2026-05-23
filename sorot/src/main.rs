mod scene;

use sorot_gpu::wgpu_backend::WgpuApp;

fn main() {
    let demo = scene::demo_scene();
    pollster::block_on(WgpuApp::run(demo.meshes, demo.sdf));
}
