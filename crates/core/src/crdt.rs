//! Local-first CRDT (Last-Write-Wins Element Set with Hybrid Logical Clocks)
//! and Theme Tokens (Light, Dark, High-Contrast) for developer-focused UX.

use crate::scene::{ColorRgba, Edge, EdgeId, Node, NodeId, SceneGraph, ThemeMode};

/// Hybrid Logical Clock timestamp for deterministic conflict-free ordering across peers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct HlcTimestamp {
    pub logical_time: u64,
    pub peer_id: u64,
}

impl HlcTimestamp {
    pub const fn new(logical_time: u64, peer_id: u64) -> Self {
        Self {
            logical_time,
            peer_id,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum CrdtOp {
    UpsertNode { ts: HlcTimestamp, node: Node },
    DeleteNode { ts: HlcTimestamp, node_id: NodeId },
    UpsertEdge { ts: HlcTimestamp, edge: Edge },
    DeleteEdge { ts: HlcTimestamp, edge_id: EdgeId },
}

impl CrdtOp {
    pub fn timestamp(&self) -> HlcTimestamp {
        match self {
            CrdtOp::UpsertNode { ts, .. }
            | CrdtOp::DeleteNode { ts, .. }
            | CrdtOp::UpsertEdge { ts, .. }
            | CrdtOp::DeleteEdge { ts, .. } => *ts,
        }
    }
}

/// Convergent Last-Write-Wins (LWW) CRDT state replica for real-time multiplayer collaboration.
#[derive(Debug, Clone)]
pub struct CrdtReplica {
    pub peer_id: u64,
    pub clock: u64,
    node_timestamps: Vec<(NodeId, HlcTimestamp, bool)>,
    edge_timestamps: Vec<(EdgeId, HlcTimestamp, bool)>,
    op_log: Vec<CrdtOp>,
}

impl CrdtReplica {
    pub fn new(peer_id: u64) -> Self {
        Self {
            peer_id,
            clock: 0,
            node_timestamps: Vec::new(),
            edge_timestamps: Vec::new(),
            op_log: Vec::new(),
        }
    }

    pub fn next_timestamp(&mut self) -> HlcTimestamp {
        self.clock += 1;
        HlcTimestamp::new(self.clock, self.peer_id)
    }

    pub fn ops(&self) -> &[CrdtOp] {
        &self.op_log
    }

    pub fn local_upsert_node(&mut self, scene: &mut SceneGraph, node: Node) -> CrdtOp {
        let ts = self.next_timestamp();
        let op = CrdtOp::UpsertNode { ts, node };
        self.apply_remote_op(scene, op.clone());
        op
    }

    pub fn local_delete_node(&mut self, scene: &mut SceneGraph, node_id: NodeId) -> CrdtOp {
        let ts = self.next_timestamp();
        let op = CrdtOp::DeleteNode { ts, node_id };
        self.apply_remote_op(scene, op.clone());
        op
    }

    pub fn local_upsert_edge(&mut self, scene: &mut SceneGraph, edge: Edge) -> CrdtOp {
        let ts = self.next_timestamp();
        let op = CrdtOp::UpsertEdge { ts, edge };
        self.apply_remote_op(scene, op.clone());
        op
    }

    pub fn local_delete_edge(&mut self, scene: &mut SceneGraph, edge_id: EdgeId) -> CrdtOp {
        let ts = self.next_timestamp();
        let op = CrdtOp::DeleteEdge { ts, edge_id };
        self.apply_remote_op(scene, op.clone());
        op
    }

    /// Apply a local or remote `CrdtOp` idempotently and commutatively.
    pub fn apply_remote_op(&mut self, scene: &mut SceneGraph, op: CrdtOp) -> bool {
        self.clock = self.clock.max(op.timestamp().logical_time);
        match &op {
            CrdtOp::UpsertNode { ts, node } => {
                if let Some(entry) = self
                    .node_timestamps
                    .iter_mut()
                    .find(|(id, _, _)| *id == node.id)
                {
                    if *ts <= entry.1 {
                        return false;
                    }
                    entry.1 = *ts;
                    entry.2 = false;
                } else {
                    self.node_timestamps.push((node.id, *ts, false));
                }
                scene.insert_node(node.clone());
                self.op_log.push(op);
                true
            }
            CrdtOp::DeleteNode { ts, node_id } => {
                if let Some(entry) = self
                    .node_timestamps
                    .iter_mut()
                    .find(|(id, _, _)| *id == *node_id)
                {
                    if *ts <= entry.1 {
                        return false;
                    }
                    entry.1 = *ts;
                    entry.2 = true;
                } else {
                    self.node_timestamps.push((*node_id, *ts, true));
                }
                scene.remove_node(*node_id);
                self.op_log.push(op);
                true
            }
            CrdtOp::UpsertEdge { ts, edge } => {
                if let Some(entry) = self
                    .edge_timestamps
                    .iter_mut()
                    .find(|(id, _, _)| *id == edge.id)
                {
                    if *ts <= entry.1 {
                        return false;
                    }
                    entry.1 = *ts;
                    entry.2 = false;
                } else {
                    self.edge_timestamps.push((edge.id, *ts, false));
                }
                scene.insert_edge(edge.clone());
                self.op_log.push(op);
                true
            }
            CrdtOp::DeleteEdge { ts, edge_id } => {
                if let Some(entry) = self
                    .edge_timestamps
                    .iter_mut()
                    .find(|(id, _, _)| *id == *edge_id)
                {
                    if *ts <= entry.1 {
                        return false;
                    }
                    entry.1 = *ts;
                    entry.2 = true;
                } else {
                    self.edge_timestamps.push((*edge_id, *ts, true));
                }
                scene.remove_edge(*edge_id);
                self.op_log.push(op);
                true
            }
        }
    }

    pub fn merge_from(&mut self, scene: &mut SceneGraph, remote_ops: &[CrdtOp]) -> usize {
        let mut applied = 0;
        for op in remote_ops {
            if self.apply_remote_op(scene, op.clone()) {
                applied += 1;
            }
        }
        applied
    }
}

/// Theme design tokens for Light, Dark, and High-Contrast developer modes.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ThemeTokens {
    pub mode: ThemeMode,
    pub canvas_bg: ColorRgba,
    pub grid_dot: ColorRgba,
    pub card_fill: ColorRgba,
    pub card_header_fill: ColorRgba,
    pub card_stroke: ColorRgba,
    pub text_primary: ColorRgba,
    pub edge_default: ColorRgba,
}

impl ThemeTokens {
    pub const fn for_mode(mode: ThemeMode) -> Self {
        match mode {
            ThemeMode::Light => Self {
                mode,
                canvas_bg: ColorRgba::new(0.97, 0.98, 0.99, 1.0),
                grid_dot: ColorRgba::new(0.78, 0.81, 0.86, 1.0),
                card_fill: ColorRgba::WHITE,
                card_header_fill: ColorRgba::new(0.95, 0.96, 0.99, 1.0),
                card_stroke: ColorRgba::new(0.82, 0.85, 0.90, 1.0),
                text_primary: ColorRgba::SLATE_900,
                edge_default: ColorRgba::INDIGO_500,
            },
            ThemeMode::Dark => Self {
                mode,
                canvas_bg: ColorRgba::new(0.06, 0.09, 0.16, 1.0),
                grid_dot: ColorRgba::new(0.20, 0.25, 0.33, 1.0),
                card_fill: ColorRgba::new(0.12, 0.16, 0.23, 1.0),
                card_header_fill: ColorRgba::new(0.15, 0.20, 0.29, 1.0),
                card_stroke: ColorRgba::new(0.28, 0.34, 0.45, 1.0),
                text_primary: ColorRgba::new(0.95, 0.96, 0.98, 1.0),
                edge_default: ColorRgba::new(0.51, 0.55, 0.97, 1.0),
            },
            ThemeMode::HighContrast => Self {
                mode,
                canvas_bg: ColorRgba::new(0.0, 0.0, 0.0, 1.0),
                grid_dot: ColorRgba::new(0.45, 0.45, 0.45, 1.0),
                card_fill: ColorRgba::new(0.05, 0.05, 0.05, 1.0),
                card_header_fill: ColorRgba::new(0.12, 0.12, 0.12, 1.0),
                card_stroke: ColorRgba::new(1.0, 1.0, 0.0, 1.0),
                text_primary: ColorRgba::WHITE,
                edge_default: ColorRgba::new(0.0, 1.0, 1.0, 1.0),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Rect;

    #[test]
    fn crdt_commutative_convergence_across_peers() {
        let mut scene_a = SceneGraph::new();
        let mut scene_b = SceneGraph::new();
        let mut peer_a = CrdtReplica::new(1);
        let mut peer_b = CrdtReplica::new(2);

        let op1 = peer_a.local_upsert_node(
            &mut scene_a,
            Node::new_react_flow_card(1, "Peer A Node", Rect::new(20.0, 20.0, 140.0, 60.0)),
        );
        let op2 = peer_b.local_upsert_node(
            &mut scene_b,
            Node::new_react_flow_card(
                1,
                "Peer B Concurrent Edit",
                Rect::new(40.0, 40.0, 140.0, 60.0),
            ),
        );

        // Merge in opposite orders on each peer
        peer_a.apply_remote_op(&mut scene_a, op2);
        peer_b.apply_remote_op(&mut scene_b, op1);

        // Both peers converge to the exact same state (peer 2 wins tie at logical_time=1)
        assert_eq!(
            scene_a.get_node(1).unwrap().title,
            scene_b.get_node(1).unwrap().title
        );
        assert_eq!(scene_a.get_node(1).unwrap().title, "Peer B Concurrent Edit");
    }
}
