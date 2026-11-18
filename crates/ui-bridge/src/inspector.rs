//! Right Sidebar Format Inspector (Style, Text, Arrange tabs) and Draw.io Keyboard Shortcuts.

use rustflow_core::{
    Command, CommandHistory, NodeId, NodeStyle, Point, Rect, SceneGraph, SelectionState, TextStyle,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InspectorTab {
    Style,
    Text,
    Arrange,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ArrangeSnapshot {
    pub node_id: NodeId,
    pub bounds: Rect,
    pub rotation_deg: f32,
    pub z_index: i32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct InspectorSnapshot {
    pub active_tab: InspectorTab,
    pub selected_count: usize,
    pub style: Option<NodeStyle>,
    pub text: Option<TextStyle>,
    pub arrange: Option<ArrangeSnapshot>,
}

impl InspectorSnapshot {
    pub fn from_selection(
        active_tab: InspectorTab,
        scene: &SceneGraph,
        selection: &SelectionState,
    ) -> Self {
        let primary_node = selection
            .selected_nodes
            .first()
            .and_then(|&id| scene.get_node(id));

        Self {
            active_tab,
            selected_count: selection.selected_nodes.len() + selection.selected_edges.len(),
            style: primary_node.map(|n| n.style.clone()),
            text: primary_node.map(|n| n.text_style.clone()),
            arrange: primary_node.map(|n| ArrangeSnapshot {
                node_id: n.id,
                bounds: n.bounds,
                rotation_deg: n.rotation_deg,
                z_index: n.z_index,
            }),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShortcutCommand {
    Duplicate,
    Group,
    Ungroup,
    Delete,
    Undo,
    Redo,
    BringToFront,
    SendToBack,
    NudgeUp,
    NudgeDown,
    NudgeLeft,
    NudgeRight,
}

pub fn dispatch_shortcut(
    shortcut: ShortcutCommand,
    scene: &mut SceneGraph,
    selection: &mut SelectionState,
    history: &mut CommandHistory,
    grid_step: f32,
) {
    let step = grid_step.max(1.0);
    match shortcut {
        ShortcutCommand::Duplicate => {
            selection.duplicate_selected(scene, history, Point::new(step, step));
        }
        ShortcutCommand::Group => {
            selection.group_selected(scene, history);
        }
        ShortcutCommand::Ungroup => {
            selection.ungroup_selected(scene, history);
        }
        ShortcutCommand::Delete => {
            selection.delete_selected(scene, history);
        }
        ShortcutCommand::Undo => {
            history.undo(scene);
        }
        ShortcutCommand::Redo => {
            history.redo(scene);
        }
        ShortcutCommand::BringToFront => {
            selection.bring_selected_to_front(scene, history);
        }
        ShortcutCommand::SendToBack => {
            selection.send_selected_to_back(scene, history);
        }
        ShortcutCommand::NudgeUp => {
            selection.nudge_selected(scene, history, 0.0, -step);
        }
        ShortcutCommand::NudgeDown => {
            selection.nudge_selected(scene, history, 0.0, step);
        }
        ShortcutCommand::NudgeLeft => {
            selection.nudge_selected(scene, history, -step, 0.0);
        }
        ShortcutCommand::NudgeRight => {
            selection.nudge_selected(scene, history, step, 0.0);
        }
    }
}

pub fn apply_inspector_style(
    node_id: NodeId,
    new_style: NodeStyle,
    scene: &mut SceneGraph,
    history: &mut CommandHistory,
) -> bool {
    let Some(old_style) = scene.get_node(node_id).map(|n| n.style.clone()) else {
        return false;
    };
    history.execute(
        Command::UpdateNodeStyle {
            id: node_id,
            from: old_style,
            to: new_style,
        },
        scene,
    );
    true
}

pub fn apply_inspector_text_style(
    node_id: NodeId,
    new_text: TextStyle,
    scene: &mut SceneGraph,
    history: &mut CommandHistory,
) -> bool {
    let Some(old_text) = scene.get_node(node_id).map(|n| n.text_style.clone()) else {
        return false;
    };
    history.execute(
        Command::UpdateTextStyle {
            id: node_id,
            from: old_text,
            to: new_text,
        },
        scene,
    );
    true
}
