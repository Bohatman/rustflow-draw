//! Auto-layout algorithms for RustFlow Draw.

use rustflow_core::Point;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LayoutAlgorithm {
    Grid,
    Hierarchical,
    ForceDirected,
}

pub fn snap_to_grid_point(pt: Point, spacing: f32) -> Point {
    if spacing <= 0.0 {
        return pt;
    }
    Point::new(
        (pt.x / spacing).round() * spacing,
        (pt.y / spacing).round() * spacing,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn layout_grid_snap_works() {
        let snapped = snap_to_grid_point(Point::new(23.0, 39.0), 20.0);
        assert_eq!(snapped, Point::new(20.0, 40.0));
    }
}
