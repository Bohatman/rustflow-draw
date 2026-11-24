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


pub(super) fn minimap_rects(canvas_rect: EguiRect) -> (EguiRect, EguiRect, EguiRect) {
    let size = Vec2::new(192.0, 124.0);
    let outer = EguiRect::from_min_size(canvas_rect.max - size - Vec2::new(12.0, 12.0), size);
    let inner = EguiRect::from_min_max(
        Pos2::new(outer.min.x + 4.0, outer.min.y + 18.0),
        Pos2::new(outer.max.x - 4.0, outer.max.y - 4.0),
    );
    let close_rect = EguiRect::from_min_size(
        Pos2::new(outer.max.x - 17.0, outer.min.y + 3.0),
        Vec2::splat(12.0),
    );
    (outer, inner, close_rect)
}

pub(super) fn minimap_restore_pill_rect(canvas_rect: EguiRect) -> EguiRect {
    let size = Vec2::new(52.0, 20.0);
    EguiRect::from_min_size(canvas_rect.max - size - Vec2::new(12.0, 12.0), size)
}

pub(super) fn draw_minimap_restore_pill(
    painter: &Painter,
    canvas_rect: EguiRect,
    pointer_pos: Option<Pos2>,
) {
    let rect = minimap_restore_pill_rect(canvas_rect);
    let hovered = pointer_pos.is_some_and(|p| rect.contains(p));
    let bg = if hovered {
        Color32::from_rgb(232, 238, 255)
    } else {
        Color32::from_rgba_unmultiplied(232, 234, 240, 235)
    };
    let border = if hovered {
        Color32::from_rgb(0, 0, 160)
    } else {
        Color32::from_rgb(120, 126, 144)
    };
    painter.rect_filled(rect, 3.0, bg);
    painter.rect_stroke(rect, 3.0, Stroke::new(1.0_f32, border), egui::StrokeKind::Inside);
    painter.text(
        rect.center(),
        Align2::CENTER_CENTER,
        "Minimap",
        FontId::proportional(9.5),
        Color32::from_rgb(24, 28, 42),
    );
}

pub(super) fn pan_viewport_from_minimap(app: &mut BohatDrawApp, inner: EguiRect, pos: Pos2) {
    let snap = app
        .bridge
        .minimap_snapshot(inner.width().max(1.0), inner.height().max(1.0));
    let local_x = (pos.x - inner.min.x).clamp(0.0, inner.width());
    let local_y = (pos.y - inner.min.y).clamp(0.0, inner.height());
    let world_center = snap.minimap_point_to_world(Point::new(local_x, local_y));
    let vp = &mut app.bridge.viewport;
    vp.pan.x = vp.screen_width * 0.5 - world_center.x * vp.zoom;
    vp.pan.y = vp.screen_height * 0.5 - world_center.y * vp.zoom;
}

pub(super) fn draw_minimap(
    painter: &Painter,
    canvas_rect: EguiRect,
    app: &BohatDrawApp,
    pointer_pos: Option<Pos2>,
) {
    let (mrect, inner, close_rect) = minimap_rects(canvas_rect);
    let header_rect = EguiRect::from_min_max(mrect.min, Pos2::new(mrect.max.x, inner.min.y));

    painter.rect_filled(
        mrect.translate(Vec2::new(2.0, 3.0)),
        3.0,
        Color32::from_rgba_unmultiplied(15, 23, 42, 42),
    );
    painter.rect_filled(mrect, 3.0, Color32::from_rgba_unmultiplied(244, 246, 250, 238));
    painter.rect_filled(header_rect, 2.0, Color32::from_rgb(214, 218, 228));
    painter.line_segment(
        [header_rect.left_bottom(), header_rect.right_bottom()],
        Stroke::new(1.0_f32, Color32::from_rgb(148, 154, 172)),
    );
    painter.text(
        Pos2::new(header_rect.min.x + 6.0, header_rect.center().y),
        Align2::LEFT_CENTER,
        "Minimap",
        FontId::proportional(9.5),
        Color32::from_rgb(24, 28, 42),
    );
    painter.text(
        Pos2::new(close_rect.min.x - 6.0, header_rect.center().y),
        Align2::RIGHT_CENTER,
        format!("{:.0}%", app.bridge.viewport.zoom * 100.0),
        FontId::monospace(9.0),
        Color32::from_rgb(0, 0, 140),
    );
    let close_hovered = pointer_pos.is_some_and(|p| close_rect.expand(2.0).contains(p));
    let close_bg = if close_hovered {
        Color32::from_rgb(220, 38, 38)
    } else {
        Color32::from_rgb(228, 230, 238)
    };
    let close_fg = if close_hovered {
        Color32::WHITE
    } else {
        Color32::from_rgb(76, 82, 98)
    };
    painter.rect_filled(close_rect, 1.5, close_bg);
    painter.rect_stroke(
        close_rect,
        1.5,
        Stroke::new(1.0_f32, Color32::from_rgb(120, 126, 144)),
        egui::StrokeKind::Inside,
    );
    let cc = close_rect.center();
    painter.line_segment(
        [Pos2::new(cc.x - 2.4, cc.y - 2.4), Pos2::new(cc.x + 2.4, cc.y + 2.4)],
        Stroke::new(1.2_f32, close_fg),
    );
    painter.line_segment(
        [Pos2::new(cc.x + 2.4, cc.y - 2.4), Pos2::new(cc.x - 2.4, cc.y + 2.4)],
        Stroke::new(1.2_f32, close_fg),
    );

    let snap = app
        .bridge
        .minimap_snapshot(inner.width().max(1.0), inner.height().max(1.0));
    let world_to_mini = |pt: Point| -> Pos2 {
        Pos2::new(
            inner.min.x + (pt.x - snap.world_envelope.x) * snap.scale,
            inner.min.y + (pt.y - snap.world_envelope.y) * snap.scale,
        )
    };
    let edge_stroke = Stroke::new(0.8_f32, Color32::from_rgba_unmultiplied(110, 120, 145, 130));
    for edge in app.bridge.scene.edges() {
        if let (Some(sn), Some(tn)) = (
            app.bridge.scene.get_node(edge.source_node),
            app.bridge.scene.get_node(edge.target_node),
        ) {
            let p0 = world_to_mini(sn.bounds.center());
            let p1 = world_to_mini(tn.bounds.center());
            if inner.contains(p0) && inner.contains(p1) {
                painter.line_segment([p0, p1], edge_stroke);
            }
        }
    }
    for nb in &snap.node_boxes {
        let r = EguiRect::from_min_size(
            inner.min + Vec2::new(nb.rect.x, nb.rect.y),
            Vec2::new(nb.rect.width.max(3.5), nb.rect.height.max(3.0)),
        )
        .intersect(inner);
        let col = if app.bridge.selection.selected_nodes.contains(&nb.node_id) {
            Color32::from_rgb(14, 165, 233)
        } else if let Some(icon) = app.bridge.scene.get_node(nb.node_id).and_then(|n| n.icon) {
            symbol_icon_accent(icon)
        } else {
            Color32::from_rgb(36, 56, 148)
        };
        painter.rect_filled(r, 1.0, col);
    }
    let vr = EguiRect::from_min_size(
        inner.min + Vec2::new(snap.viewport_frustum_rect.x, snap.viewport_frustum_rect.y),
        Vec2::new(
            snap.viewport_frustum_rect.width.max(10.0),
            snap.viewport_frustum_rect.height.max(8.0),
        ),
    )
    .intersect(inner);
    painter.rect_filled(
        vr,
        1.5,
        Color32::from_rgba_unmultiplied(37, 99, 235, 36),
    );
    painter.rect_stroke(
        vr,
        1.5,
        Stroke::new(1.5_f32, Color32::from_rgb(29, 78, 216)),
        egui::StrokeKind::Inside,
    );
    painter.rect_stroke(
        mrect,
        3.0,
        Stroke::new(
            1.2_f32,
            if app.dragging_minimap {
                Color32::from_rgb(0, 0, 160)
            } else {
                Color32::from_rgb(96, 102, 122)
            },
        ),
        egui::StrokeKind::Inside,
    );
}

