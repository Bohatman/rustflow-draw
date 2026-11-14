//! Shared UI bridge for Web (WASM) and Desktop (Tauri v2).

use rustflow_core::{Edge, EdgeId, Node, NodeId, Point, Rect, SceneGraph, Viewport};
use rustflow_io_formats::NATIVE_FORMAT_VERSION;
use rustflow_layout::snap_to_grid_point;
use rustflow_renderer::{compute_visible_grid_dots, GridUniforms, RenderFrame};

#[derive(Debug, Clone)]
pub struct EngineBridge {
    pub viewport: Viewport,
    pub scene: SceneGraph,
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
            scene: SceneGraph::new(),
        }
    }

    pub fn format_version(&self) -> &'static str {
        NATIVE_FORMAT_VERSION
    }

    pub fn pan(&mut self, dx: f32, dy: f32) {
        self.viewport.pan_by(dx, dy);
    }

    pub fn zoom_at(&mut self, zoom: f32, cursor_x: f32, cursor_y: f32) {
        self.viewport.zoom_at(zoom, Point::new(cursor_x, cursor_y));
    }

    pub fn add_react_flow_card(
        &mut self,
        title: impl Into<String>,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
    ) -> NodeId {
        let id = self.scene.allocate_id();
        let origin = if self.viewport.snap_to_grid {
            snap_to_grid_point(Point::new(x, y), self.viewport.grid_size)
        } else {
            Point::new(x, y)
        };
        let node = Node::new_react_flow_card(
            id,
            title,
            Rect::new(origin.x, origin.y, width, height),
        );
        self.scene.insert_node(node);
        id
    }

    pub fn connect_bezier(&mut self, source: NodeId, target: NodeId) -> EdgeId {
        let id = self.scene.allocate_id();
        self.scene.insert_edge(Edge::new_bezier(id, source, target));
        id
    }

    pub fn build_render_frame(&self, time_seconds: f32) -> RenderFrame {
        RenderFrame::build(&self.scene, &self.viewport, time_seconds)
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
    fn bridge_milestone1_cards_and_bezier_render_frame() {
        let mut bridge = EngineBridge::new(800.0, 600.0);
        let n1 = bridge.add_react_flow_card("Ingress", 103.0, 98.0, 160.0, 72.0);
        let n2 = bridge.add_react_flow_card("Worker", 380.0, 220.0, 160.0, 72.0);
        bridge.connect_bezier(n1, n2);

        let frame = bridge.build_render_frame(0.0);
        assert_eq!(frame.node_instances.len(), 2);
        assert!(!frame.edge_vertices.is_empty());
    }
}
