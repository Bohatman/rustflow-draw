//! Auto-layout engines (Grid, Sugiyama Hierarchical, Force-Directed) and Spring-Physics transitions.

use rustflow_core::{NodeId, Point, SceneGraph};
use std::collections::HashMap;

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

/// Compute target positions for a uniform Grid layout.
pub fn compute_grid_layout(
    scene: &SceneGraph,
    columns: usize,
    origin: Point,
    col_gap: f32,
    row_gap: f32,
) -> Vec<(NodeId, Point, Point)> {
    let cols = columns.max(1);
    let mut moves = Vec::with_capacity(scene.nodes().len());
    for (idx, node) in scene.nodes().iter().enumerate() {
        let col = idx % cols;
        let row = idx / cols;
        let target = Point::new(
            origin.x + col as f32 * (node.bounds.width + col_gap),
            origin.y + row as f32 * (node.bounds.height + row_gap),
        );
        moves.push((node.id, Point::new(node.bounds.x, node.bounds.y), target));
    }
    moves
}

/// Compute target positions for a Sugiyama-style Hierarchical layered layout.
pub fn compute_hierarchical_layout(
    scene: &SceneGraph,
    origin: Point,
    layer_gap: f32,
    sibling_gap: f32,
) -> Vec<(NodeId, Point, Point)> {
    let mut layer_of: HashMap<NodeId, usize> = HashMap::new();
    for node in scene.nodes() {
        layer_of.insert(node.id, 0);
    }

    // Bellman-Ford / longest-path layer assignment (bounded by node count to handle cycles safely)
    let n_count = scene.nodes().len();
    for _ in 0..n_count {
        let mut changed = false;
        for edge in scene.edges() {
            let src_layer = *layer_of.get(&edge.source_node).unwrap_or(&0);
            let dst_layer = *layer_of.get(&edge.target_node).unwrap_or(&0);
            if dst_layer <= src_layer && edge.source_node != edge.target_node {
                layer_of.insert(edge.target_node, (src_layer + 1).min(n_count));
                changed = true;
            }
        }
        if !changed {
            break;
        }
    }

    let mut layers: Vec<Vec<NodeId>> = Vec::new();
    for node in scene.nodes() {
        let l = *layer_of.get(&node.id).unwrap_or(&0);
        if layers.len() <= l {
            layers.resize_with(l + 1, Vec::new);
        }
        layers[l].push(node.id);
    }

    let mut moves = Vec::with_capacity(scene.nodes().len());
    for (layer_idx, group) in layers.iter().enumerate() {
        for (sib_idx, &nid) in group.iter().enumerate() {
            if let Some(node) = scene.get_node(nid) {
                let target = Point::new(
                    origin.x + sib_idx as f32 * (node.bounds.width + sibling_gap),
                    origin.y + layer_idx as f32 * (node.bounds.height + layer_gap),
                );
                moves.push((nid, Point::new(node.bounds.x, node.bounds.y), target));
            }
        }
    }
    moves
}

/// Compute target positions using Fruchterman-Reingold Force-Directed layout.
pub fn compute_force_directed_layout(
    scene: &SceneGraph,
    iterations: usize,
    ideal_distance: f32,
) -> Vec<(NodeId, Point, Point)> {
    let nodes = scene.nodes();
    if nodes.is_empty() {
        return Vec::new();
    }
    let k = ideal_distance.max(80.0);
    let mut positions: Vec<Point> = nodes
        .iter()
        .enumerate()
        .map(|(i, n)| {
            Point::new(
                n.bounds.x + (i % 5) as f32 * 5.0,
                n.bounds.y + (i / 5) as f32 * 5.0,
            )
        })
        .collect();
    let id_to_idx: HashMap<NodeId, usize> =
        nodes.iter().enumerate().map(|(i, n)| (n.id, i)).collect();

    for iter in 0..iterations.max(1) {
        let temp = k * (1.0 - iter as f32 / (iterations.max(1) as f32 + 1.0));
        let mut disp = vec![Point::ZERO; nodes.len()];

        // Repulsive forces between all node pairs
        for i in 0..nodes.len() {
            for j in (i + 1)..nodes.len() {
                let dx = positions[i].x - positions[j].x;
                let dy = positions[i].y - positions[j].y;
                let dist = (dx * dx + dy * dy).sqrt().max(1.0);
                let force = (k * k) / dist;
                let fx = (dx / dist) * force;
                let fy = (dy / dist) * force;
                disp[i].x += fx;
                disp[i].y += fy;
                disp[j].x -= fx;
                disp[j].y -= fy;
            }
        }

        // Attractive forces along edges
        for edge in scene.edges() {
            if let (Some(&u), Some(&v)) = (
                id_to_idx.get(&edge.source_node),
                id_to_idx.get(&edge.target_node),
            ) {
                if u == v {
                    continue;
                }
                let dx = positions[u].x - positions[v].x;
                let dy = positions[u].y - positions[v].y;
                let dist = (dx * dx + dy * dy).sqrt().max(1.0);
                let force = (dist * dist) / k;
                let fx = (dx / dist) * force;
                let fy = (dy / dist) * force;
                disp[u].x -= fx;
                disp[u].y -= fy;
                disp[v].x += fx;
                disp[v].y += fy;
            }
        }

        for i in 0..nodes.len() {
            let d = (disp[i].x * disp[i].x + disp[i].y * disp[i].y)
                .sqrt()
                .max(1.0);
            let step = d.min(temp);
            positions[i].x += (disp[i].x / d) * step;
            positions[i].y += (disp[i].y / d) * step;
        }
    }

    nodes
        .iter()
        .enumerate()
        .map(|(i, n)| (n.id, Point::new(n.bounds.x, n.bounds.y), positions[i]))
        .collect()
}

/// Spring-physics / Cubic-Bezier layout transition interpolator.
#[derive(Debug, Clone)]
pub struct LayoutTransition {
    pub moves: Vec<(NodeId, Point, Point)>,
    pub duration_seconds: f32,
    pub elapsed_seconds: f32,
}

impl LayoutTransition {
    pub fn new(moves: Vec<(NodeId, Point, Point)>, duration_seconds: f32) -> Self {
        Self {
            moves,
            duration_seconds: duration_seconds.max(0.05),
            elapsed_seconds: 0.0,
        }
    }

    /// Critically-damped spring / cubic-bezier ease-in-out curve.
    pub fn spring_progress(&self) -> f32 {
        let t = (self.elapsed_seconds / self.duration_seconds).clamp(0.0, 1.0);
        // Smooth cubic-bezier + damped spring blend
        1.0 - (1.0 - t).powi(3) * (1.0 + 3.0 * t)
    }

    pub fn step(&mut self, dt_seconds: f32, scene: &mut SceneGraph) -> bool {
        self.elapsed_seconds = (self.elapsed_seconds + dt_seconds).min(self.duration_seconds);
        let alpha = self.spring_progress();
        for &(id, from, to) in &self.moves {
            let pos = from.lerp(to, alpha);
            scene.move_node_to(id, pos);
        }
        self.elapsed_seconds >= self.duration_seconds
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlignMode {
    Left,
    CenterH,
    Right,
    Top,
    CenterV,
    Bottom,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DistributeAxis {
    Horizontal,
    Vertical,
}

pub fn align_nodes(scene: &mut SceneGraph, ids: &[NodeId], mode: AlignMode) {
    let nodes: Vec<(NodeId, rustflow_core::Rect)> = ids
        .iter()
        .filter_map(|&id| scene.get_node(id).map(|n| (id, n.bounds)))
        .collect();
    if nodes.len() < 2 {
        return;
    }
    let min_x = nodes.iter().map(|(_, r)| r.x).fold(f32::INFINITY, f32::min);
    let max_r = nodes.iter().map(|(_, r)| r.right()).fold(f32::NEG_INFINITY, f32::max);
    let min_y = nodes.iter().map(|(_, r)| r.y).fold(f32::INFINITY, f32::min);
    let max_b = nodes.iter().map(|(_, r)| r.bottom()).fold(f32::NEG_INFINITY, f32::max);
    let mid_x = (min_x + max_r) * 0.5;
    let mid_y = (min_y + max_b) * 0.5;

    for (id, r) in nodes {
        let target = match mode {
            AlignMode::Left => Point::new(min_x, r.y),
            AlignMode::CenterH => Point::new(mid_x - r.width * 0.5, r.y),
            AlignMode::Right => Point::new(max_r - r.width, r.y),
            AlignMode::Top => Point::new(r.x, min_y),
            AlignMode::CenterV => Point::new(r.x, mid_y - r.height * 0.5),
            AlignMode::Bottom => Point::new(r.x, max_b - r.height),
        };
        scene.move_node_to(id, target);
    }
}

pub fn distribute_nodes(scene: &mut SceneGraph, ids: &[NodeId], axis: DistributeAxis) {
    let mut nodes: Vec<(NodeId, rustflow_core::Rect)> = ids
        .iter()
        .filter_map(|&id| scene.get_node(id).map(|n| (id, n.bounds)))
        .collect();
    if nodes.len() < 3 {
        return;
    }
    match axis {
        DistributeAxis::Horizontal => {
            nodes.sort_by(|a, b| a.1.x.total_cmp(&b.1.x));
            let first = nodes.first().unwrap().1.x;
            let last_r = nodes.last().unwrap().1.right();
            let total_w: f32 = nodes.iter().map(|(_, r)| r.width).sum();
            let gap = ((last_r - first) - total_w) / (nodes.len() - 1) as f32;
            let mut cursor = first;
            for (id, r) in nodes {
                scene.move_node_to(id, Point::new(cursor, r.y));
                cursor += r.width + gap;
            }
        }
        DistributeAxis::Vertical => {
            nodes.sort_by(|a, b| a.1.y.total_cmp(&b.1.y));
            let first = nodes.first().unwrap().1.y;
            let last_b = nodes.last().unwrap().1.bottom();
            let total_h: f32 = nodes.iter().map(|(_, r)| r.height).sum();
            let gap = ((last_b - first) - total_h) / (nodes.len() - 1) as f32;
            let mut cursor = first;
            for (id, r) in nodes {
                scene.move_node_to(id, Point::new(r.x, cursor));
                cursor += r.height + gap;
            }
        }
    }
}

pub fn layout_overlap_stack(scene: &mut SceneGraph, ids: &[NodeId]) {
    let nodes: Vec<(NodeId, rustflow_core::Rect)> = ids
        .iter()
        .filter_map(|&id| scene.get_node(id).map(|n| (id, n.bounds)))
        .collect();
    if nodes.len() < 2 {
        return;
    }
    let anchor = nodes[0].1.center();
    for (i, (id, r)) in nodes.iter().enumerate() {
        let pos = Point::new(anchor.x - r.width * 0.5, anchor.y - r.height * 0.5);
        scene.move_node_to(*id, pos);
        if let Some(n) = scene.get_node_mut(*id) {
            n.z_index = i as i32;
        }
    }
    scene.normalize_layer_order();
}

pub fn layout_cascade_pile(scene: &mut SceneGraph, ids: &[NodeId], dx: f32, dy: f32) {
    let nodes: Vec<(NodeId, rustflow_core::Rect)> = ids
        .iter()
        .filter_map(|&id| scene.get_node(id).map(|n| (id, n.bounds)))
        .collect();
    if nodes.len() < 2 {
        return;
    }
    let base_x = nodes[0].1.x;
    let base_y = nodes[0].1.y;
    for (i, (id, _)) in nodes.iter().enumerate() {
        scene.move_node_to(*id, Point::new(base_x + dx * i as f32, base_y + dy * i as f32));
        if let Some(n) = scene.get_node_mut(*id) {
            n.z_index = i as i32;
        }
    }
    scene.normalize_layer_order();
}

pub fn layout_heap_pile(scene: &mut SceneGraph, ids: &[NodeId]) {
    let nodes: Vec<(NodeId, rustflow_core::Rect)> = ids
        .iter()
        .filter_map(|&id| scene.get_node(id).map(|n| (id, n.bounds)))
        .collect();
    if nodes.len() < 2 {
        return;
    }
    let cx = nodes.iter().map(|(_, r)| r.center().x).sum::<f32>() / nodes.len() as f32;
    let cy = nodes.iter().map(|(_, r)| r.center().y).sum::<f32>() / nodes.len() as f32;
    for (i, (id, r)) in nodes.iter().enumerate() {
        let angle = (i as f32) * 2.3999632;
        let radius = 14.0 * (i as f32).sqrt();
        let x = cx + angle.cos() * radius - r.width * 0.5;
        let y = cy + angle.sin() * radius - r.height * 0.5;
        scene.move_node_to(*id, Point::new(x, y));
        if let Some(n) = scene.get_node_mut(*id) {
            n.z_index = i as i32;
        }
    }
    scene.normalize_layer_order();
}

pub fn layout_separate_overlaps(scene: &mut SceneGraph, ids: &[NodeId], padding: f32) {
    let mut items: Vec<(NodeId, rustflow_core::Rect)> = ids
        .iter()
        .filter_map(|&id| scene.get_node(id).map(|n| (id, n.bounds)))
        .collect();
    if items.len() < 2 {
        return;
    }
    for _ in 0..24 {
        let mut moved = false;
        for i in 0..items.len() {
            for j in (i + 1)..items.len() {
                let a = items[i].1.expand(padding * 0.5);
                let b = items[j].1.expand(padding * 0.5);
                if a.intersects(&b) {
                    let ac = a.center();
                    let bc = b.center();
                    let mut dx = bc.x - ac.x;
                    let dy = bc.y - ac.y;
                    if dx.abs() < 0.01 && dy.abs() < 0.01 {
                        dx = 1.0;
                    }
                    let ox = (a.width + b.width) * 0.5 - dx.abs();
                    let oy = (a.height + b.height) * 0.5 - dy.abs();
                    if ox < oy {
                        let shift = (ox * 0.5 + 1.0) * dx.signum();
                        items[i].1.x -= shift;
                        items[j].1.x += shift;
                    } else {
                        let shift = (oy * 0.5 + 1.0) * dy.signum();
                        items[i].1.y -= shift;
                        items[j].1.y += shift;
                    }
                    moved = true;
                }
            }
        }
        if !moved {
            break;
        }
    }
    for (id, r) in items {
        scene.move_node_to(id, Point::new(r.x, r.y));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rustflow_core::{Edge, Node, Rect};

    #[test]
    fn hierarchical_force_and_spring_transition_work() {
        let mut scene = SceneGraph::new();
        scene.insert_node(Node::new_react_flow_card(
            1,
            "Root",
            Rect::new(0.0, 0.0, 140.0, 60.0),
        ));
        scene.insert_node(Node::new_react_flow_card(
            2,
            "Child A",
            Rect::new(0.0, 0.0, 140.0, 60.0),
        ));
        scene.insert_node(Node::new_react_flow_card(
            3,
            "Child B",
            Rect::new(0.0, 0.0, 140.0, 60.0),
        ));
        scene.insert_edge(Edge::new_bezier(10, 1, 2));
        scene.insert_edge(Edge::new_bezier(11, 1, 3));

        let moves = compute_hierarchical_layout(&scene, Point::new(80.0, 60.0), 100.0, 60.0);
        let mut transition = LayoutTransition::new(moves, 0.4);
        assert!(!transition.step(0.2, &mut scene));
        assert!(transition.step(0.2, &mut scene));

        let root_y = scene.get_node(1).unwrap().bounds.y;
        let child_a_y = scene.get_node(2).unwrap().bounds.y;
        assert!(child_a_y > root_y);

        let force_moves = compute_force_directed_layout(&scene, 15, 160.0);
        assert_eq!(force_moves.len(), 3);
    }
}
