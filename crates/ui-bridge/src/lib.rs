//! Shared UI bridge for Web (WASM) and Desktop (Tauri v2).

pub mod inspector;
pub mod palette;

pub use inspector::{
    apply_inspector_style, apply_inspector_text_style, dispatch_shortcut, ArrangeSnapshot,
    InspectorSnapshot, InspectorTab, ShortcutCommand,
};
pub use palette::{PaletteCatalog, StencilCategory, StencilItem};

use rustflow_core::{
    compute_smart_guides, Command, CommandHistory, Edge, EdgeId, Node, NodeId, Point, Rect,
    SceneGraph, SelectionState, SmartSnapResult, Viewport,
};
use rustflow_io_formats::NATIVE_FORMAT_VERSION;
use rustflow_layout::snap_to_grid_point;
use rustflow_renderer::{compute_visible_grid_dots, GridUniforms, RenderFrame};

#[derive(Debug, Clone)]
pub struct EngineBridge {
    pub viewport: Viewport,
    pub scene: SceneGraph,
    pub history: CommandHistory,
    pub selection: SelectionState,
    pub palette: PaletteCatalog,
    pub active_inspector_tab: InspectorTab,
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
            history: CommandHistory::new(256),
            selection: SelectionState::new(),
            palette: PaletteCatalog::built_in(),
            active_inspector_tab: InspectorTab::Style,
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
        self.history.execute(Command::AddNode(node), &mut self.scene);
        id
    }

    pub fn drop_stencil(&mut self, stencil_id: &str, world_x: f32, world_y: f32) -> Option<NodeId> {
        let id = self.scene.allocate_id();
        let origin = if self.viewport.snap_to_grid {
            snap_to_grid_point(Point::new(world_x, world_y), self.viewport.grid_size)
        } else {
            Point::new(world_x, world_y)
        };
        let node = self.palette.instantiate(stencil_id, id, origin)?;
        self.history.execute(Command::AddNode(node), &mut self.scene);
        self.selection.select_node(id, false);
        Some(id)
    }

    pub fn connect_bezier(&mut self, source: NodeId, target: NodeId) -> EdgeId {
        let id = self.scene.allocate_id();
        let edge = Edge::new_bezier(id, source, target);
        self.history.execute(Command::AddEdge(edge), &mut self.scene);
        id
    }

    pub fn edit_node_text(
        &mut self,
        node_id: NodeId,
        new_title: impl Into<String>,
        new_body: impl Into<String>,
    ) -> bool {
        let Some(node) = self.scene.get_node(node_id) else {
            return false;
        };
        let cmd = Command::EditNodeText {
            id: node_id,
            from_title: node.title.clone(),
            from_body: node.body.clone(),
            to_title: new_title.into(),
            to_body: new_body.into(),
        };
        self.history.execute(cmd, &mut self.scene);
        true
    }

    pub fn edit_edge_label(&mut self, edge_id: EdgeId, new_label: impl Into<String>) -> bool {
        let Some(edge) = self.scene.get_edge(edge_id) else {
            return false;
        };
        let cmd = Command::EditEdgeLabel {
            id: edge_id,
            from: edge.label.clone(),
            to: new_label.into(),
        };
        self.history.execute(cmd, &mut self.scene);
        true
    }

    pub fn drag_node_with_smart_guides(
        &mut self,
        node_id: NodeId,
        target_x: f32,
        target_y: f32,
    ) -> Option<SmartSnapResult> {
        let node = self.scene.get_node(node_id)?.clone();
        let candidate = Rect::new(target_x, target_y, node.bounds.width, node.bounds.height);
        let snap = compute_smart_guides(&self.scene, node_id, candidate, 6.0);
        let from = Point::new(node.bounds.x, node.bounds.y);
        let to = Point::new(snap.snapped_bounds.x, snap.snapped_bounds.y);
        self.history.execute(
            Command::MoveNodes {
                moves: vec![(node_id, from, to)],
            },
            &mut self.scene,
        );
        Some(snap)
    }

    pub fn handle_shortcut(&mut self, shortcut: ShortcutCommand) {
        dispatch_shortcut(
            shortcut,
            &mut self.scene,
            &mut self.selection,
            &mut self.history,
            self.viewport.grid_size,
        );
    }

    pub fn inspector_snapshot(&self) -> InspectorSnapshot {
        InspectorSnapshot::from_selection(self.active_inspector_tab, &self.scene, &self.selection)
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
    fn bridge_milestone2_palette_inspector_and_shortcuts() {
        let mut bridge = EngineBridge::new(1024.0, 768.0);
        assert_eq!(bridge.palette.search("aws").len(), 1);

        let n1 = bridge
            .drop_stencil("cloud.aws.lambda", 100.0, 100.0)
            .expect("stencil should instantiate");
        let n2 = bridge
            .drop_stencil("flowchart.database", 320.0, 100.0)
            .expect("stencil should instantiate");
        let e1 = bridge.connect_bezier(n1, n2);

        bridge.edit_node_text(n1, "Auth Handler", "Rust 1.100 runtime");
        bridge.edit_edge_label(e1, "gRPC TLS");
        assert_eq!(bridge.scene.get_node(n1).unwrap().title, "Auth Handler");
        assert_eq!(bridge.scene.get_edge(e1).unwrap().label, "gRPC TLS");

        bridge.handle_shortcut(ShortcutCommand::Undo);
        assert_eq!(bridge.scene.get_edge(e1).unwrap().label, "");
        bridge.handle_shortcut(ShortcutCommand::Redo);
        assert_eq!(bridge.scene.get_edge(e1).unwrap().label, "gRPC TLS");

        let snap = bridge.inspector_snapshot();
        assert_eq!(snap.selected_count, 1);
        assert!(snap.arrange.is_some());
    }
}
