//! Command pattern and deterministic Undo/Redo history stack for RustFlow Draw.

use crate::scene::{Edge, EdgeId, GroupId, Node, NodeId, NodeStyle, SceneGraph, TextStyle};
use crate::{Point, Rect};

#[derive(Debug, Clone, PartialEq)]
pub enum Command {
    AddNode(Node),
    RemoveNode {
        node: Node,
        incident_edges: Vec<Edge>,
    },
    MoveNodes {
        moves: Vec<(NodeId, Point, Point)>,
    },
    ResizeNode {
        id: NodeId,
        from: Rect,
        to: Rect,
        lock_aspect: bool,
    },
    RotateNode {
        id: NodeId,
        from_deg: f32,
        to_deg: f32,
    },
    UpdateNodeStyle {
        id: NodeId,
        from: NodeStyle,
        to: NodeStyle,
    },
    UpdateTextStyle {
        id: NodeId,
        from: TextStyle,
        to: TextStyle,
    },
    EditNodeText {
        id: NodeId,
        from_title: String,
        from_body: String,
        to_title: String,
        to_body: String,
    },
    EditEdgeLabel {
        id: EdgeId,
        from: String,
        to: String,
    },
    AddEdge(Edge),
    RemoveEdge(Edge),
    GroupNodes {
        group_id: GroupId,
        prev_groups: Vec<(NodeId, Option<GroupId>)>,
    },
    UngroupNodes {
        prev_groups: Vec<(NodeId, Option<GroupId>)>,
    },
    ChangeZIndex {
        id: NodeId,
        from: i32,
        to: i32,
    },
    Batch(Vec<Command>),
}

impl Command {
    pub fn apply(&self, scene: &mut SceneGraph) {
        match self {
            Command::AddNode(node) => {
                scene.insert_node(node.clone());
            }
            Command::RemoveNode { node, .. } => {
                scene.remove_node(node.id);
            }
            Command::MoveNodes { moves } => {
                for &(id, _from, to) in moves {
                    scene.move_node_to(id, to);
                }
            }
            Command::ResizeNode {
                id,
                to,
                lock_aspect,
                ..
            } => {
                scene.resize_node(*id, *to, *lock_aspect);
            }
            Command::RotateNode { id, to_deg, .. } => {
                if let Some(node) = scene.get_node_mut(*id) {
                    if !node.locked {
                        node.rotation_deg = *to_deg;
                    }
                }
            }
            Command::UpdateNodeStyle { id, to, .. } => {
                if let Some(node) = scene.get_node_mut(*id) {
                    node.style = to.clone();
                }
            }
            Command::UpdateTextStyle { id, to, .. } => {
                if let Some(node) = scene.get_node_mut(*id) {
                    node.text_style = to.clone();
                }
            }
            Command::EditNodeText {
                id,
                to_title,
                to_body,
                ..
            } => {
                if let Some(node) = scene.get_node_mut(*id) {
                    node.title = to_title.clone();
                    node.body = to_body.clone();
                }
            }
            Command::EditEdgeLabel { id, to, .. } => {
                if let Some(edge) = scene.get_edge_mut(*id) {
                    edge.label = to.clone();
                }
            }
            Command::AddEdge(edge) => {
                scene.insert_edge(edge.clone());
            }
            Command::RemoveEdge(edge) => {
                scene.remove_edge(edge.id);
            }
            Command::GroupNodes {
                group_id,
                prev_groups,
            } => {
                for &(id, _) in prev_groups {
                    if let Some(node) = scene.get_node_mut(id) {
                        node.group_id = Some(*group_id);
                    }
                }
            }
            Command::UngroupNodes { prev_groups } => {
                for &(id, _) in prev_groups {
                    if let Some(node) = scene.get_node_mut(id) {
                        node.group_id = None;
                    }
                }
            }
            Command::ChangeZIndex { id, to, .. } => {
                if let Some(node) = scene.get_node_mut(*id) {
                    node.z_index = *to;
                }
            }
            Command::Batch(cmds) => {
                for cmd in cmds {
                    cmd.apply(scene);
                }
            }
        }
    }

    pub fn revert(&self, scene: &mut SceneGraph) {
        match self {
            Command::AddNode(node) => {
                scene.remove_node(node.id);
            }
            Command::RemoveNode {
                node,
                incident_edges,
            } => {
                scene.insert_node(node.clone());
                for edge in incident_edges {
                    scene.insert_edge(edge.clone());
                }
            }
            Command::MoveNodes { moves } => {
                for &(id, from, _to) in moves {
                    scene.move_node_to(id, from);
                }
            }
            Command::ResizeNode { id, from, .. } => {
                scene.resize_node(*id, *from, false);
            }
            Command::RotateNode { id, from_deg, .. } => {
                if let Some(node) = scene.get_node_mut(*id) {
                    node.rotation_deg = *from_deg;
                }
            }
            Command::UpdateNodeStyle { id, from, .. } => {
                if let Some(node) = scene.get_node_mut(*id) {
                    node.style = from.clone();
                }
            }
            Command::UpdateTextStyle { id, from, .. } => {
                if let Some(node) = scene.get_node_mut(*id) {
                    node.text_style = from.clone();
                }
            }
            Command::EditNodeText {
                id,
                from_title,
                from_body,
                ..
            } => {
                if let Some(node) = scene.get_node_mut(*id) {
                    node.title = from_title.clone();
                    node.body = from_body.clone();
                }
            }
            Command::EditEdgeLabel { id, from, .. } => {
                if let Some(edge) = scene.get_edge_mut(*id) {
                    edge.label = from.clone();
                }
            }
            Command::AddEdge(edge) => {
                scene.remove_edge(edge.id);
            }
            Command::RemoveEdge(edge) => {
                scene.insert_edge(edge.clone());
            }
            Command::GroupNodes { prev_groups, .. } | Command::UngroupNodes { prev_groups } => {
                for &(id, prev) in prev_groups {
                    if let Some(node) = scene.get_node_mut(id) {
                        node.group_id = prev;
                    }
                }
            }
            Command::ChangeZIndex { id, from, .. } => {
                if let Some(node) = scene.get_node_mut(*id) {
                    node.z_index = *from;
                }
            }
            Command::Batch(cmds) => {
                for cmd in cmds.iter().rev() {
                    cmd.revert(scene);
                }
            }
        }
    }
}

/// Undo/Redo command stack with configurable capacity.
#[derive(Debug, Clone, Default)]
pub struct CommandHistory {
    undo_stack: Vec<Command>,
    redo_stack: Vec<Command>,
    max_history: usize,
}

impl CommandHistory {
    pub fn new(max_history: usize) -> Self {
        Self {
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            max_history: max_history.max(16),
        }
    }

    pub fn execute(&mut self, cmd: Command, scene: &mut SceneGraph) {
        cmd.apply(scene);
        self.redo_stack.clear();
        if self.max_history > 0 && self.undo_stack.len() >= self.max_history {
            self.undo_stack.remove(0);
        }
        self.undo_stack.push(cmd);
    }

    pub fn undo(&mut self, scene: &mut SceneGraph) -> bool {
        let Some(cmd) = self.undo_stack.pop() else {
            return false;
        };
        cmd.revert(scene);
        self.redo_stack.push(cmd);
        true
    }

    pub fn redo(&mut self, scene: &mut SceneGraph) -> bool {
        let Some(cmd) = self.redo_stack.pop() else {
            return false;
        };
        cmd.apply(scene);
        self.undo_stack.push(cmd);
        true
    }

    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    pub fn undo_len(&self) -> usize {
        self.undo_stack.len()
    }

    pub fn redo_len(&self) -> usize {
        self.redo_stack.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn command_history_undo_redo_invariants() {
        let mut scene = SceneGraph::new();
        let mut history = CommandHistory::new(100);

        let n1 = Node::new_react_flow_card(1, "Initial Title", Rect::new(20.0, 20.0, 140.0, 60.0));
        history.execute(Command::AddNode(n1), &mut scene);
        assert_eq!(scene.nodes().len(), 1);

        history.execute(
            Command::EditNodeText {
                id: 1,
                from_title: "Initial Title".into(),
                from_body: String::new(),
                to_title: "Updated Title".into(),
                to_body: "Subtitle".into(),
            },
            &mut scene,
        );
        assert_eq!(scene.get_node(1).unwrap().title, "Updated Title");

        assert!(history.undo(&mut scene));
        assert_eq!(scene.get_node(1).unwrap().title, "Initial Title");

        assert!(history.redo(&mut scene));
        assert_eq!(scene.get_node(1).unwrap().title, "Updated Title");

        assert!(history.undo(&mut scene));
        assert!(history.undo(&mut scene));
        assert!(scene.nodes().is_empty());
    }
}
