//! Headless GPU / Framebuffer Visual Regression Testing (VRT) suite for RustFlow Draw.

use rustflow_core::{ColorRgba, Edge, EdgeAnimation, Node, Rect, SceneGraph, Viewport};
use rustflow_renderer::HeadlessFramebuffer;

pub fn render_reference_diagram(time_seconds: f32) -> HeadlessFramebuffer {
    let mut scene = SceneGraph::new();
    scene.insert_node(Node::new_react_flow_card(
        1,
        "API Gateway",
        Rect::new(40.0, 50.0, 90.0, 44.0),
    ));
    scene.insert_node(Node::new_react_flow_card(
        2,
        "Auth Service",
        Rect::new(180.0, 90.0, 90.0, 44.0),
    ));
    let mut edge = Edge::new_bezier(10, 1, 2);
    edge.animation = EdgeAnimation::PulseParticles {
        speed: 0.5,
        particle_count: 2,
        radius: 3.0,
        color: ColorRgba::INDIGO_500,
    };
    scene.insert_edge(edge);

    let vp = Viewport::new(320.0, 180.0);
    let mut fb = HeadlessFramebuffer::new(320, 180, [248, 250, 252, 255]);
    fb.render_scene(&scene, &vp, time_seconds);
    fb
}

#[cfg(test)]
mod tests {
    use super::*;
    use rustflow_renderer::compute_visible_grid_dots;

    #[test]
    fn phase1_pan_zoom_grid_snapshot() {
        let vp = Viewport::new(320.0, 200.0);
        let dots = compute_visible_grid_dots(&vp);
        assert_eq!(dots.len(), 17 * 11);
    }

    #[test]
    fn vrt_headless_diagram_golden_pixel_diff_and_animation() {
        let frame_t0 = render_reference_diagram(0.0);
        let frame_t0_repeat = render_reference_diagram(0.0);
        assert_eq!(frame_t0.checksum(), frame_t0_repeat.checksum());
        assert_eq!(frame_t0.pixel_diff_ratio(&frame_t0_repeat), 0.0);

        // Advancing animation time moves pulse particles along the Bezier spline
        let frame_t1 = render_reference_diagram(0.5);
        let diff = frame_t0.pixel_diff_ratio(&frame_t1);
        assert!(diff > 0.0 && diff < 0.05);
    }
}
