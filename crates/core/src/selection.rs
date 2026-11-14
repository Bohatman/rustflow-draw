//! Multi-selection, marquee drag, smart alignment guides, and spacing indicators.

use crate::command::{Command, CommandHistory};
use crate::scene::{EdgeId, GroupId, Node, NodeId, SceneGraph};
use crate::{Point, Rect};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GuideOrientation {
    Vertical,
    Horizontal,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AlignmentGuide {
    pub orientation: GuideOrientation,
    pub coordinate: f32,
    pub start: f32,
    pub end: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SpacingIndicator {
    pub orientation: GuideOrientation,
    pub from: Point,
    pub to: Point,
    pub distance: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SmartSnapResult {
    pub snapped_bounds: Rect,
    pub guides: Vec<AlignmentGuide>,
    pub spacing_indicators: Vec<SpacingIndicator>,
}

/// Multi-selection state supporting marquee selection, Shift+Click, grouping, and smart guides.
#[derive(Debug, Clone, Default)]
pub struct SelectionState {
    pub selected_nodes: Vec<NodeId>,
    pub selected_edges: Vec<EdgeId>,
}

impl SelectionState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn clear(&mut self) {
        self.selected_nodes.clear();
        self.selected_edges.clear();
    }

    pub fn select_node(&mut self, id: NodeId, additive: bool) {
        if !additive {
            self.selected_nodes.clear();
            self.selected_edges.clear();
            self.selected_nodes.push(id);
        } else if let Some(pos) = self.selected_nodes.iter().position(|&n| n == id) {
            self.selected_nodes.remove(pos);
        } else {
            self.selected_nodes.push(id);
        }
    }

    pub fn select_edge(&mut self, id: EdgeId, additive: bool) {
        if !additive {
            self.selected_nodes.clear();
            self.selected_edges.clear();
            self.selected_edges.push(id);
        } else if !self.selected_edges.contains(&id) {
            self.selected_edges.push(id);
        }
    }

    /// Marquee drag selection in world space.
    pub fn select_marquee(&mut self, scene: &SceneGraph, marquee: &Rect, additive: bool) {
        if !additive {
            self.clear();
        }
        for node in scene.query_visible_nodes(marquee) {
            if marquee.intersects(&node.bounds) && !self.selected_nodes.contains(&node.id) {
                self.selected_nodes.push(node.id);
            }
        }
    }

    pub fn is_node_selected(&self, id: NodeId) -> bool {
        self.selected_nodes.contains(&id)
    }

    /// Duplicate selected nodes (`Ctrl/Cmd + D`) with an offset and record in Undo/Redo history.
    pub fn duplicate_selected(
        &mut self,
        scene: &mut SceneGraph,
        history: &mut CommandHistory,
        offset: Point,
    ) -> Vec<NodeId> {
        let mut cmds = Vec::new();
        let mut new_ids = Vec::new();
        for &nid in &self.selected_nodes {
            if let Some(orig) = scene.get_node(nid).cloned() {
                let new_id = scene.allocate_id();
                let mut copy = Node::new_react_flow_card(
                    new_id,
                    orig.title,
                    Rect::new(
                        orig.bounds.x + offset.x,
                        orig.bounds.y + offset.y,
                        orig.bounds.width,
                        orig.bounds.height,
                    ),
                );
                copy.body = orig.body;
                copy.shape = orig.shape;
                copy.style = orig.style;
                copy.text_style = orig.text_style;
                copy.rotation_deg = orig.rotation_deg;
                copy.z_index = orig.z_index + 1;
                cmds.push(Command::AddNode(copy));
                new_ids.push(new_id);
            }
        }
        if !cmds.is_empty() {
            history.execute(Command::Batch(cmds), scene);
            self.selected_nodes = new_ids.clone();
            self.selected_edges.clear();
        }
        new_ids
    }

    /// Group selected nodes (`Ctrl/Cmd + G`).
    pub fn group_selected(
        &self,
        scene: &mut SceneGraph,
        history: &mut CommandHistory,
    ) -> Option<GroupId> {
        if self.selected_nodes.len() < 2 {
            return None;
        }
        let group_id = scene.allocate_id();
        let prev_groups: Vec<(NodeId, Option<GroupId>)> = self
            .selected_nodes
            .iter()
            .filter_map(|&id| scene.get_node(id).map(|n| (id, n.group_id)))
            .collect();
        history.execute(
            Command::GroupNodes {
                group_id,
                prev_groups,
            },
            scene,
        );
        Some(group_id)
    }

    /// Ungroup selected nodes (`Ctrl/Cmd + Shift + G`).
    pub fn ungroup_selected(&self, scene: &mut SceneGraph, history: &mut CommandHistory) {
        let prev_groups: Vec<(NodeId, Option<GroupId>)> = self
            .selected_nodes
            .iter()
            .filter_map(|&id| scene.get_node(id).map(|n| (id, n.group_id)))
            .collect();
        if !prev_groups.is_empty() {
            history.execute(Command::UngroupNodes { prev_groups }, scene);
        }
    }

    /// Nudge selected nodes by `(dx, dy)` via arrow keys.
    pub fn nudge_selected(
        &self,
        scene: &mut SceneGraph,
        history: &mut CommandHistory,
        dx: f32,
        dy: f32,
    ) {
        let moves: Vec<(NodeId, Point, Point)> = self
            .selected_nodes
            .iter()
            .filter_map(|&id| {
                scene.get_node(id).filter(|n| !n.locked).map(|n| {
                    let from = Point::new(n.bounds.x, n.bounds.y);
                    let to = Point::new(from.x + dx, from.y + dy);
                    (id, from, to)
                })
            })
            .collect();
        if !moves.is_empty() {
            history.execute(Command::MoveNodes { moves }, scene);
        }
    }

    /// Delete selected nodes and edges (`Delete` / `Backspace`).
    pub fn delete_selected(&mut self, scene: &mut SceneGraph, history: &mut CommandHistory) {
        let mut cmds = Vec::new();
        for &eid in &self.selected_edges {
            if let Some(edge) = scene.get_edge(eid).cloned() {
                cmds.push(Command::RemoveEdge(edge));
            }
        }
        for &nid in &self.selected_nodes {
            if let Some(node) = scene.get_node(nid).cloned() {
                let incident_edges: Vec<_> = scene
                    .edges()
                    .iter()
                    .filter(|e| e.source_node == nid || e.target_node == nid)
                    .cloned()
                    .collect();
                cmds.push(Command::RemoveNode {
                    node,
                    incident_edges,
                });
            }
        }
        if !cmds.is_empty() {
            history.execute(Command::Batch(cmds), scene);
            self.clear();
        }
    }

    pub fn bring_selected_to_front(&self, scene: &mut SceneGraph, history: &mut CommandHistory) {
        let max_z = scene.nodes().iter().map(|n| n.z_index).max().unwrap_or(0);
        let mut cmds = Vec::new();
        for (idx, &nid) in self.selected_nodes.iter().enumerate() {
            if let Some(node) = scene.get_node(nid) {
                cmds.push(Command::ChangeZIndex {
                    id: nid,
                    from: node.z_index,
                    to: max_z + 1 + idx as i32,
                });
            }
        }
        if !cmds.is_empty() {
            history.execute(Command::Batch(cmds), scene);
        }
    }

    pub fn send_selected_to_back(&self, scene: &mut SceneGraph, history: &mut CommandHistory) {
        let min_z = scene.nodes().iter().map(|n| n.z_index).min().unwrap_or(0);
        let mut cmds = Vec::new();
        for (idx, &nid) in self.selected_nodes.iter().enumerate() {
            if let Some(node) = scene.get_node(nid) {
                cmds.push(Command::ChangeZIndex {
                    id: nid,
                    from: node.z_index,
                    to: min_z - 1 - idx as i32,
                });
            }
        }
        if !cmds.is_empty() {
            history.execute(Command::Batch(cmds), scene);
        }
    }
}

/// Compute dynamic alignment guides (center/edge alignment) and live spacing indicators.
pub fn compute_smart_guides(
    scene: &SceneGraph,
    moving_node_id: NodeId,
    candidate_bounds: Rect,
    snap_threshold: f32,
) -> SmartSnapResult {
    let mut snapped = candidate_bounds;
    let mut guides = Vec::new();
    let mut spacing_indicators = Vec::new();

    let search_area = candidate_bounds.expand(600.0);
    let others: Vec<&Node> = scene
        .query_visible_nodes(&search_area)
        .into_iter()
        .filter(|n| n.id != moving_node_id)
        .collect();

    let mut best_dx: Option<(f32, AlignmentGuide)> = None;
    let mut best_dy: Option<(f32, AlignmentGuide)> = None;

    let cand_x_points = [
        candidate_bounds.x,
        candidate_bounds.center().x,
        candidate_bounds.right(),
    ];
    let cand_y_points = [
        candidate_bounds.y,
        candidate_bounds.center().y,
        candidate_bounds.bottom(),
    ];

    for other in &others {
        let other_x_points = [other.bounds.x, other.bounds.center().x, other.bounds.right()];
        for &cx in &cand_x_points {
            for &ox in &other_x_points {
                let diff = ox - cx;
                if diff.abs() <= snap_threshold {
                    if best_dx.as_ref().map_or(true, |(b, _)| diff.abs() < b.abs()) {
                        let start = candidate_bounds.y.min(other.bounds.y);
                        let end = candidate_bounds.bottom().max(other.bounds.bottom());
                        best_dx = Some((
                            diff,
                            AlignmentGuide {
                                orientation: GuideOrientation::Vertical,
                                coordinate: ox,
                                start,
                                end,
                            },
                        ));
                    }
                }
            }
        }

        let other_y_points = [other.bounds.y, other.bounds.center().y, other.bounds.bottom()];
        for &cy in &cand_y_points {
            for &oy in &other_y_points {
                let diff = oy - cy;
                if diff.abs() <= snap_threshold {
                    if best_dy.as_ref().map_or(true, |(b, _)| diff.abs() < b.abs()) {
                        let start = candidate_bounds.x.min(other.bounds.x);
                        let end = candidate_bounds.right().max(other.bounds.right());
                        best_dy = Some((
                            diff,
                            AlignmentGuide {
                                orientation: GuideOrientation::Horizontal,
                                coordinate: oy,
                                start,
                                end,
                            },
                        ));
                    }
                }
            }
        }
    }

    if let Some((dx, guide)) = best_dx {
        snapped.x += dx;
        guides.push(guide);
    }
    if let Some((dy, guide)) = best_dy {
        snapped.y += dy;
        guides.push(guide);
    }

    // Compute horizontal distance indicators to closest left/right neighbors
    for other in &others {
        let vertical_overlap =
            snapped.y <= other.bounds.bottom() && snapped.bottom() >= other.bounds.y;
        if vertical_overlap && other.bounds.right() <= snapped.x {
            let dist = snapped.x - other.bounds.right();
            let mid_y = (snapped.center().y + other.bounds.center().y) * 0.5;
            spacing_indicators.push(SpacingIndicator {
                orientation: GuideOrientation::Horizontal,
                from: Point::new(other.bounds.right(), mid_y),
                to: Point::new(snapped.x, mid_y),
                distance: dist,
            });
        }
    }

    SmartSnapResult {
        snapped_bounds: snapped,
        guides,
        spacing_indicators,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smart_alignment_guides_and_multi_selection_commands() {
        let mut scene = SceneGraph::new();
        let mut history = CommandHistory::new(50);
        let mut sel = SelectionState::new();

        scene.insert_node(Node::new_react_flow_card(
            1,
            "Node A",
            Rect::new(100.0, 100.0, 120.0, 60.0),
        ));
        scene.insert_node(Node::new_react_flow_card(
            2,
            "Node B",
            Rect::new(280.0, 103.0, 120.0, 60.0),
        ));

        let snap = compute_smart_guides(&scene, 2, Rect::new(280.0, 103.0, 120.0, 60.0), 6.0);
        assert_eq!(snap.snapped_bounds.y, 100.0);
        assert!(!snap.guides.is_empty());
        assert!(!snap.spacing_indicators.is_empty());

        sel.select_marquee(&scene, &Rect::new(0.0, 0.0, 500.0, 300.0), false);
        assert_eq!(sel.selected_nodes.len(), 2);

        let group_id = sel.group_selected(&mut scene, &mut history);
        assert!(group_id.is_some());
        assert_eq!(scene.get_node(1).unwrap().group_id, group_id);

        let dup_ids = sel.duplicate_selected(&mut scene, &mut history, Point::new(20.0, 20.0));
        assert_eq!(dup_ids.len(), 2);
        assert_eq!(scene.nodes().len(), 4);
    }
}
