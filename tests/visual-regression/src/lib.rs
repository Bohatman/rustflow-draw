//! Visual regression test suite for RustFlow Draw.

#[cfg(test)]
mod tests {
    use rustflow_core::Viewport;
    use rustflow_renderer::compute_visible_grid_dots;

    #[test]
    fn phase1_pan_zoom_grid_snapshot() {
        let vp = Viewport::new(320.0, 200.0);
        let dots = compute_visible_grid_dots(&vp);
        assert_eq!(dots.len(), 17 * 11);
    }
}
