//! Headless GPU / Framebuffer Visual Regression Testing (VRT) suite for RustFlow Draw.

use rustflow_core::{
    ColorRgba, Edge, EdgeAnimation, EdgeKind, Node, Rect, SceneGraph, ThemeMode, ThemeTokens,
    Viewport,
};
use rustflow_renderer::HeadlessFramebuffer;

pub fn render_reference_diagram(time_seconds: f32) -> HeadlessFramebuffer {
    render_themed_diagram(ThemeMode::Light, EdgeKind::Bezier, time_seconds)
}

pub fn render_themed_diagram(
    theme: ThemeMode,
    edge_kind: EdgeKind,
    time_seconds: f32,
) -> HeadlessFramebuffer {
    let tokens = ThemeTokens::for_mode(theme);
    let mut scene = SceneGraph::new();

    let mut n1 = Node::new_react_flow_card(1, "API Gateway", Rect::new(40.0, 50.0, 90.0, 44.0));
    n1.style.fill = tokens.card_fill;
    n1.style.stroke = tokens.card_stroke;
    scene.insert_node(n1);

    let mut n2 = Node::new_react_flow_card(2, "Auth Service", Rect::new(180.0, 90.0, 90.0, 44.0));
    n2.style.fill = tokens.card_fill;
    n2.style.stroke = tokens.card_stroke;
    scene.insert_node(n2);

    let mut edge = Edge::new_bezier(10, 1, 2);
    edge.kind = edge_kind;
    edge.stroke = tokens.edge_default;
    edge.animation = EdgeAnimation::PulseParticles {
        speed: 0.5,
        particle_count: 2,
        radius: 3.0,
        color: ColorRgba::INDIGO_500,
    };
    scene.insert_edge(edge);

    let vp = Viewport::new(320.0, 180.0);
    let clear = [
        (tokens.canvas_bg.r * 255.0) as u8,
        (tokens.canvas_bg.g * 255.0) as u8,
        (tokens.canvas_bg.b * 255.0) as u8,
        255,
    ];
    let mut fb = HeadlessFramebuffer::new(320, 180, clear);
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

        let frame_t1 = render_reference_diagram(0.5);
        let diff = frame_t0.pixel_diff_ratio(&frame_t1);
        assert!(diff > 0.0 && diff < 0.05);
    }

    #[test]
    fn vrt_multi_theme_and_edge_routing_parity() {
        let light_bezier = render_themed_diagram(ThemeMode::Light, EdgeKind::Bezier, 0.25);
        let dark_bezier = render_themed_diagram(ThemeMode::Dark, EdgeKind::Bezier, 0.25);
        let contrast_ortho =
            render_themed_diagram(ThemeMode::HighContrast, EdgeKind::Orthogonal, 0.25);
        let light_smooth = render_themed_diagram(ThemeMode::Light, EdgeKind::SmoothStep, 0.25);

        assert_ne!(light_bezier.checksum(), dark_bezier.checksum());
        assert_ne!(dark_bezier.checksum(), contrast_ortho.checksum());
        assert_ne!(light_bezier.checksum(), light_smooth.checksum());
    }
}
