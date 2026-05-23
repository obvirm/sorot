use srruntime::AetherApp;

fn main() {
    let app = AetherApp::new("sorot — AetherRender", 800, 600);
    app.run().unwrap();
}
