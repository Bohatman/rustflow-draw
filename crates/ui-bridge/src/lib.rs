//! Shared UI bridge for Web (WASM) and Desktop (Tauri v2).

use rustflow_core::{Point, Viewport};
use rustflow_renderer::{compute_visible_grid_dots, GridUniforms};

#[derive(Debug, Clone)]
pub struct EngineBridge {
    pub viewport: Viewport,
}

impl Default for EngineBridge {
    fn default() -> Self {
        Self::new(1280.0, 720.0)
    }
}

impl EngineBridge {
    pub fn new(width: f32, height: f32) -> Self {
        Self {
            viewport: Viewport::new(width, height),
        }
    }

    pub fn pan(&mut self, dx: f32, dy: f32) {
        self.viewport.pan_by(dx, dy);
    }

    pub fn zoom_at(&mut self, zoom: f32, cursor_x: f32, cursor_y: f32) {
        self.viewport.zoom_at(zoom, Point::new(cursor_x, cursor_y));
    }

    pub fn grid_uniforms(&self) -> GridUniforms {
        GridUniforms::from_viewport(&self.viewport)
    }

    pub fn visible_grid_dot_count(&self) -> usize {
        compute_visible_grid_dots(&self.viewport).len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bridge_pan_zoom_grid_works() {
        let mut bridge = EngineBridge::new(400.0, 300.0);
        let initial_dots = bridge.visible_grid_dot_count();
        assert!(initial_dots > 0);
        bridge.pan(40.0, 20.0);
        bridge.zoom_at(2.0, 200.0, 150.0);
        assert_eq!(bridge.grid_uniforms().zoom, 2.0);
    }
}
