use rustflow_core::{build_dogfood_architecture_scene, Point, TelemetryCollector};
use rustflow_renderer::HeadlessFramebuffer;
use rustflow_ui_bridge::EngineBridge;
use std::time::Instant;

fn main() {
    let mut bridge = EngineBridge::new(1280.0, 720.0);
    bridge.scene = build_dogfood_architecture_scene();
    bridge.pan(40.0, 20.0);
    bridge.zoom_at(1.25, 640.0, 360.0);

    let mut telemetry = TelemetryCollector::new(true, 128);
    let t0 = Instant::now();
    let frame = bridge.build_render_frame(0.5);
    let mut fb = HeadlessFramebuffer::new(320, 180, [248, 250, 252, 255]);
    fb.render_scene(&bridge.scene, &bridge.viewport, 0.5);
    telemetry.record_frame_micros(t0.elapsed().as_micros() as u64);

    let world_center = bridge.viewport.screen_to_world(Point::new(640.0, 360.0));

    println!(
        "RustFlow Draw Desktop v{}: nodes={}, edge_verts={}, center=({:.1},{:.1}), checksum={:#x}, telemetry=[{}]",
        bridge.format_version(),
        frame.node_instances.len(),
        frame.edge_vertices.len(),
        world_center.x,
        world_center.y,
        fb.checksum(),
        telemetry.summary_report()
    );
}
