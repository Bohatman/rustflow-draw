use rustflow_core::Point;
use rustflow_renderer::HeadlessFramebuffer;
use rustflow_ui_bridge::EngineBridge;

fn main() {
    let mut bridge = EngineBridge::new(1280.0, 720.0);
    let n1 = bridge.add_react_flow_card("Frontend Gateway", 120.0, 140.0, 180.0, 80.0);
    let n2 = bridge.add_react_flow_card("Rust Core Engine", 420.0, 240.0, 180.0, 80.0);
    bridge.connect_bezier(n1, n2);
    bridge.pan(40.0, 20.0);
    bridge.zoom_at(1.25, 640.0, 360.0);

    let frame = bridge.build_render_frame(0.5);
    let mut fb = HeadlessFramebuffer::new(320, 180, [248, 250, 252, 255]);
    fb.render_scene(&bridge.scene, &bridge.viewport, 0.5);
    let world_center = bridge.viewport.screen_to_world(Point::new(640.0, 360.0));

    println!(
        "RustFlow Draw Desktop v{}: nodes={}, edge_verts={}, center=({:.1},{:.1}), checksum={:#x}",
        bridge.format_version(),
        frame.node_instances.len(),
        frame.edge_vertices.len(),
        world_center.x,
        world_center.y,
        fb.checksum()
    );
}
