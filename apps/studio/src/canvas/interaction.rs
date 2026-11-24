#![allow(unused_imports)]
use crate::app::{BohatDrawApp, CanvasBgStyle, ResizeHandle};
use crate::theme::{canvas_bg_color, draw_symbol_icon, symbol_icon_accent, to_color32};
use egui::{Align2, Color32, CornerRadius, FontId, Painter, Pos2, Rect as EguiRect, Sense, Stroke, Vec2};
use rustflow_collab_protocol::ClientMsg;
use rustflow_core::{
    BorderStyle, Edge, EdgeAnimation, EdgeArrowStyle, EdgeId, EdgeKind, Node, NodeId, Point,
    PortId, PortSide, Rect, ShapeKind,
};
use rustflow_renderer::{
    tessellate_cubic_bezier, tessellate_edge_path, tessellate_self_loop,
    tessellate_smooth_step_oriented,
};

use super::{node_supports_body, rect_to_screen, screen_to_world, world_to_screen};
use super::minimap::{minimap_rects, minimap_restore_pill_rect, pan_viewport_from_minimap};
use super::render::{EdgeHandleHit, hit_test_selected_edge_handle};

fn hit_test_edge(app: &BohatDrawApp, world_pt: Point, tol: f32) -> Option<EdgeId> {
    for edge in app.bridge.scene.edges() {
        let pts = tessellate_edge_path(&app.bridge.scene, edge);
        for w in pts.windows(2) {
            if dist_point_to_segment(world_pt, w[0], w[1]) <= tol {
                return Some(edge.id);
            }
        }
    }
    None
}

fn dist_point_to_segment(p: Point, a: Point, b: Point) -> f32 {
    let abx = b.x - a.x;
    let aby = b.y - a.y;
    let len_sq = abx * abx + aby * aby;
    if len_sq < 1e-6 {
        return p.distance_to(a);
    }
    let t = (((p.x - a.x) * abx + (p.y - a.y) * aby) / len_sq).clamp(0.0, 1.0);
    p.distance_to(Point::new(a.x + abx * t, a.y + aby * t))
}

pub(super) fn hit_test_port(app: &BohatDrawApp, canvas_rect: EguiRect, screen_pos: Pos2) -> Option<(NodeId, PortId)> {
    let mut best: Option<(f32, NodeId, PortId)> = None;
    for node in app.bridge.scene.nodes().iter().rev() {
        if !node.visible {
            continue;
        }
        let mut draw_bounds = node.bounds;
        if matches!(node.shape, ShapeKind::Collapsible) && node.collapsed {
            draw_bounds.height = 34.0;
        }
        for port in &node.ports {
            let pt_s = world_to_screen(canvas_rect, app, port.world_position(&draw_bounds));
            let d = pt_s.distance(screen_pos);
            if d <= 14.0 && best.is_none_or(|(bd, ..)| d < bd) {
                best = Some((d, node.id, port.id));
            }
        }
    }
    best.map(|(_, nid, pid)| (nid, pid))
}

pub(super) fn handle_canvas_input(
    app: &mut BohatDrawApp,
    ui: &mut egui::Ui,
    response: &egui::Response,
    canvas_rect: EguiRect,
) {
    let (scroll_delta, zoom_delta, pointer_pos, modifiers, primary_down, primary_pressed, middle_down) =
        ui.input(|i| {
            (
                i.raw_scroll_delta,
                i.zoom_delta(),
                i.pointer.latest_pos().or(i.pointer.hover_pos()),
                i.modifiers,
                i.pointer.primary_down(),
                i.pointer.primary_pressed(),
                i.pointer.middle_down(),
            )
        });

    if let Some(touch) = ui.input(|i| i.multi_touch()) {
        if touch.translation_delta.length_sq() > 0.1 {
            app.target_zoom = None;
            app.bridge.pan(touch.translation_delta.x, touch.translation_delta.y);
        }
        if (touch.zoom_delta - 1.0).abs() > 0.002 {
            app.target_zoom = None;
            let new_zoom = (app.bridge.viewport.zoom * touch.zoom_delta).clamp(0.15, 4.0);
            app.bridge.zoom_at(
                new_zoom,
                touch.center_pos.x - canvas_rect.min.x,
                touch.center_pos.y - canvas_rect.min.y,
            );
        }
    }

    if canvas_rect.width() > 480.0 && canvas_rect.height() > 340.0 {
        if app.show_minimap {
            let (mrect, inner, close_rect) = minimap_rects(canvas_rect);
            let in_minimap = pointer_pos.is_some_and(|p| mrect.contains(p));
            let in_close = pointer_pos.is_some_and(|p| close_rect.expand(3.0).contains(p));
            if app.dragging_minimap {
                if primary_down {
                    if let Some(pos) = pointer_pos {
                        app.target_zoom = None;
                        pan_viewport_from_minimap(app, inner, pos);
                    }
                    ui.ctx().set_cursor_icon(egui::CursorIcon::Grabbing);
                    return;
                } else {
                    app.dragging_minimap = false;
                }
            } else if in_minimap && !app.is_interacting() {
                if in_close {
                    ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                    if primary_pressed {
                        app.show_minimap = false;
                        app.dragging_minimap = false;
                        return;
                    }
                } else {
                    ui.ctx().set_cursor_icon(egui::CursorIcon::Grab);
                    if primary_pressed {
                        if let Some(pos) = pointer_pos {
                            if inner.contains(pos) {
                                app.dragging_minimap = true;
                                app.target_zoom = None;
                                pan_viewport_from_minimap(app, inner, pos);
                            }
                        }
                        return;
                    }
                }
                if scroll_delta.y.abs() > 0.1 {
                    let factor = (1.0 + scroll_delta.y * 0.0025).clamp(0.8, 1.25);
                    let base = app
                        .target_zoom
                        .map(|(z, ..)| z)
                        .unwrap_or(app.bridge.viewport.zoom);
                    let target = (base * factor).clamp(0.15, 4.0);
                    app.target_zoom = Some((
                        target,
                        canvas_rect.width() * 0.5,
                        canvas_rect.height() * 0.5,
                    ));
                    return;
                }
            }
        } else if !app.is_interacting() {
            let pill = minimap_restore_pill_rect(canvas_rect);
            if pointer_pos.is_some_and(|p| pill.contains(p)) {
                ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                if primary_pressed {
                    app.show_minimap = true;
                    return;
                }
            }
        }
    }

    if response.hovered() {
        if (zoom_delta - 1.0).abs() > 0.001 {
            if let Some(pos) = pointer_pos {
                let new_zoom = app.bridge.viewport.zoom * zoom_delta;
                app.bridge.zoom_at(
                    new_zoom,
                    pos.x - canvas_rect.min.x,
                    pos.y - canvas_rect.min.y,
                );
            }
        } else if scroll_delta.y.abs() > 0.1 {
            if modifiers.ctrl || modifiers.command {
                if let Some(pos) = pointer_pos {
                    let factor = (1.0 + scroll_delta.y * 0.0025).clamp(0.8, 1.25);
                    let base = app
                        .target_zoom
                        .map(|(z, ..)| z)
                        .unwrap_or(app.bridge.viewport.zoom);
                    let target = (base * factor).clamp(0.15, 4.0);
                    app.target_zoom = Some((
                        target,
                        pos.x - canvas_rect.min.x,
                        pos.y - canvas_rect.min.y,
                    ));
                }
            } else {
                app.target_zoom = None;
                app.bridge.pan(scroll_delta.x, scroll_delta.y);
            }
        }
    }

    if let Some(pos) = pointer_pos {
        if response.hovered() && !app.is_interacting() {
            if let Some((_, handle, _)) = hit_test_resize_handle(app, canvas_rect, pos) {
                ui.ctx().set_cursor_icon(resize_handle_cursor(handle));
            }
        }
        if response.hovered() && app.collab.connected {
            let wp = screen_to_world(canvas_rect, app, pos);
            app.maybe_send_cursor(wp);
        }
    }

    if response.double_clicked() {
        if let Some(pos) = response.interact_pointer_pos() {
            if let Some(EdgeHandleHit::Waypoint(eid, wp_idx)) =
                hit_test_selected_edge_handle(app, canvas_rect, pos)
            {
                app.push_undo();
                if let Some(e) = app.bridge.scene.get_edge_mut(eid) {
                    if wp_idx < e.waypoints.len() {
                        e.waypoints.remove(wp_idx);
                    }
                }
                app.broadcast_scene_snapshot();
                return;
            }
            let wp = screen_to_world(canvas_rect, app, pos);
            if let Some(nid) = app.bridge.scene.hit_test_node(wp) {
                let focus_body = app
                    .bridge
                    .scene
                    .get_node(nid)
                    .map(|n| {
                        node_supports_body(&n.shape)
                            && n.bounds.height > 54.0
                            && wp.y > n.bounds.y + 32.0
                    })
                    .unwrap_or(false);
                app.start_inline_edit(nid, focus_body);
                app.bridge.selection.select_node(nid, false);
                return;
            }
            let edge_tol = (10.0 / app.bridge.viewport.zoom).max(6.0);
            if let Some(eid) = hit_test_edge(app, wp, edge_tol) {
                app.bridge.selection.clear();
                app.bridge.selection.selected_edges.push(eid);
                app.start_edge_inline_edit(eid);
                return;
            }
        }
    }

    if response.drag_started() {
        app.finish_inline_edit(true);
        app.finish_edge_inline_edit(true);
        if let Some(pos) = response.interact_pointer_pos() {
            let start_pos = ui.input(|i| i.pointer.press_origin()).unwrap_or(pos);
            let wp = screen_to_world(canvas_rect, app, start_pos);
            if middle_down || ui.input(|i| i.key_down(egui::Key::Space)) {
                app.panning = true;
            } else if let Some(hit) = hit_test_selected_edge_handle(app, canvas_rect, start_pos) {
                app.push_undo();
                match hit {
                    EdgeHandleHit::Waypoint(eid, wp_idx) => {
                        app.dragging_waypoint = Some((eid, wp_idx));
                    }
                    EdgeHandleHit::Midpoint(eid, insert_idx, mid_w) => {
                        if let Some(e) = app.bridge.scene.get_edge_mut(eid) {
                            let idx = insert_idx.min(e.waypoints.len());
                            e.waypoints.insert(idx, mid_w);
                            app.dragging_waypoint = Some((eid, idx));
                        }
                    }
                }
            } else if let Some((nid, pid)) = hit_test_port(app, canvas_rect, start_pos) {
                app.connecting_from = Some((nid, pid));
            } else {
                let hit_resize = hit_test_resize_handle(app, canvas_rect, start_pos);
                if let Some((rid, handle, orig_bounds)) = hit_resize {
                    app.push_undo();
                    app.resizing_node = Some((rid, handle, orig_bounds));
                    app.drag_origin_world = wp;
                } else if let Some(nid) = app.bridge.scene.hit_test_node(wp) {
                    if app.collab.locked_by_other(nid).is_some() {
                        let label = app
                            .bridge
                            .scene
                            .get_node(nid)
                            .map(|n| n.title.clone())
                            .unwrap_or_else(|| format!("Node {nid}"));
                        app.collab.send(&ClientMsg::DuelStart {
                            item_id: nid,
                            item_label: label,
                        });
                    } else {
                        if !app.bridge.selection.selected_nodes.contains(&nid) {
                            app.bridge.selection.select_node(nid, modifiers.shift);
                        }
                        app.push_undo();
                        app.dragging_nodes = true;
                        app.drag_origin_world = wp;
                        let mut moving_ids: Vec<NodeId> = Vec::new();
                        for &sid in &app.bridge.selection.selected_nodes {
                            if !moving_ids.contains(&sid) {
                                moving_ids.push(sid);
                            }
                            for did in app.bridge.scene.descendant_ids(sid) {
                                if !moving_ids.contains(&did) {
                                    moving_ids.push(did);
                                }
                            }
                        }
                        app.drag_start_positions = moving_ids
                            .into_iter()
                            .filter_map(|id| {
                                app.bridge
                                    .scene
                                    .get_node(id)
                                    .filter(|n| !n.locked)
                                    .map(|n| (id, Point::new(n.bounds.x, n.bounds.y)))
                            })
                            .collect();
                        app.active_guides.clear();
                        app.collab.send(&ClientMsg::Lock { item_id: nid });
                    }
                } else if app.hand_pan_mode {
                    app.panning = true;
                } else {
                    app.marquee_screen = Some((pos, pos));
                }
            }
        }
    }

    if response.dragged() {
        let delta = response.drag_delta();
        if app.panning {
            app.bridge.pan(delta.x, delta.y);
        } else if let Some((eid, wp_idx)) = app.dragging_waypoint {
            if let Some(cur_pos) = pointer_pos.or(response.interact_pointer_pos()) {
                let cur_w = screen_to_world(canvas_rect, app, cur_pos);
                if let Some(e) = app.bridge.scene.get_edge_mut(eid) {
                    if let Some(pt) = e.waypoints.get_mut(wp_idx) {
                        *pt = cur_w;
                    }
                }
            }
            ui.ctx().set_cursor_icon(egui::CursorIcon::Grabbing);
        } else if let Some((rid, handle, orig)) = app.resizing_node {
            if let Some(cur_pos) = pointer_pos.or(response.interact_pointer_pos()) {
                let cur_w = screen_to_world(canvas_rect, app, cur_pos);
                let dx = cur_w.x - app.drag_origin_world.x;
                let dy = cur_w.y - app.drag_origin_world.y;
                let min_w = 40.0_f32;
                let min_h = 30.0_f32;
                let mut x0 = orig.x;
                let mut y0 = orig.y;
                let mut x1 = orig.x + orig.width;
                let mut y1 = orig.y + orig.height;
                match handle {
                    ResizeHandle::NW => {
                        x0 = (orig.x + dx).min(x1 - min_w);
                        y0 = (orig.y + dy).min(y1 - min_h);
                    }
                    ResizeHandle::N => {
                        y0 = (orig.y + dy).min(y1 - min_h);
                    }
                    ResizeHandle::NE => {
                        x1 = (orig.x + orig.width + dx).max(x0 + min_w);
                        y0 = (orig.y + dy).min(y1 - min_h);
                    }
                    ResizeHandle::E => {
                        x1 = (orig.x + orig.width + dx).max(x0 + min_w);
                    }
                    ResizeHandle::SE => {
                        x1 = (orig.x + orig.width + dx).max(x0 + min_w);
                        y1 = (orig.y + orig.height + dy).max(y0 + min_h);
                    }
                    ResizeHandle::S => {
                        y1 = (orig.y + orig.height + dy).max(y0 + min_h);
                    }
                    ResizeHandle::SW => {
                        x0 = (orig.x + dx).min(x1 - min_w);
                        y1 = (orig.y + orig.height + dy).max(y0 + min_h);
                    }
                    ResizeHandle::W => {
                        x0 = (orig.x + dx).min(x1 - min_w);
                    }
                }
                if modifiers.alt {
                    let cx = orig.x + orig.width * 0.5;
                    let cy = orig.y + orig.height * 0.5;
                    let half_w = ((x1 - x0) * 0.5).max(min_w * 0.5);
                    let half_h = ((y1 - y0) * 0.5).max(min_h * 0.5);
                    x0 = cx - half_w;
                    x1 = cx + half_w;
                    y0 = cy - half_h;
                    y1 = cy + half_h;
                }
                let nb = Rect::new(x0, y0, (x1 - x0).max(min_w), (y1 - y0).max(min_h));
                app.bridge.scene.resize_node(rid, nb, modifiers.shift);
                if let Some(nm) = app.bridge.scene.get_node_mut(rid) {
                    if matches!(nm.port_density, rustflow_core::PortDensity::Auto) {
                        nm.rebuild_ports();
                    }
                }
                ui.ctx().set_cursor_icon(resize_handle_cursor(handle));
            }
        } else if app.dragging_nodes {
            if let Some(cur_pos) = pointer_pos.or(response.interact_pointer_pos()) {
                let cur_w = screen_to_world(canvas_rect, app, cur_pos);
                let mut dx = cur_w.x - app.drag_origin_world.x;
                let mut dy = cur_w.y - app.drag_origin_world.y;
                app.active_guides.clear();

                let drag_speed = ui.input(|i| i.pointer.velocity().length());
                if !modifiers.alt && drag_speed < 380.0 {
                    if let Some(&(primary_id, primary_start)) = app.drag_start_positions.first() {
                        if let Some(pnode) = app.bridge.scene.get_node(primary_id) {
                            let pw = pnode.bounds.width;
                            let ph = pnode.bounds.height;
                            let raw_x = primary_start.x + dx;
                            let raw_y = primary_start.y + dy;
                            let thresh = (3.5 / app.bridge.viewport.zoom).clamp(2.5, 5.5);

                            let mut best_x: Option<(f32, f32, f32, bool)> = None;
                            let mut best_y: Option<(f32, f32, f32, bool)> = None;

                            if app.bridge.scene.nodes().len() <= 60 {
                                for other in app.bridge.scene.nodes() {
                                    if !other.visible
                                        || app.drag_start_positions.iter().any(|(id, _)| *id == other.id)
                                    {
                                        continue;
                                    }
                                    let ob = other.bounds;
                                    let close_y = (raw_y + ph * 0.5 - ob.center().y).abs() <= 420.0;
                                    let close_x = (raw_x + pw * 0.5 - ob.center().x).abs() <= 420.0;
                                    if close_y {
                                        let x_pairs = [
                                            (raw_x + pw * 0.5, ob.center().x, true),
                                            (raw_x, ob.x, false),
                                            (raw_x + pw, ob.right(), false),
                                        ];
                                        for (a, target, is_center) in x_pairs {
                                            let diff = target - a;
                                            let ad = diff.abs();
                                            let score = if is_center { ad * 0.7 } else { ad };
                                            if ad <= thresh && best_x.is_none_or(|(bd, ..)| score < bd) {
                                                best_x = Some((score, diff, target, is_center));
                                            }
                                        }
                                    }
                                    if close_x {
                                        let y_pairs = [
                                            (raw_y + ph * 0.5, ob.center().y, true),
                                            (raw_y, ob.y, false),
                                            (raw_y + ph, ob.bottom(), false),
                                        ];
                                        for (a, target, is_center) in y_pairs {
                                            let diff = target - a;
                                            let ad = diff.abs();
                                            let score = if is_center { ad * 0.7 } else { ad };
                                            if ad <= thresh && best_y.is_none_or(|(bd, ..)| score < bd) {
                                                best_y = Some((score, diff, target, is_center));
                                            }
                                        }
                                    }
                                }
                            }

                            for edge in app.bridge.scene.edges() {
                                let pair = if edge.source_node == primary_id
                                    && !app.drag_start_positions.iter().any(|(id, _)| *id == edge.target_node)
                                {
                                    Some((edge.source_port, edge.target_node, edge.target_port))
                                } else if edge.target_node == primary_id
                                    && !app.drag_start_positions.iter().any(|(id, _)| *id == edge.source_node)
                                {
                                    Some((edge.target_port, edge.source_node, edge.source_port))
                                } else {
                                    None
                                };
                                if let Some((my_pid, other_nid, other_pid)) = pair {
                                    if let Some(other_node) = app.bridge.scene.get_node(other_nid) {
                                        let my_rel = my_pid
                                            .and_then(|pid| pnode.port_position(pid))
                                            .map(|pt| Point::new(pt.x - pnode.bounds.x, pt.y - pnode.bounds.y))
                                            .unwrap_or_else(|| Point::new(pw * 0.5, ph * 0.5));
                                        let other_pt = other_pid
                                            .and_then(|pid| other_node.port_position(pid))
                                            .unwrap_or_else(|| other_node.bounds.center());
                                        let diff_x = other_pt.x - (raw_x + my_rel.x);
                                        let ad_x = diff_x.abs();
                                        let score_x = ad_x * 0.5;
                                        if ad_x <= thresh && best_x.is_none_or(|(bd, ..)| score_x < bd) {
                                            best_x = Some((score_x, diff_x, other_pt.x, true));
                                        }
                                        let diff_y = other_pt.y - (raw_y + my_rel.y);
                                        let ad_y = diff_y.abs();
                                        let score_y = ad_y * 0.5;
                                        if ad_y <= thresh && best_y.is_none_or(|(bd, ..)| score_y < bd) {
                                            best_y = Some((score_y, diff_y, other_pt.y, true));
                                        }
                                    }
                                }
                            }

                            if let Some((_, snap_dx, guide_x, is_c)) = best_x {
                                dx += snap_dx;
                                app.active_guides.push((true, guide_x, is_c));
                            }

                            if let Some((_, snap_dy, guide_y, is_c)) = best_y {
                                dy += snap_dy;
                                app.active_guides.push((false, guide_y, is_c));
                            }
                        }
                    }
                }

                for i in 0..app.drag_start_positions.len() {
                    let (nid, start_pt) = app.drag_start_positions[i];
                    if let Some(node) = app.bridge.scene.get_node_mut(nid) {
                        if !node.locked {
                            node.bounds.x = start_pt.x + dx;
                            node.bounds.y = start_pt.y + dy;
                        }
                    }
                }
            }
        } else if let Some((start_s, _)) = app.marquee_screen {
            if let Some(cur) = response.interact_pointer_pos() {
                app.marquee_screen = Some((start_s, cur));
            }
        }
    }

    if response.drag_stopped() || (!primary_down && !middle_down && app.is_interacting()) {
        if let Some((src_nid, src_pid)) = app.connecting_from.take() {
            if let Some(pos) = response.interact_pointer_pos().or(pointer_pos) {
                let wp = screen_to_world(canvas_rect, app, pos);
                if let Some((dst_nid, dst_pid, _)) = app
                    .bridge
                    .scene
                    .resolve_connection_target_from_port(
                        wp,
                        34.0 / app.bridge.viewport.zoom,
                        src_nid,
                        src_pid,
                    )
                {
                    app.push_undo();
                    let eid = app.bridge.scene.allocate_id();
                    let mut edge = Edge::new_bezier(eid, src_nid, dst_nid);
                    edge.source_port = Some(src_pid);
                    edge.target_port = dst_pid;
                    edge.kind = app.default_edge_kind;
                    edge.arrow = app.default_edge_arrow;
                    edge.animation = app.default_edge_animation(edge.stroke);
                    app.bridge.scene.insert_edge(edge);
                    app.broadcast_scene_snapshot();
                }
            }
        }
        if app.dragging_nodes {
            if app.snap_to_grid && app.active_guides.is_empty() && !modifiers.alt {
                let g = app.bridge.viewport.grid_size.max(4.0);
                if let Some(&(primary_id, _)) = app.drag_start_positions.first() {
                    if let Some(n) = app.bridge.scene.get_node(primary_id) {
                        let sdx = (n.bounds.x / g).round() * g - n.bounds.x;
                        let sdy = (n.bounds.y / g).round() * g - n.bounds.y;
                        let moved: Vec<NodeId> =
                            app.drag_start_positions.iter().map(|(id, _)| *id).collect();
                        for mid in moved {
                            if let Some(mn) = app.bridge.scene.get_node(mid) {
                                app.bridge.scene.move_node_to(
                                    mid,
                                    Point::new(mn.bounds.x + sdx, mn.bounds.y + sdy),
                                );
                            }
                        }
                    }
                }
            }
            app.bridge.scene.rebuild_spatial_index();
            let ids = app.bridge.selection.selected_nodes.clone();
            for nid in ids {
                app.bridge.scene.auto_attach_to_section(nid);
            }
            app.broadcast_scene_snapshot();
        }
        if let Some((eid, wp_idx)) = app.dragging_waypoint.take() {
            if app.snap_to_grid && !modifiers.alt {
                let g = app.bridge.viewport.grid_size.max(4.0);
                if let Some(e) = app.bridge.scene.get_edge_mut(eid) {
                    if let Some(pt) = e.waypoints.get_mut(wp_idx) {
                        pt.x = (pt.x / g).round() * g;
                        pt.y = (pt.y / g).round() * g;
                    }
                }
            }
            app.broadcast_scene_snapshot();
        }
        if app.resizing_node.is_some() {
            app.broadcast_scene_snapshot();
        }
        if let Some((start_s, cur_s)) = app.marquee_screen.take() {
            let mrect = EguiRect::from_two_pos(start_s, cur_s);
            let w0 = screen_to_world(canvas_rect, app, mrect.min);
            let w1 = screen_to_world(canvas_rect, app, mrect.max);
            let wrect = Rect::new(w0.x, w0.y, (w1.x - w0.x).abs(), (w1.y - w0.y).abs());
            app.bridge.selection.clear();
            for n in app.bridge.scene.nodes() {
                if n.visible && wrect.intersects(&n.bounds) {
                    app.bridge.selection.selected_nodes.push(n.id);
                }
            }
        }
        app.panning = false;
        app.dragging_nodes = false;
        app.drag_start_positions.clear();
        app.active_guides.clear();
        app.resizing_node = None;
    }

    if response.clicked() {
        app.finish_inline_edit(true);
        if let Some(pos) = response.interact_pointer_pos() {
            let wp = screen_to_world(canvas_rect, app, pos);
            if let Some(nid) = app.bridge.scene.hit_test_node(wp) {
                let mut toggled_special = false;
                if let Some(node) = app.bridge.scene.get_node_mut(nid) {
                    if matches!(node.shape, ShapeKind::Collapsible) && wp.y <= node.bounds.y + 32.0 {
                        node.collapsed = !node.collapsed;
                        toggled_special = true;
                    } else if matches!(node.shape, ShapeKind::FolderTree) && wp.y > node.bounds.y + 24.0 {
                        let row_idx = ((wp.y - node.bounds.y - 26.0) / 20.0).floor() as usize;
                        let mut visible_idx = 0usize;
                        let mut hidden_depth: Option<u8> = None;
                        for item in &mut node.tree_items {
                            if let Some(hd) = hidden_depth {
                                if item.depth > hd {
                                    continue;
                                } else {
                                    hidden_depth = None;
                                }
                            }
                            if visible_idx == row_idx && item.is_folder {
                                item.expanded = !item.expanded;
                                toggled_special = true;
                                break;
                            }
                            if item.is_folder && !item.expanded {
                                hidden_depth = Some(item.depth);
                            }
                            visible_idx += 1;
                        }
                    }
                }
                if !toggled_special {
                    if app.collab.locked_by_other(nid).is_some() {
                        let label = app
                            .bridge
                            .scene
                            .get_node(nid)
                            .map(|n| n.title.clone())
                            .unwrap_or_else(|| format!("Node {nid}"));
                        app.collab.send(&ClientMsg::DuelStart {
                            item_id: nid,
                            item_label: label,
                        });
                    } else {
                        app.bridge.selection.select_node(nid, modifiers.shift);
                        app.collab.send(&ClientMsg::Lock { item_id: nid });
                    }
                }
            } else if let Some(eid) = hit_test_edge(app, wp, 10.0 / app.bridge.viewport.zoom) {
                app.bridge.selection.clear();
                app.bridge.selection.selected_edges.push(eid);
            } else {
                app.bridge.selection.clear();
            }
        }
    }
}

pub(super) fn render_context_menu(app: &mut BohatDrawApp, response: &egui::Response, canvas_rect: EguiRect) {
    response.context_menu(|ui| {
        if let Some(pos) = ui.input(|i| i.pointer.interact_pos()) {
            let wp = screen_to_world(canvas_rect, app, pos);
            if app.bridge.selection.selected_nodes.is_empty()
                && app.bridge.selection.selected_edges.is_empty()
            {
                if let Some(nid) = app.bridge.scene.hit_test_node(wp) {
                    app.bridge.selection.select_node(nid, false);
                } else if let Some(eid) = hit_test_edge(app, wp, 10.0 / app.bridge.viewport.zoom) {
                    app.bridge.selection.selected_edges.push(eid);
                }
            }
        }

        if !app.bridge.selection.selected_nodes.is_empty() {
            let ids = app.bridge.selection.selected_nodes.clone();
            ui.label(format!("Item ({})", ids.len()));
            ui.separator();
            if ids.len() == 1 && ui.button("Edit Text").clicked() {
                app.start_inline_edit(ids[0], false);
                ui.close_menu();
            }
            ui.menu_button("Layer Order", |ui| {
                if ui.button("Bring to Front").clicked() {
                    app.push_undo();
                    app.bridge.scene.bring_to_front(&ids);
                    app.broadcast_scene_snapshot();
                    ui.close_menu();
                }
                if ui.button("Bring Forward").clicked() {
                    app.push_undo();
                    app.bridge.scene.bring_forward(&ids);
                    app.broadcast_scene_snapshot();
                    ui.close_menu();
                }
                if ui.button("Send Backward").clicked() {
                    app.push_undo();
                    app.bridge.scene.send_backward(&ids);
                    app.broadcast_scene_snapshot();
                    ui.close_menu();
                }
                if ui.button("Send to Back").clicked() {
                    app.push_undo();
                    app.bridge.scene.send_to_back(&ids);
                    app.broadcast_scene_snapshot();
                    ui.close_menu();
                }
            });
            ui.menu_button("Border Style", |ui| {
                for (name, bs) in [
                    ("Solid", BorderStyle::Solid),
                    ("Dashed", BorderStyle::Dashed),
                    ("Dotted", BorderStyle::Dotted),
                    ("Dash-Dot", BorderStyle::DashDot),
                    ("Double", BorderStyle::Double),
                    ("None", BorderStyle::None),
                ] {
                    if ui.button(name).clicked() {
                        app.push_undo();
                        for &id in &ids {
                            if let Some(n) = app.bridge.scene.get_node_mut(id) {
                                n.style.border_style = bs;
                            }
                        }
                        app.broadcast_scene_snapshot();
                        ui.close_menu();
                    }
                }
            });
            if ui.button("Group into Section").clicked() {
                app.push_undo();
                app.bridge.scene.group_into_section(&ids, "Section");
                app.broadcast_scene_snapshot();
                ui.close_menu();
            }
            if ui.button("Detach from Section").clicked() {
                app.push_undo();
                app.bridge.scene.detach_from_parent(&ids);
                app.broadcast_scene_snapshot();
                ui.close_menu();
            }
            if ui.button("Copy").clicked() {
                app.copy_selection();
                ui.close_menu();
            }
            if ui.button("Duplicate").clicked() {
                app.duplicate_selection();
                ui.close_menu();
            }
            if ui.button("Delete").clicked() {
                app.delete_selection();
                ui.close_menu();
            }
            ui.separator();
        } else if let Some(&eid) = app.bridge.selection.selected_edges.first() {
            ui.label("Edge");
            ui.separator();
            ui.menu_button("Routing", |ui| {
                for (name, kind) in [
                    ("Bezier", EdgeKind::Bezier),
                    ("Smooth Step", EdgeKind::SmoothStep),
                    ("Straight", EdgeKind::Straight),
                    ("Orthogonal", EdgeKind::Orthogonal),
                ] {
                    if ui.button(name).clicked() {
                        if let Some(e) = app.bridge.scene.get_edge_mut(eid) {
                            e.kind = kind;
                        }
                        app.broadcast_scene_snapshot();
                        ui.close_menu();
                    }
                }
            });
            ui.menu_button("Arrowhead", |ui| {
                for (name, arr) in [
                    ("1-Way", EdgeArrowStyle::OneWay),
                    ("2-Way", EdgeArrowStyle::TwoWay),
                    ("None", EdgeArrowStyle::None),
                    ("Open", EdgeArrowStyle::Open),
                    ("Diamond", EdgeArrowStyle::Diamond),
                ] {
                    if ui.button(name).clicked() {
                        if let Some(e) = app.bridge.scene.get_edge_mut(eid) {
                            e.arrow = arr;
                        }
                        app.broadcast_scene_snapshot();
                        ui.close_menu();
                    }
                }
            });
            ui.menu_button("Animation", |ui| {
                if ui.button("Static").clicked() {
                    if let Some(e) = app.bridge.scene.get_edge_mut(eid) {
                        e.animation = EdgeAnimation::None;
                    }
                    ui.close_menu();
                }
                if ui.button("Marching Ants").clicked() {
                    if let Some(e) = app.bridge.scene.get_edge_mut(eid) {
                        e.animation = EdgeAnimation::MarchingAnts {
                            speed: 45.0,
                            dash_length: 8.0,
                            gap_length: 5.0,
                            reverse: false,
                        };
                    }
                    ui.close_menu();
                }
                if ui.button("Pulse Particles").clicked() {
                    if let Some(e) = app.bridge.scene.get_edge_mut(eid) {
                        e.animation = EdgeAnimation::PulseParticles {
                            speed: 65.0,
                            particle_count: 3,
                            radius: 4.0,
                            color: rustflow_core::ColorRgba::INDIGO_500,
                        };
                    }
                    ui.close_menu();
                }
            });
            let has_manual = app
                .bridge
                .scene
                .get_edge(eid)
                .is_some_and(|e| !e.waypoints.is_empty());
            if has_manual && ui.button("Reset to Auto Route").clicked() {
                app.push_undo();
                if let Some(e) = app.bridge.scene.get_edge_mut(eid) {
                    e.waypoints.clear();
                }
                app.broadcast_scene_snapshot();
                ui.close_menu();
            }
            if ui.button("Reverse Direction").clicked() {
                if let Some(e) = app.bridge.scene.get_edge_mut(eid) {
                    std::mem::swap(&mut e.source_node, &mut e.target_node);
                    std::mem::swap(&mut e.source_port, &mut e.target_port);
                }
                app.broadcast_scene_snapshot();
                ui.close_menu();
            }
            if ui.button("Delete Edge").clicked() {
                app.push_undo();
                app.bridge.scene.remove_edge(eid);
                app.bridge.selection.clear();
                app.broadcast_scene_snapshot();
                ui.close_menu();
            }
            ui.separator();
        }

        if !app.clipboard_nodes.is_empty() && ui.button("Paste").clicked() {
            app.paste_clipboard();
            ui.close_menu();
        }
        if ui.button("Copy as 2x PNG").clicked() {
            app.copy_diagram_png();
            ui.close_menu();
        }
        if ui.button("Copy as Animated GIF").clicked() {
            app.copy_diagram_gif();
            ui.close_menu();
        }
        if ui.button("Fit View").clicked() {
            app.fit_view();
            ui.close_menu();
        }
        let map_lbl = if app.show_minimap {
            "Hide Minimap"
        } else {
            "Show Minimap"
        };
        if ui.button(map_lbl).clicked() {
            app.show_minimap = !app.show_minimap;
            ui.close_menu();
        }
    });
}

pub(super) fn resize_handle_positions(r: EguiRect) -> [(ResizeHandle, Pos2); 8] {
    let cx = r.center().x;
    let cy = r.center().y;
    [
        (ResizeHandle::NW, Pos2::new(r.min.x, r.min.y)),
        (ResizeHandle::N, Pos2::new(cx, r.min.y)),
        (ResizeHandle::NE, Pos2::new(r.max.x, r.min.y)),
        (ResizeHandle::E, Pos2::new(r.max.x, cy)),
        (ResizeHandle::SE, Pos2::new(r.max.x, r.max.y)),
        (ResizeHandle::S, Pos2::new(cx, r.max.y)),
        (ResizeHandle::SW, Pos2::new(r.min.x, r.max.y)),
        (ResizeHandle::W, Pos2::new(r.min.x, cy)),
    ]
}

fn resize_handle_cursor(h: ResizeHandle) -> egui::CursorIcon {
    match h {
        ResizeHandle::NW | ResizeHandle::SE => egui::CursorIcon::ResizeNwSe,
        ResizeHandle::NE | ResizeHandle::SW => egui::CursorIcon::ResizeNeSw,
        ResizeHandle::N | ResizeHandle::S => egui::CursorIcon::ResizeVertical,
        ResizeHandle::E | ResizeHandle::W => egui::CursorIcon::ResizeHorizontal,
    }
}

fn hit_test_resize_handle(
    app: &BohatDrawApp,
    canvas_rect: EguiRect,
    pos: Pos2,
) -> Option<(NodeId, ResizeHandle, Rect)> {
    for &sid in &app.bridge.selection.selected_nodes {
        if let Some(n) = app.bridge.scene.get_node(sid) {
            if n.locked {
                continue;
            }
            let box_rect = rect_to_screen(canvas_rect, app, n.bounds).expand(3.0);
            let show_mid = box_rect.width() >= 48.0 && box_rect.height() >= 40.0;
            for (kind, hpos) in resize_handle_positions(box_rect) {
                if !show_mid
                    && matches!(
                        kind,
                        ResizeHandle::N | ResizeHandle::E | ResizeHandle::S | ResizeHandle::W
                    )
                {
                    continue;
                }
                if hpos.distance(pos) <= 9.0 {
                    return Some((sid, kind, n.bounds));
                }
            }
        }
    }
    None
}

pub(super) fn render_selection_floating_bar(app: &mut BohatDrawApp, ui: &mut egui::Ui, canvas_rect: EguiRect) {
    let mut sel_rect: Option<EguiRect> = None;
    for &nid in &app.bridge.selection.selected_nodes {
        if let Some(n) = app.bridge.scene.get_node(nid) {
            let sr = rect_to_screen(canvas_rect, app, n.bounds);
            sel_rect = Some(match sel_rect {
                Some(r) => r.union(sr),
                None => sr,
            });
        }
    }
    let has_nodes = !app.bridge.selection.selected_nodes.is_empty();
    let has_edges = !app.bridge.selection.selected_edges.is_empty();
    if !has_nodes && has_edges {
        if let Some(&eid) = app.bridge.selection.selected_edges.first() {
            if let Some(e) = app.bridge.scene.get_edge(eid) {
                let pts = tessellate_edge_path(&app.bridge.scene, e);
                if !pts.is_empty() {
                    let mid = world_to_screen(canvas_rect, app, pts[pts.len() / 2]);
                    sel_rect = Some(EguiRect::from_center_size(mid, Vec2::new(20.0, 20.0)));
                }
            }
        }
    }
    let Some(sr) = sel_rect else {
        return;
    };
    if !canvas_rect.intersects(sr.expand(40.0)) {
        return;
    }

    let bar_w = if has_nodes { 216.0 } else { 228.0 };
    let bar_h = 28.0;
    let px = (sr.center().x - bar_w * 0.5)
        .clamp(canvas_rect.min.x + 8.0, (canvas_rect.max.x - bar_w - 8.0).max(canvas_rect.min.x + 8.0));
    let above_y = sr.min.y - bar_h - 12.0;
    let py = if above_y >= canvas_rect.min.y + 8.0 {
        above_y
    } else {
        (sr.max.y + 12.0).min((canvas_rect.max.y - bar_h - 8.0).max(canvas_rect.min.y + 8.0))
    };

    let is_dark = matches!(app.theme_mode, rustflow_core::ThemeMode::Dark);
    let bg = if is_dark {
        Color32::from_rgba_unmultiplied(36, 40, 50, 242)
    } else {
        Color32::from_rgba_unmultiplied(255, 255, 255, 244)
    };
    let border = if is_dark {
        Color32::from_rgb(72, 78, 94)
    } else {
        Color32::from_rgb(208, 214, 226)
    };

    egui::Area::new(egui::Id::new("selection_quick_toolbar"))
        .order(egui::Order::Foreground)
        .fixed_pos(Pos2::new(px, py))
        .show(ui.ctx(), |ui| {
            egui::Frame::new()
                .fill(bg)
                .stroke(Stroke::new(1.0_f32, border))
                .corner_radius(egui::CornerRadius::same(8))
                .shadow(egui::epaint::Shadow {
                    offset: [0, 3],
                    blur: 10,
                    spread: 0,
                    color: Color32::from_black_alpha(32),
                })
                .inner_margin(egui::Margin::symmetric(6, 3))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = 4.0;
                        if has_nodes {
                            let quick_fills = [
                                ("White", Color32::WHITE),
                                ("Blue", Color32::from_rgb(219, 234, 254)),
                                ("Mint", Color32::from_rgb(209, 250, 229)),
                                ("Amber", Color32::from_rgb(254, 243, 199)),
                                ("Rose", Color32::from_rgb(255, 228, 230)),
                            ];
                            for (name, c32) in quick_fills {
                                let (r, resp) =
                                    ui.allocate_exact_size(Vec2::splat(16.0), Sense::click());
                                ui.painter().circle_filled(r.center(), 6.5, c32);
                                ui.painter().circle_stroke(
                                    r.center(),
                                    6.5,
                                    Stroke::new(1.0_f32, Color32::from_rgb(148, 163, 184)),
                                );
                                if resp.on_hover_text(format!("Fill: {name}")).clicked() {
                                    app.push_undo();
                                    let rgba = crate::theme::from_color32(c32);
                                    let ids = app.bridge.selection.selected_nodes.clone();
                                    for id in ids {
                                        if let Some(n) = app.bridge.scene.get_node_mut(id) {
                                            n.style.fill = rgba;
                                        }
                                    }
                                    app.broadcast_scene_snapshot();
                                }
                            }
                            ui.separator();
                            if ui
                                .small_button("Dup")
                                .on_hover_text("Duplicate (Ctrl+D)")
                                .clicked()
                            {
                                app.duplicate_selection();
                            }
                            if ui
                                .small_button("Front")
                                .on_hover_text("Bring to Front")
                                .clicked()
                            {
                                app.push_undo();
                                let ids = app.bridge.selection.selected_nodes.clone();
                                app.bridge.scene.bring_to_front(&ids);
                                app.broadcast_scene_snapshot();
                            }
                            if ui
                                .small_button(
                                    egui::RichText::new("Del")
                                        .color(Color32::from_rgb(220, 38, 38)),
                                )
                                .on_hover_text("Delete (Del)")
                                .clicked()
                            {
                                app.delete_selection();
                            }
                        } else if let Some(&eid) = app.bridge.selection.selected_edges.first() {
                            let cur_anim = app.bridge.scene.get_edge(eid).map(|e| match e.animation {
                                EdgeAnimation::None => 0u8,
                                EdgeAnimation::MarchingAnts { .. } => 1u8,
                                EdgeAnimation::PulseParticles { .. } => 2u8,
                            }).unwrap_or(0);
                            for (mode, lbl) in [(0u8, "Static"), (1, "Ants"), (2, "Pulse")] {
                                if ui.selectable_label(cur_anim == mode, lbl).clicked() {
                                    app.push_undo();
                                    let eids = app.bridge.selection.selected_edges.clone();
                                    for id in eids {
                                        if let Some(e) = app.bridge.scene.get_edge_mut(id) {
                                            let st = e.stroke;
                                            e.animation = match mode {
                                                1 => EdgeAnimation::MarchingAnts {
                                                    speed: 45.0,
                                                    dash_length: 8.0,
                                                    gap_length: 5.0,
                                                    reverse: false,
                                                },
                                                2 => EdgeAnimation::PulseParticles {
                                                    speed: 65.0,
                                                    particle_count: 3,
                                                    radius: 4.0,
                                                    color: st,
                                                },
                                                _ => EdgeAnimation::None,
                                            };
                                        }
                                    }
                                    app.broadcast_scene_snapshot();
                                }
                            }
                            ui.separator();
                            if ui.small_button("Label").on_hover_text("Edit Label").clicked() {
                                app.start_edge_inline_edit(eid);
                            }
                            if ui
                                .small_button(
                                    egui::RichText::new("Del")
                                        .color(Color32::from_rgb(220, 38, 38)),
                                )
                                .clicked()
                            {
                                app.delete_selection();
                            }
                        }
                    });
                });
        });
}
