#![allow(unused_imports)]
mod interaction;
mod minimap;
mod render;

pub use render::draw_simple_layer_shape;
use interaction::{
    handle_canvas_input, hit_test_port, render_context_menu, render_selection_floating_bar,
};
use minimap::{draw_minimap, draw_minimap_restore_pill};
use render::{
    draw_arrowhead, draw_canvas_background, draw_edge, draw_node, draw_remote_cursors,
    draw_smart_guides, draw_stroked_polyline, find_end_tangent,
};

use crate::app::{BohatDrawApp, CanvasBgStyle, ResizeHandle};
use crate::theme::{canvas_bg_color, draw_symbol_icon, symbol_icon_accent, to_color32};

pub fn node_supports_body(shape: &ShapeKind) -> bool {
    matches!(
        shape,
        ShapeKind::ReactFlowCard
            | ShapeKind::Rectangle
            | ShapeKind::RoundedRect
            | ShapeKind::Note
            | ShapeKind::Document
            | ShapeKind::Collapsible
            | ShapeKind::BrowserWindow
            | ShapeKind::MobileScreen
            | ShapeKind::Callout
            | ShapeKind::Package
            | ShapeKind::Component
    )
}
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

pub fn world_to_screen(canvas_rect: EguiRect, app: &BohatDrawApp, pt: Point) -> Pos2 {
    let vp = &app.bridge.viewport;
    Pos2::new(
        canvas_rect.min.x + pt.x * vp.zoom + vp.pan.x,
        canvas_rect.min.y + pt.y * vp.zoom + vp.pan.y,
    )
}

pub fn screen_to_world(canvas_rect: EguiRect, app: &BohatDrawApp, pos: Pos2) -> Point {
    let vp = &app.bridge.viewport;
    Point::new(
        (pos.x - canvas_rect.min.x - vp.pan.x) / vp.zoom,
        (pos.y - canvas_rect.min.y - vp.pan.y) / vp.zoom,
    )
}

pub fn rect_to_screen(canvas_rect: EguiRect, app: &BohatDrawApp, r: Rect) -> EguiRect {
    let min = world_to_screen(canvas_rect, app, Point::new(r.x, r.y));
    let max = world_to_screen(canvas_rect, app, Point::new(r.right(), r.bottom()));
    EguiRect::from_min_max(min, max)
}

pub fn render_canvas(app: &mut BohatDrawApp, ui: &mut egui::Ui) {
    let available = ui.available_size();
    let (response, painter) = ui.allocate_painter(available, Sense::click_and_drag());
    let canvas_rect = response.rect;
    app.bridge.viewport.screen_width = canvas_rect.width().max(1.0);
    app.bridge.viewport.screen_height = canvas_rect.height().max(1.0);

    handle_canvas_input(app, ui, &response, canvas_rect);

    let bg = canvas_bg_color(app.theme_mode);
    painter.rect_filled(canvas_rect, 0.0, bg);

    if app.show_grid && !matches!(app.bg_style, CanvasBgStyle::None) {
        draw_canvas_background(&painter, canvas_rect, app, ui.ctx());
    }

    let now_sec = ui.input(|i| i.time);
    let now_f32 = now_sec as f32;
    let pointer_pos = ui.input(|i| i.pointer.latest_pos().or(i.pointer.hover_pos()));
    let busy_dragging = app.dragging_nodes
        || app.panning
        || app.resizing_node.is_some()
        || app.marquee_screen.is_some();
    let hovered_port = if busy_dragging {
        None
    } else {
        pointer_pos.and_then(|p| hit_test_port(app, canvas_rect, p))
    };
    if hovered_port.is_some() || app.connecting_from.is_some() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::Crosshair);
    }
    let hovered_node = if busy_dragging {
        None
    } else {
        hovered_port.map(|(nid, _)| nid).or_else(|| {
            pointer_pos.and_then(|p| {
                app.bridge
                    .scene
                    .nodes()
                    .iter()
                    .filter(|n| n.visible && rect_to_screen(canvas_rect, app, n.bounds).expand(16.0).contains(p))
                    .max_by_key(|n| (n.z_index, n.id))
                    .map(|n| n.id)
            })
        })
    };

    let mut node_indices: Vec<usize> = app
        .bridge
        .scene
        .nodes()
        .iter()
        .enumerate()
        .filter(|(_, n)| n.visible)
        .map(|(i, _)| i)
        .collect();
    node_indices.sort_by_key(|&i| {
        let n = &app.bridge.scene.nodes()[i];
        (n.z_index, n.id)
    });

    let skip_shadows = app.is_interacting() || node_indices.len() > 36;
    let cull_rect = canvas_rect.expand(48.0);
    for &idx in &node_indices {
        let node = &app.bridge.scene.nodes()[idx];
        if !(node.is_section || matches!(node.shape, ShapeKind::Section)) {
            continue;
        }
        if !rect_to_screen(canvas_rect, app, node.bounds).intersects(cull_rect) {
            continue;
        }
        let is_selected = app.bridge.selection.selected_nodes.contains(&node.id);
        let is_hovered = hovered_node == Some(node.id);
        draw_node(
            &painter,
            canvas_rect,
            app,
            node,
            is_selected,
            is_hovered,
            hovered_port,
            pointer_pos,
            busy_dragging,
            skip_shadows,
            now_f32,
        );
    }

    let mut placed_labels: Vec<Rect> = Vec::with_capacity(app.bridge.scene.edges().len());
    let edge_count = app.bridge.scene.edges().len();
    for edge_idx in 0..edge_count {
        draw_edge(&painter, canvas_rect, app, edge_idx, now_f32, &mut placed_labels);
    }

    for idx in node_indices {
        let node = &app.bridge.scene.nodes()[idx];
        if node.is_section || matches!(node.shape, ShapeKind::Section) {
            continue;
        }
        if !rect_to_screen(canvas_rect, app, node.bounds).intersects(cull_rect) {
            continue;
        }
        let is_selected = app.bridge.selection.selected_nodes.contains(&node.id);
        let is_hovered = hovered_node == Some(node.id);
        draw_node(
            &painter,
            canvas_rect,
            app,
            node,
            is_selected,
            is_hovered,
            hovered_port,
            pointer_pos,
            busy_dragging,
            skip_shadows,
            now_f32,
        );
    }

    if app.dragging_nodes && !app.active_guides.is_empty() {
        draw_smart_guides(&painter, canvas_rect, app);
    }

    if let Some((src_nid, src_pid)) = app.connecting_from {
        if let Some(src_node) = app.bridge.scene.get_node(src_nid) {
            let src_bounds = src_node.bounds;
            let start_w = src_node
                .port_position(src_pid)
                .unwrap_or_else(|| src_node.bounds.center());
            let start_side = src_node.port_side(src_pid).unwrap_or(PortSide::Right);
            if let Some(cur_pos) = pointer_pos {
                let cur_w = screen_to_world(canvas_rect, app, cur_pos);
                let snap_r = 34.0 / app.bridge.viewport.zoom;
                let snapped = app
                    .bridge
                    .scene
                    .resolve_connection_target_from_port(cur_w, snap_r, src_nid, src_pid);
                let (end_w, end_side, is_self_target) = if let Some((dst_nid, dst_pid, pt)) = snapped {
                    let side = dst_pid
                        .and_then(|pid| app.bridge.scene.get_node(dst_nid)?.port_side(pid))
                        .unwrap_or(PortSide::Left);
                    let snap_s = world_to_screen(canvas_rect, app, pt);
                    painter.circle_stroke(
                        snap_s,
                        9.0,
                        Stroke::new(2.0_f32, Color32::from_rgb(16, 185, 129)),
                    );
                    painter.circle_filled(snap_s, 4.5, Color32::from_rgb(16, 185, 129));
                    (pt, side, dst_nid == src_nid)
                } else {
                    let dx = cur_w.x - start_w.x;
                    let dy = cur_w.y - start_w.y;
                    let inferred = if dx.abs() >= dy.abs() {
                        if dx >= 0.0 { PortSide::Left } else { PortSide::Right }
                    } else if dy >= 0.0 {
                        PortSide::Top
                    } else {
                        PortSide::Bottom
                    };
                    (cur_w, inferred, false)
                };
                let preview_w = if is_self_target {
                    tessellate_self_loop(
                        src_bounds,
                        start_w,
                        end_w,
                        start_side,
                        end_side,
                        app.default_edge_kind,
                    )
                } else {
                    match app.default_edge_kind {
                        EdgeKind::Straight => vec![start_w, end_w],
                        EdgeKind::Bezier => tessellate_cubic_bezier(start_w, end_w, start_side, end_side, 20),
                        EdgeKind::SmoothStep | EdgeKind::Orthogonal => {
                            tessellate_smooth_step_oriented(start_w, end_w, start_side, end_side)
                        }
                    }
                };
                let preview_s: Vec<Pos2> = preview_w
                    .iter()
                    .map(|&p| world_to_screen(canvas_rect, app, p))
                    .collect();
                let preview_col = if snapped.is_some() {
                    Color32::from_rgb(16, 185, 129)
                } else {
                    Color32::from_rgb(0, 0, 140)
                };
                draw_stroked_polyline(
                    &painter,
                    &preview_s,
                    Stroke::new(2.0_f32, preview_col),
                    BorderStyle::Dashed,
                    -now_f32 * 40.0,
                );
                if let Some((tip, prev)) = find_end_tangent(&preview_s) {
                    draw_arrowhead(
                        &painter,
                        tip,
                        prev,
                        preview_col,
                        app.default_edge_arrow,
                        app.bridge.viewport.zoom,
                    );
                }
            }
        }
    }

    if let Some((start_s, cur_s)) = app.marquee_screen {
        let mrect = EguiRect::from_two_pos(start_s, cur_s);
        painter.rect(
            mrect,
            0.0,
            Color32::from_rgba_unmultiplied(0, 0, 128, 28),
            Stroke::new(1.0_f32, Color32::from_rgb(0, 0, 128)),
            egui::StrokeKind::Inside,
        );
    }

    draw_remote_cursors(&painter, canvas_rect, app);

    if canvas_rect.width() > 480.0 && canvas_rect.height() > 340.0 {
        if app.show_minimap {
            draw_minimap(&painter, canvas_rect, app, pointer_pos);
        } else {
            draw_minimap_restore_pill(&painter, canvas_rect, pointer_pos);
        }
    }

    if let Some((msg, until)) = app.toast.clone() {
        if now_sec <= until {
            let w = (msg.len() as f32 * 6.8 + 24.0).min(canvas_rect.width() - 32.0);
            let trect = EguiRect::from_center_size(
                Pos2::new(canvas_rect.center().x, canvas_rect.min.y + 22.0),
                Vec2::new(w, 24.0),
            );
            painter.rect_filled(trect, 3.0, Color32::from_rgb(0, 0, 128));
            painter.text(
                trect.center(),
                Align2::CENTER_CENTER,
                msg,
                FontId::proportional(11.5),
                Color32::WHITE,
            );
            ui.ctx().request_repaint();
        } else {
            app.toast = None;
        }
    }

    if let Some(edit_id) = app.editing_node {
        if let Some((bounds, supports_body)) = app
            .bridge
            .scene
            .get_node(edit_id)
            .map(|n| (n.bounds, node_supports_body(&n.shape)))
        {
            let srect = rect_to_screen(canvas_rect, app, bounds);
            let editor_w = if supports_body {
                srect.width().clamp(200.0, 290.0)
            } else {
                srect.width().clamp(160.0, 240.0)
            };
            let est_h = if supports_body { 150.0 } else { 36.0 };
            let px = (srect.center().x - editor_w * 0.5)
                .clamp(canvas_rect.min.x + 8.0, (canvas_rect.max.x - editor_w - 8.0).max(canvas_rect.min.x + 8.0));
            let py = (if supports_body {
                srect.min.y - 4.0
            } else {
                srect.center().y - 16.0
            })
            .clamp(canvas_rect.min.y + 8.0, (canvas_rect.max.y - est_h - 8.0).max(canvas_rect.min.y + 8.0));

            let mut commit = false;
            let mut cancel = false;
            let needs_focus = app.editing_needs_focus;
            let focus_body = app.editing_focus_body;
            let mut focused_now = false;

            egui::Area::new(egui::Id::new("inline_node_editor"))
                .order(egui::Order::Foreground)
                .fixed_pos(Pos2::new(px, py))
                .show(ui.ctx(), |ui| {
                    egui::Frame::popup(ui.style())
                        .inner_margin(egui::Margin::same(6))
                        .show(ui, |ui| {
                            ui.set_width(editor_w);
                            if let Some(node) = app.bridge.scene.get_node_mut(edit_id) {
                                let (enter_pressed, cmd_enter, esc_pressed) = ui.input(|i| {
                                    let enter = i.key_pressed(egui::Key::Enter);
                                    let cmd = i.modifiers.command || i.modifiers.ctrl;
                                    (enter, enter && cmd, i.key_pressed(egui::Key::Escape))
                                });
                                if supports_body {
                                    ui.label(egui::RichText::new("Title").small().strong());
                                    let title_resp = ui.add_sized(
                                        [editor_w, 24.0],
                                        egui::TextEdit::singleline(&mut node.title)
                                            .hint_text("Title..."),
                                    );
                                    if needs_focus && !focus_body {
                                        title_resp.request_focus();
                                        focused_now = true;
                                    }
                                    ui.add_space(4.0);
                                    ui.label(egui::RichText::new("Body").small().strong());
                                    let body_resp = ui.add_sized(
                                        [editor_w, 66.0],
                                        egui::TextEdit::multiline(&mut node.body)
                                            .desired_rows(3)
                                            .hint_text("Body text..."),
                                    );
                                    if needs_focus && focus_body {
                                        body_resp.request_focus();
                                        focused_now = true;
                                    }
                                    ui.add_space(4.0);
                                    ui.horizontal(|ui| {
                                        ui.label(
                                            egui::RichText::new("Ctrl+Enter: Done • Esc: Cancel")
                                                .small()
                                                .weak(),
                                        );
                                        ui.with_layout(
                                            egui::Layout::right_to_left(egui::Align::Center),
                                            |ui| {
                                                if ui.small_button("Done").clicked() {
                                                    commit = true;
                                                }
                                            },
                                        );
                                    });
                                    if cmd_enter || (enter_pressed && title_resp.has_focus()) {
                                        commit = true;
                                    }
                                    if esc_pressed {
                                        cancel = true;
                                    }
                                } else {
                                    let resp = ui.add_sized(
                                        [editor_w, 24.0],
                                        egui::TextEdit::singleline(&mut node.title)
                                            .hint_text("Title..."),
                                    );
                                    if needs_focus {
                                        resp.request_focus();
                                        focused_now = true;
                                    }
                                    if enter_pressed {
                                        commit = true;
                                    }
                                    if esc_pressed {
                                        cancel = true;
                                    }
                                }
                            }
                        });
                });
            if focused_now {
                app.editing_needs_focus = false;
            }
            if commit {
                app.finish_inline_edit(true);
            } else if cancel {
                app.finish_inline_edit(false);
            }
        } else {
            app.finish_inline_edit(false);
        }
    }

    if let Some(edit_eid) = app.editing_edge {
        let edge_mid_screen = app.bridge.scene.get_edge(edit_eid).and_then(|e| {
            let pts = tessellate_edge_path(&app.bridge.scene, e);
            if pts.is_empty() {
                None
            } else {
                let mid = pts[pts.len() / 2];
                Some(world_to_screen(canvas_rect, app, mid))
            }
        });
        if let Some(mid_s) = edge_mid_screen {
            let editor_w = 190.0_f32;
            let px = (mid_s.x - editor_w * 0.5)
                .clamp(canvas_rect.min.x + 8.0, (canvas_rect.max.x - editor_w - 8.0).max(canvas_rect.min.x + 8.0));
            let py = (mid_s.y - 18.0)
                .clamp(canvas_rect.min.y + 8.0, (canvas_rect.max.y - 44.0).max(canvas_rect.min.y + 8.0));
            let mut commit = false;
            let mut cancel = false;
            let needs_focus = app.editing_edge_needs_focus;
            let mut focused_now = false;
            egui::Area::new(egui::Id::new("inline_edge_editor"))
                .order(egui::Order::Foreground)
                .fixed_pos(Pos2::new(px, py))
                .show(ui.ctx(), |ui| {
                    egui::Frame::popup(ui.style())
                        .inner_margin(egui::Margin::same(5))
                        .show(ui, |ui| {
                            ui.set_width(editor_w);
                            if let Some(edge) = app.bridge.scene.get_edge_mut(edit_eid) {
                                let (enter_pressed, esc_pressed) = ui.input(|i| {
                                    (
                                        i.key_pressed(egui::Key::Enter),
                                        i.key_pressed(egui::Key::Escape),
                                    )
                                });
                                let resp = ui.add_sized(
                                    [editor_w, 24.0],
                                    egui::TextEdit::singleline(&mut edge.label)
                                        .hint_text("Edge label..."),
                                );
                                if needs_focus {
                                    resp.request_focus();
                                    focused_now = true;
                                }
                                if enter_pressed {
                                    commit = true;
                                }
                                if esc_pressed {
                                    cancel = true;
                                }
                            }
                        });
                });
            if focused_now {
                app.editing_edge_needs_focus = false;
            }
            if commit {
                app.finish_edge_inline_edit(true);
            } else if cancel {
                app.finish_edge_inline_edit(false);
            }
        } else {
            app.finish_edge_inline_edit(false);
        }
    }

    // Floating quick-action mini-toolbar above selection (never shown during active drag/edit)
    if !app.is_interacting()
        && app.editing_node.is_none()
        && app.editing_edge.is_none()
        && (!app.bridge.selection.selected_nodes.is_empty()
            || !app.bridge.selection.selected_edges.is_empty())
    {
        render_selection_floating_bar(app, ui, canvas_rect);
    }

    if let Some(drag) = app.dragging_stencil.clone() {
        let primary_down = ui.input(|i| i.pointer.primary_down());
        if let Some(cur_pos) = ui.input(|i| {
            i.pointer
                .latest_pos()
                .or(i.pointer.hover_pos())
                .or(i.pointer.interact_pos())
        }) {
            let overlay = ui.ctx().layer_painter(egui::LayerId::new(
                egui::Order::Tooltip,
                egui::Id::new("stencil_drag_ghost"),
            ));
            let gw = (drag.width * app.bridge.viewport.zoom).clamp(48.0, 220.0);
            let gh = (drag.height * app.bridge.viewport.zoom).clamp(36.0, 160.0);
            let grect = EguiRect::from_center_size(cur_pos, Vec2::new(gw, gh));
            if let Some(icon) = drag.icon {
                let ir = EguiRect::from_center_size(
                    Pos2::new(grect.center().x, grect.min.y + grect.height() * 0.40),
                    Vec2::splat((grect.width().min(grect.height() * 0.68)).clamp(24.0, 52.0)),
                );
                draw_symbol_icon(&overlay, ir, icon);
                overlay.text(
                    Pos2::new(grect.center().x, ir.max.y + 10.0),
                    Align2::CENTER_CENTER,
                    &drag.label,
                    FontId::proportional(11.0),
                    Color32::from_rgb(18, 22, 36),
                );
            } else {
                overlay.rect_filled(
                    grect,
                    4.0,
                    Color32::from_rgba_unmultiplied(236, 240, 255, 190),
                );
                overlay.rect_stroke(
                    grect,
                    4.0,
                    Stroke::new(1.5_f32, Color32::from_rgb(0, 0, 140)),
                    egui::StrokeKind::Inside,
                );
                overlay.text(
                    grect.center(),
                    Align2::CENTER_CENTER,
                    &drag.label,
                    FontId::proportional(11.5),
                    Color32::from_rgb(0, 0, 128),
                );
            }
            if !primary_down {
                if canvas_rect.contains(cur_pos) {
                    let wp = screen_to_world(canvas_rect, app, cur_pos);
                    app.spawn_stencil_at_world(
                        &drag.label,
                        drag.width,
                        drag.height,
                        drag.shape,
                        drag.icon,
                        drag.layers,
                        wp,
                    );
                }
                app.dragging_stencil = None;
            }
        } else if !primary_down {
            app.dragging_stencil = None;
        }
    }

    render_context_menu(app, &response, canvas_rect);

    if app.has_live_animations() || app.is_interacting() {
        ui.ctx().request_repaint();
    }
}

