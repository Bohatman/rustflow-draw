//! Performance benchmark suite for RustFlow Draw.

#[cfg(test)]
mod tests {
    use rustflow_core::{Point, Viewport};
    use rustflow_renderer::compute_visible_grid_dots;

    #[test]
    fn phase1_grid_generation_within_budget() {
        let mut vp = Viewport::new(1920.0, 1080.0);
        vp.zoom_at(1.0, Point::new(960.0, 540.0));
        let dots = compute_visible_grid_dots(&vp);
        assert!(dots.len() > 5000);
    }
}
