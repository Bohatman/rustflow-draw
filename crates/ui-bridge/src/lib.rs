//! Shared UI bridge for Web (WASM) and Desktop (Tauri v2).

pub mod inspector;
pub mod palette;

pub use inspector::{
    apply_inspector_style, apply_inspector_text_style, dispatch_shortcut, ArrangeSnapshot,
    InspectorSnapshot, InspectorTab, ShortcutCommand,
};
pub use palette::{PaletteCatalog, StencilCategory, StencilItem};

use rustflow_core::{
    compute_smart_guides, ColorRgba, Command, CommandHistory, Document, Edge, EdgeAnimation,
    EdgeId, MinimapSnapshot, Node, NodeId, PageId, Point, Rect, SceneGraph, SelectionState,
    SmartSnapResult, Viewport,
};
use rustflow_io_formats::NATIVE_FORMAT_VERSION;
use rustflow_layout::{
    compute_force_directed_layout, compute_grid_layout, compute_hierarchical_layout,
    snap_to_grid_point, LayoutAlgorithm, LayoutTransition,
};
use rustflow_renderer::{compute_visible_grid_dots, GridUniforms, RenderFrame};

#[derive(Debug, Clone)]
pub struct EngineBridge {
    pub viewport: Viewport,
    pub scene: SceneGraph,
    pub history: CommandHistory,
    pub selection: SelectionState,
    pub document: Document,
    pub palette: PaletteCatalog,
    pub active_inspector_tab: InspectorTab,
    pub active_transition: Option<LayoutTransition>,
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
            document: Document::new("Untitled Diagram", width, height),
            palette: PaletteCatalog::built_in(),
            active_inspector_tab: InspectorTab::Style,
            active_transition: None,
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

    pub fn add_page(&mut self, name: impl Into<String>) -> PageId {
        self.sync_active_page_to_document();
        let new_id = self.document.add_page(name);
        self.load_active_page_from_document();
        new_id
    }

    pub fn switch_page(&mut self, page_id: PageId) -> bool {
        self.sync_active_page_to_document();
        if self.document.switch_page(page_id) {
            self.load_active_page_from_document();
            true
        } else {
            false
        }
    }

    pub fn sync_active_page_to_document(&mut self) {
        let page = self.document.active_page_mut();
        page.scene = self.scene.clone();
        page.viewport = self.viewport.clone();
        page.history = self.history.clone();
        page.selection = self.selection.clone();
    }

    pub fn load_active_page_from_document(&mut self) {
        let page = self.document.active_page();
        self.scene = page.scene.clone();
        self.viewport = page.viewport.clone();
        self.history = page.history.clone();
        self.selection = page.selection.clone();
    }

    pub fn minimap_snapshot(&self, minimap_width: f32, minimap_height: f32) -> MinimapSnapshot {
        MinimapSnapshot::compute(&self.scene, &self.viewport, minimap_width, minimap_height)
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

    pub fn set_edge_marching_ants(
        &mut self,
        edge_id: EdgeId,
        speed: f32,
        dash_length: f32,
        gap_length: f32,
        reverse: bool,
    ) -> bool {
        let Some(edge) = self.scene.get_edge_mut(edge_id) else {
            return false;
        };
        edge.animation = EdgeAnimation::MarchingAnts {
            speed,
            dash_length,
            gap_length,
            reverse,
        };
        true
    }

    pub fn set_edge_pulse_particles(
        &mut self,
        edge_id: EdgeId,
        speed: f32,
        particle_count: u32,
        radius: f32,
        color: ColorRgba,
    ) -> bool {
        let Some(edge) = self.scene.get_edge_mut(edge_id) else {
            return false;
        };
        edge.animation = EdgeAnimation::PulseParticles {
            speed,
            particle_count,
            radius,
            color,
        };
        true
    }

    pub fn trigger_auto_layout(&mut self, algorithm: LayoutAlgorithm, duration_seconds: f32) {
        let moves = match algorithm {
            LayoutAlgorithm::Grid => {
                compute_grid_layout(&self.scene, 4, Point::new(80.0, 80.0), 60.0, 60.0)
            }
            LayoutAlgorithm::Hierarchical => {
                compute_hierarchical_layout(&self.scene, Point::new(80.0, 80.0), 100.0, 60.0)
            }
            LayoutAlgorithm::ForceDirected => {
                compute_force_directed_layout(&self.scene, 30, 180.0)
            }
        };
        if duration_seconds <= 0.0 {
            self.history
                .execute(Command::MoveNodes { moves }, &mut self.scene);
        } else {
            self.active_transition = Some(LayoutTransition::new(moves, duration_seconds));
        }
    }

    pub fn step_animations(&mut self, dt_seconds: f32) -> bool {
        if let Some(transition) = &mut self.active_transition {
            let done = transition.step(dt_seconds, &mut self.scene);
            if done {
                let completed_moves = transition.moves.clone();
                self.active_transition = None;
                self.history.execute(
                    Command::MoveNodes {
                        moves: completed_moves,
                    },
                    &mut self.scene,
                );
            }
            return true;
        }
        false
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
    fn bridge_milestone3_animations_auto_layout_minimap_and_pages() {
        let mut bridge = EngineBridge::new(1024.0, 768.0);
        let n1 = bridge.add_react_flow_card("API", 100.0, 100.0, 160.0, 72.0);
        let n2 = bridge.add_react_flow_card("Queue", 100.0, 100.0, 160.0, 72.0);
        let e1 = bridge.connect_bezier(n1, n2);

        bridge.set_edge_marching_ants(e1, 30.0, 10.0, 6.0, false);
        bridge.trigger_auto_layout(LayoutAlgorithm::Hierarchical, 0.3);
        bridge.step_animations(0.15);
        bridge.step_animations(0.20);
        assert!(bridge.active_transition.is_none());

        let minimap = bridge.minimap_snapshot(200.0, 140.0);
        assert_eq!(minimap.node_boxes.len(), 2);

        let page2 = bridge.add_page("Page-2");
        assert_eq!(bridge.scene.nodes().len(), 0);
        bridge.switch_page(1);
        assert_eq!(bridge.scene.nodes().len(), 2);
        bridge.switch_page(page2);
        assert_eq!(bridge.scene.nodes().len(), 0);
    }
}
