mod scene;

use sorot_gpu::wgpu_backend::WgpuApp;

fn main() {
    let (meshes, sdf) = scene::build_demo();
    pollster::block_on(WgpuApp::run(meshes, sdf));
}
