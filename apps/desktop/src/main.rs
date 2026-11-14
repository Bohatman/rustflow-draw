use rustflow_ui_bridge::EngineBridge;

fn main() {
    let mut bridge = EngineBridge::new(1280.0, 720.0);
    bridge.pan(100.0, 60.0);
    bridge.zoom_at(1.25, 640.0, 360.0);
    let uniforms = bridge.grid_uniforms();
    println!(
        "RustFlow Draw Desktop Shell initialized: viewport={}x{}, zoom={:.2}, visible_grid_dots={}",
        uniforms.screen_size[0],
        uniforms.screen_size[1],
        uniforms.zoom,
        bridge.visible_grid_dot_count()
    );
}
