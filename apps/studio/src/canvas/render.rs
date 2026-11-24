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

use super::{node_supports_body, rect_to_screen, world_to_screen};
use super::interaction::resize_handle_positions;

pub(super) fn draw_smart_guides(painter: &Painter, canvas_rect: EguiRect, app: &BohatDrawApp) {
    for &(is_vertical, world_coord, is_center) in &app.active_guides {
        let color = if is_center {
            Color32::from_rgb(217, 70, 239)
        } else {
            Color32::from_rgb(239, 68, 68)
        };
        let guide_stroke = Stroke::new(if is_center { 1.4_f32 } else { 1.0_f32 }, color);
        if is_vertical {
            let sx = world_to_screen(canvas_rect, app, Point::new(world_coord, 0.0)).x;
            painter.line_segment(
                [Pos2::new(sx, canvas_rect.min.y), Pos2::new(sx, canvas_rect.max.y)],
                guide_stroke,
            );
        } else {
            let sy = world_to_screen(canvas_rect, app, Point::new(0.0, world_coord)).y;
            painter.line_segment(
                [Pos2::new(canvas_rect.min.x, sy), Pos2::new(canvas_rect.max.x, sy)],
                guide_stroke,
            );
        }
    }
}

fn push_mesh_segment(mesh: &mut egui::Mesh, a: Pos2, b: Pos2, width: f32, color: Color32) {
    let seg = b - a;
    let len = seg.length();
    if len < 0.001 {
        return;
    }
    let half = width * 0.5;
    let perp = Vec2::new(-seg.y / len * half, seg.x / len * half);
    let idx = mesh.vertices.len() as u32;
    let uv = egui::epaint::WHITE_UV;
    mesh.vertices.push(egui::epaint::Vertex { pos: a + perp, uv, color });
    mesh.vertices.push(egui::epaint::Vertex { pos: b + perp, uv, color });
    mesh.vertices.push(egui::epaint::Vertex { pos: b - perp, uv, color });
    mesh.vertices.push(egui::epaint::Vertex { pos: a - perp, uv, color });
    mesh.indices
        .extend_from_slice(&[idx, idx + 1, idx + 2, idx, idx + 2, idx + 3]);
}

fn ensure_bg_tile_texture(
    app: &mut BohatDrawApp,
    ctx: &egui::Context,
) -> Option<egui::TextureId> {
    let style = app.bg_style;
    let mode = app.theme_mode;
    if matches!(style, CanvasBgStyle::None | CanvasBgStyle::Isometric) {
        return None;
    }
    let needs_rebuild = app
        .bg_tile_tex
        .as_ref()
        .is_none_or(|(s, m, _)| *s != style || *m != mode);
    if needs_rebuild {
        let (dot_col, minor_col, major_col) = match mode {
            rustflow_core::ThemeMode::Dark => (
                Color32::from_rgba_unmultiplied(150, 165, 195, 68),
                Color32::from_rgba_unmultiplied(120, 135, 165, 36),
                Color32::from_rgba_unmultiplied(150, 165, 200, 68),
            ),
            _ => (
                Color32::from_rgba_unmultiplied(92, 104, 134, 88),
                Color32::from_rgba_unmultiplied(130, 142, 168, 52),
                Color32::from_rgba_unmultiplied(100, 114, 146, 86),
            ),
        };
        let sz: usize = 32;
        let mut pixels = vec![Color32::TRANSPARENT; sz * sz];
        match style {
            CanvasBgStyle::Dots => {
                for dy in 0..3 {
                    for dx in 0..3 {
                        pixels[dy * sz + dx] = dot_col;
                    }
                }
            }
            CanvasBgStyle::Grid => {
                for i in 0..sz {
                    pixels[i] = minor_col;
                    pixels[i * sz] = minor_col;
                }
                pixels[0] = major_col;
            }
            CanvasBgStyle::Crosshair => {
                let c = 16usize;
                for d in (c - 4)..=(c + 4) {
                    pixels[c * sz + d] = major_col;
                    pixels[d * sz + c] = major_col;
                }
            }
            CanvasBgStyle::None | CanvasBgStyle::Isometric => {}
        }
        let img = egui::ColorImage {
            size: [sz, sz],
            pixels,
        };
        let opts = egui::TextureOptions {
            magnification: egui::TextureFilter::Linear,
            minification: egui::TextureFilter::Linear,
            wrap_mode: egui::TextureWrapMode::Repeat,
            mipmap_mode: None,
        };
        let handle = ctx.load_texture("canvas_bg_tile", img, opts);
        app.bg_tile_tex = Some((style, mode, handle));
    }
    app.bg_tile_tex.as_ref().map(|(_, _, h)| h.id())
}

pub(super) fn draw_canvas_background(
    painter: &Painter,
    canvas_rect: EguiRect,
    app: &mut BohatDrawApp,
    ctx: &egui::Context,
) {
    let vp = &app.bridge.viewport;
    let mut step = (vp.grid_size * vp.zoom).max(2.0);
    while step < 18.0 {
        step *= 2.0;
    }
    step = step.min(80.0);
    let pan = vp.pan;

    if let Some(tex_id) = ensure_bg_tile_texture(app, ctx) {
        let u0 = -pan.x / step;
        let v0 = -pan.y / step;
        let u1 = (canvas_rect.width() - pan.x) / step;
        let v1 = (canvas_rect.height() - pan.y) / step;
        let uv = EguiRect::from_min_max(Pos2::new(u0, v0), Pos2::new(u1, v1));
        painter.image(tex_id, canvas_rect, uv, Color32::WHITE);
        return;
    }

    if matches!(app.bg_style, CanvasBgStyle::Isometric) {
        let minor_col = match app.theme_mode {
            rustflow_core::ThemeMode::Dark => Color32::from_rgba_unmultiplied(120, 135, 165, 32),
            _ => Color32::from_rgba_unmultiplied(130, 142, 168, 44),
        };
        let line_stroke = Stroke::new(1.0_f32, minor_col);
        let iso_step = step * 1.6;
        let h = canvas_rect.height();
        let w = canvas_rect.width();
        let diag = w + h * 2.0;
        let mut d = -h * 1.8 + pan.x.rem_euclid(iso_step);
        while d < diag {
            let p0 = Pos2::new(canvas_rect.min.x + d, canvas_rect.min.y);
            let p1 = Pos2::new(canvas_rect.min.x + d + h * 1.732, canvas_rect.max.y);
            painter.line_segment([p0, p1], line_stroke);
            let q0 = Pos2::new(canvas_rect.min.x + d, canvas_rect.min.y);
            let q1 = Pos2::new(canvas_rect.min.x + d - h * 1.732, canvas_rect.max.y);
            painter.line_segment([q0, q1], line_stroke);
            d += iso_step;
        }
    }
}

pub(super) fn draw_stroked_polyline(
    painter: &Painter,
    pts: &[Pos2],
    stroke: Stroke,
    style: BorderStyle,
    dash_offset: f32,
) {
    if pts.len() < 2 || matches!(style, BorderStyle::None) {
        return;
    }
    match style {
        BorderStyle::Solid => {
            painter.add(egui::Shape::line(pts.to_vec(), stroke));
        }
        BorderStyle::Double => {
            painter.add(egui::Shape::line(
                pts.to_vec(),
                Stroke::new(stroke.width + 2.0, stroke.color),
            ));
            painter.add(egui::Shape::line(
                pts.to_vec(),
                Stroke::new((stroke.width - 0.5).max(0.8), Color32::WHITE),
            ));
        }
        BorderStyle::Dashed | BorderStyle::Dotted | BorderStyle::DashDot => {
            let (dash, gap) = match style {
                BorderStyle::Dashed => (8.0, 5.0),
                BorderStyle::Dotted => (2.5, 4.0),
                BorderStyle::DashDot => (9.0, 4.0),
                _ => (8.0, 5.0),
            };
            let period = dash + gap;
            let mut dist_acc = dash_offset.rem_euclid(period);
            let mut mesh = egui::Mesh::default();
            for w in pts.windows(2) {
                let a = w[0];
                let b = w[1];
                let seg = b - a;
                let len = seg.length();
                if len < 0.001 {
                    continue;
                }
                let dir = seg / len;
                let mut pos = 0.0;
                while pos < len {
                    let phase = (dist_acc + pos).rem_euclid(period);
                    if phase < dash {
                        let step = (dash - phase).min(len - pos);
                        push_mesh_segment(
                            &mut mesh,
                            a + dir * pos,
                            a + dir * (pos + step),
                            stroke.width,
                            stroke.color,
                        );
                        pos += step;
                    } else {
                        pos += (period - phase).min(len - pos);
                    }
                }
                dist_acc += len;
            }
            if !mesh.vertices.is_empty() {
                painter.add(egui::Shape::mesh(mesh));
            }
        }
        BorderStyle::None => {}
    }
}

pub(super) fn draw_rect_border(
    painter: &Painter,
    r: EguiRect,
    corner: f32,
    stroke: Stroke,
    style: BorderStyle,
    dash_offset: f32,
) {
    match style {
        BorderStyle::None => {}
        BorderStyle::Solid => {
            painter.rect_stroke(r, CornerRadius::same(corner as u8), stroke, egui::StrokeKind::Inside);
        }
        BorderStyle::Double => {
            painter.rect_stroke(r, CornerRadius::same(corner as u8), stroke, egui::StrokeKind::Inside);
            let inner = r.shrink(3.0);
            if inner.width() > 4.0 && inner.height() > 4.0 {
                painter.rect_stroke(
                    inner,
                    CornerRadius::same((corner - 2.0).max(0.0) as u8),
                    Stroke::new((stroke.width * 0.8).max(1.0), stroke.color),
                    egui::StrokeKind::Inside,
                );
            }
        }
        _ => {
            let pts = [
                r.left_top(),
                r.right_top(),
                r.right_bottom(),
                r.left_bottom(),
                r.left_top(),
            ];
            draw_stroked_polyline(painter, &pts, stroke, style, dash_offset);
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn draw_node(
    painter: &Painter,
    canvas_rect: EguiRect,
    app: &BohatDrawApp,
    node: &Node,
    is_selected: bool,
    _is_hovered: bool,
    hovered_port: Option<(NodeId, PortId)>,
    pointer_pos: Option<Pos2>,
    busy_dragging: bool,
    skip_shadows: bool,
    now_sec: f32,
) {
    let mut draw_bounds = node.bounds;
    if matches!(node.shape, ShapeKind::Collapsible) && node.collapsed {
        draw_bounds.height = 34.0;
    }
    let srect = rect_to_screen(canvas_rect, app, draw_bounds);
    if !srect.intersects(canvas_rect.expand(40.0)) {
        return;
    }

    let opacity = node.style.opacity.clamp(0.08, 1.0);
    let mut fill = to_color32(node.style.fill);
    fill = Color32::from_rgba_unmultiplied(
        fill.r(),
        fill.g(),
        fill.b(),
        (fill.a() as f32 * opacity).round() as u8,
    );
    let stroke_col = to_color32(node.style.stroke);
    let stroke = Stroke::new((node.style.stroke_width * app.bridge.viewport.zoom).max(1.0), stroke_col);
    let dash_offset = if node.style.border_anim { -now_sec * 36.0 } else { 0.0 };
    let corner = (node.style.corner_radius * app.bridge.viewport.zoom).clamp(0.0, 24.0);

    let is_icon_only = node.icon.is_some() && node.custom_layers.is_empty();
    let shadow_col = Color32::from_rgba_unmultiplied(15, 23, 42, (28.0 * opacity) as u8);
    let is_box_shadow_shape = !is_icon_only
        && matches!(
            node.shape,
            ShapeKind::ReactFlowCard
                | ShapeKind::Rectangle
                | ShapeKind::RoundedRect
                | ShapeKind::BrowserWindow
                | ShapeKind::MobileScreen
                | ShapeKind::Server
                | ShapeKind::Firewall
                | ShapeKind::Queue
                | ShapeKind::FolderTree
                | ShapeKind::Collapsible
                | ShapeKind::ImageNode
        );
    if !skip_shadows && is_box_shadow_shape {
        let shadow_rect = srect.translate(Vec2::new(3.0, 4.0));
        let scr = if matches!(node.shape, ShapeKind::MobileScreen) {
            12
        } else if matches!(node.shape, ShapeKind::Queue) {
            (srect.height() * 0.35).min(24.0) as u8
        } else {
            corner as u8
        };
        painter.rect_filled(shadow_rect, CornerRadius::same(scr), shadow_col);
    }

    if !node.custom_layers.is_empty() {
        for layer in &node.custom_layers {
            let lr = EguiRect::from_min_size(
                Pos2::new(srect.min.x + layer.x * srect.width(), srect.min.y + layer.y * srect.height()),
                Vec2::new((layer.w * srect.width()).max(4.0), (layer.h * srect.height()).max(4.0)),
            );
            let lf = to_color32(layer.fill);
            let ls = Stroke::new((1.6 * app.bridge.viewport.zoom).max(1.0), to_color32(layer.stroke));
            draw_simple_layer_shape(painter, lr, &layer.shape, lf, ls);
        }
    } else if !is_icon_only {
    match &node.shape {
        ShapeKind::Diamond | ShapeKind::GatewayXor | ShapeKind::GatewayParallel => {
            let c = srect.center();
            let pts = vec![
                Pos2::new(c.x, srect.min.y),
                Pos2::new(srect.max.x, c.y),
                Pos2::new(c.x, srect.max.y),
                Pos2::new(srect.min.x, c.y),
            ];
            painter.add(egui::Shape::convex_polygon(pts.clone(), fill, Stroke::NONE));
            let mut closed = pts;
            closed.push(closed[0]);
            draw_stroked_polyline(painter, &closed, stroke, node.style.border_style, dash_offset);
            if matches!(node.shape, ShapeKind::GatewayXor) {
                let d = srect.width().min(srect.height()) * 0.16;
                painter.line_segment([Pos2::new(c.x - d, c.y - d), Pos2::new(c.x + d, c.y + d)], stroke);
                painter.line_segment([Pos2::new(c.x + d, c.y - d), Pos2::new(c.x - d, c.y + d)], stroke);
            } else if matches!(node.shape, ShapeKind::GatewayParallel) {
                let d = srect.width().min(srect.height()) * 0.18;
                painter.line_segment([Pos2::new(c.x - d, c.y), Pos2::new(c.x + d, c.y)], stroke);
                painter.line_segment([Pos2::new(c.x, c.y - d), Pos2::new(c.x, c.y + d)], stroke);
            }
        }
        ShapeKind::Ellipse | ShapeKind::TimerEvent => {
            let c = srect.center();
            let rx = srect.width() * 0.5;
            let ry = srect.height() * 0.5;
            let segs = 32;
            let mut pts = Vec::with_capacity(segs + 1);
            for i in 0..=segs {
                let a = (i as f32 / segs as f32) * std::f32::consts::TAU;
                pts.push(Pos2::new(c.x + a.cos() * rx, c.y + a.sin() * ry));
            }
            painter.add(egui::Shape::convex_polygon(pts[..segs].to_vec(), fill, Stroke::NONE));
            draw_stroked_polyline(painter, &pts, stroke, node.style.border_style, dash_offset);
            if matches!(node.shape, ShapeKind::TimerEvent) {
                painter.line_segment([c, Pos2::new(c.x, c.y - ry * 0.55)], stroke);
                painter.line_segment([c, Pos2::new(c.x + rx * 0.4, c.y)], stroke);
            }
        }
        ShapeKind::Cylinder => {
            let cap_h = (srect.height() * 0.18).clamp(8.0, 22.0);
            let body = EguiRect::from_min_max(
                Pos2::new(srect.min.x, srect.min.y + cap_h * 0.5),
                Pos2::new(srect.max.x, srect.max.y - cap_h * 0.5),
            );
            painter.rect_filled(body, 2.0, fill);
            let segs = 24;
            let top_c = Pos2::new(srect.center().x, srect.min.y + cap_h * 0.5);
            let bot_c = Pos2::new(srect.center().x, srect.max.y - cap_h * 0.5);
            let rx = srect.width() * 0.5;
            let ry = cap_h * 0.5;
            let mut top_pts = Vec::with_capacity(segs + 1);
            let mut bot_arc = Vec::with_capacity(segs / 2 + 1);
            for i in 0..=segs {
                let a = (i as f32 / segs as f32) * std::f32::consts::TAU;
                top_pts.push(Pos2::new(top_c.x + a.cos() * rx, top_c.y + a.sin() * ry));
            }
            for i in 0..=(segs / 2) {
                let a = (i as f32 / (segs / 2) as f32) * std::f32::consts::PI;
                bot_arc.push(Pos2::new(bot_c.x + a.cos() * rx, bot_c.y + a.sin() * ry));
            }
            painter.add(egui::Shape::convex_polygon(top_pts[..segs].to_vec(), fill, Stroke::NONE));
            painter.add(egui::Shape::convex_polygon(bot_arc.clone(), fill, Stroke::NONE));
            draw_stroked_polyline(painter, &top_pts, stroke, node.style.border_style, dash_offset);
            draw_stroked_polyline(painter, &bot_arc, stroke, node.style.border_style, dash_offset);
            painter.line_segment([body.left_top(), body.left_bottom()], stroke);
            painter.line_segment([body.right_top(), body.right_bottom()], stroke);
        }
        ShapeKind::Cloud => {
            let c = srect.center();
            let r1 = srect.height() * 0.34;
            let r2 = srect.height() * 0.28;
            painter.circle_filled(Pos2::new(c.x - srect.width() * 0.22, c.y + 4.0), r2, fill);
            painter.circle_filled(Pos2::new(c.x + srect.width() * 0.22, c.y + 4.0), r2, fill);
            painter.circle_filled(Pos2::new(c.x, c.y - 4.0), r1, fill);
            painter.circle_stroke(Pos2::new(c.x - srect.width() * 0.22, c.y + 4.0), r2, stroke);
            painter.circle_stroke(Pos2::new(c.x + srect.width() * 0.22, c.y + 4.0), r2, stroke);
            painter.circle_stroke(Pos2::new(c.x, c.y - 4.0), r1, stroke);
            painter.rect_filled(
                EguiRect::from_center_size(Pos2::new(c.x, c.y + 4.0), Vec2::new(srect.width() * 0.56, r1 * 1.1)),
                4.0,
                fill,
            );
        }
        ShapeKind::Actor => {
            let cx = srect.center().x;
            let head_r = (srect.height() * 0.14).clamp(6.0, 18.0);
            let head_c = Pos2::new(cx, srect.min.y + head_r + 6.0);
            painter.circle_filled(head_c, head_r, fill);
            painter.circle_stroke(head_c, head_r, stroke);
            let neck = head_c.y + head_r;
            let hip = srect.max.y - srect.height() * 0.34;
            painter.line_segment([Pos2::new(cx, neck), Pos2::new(cx, hip)], stroke);
            let arm_y = neck + (hip - neck) * 0.32;
            let arm_w = srect.width() * 0.32;
            painter.line_segment([Pos2::new(cx - arm_w, arm_y), Pos2::new(cx + arm_w, arm_y)], stroke);
            painter.line_segment([Pos2::new(cx, hip), Pos2::new(cx - arm_w * 0.85, srect.max.y - 18.0)], stroke);
            painter.line_segment([Pos2::new(cx, hip), Pos2::new(cx + arm_w * 0.85, srect.max.y - 18.0)], stroke);
        }
        ShapeKind::Document => {
            let wave = (srect.height() * 0.14).clamp(6.0, 18.0);
            let pts = vec![
                srect.left_top(),
                srect.right_top(),
                Pos2::new(srect.max.x, srect.max.y - wave),
                Pos2::new(srect.min.x + srect.width() * 0.72, srect.max.y - wave * 1.4),
                Pos2::new(srect.min.x + srect.width() * 0.3, srect.max.y),
                Pos2::new(srect.min.x, srect.max.y - wave * 0.5),
            ];
            painter.add(egui::Shape::convex_polygon(pts.clone(), fill, Stroke::NONE));
            let mut closed = pts;
            closed.push(closed[0]);
            draw_stroked_polyline(painter, &closed, stroke, node.style.border_style, dash_offset);
        }
        ShapeKind::Callout => {
            let tail_h = (srect.height() * 0.22).clamp(10.0, 22.0);
            let body = EguiRect::from_min_max(srect.min, Pos2::new(srect.max.x, srect.max.y - tail_h));
            painter.rect_filled(body, CornerRadius::same(corner as u8), fill);
            draw_rect_border(painter, body, corner, stroke, node.style.border_style, dash_offset);
            let tail = vec![
                Pos2::new(srect.min.x + srect.width() * 0.24, body.max.y - 1.0),
                Pos2::new(srect.min.x + srect.width() * 0.18, srect.max.y),
                Pos2::new(srect.min.x + srect.width() * 0.38, body.max.y - 1.0),
            ];
            painter.add(egui::Shape::convex_polygon(tail.clone(), fill, Stroke::NONE));
            painter.line_segment([tail[0], tail[1]], stroke);
            painter.line_segment([tail[1], tail[2]], stroke);
        }
        ShapeKind::Package => {
            let tab_h = (18.0 * app.bridge.viewport.zoom).clamp(12.0, 24.0);
            let tab_w = (srect.width() * 0.42).clamp(36.0, 120.0);
            let tab = EguiRect::from_min_size(srect.min, Vec2::new(tab_w, tab_h));
            let body = EguiRect::from_min_max(Pos2::new(srect.min.x, srect.min.y + tab_h - 1.0), srect.max);
            painter.rect_filled(tab, CornerRadius::same(2), Color32::from_rgb(214, 222, 240));
            draw_rect_border(painter, tab, 2.0, stroke, node.style.border_style, dash_offset);
            painter.rect_filled(body, CornerRadius::same(2), fill);
            draw_rect_border(painter, body, 2.0, stroke, node.style.border_style, dash_offset);
        }
        ShapeKind::Component => {
            let nub_w = (14.0 * app.bridge.viewport.zoom).clamp(8.0, 20.0);
            let nub_h = (10.0 * app.bridge.viewport.zoom).clamp(6.0, 14.0);
            let body = EguiRect::from_min_max(Pos2::new(srect.min.x + nub_w * 0.5, srect.min.y), srect.max);
            painter.rect_filled(body, CornerRadius::same(2), fill);
            draw_rect_border(painter, body, 2.0, stroke, node.style.border_style, dash_offset);
            for frac in [0.28, 0.68] {
                let ny = srect.min.y + srect.height() * frac - nub_h * 0.5;
                let nub = EguiRect::from_min_size(Pos2::new(srect.min.x, ny), Vec2::new(nub_w, nub_h));
                painter.rect_filled(nub, 1.0, fill);
                draw_rect_border(painter, nub, 1.0, stroke, node.style.border_style, dash_offset);
            }
        }
        ShapeKind::Interface => {
            let r = (srect.height() * 0.28).clamp(10.0, 26.0);
            let cy = srect.center().y;
            let cx = srect.min.x + r + 8.0;
            painter.circle_filled(Pos2::new(cx, cy), r, fill);
            painter.circle_stroke(Pos2::new(cx, cy), r, stroke);
            painter.line_segment([Pos2::new(cx + r, cy), Pos2::new(srect.max.x - 4.0, cy)], stroke);
        }
        ShapeKind::Queue => {
            painter.rect_filled(srect, CornerRadius::same((srect.height() * 0.35).min(24.0) as u8), fill);
            draw_rect_border(
                painter,
                srect,
                (srect.height() * 0.35).min(24.0),
                stroke,
                node.style.border_style,
                dash_offset,
            );
            for frac in [0.28, 0.52, 0.76] {
                let x = srect.min.x + srect.width() * frac;
                painter.line_segment([Pos2::new(x, srect.min.y + 3.0), Pos2::new(x, srect.max.y - 3.0)], stroke);
            }
        }
        ShapeKind::Server => {
            painter.rect_filled(srect, CornerRadius::same(4), fill);
            draw_rect_border(painter, srect, 4.0, stroke, node.style.border_style, dash_offset);
            for frac in [0.33, 0.66] {
                let y = srect.min.y + srect.height() * frac;
                painter.line_segment([Pos2::new(srect.min.x, y), Pos2::new(srect.max.x, y)], stroke);
            }
            for frac in [0.16, 0.5, 0.83] {
                let y = srect.min.y + srect.height() * frac;
                painter.circle_filled(Pos2::new(srect.max.x - 12.0, y), 3.0, Color32::from_rgb(16, 185, 129));
            }
        }
        ShapeKind::Firewall => {
            painter.rect_filled(srect, CornerRadius::same(2), fill);
            draw_rect_border(painter, srect, 2.0, stroke, node.style.border_style, dash_offset);
            let rows = 3;
            for r in 1..rows {
                let y = srect.min.y + srect.height() * (r as f32 / rows as f32);
                painter.line_segment([Pos2::new(srect.min.x, y), Pos2::new(srect.max.x, y)], stroke);
            }
            for r in 0..rows {
                let y0 = srect.min.y + srect.height() * (r as f32 / rows as f32);
                let y1 = srect.min.y + srect.height() * ((r + 1) as f32 / rows as f32);
                let offset = if r % 2 == 0 { 0.5 } else { 0.25 };
                let x = srect.min.x + srect.width() * offset;
                painter.line_segment([Pos2::new(x, y0), Pos2::new(x, y1)], stroke);
                if r % 2 == 1 {
                    let x2 = srect.min.x + srect.width() * 0.75;
                    painter.line_segment([Pos2::new(x2, y0), Pos2::new(x2, y1)], stroke);
                }
            }
        }
        ShapeKind::Shield => {
            let cx = srect.center().x;
            let pts = vec![
                Pos2::new(cx, srect.min.y),
                Pos2::new(srect.max.x, srect.min.y + srect.height() * 0.18),
                Pos2::new(srect.max.x - srect.width() * 0.12, srect.max.y - srect.height() * 0.24),
                Pos2::new(cx, srect.max.y),
                Pos2::new(srect.min.x + srect.width() * 0.12, srect.max.y - srect.height() * 0.24),
                Pos2::new(srect.min.x, srect.min.y + srect.height() * 0.18),
            ];
            painter.add(egui::Shape::convex_polygon(pts.clone(), fill, Stroke::NONE));
            let mut closed = pts;
            closed.push(closed[0]);
            draw_stroked_polyline(painter, &closed, stroke, node.style.border_style, dash_offset);
        }
        ShapeKind::Subprocess => {
            painter.rect_filled(srect, CornerRadius::same(2), fill);
            draw_rect_border(painter, srect, 2.0, stroke, node.style.border_style, dash_offset);
            let bar = (srect.width() * 0.12).clamp(8.0, 20.0);
            painter.line_segment([Pos2::new(srect.min.x + bar, srect.min.y), Pos2::new(srect.min.x + bar, srect.max.y)], stroke);
            painter.line_segment([Pos2::new(srect.max.x - bar, srect.min.y), Pos2::new(srect.max.x - bar, srect.max.y)], stroke);
        }
        ShapeKind::ManualInput => {
            let slope = (srect.height() * 0.24).clamp(8.0, 22.0);
            let pts = vec![
                Pos2::new(srect.min.x, srect.min.y + slope),
                srect.right_top(),
                srect.right_bottom(),
                srect.left_bottom(),
            ];
            painter.add(egui::Shape::convex_polygon(pts.clone(), fill, Stroke::NONE));
            let mut closed = pts;
            closed.push(closed[0]);
            draw_stroked_polyline(painter, &closed, stroke, node.style.border_style, dash_offset);
        }
        ShapeKind::Delay => {
            let r = srect.height() * 0.5;
            let cx = (srect.max.x - r).max(srect.min.x + 8.0);
            let cy = srect.center().y;
            let mut pts = vec![srect.left_top(), Pos2::new(cx, srect.min.y)];
            let segs = 16;
            for i in 1..=segs {
                let a = -std::f32::consts::FRAC_PI_2 + (i as f32 / segs as f32) * std::f32::consts::PI;
                pts.push(Pos2::new(cx + a.cos() * r, cy + a.sin() * r));
            }
            pts.push(srect.left_bottom());
            painter.add(egui::Shape::convex_polygon(pts.clone(), fill, Stroke::NONE));
            let mut closed = pts;
            closed.push(closed[0]);
            draw_stroked_polyline(painter, &closed, stroke, node.style.border_style, dash_offset);
        }
        ShapeKind::OffPage => {
            let tip = (srect.height() * 0.28).clamp(10.0, 24.0);
            let pts = vec![
                srect.left_top(),
                srect.right_top(),
                Pos2::new(srect.max.x, srect.max.y - tip),
                Pos2::new(srect.center().x, srect.max.y),
                Pos2::new(srect.min.x, srect.max.y - tip),
            ];
            painter.add(egui::Shape::convex_polygon(pts.clone(), fill, Stroke::NONE));
            let mut closed = pts;
            closed.push(closed[0]);
            draw_stroked_polyline(painter, &closed, stroke, node.style.border_style, dash_offset);
        }
        ShapeKind::Triangle => {
            let pts = vec![
                Pos2::new(srect.center().x, srect.min.y),
                srect.right_bottom(),
                srect.left_bottom(),
            ];
            painter.add(egui::Shape::convex_polygon(pts.clone(), fill, Stroke::NONE));
            let mut closed = pts;
            closed.push(closed[0]);
            draw_stroked_polyline(painter, &closed, stroke, node.style.border_style, dash_offset);
        }
        ShapeKind::Note => {
            let fold = (srect.width().min(srect.height()) * 0.2).clamp(10.0, 22.0);
            let pts = vec![
                srect.left_top(),
                Pos2::new(srect.max.x - fold, srect.min.y),
                Pos2::new(srect.max.x, srect.min.y + fold),
                srect.right_bottom(),
                srect.left_bottom(),
            ];
            painter.add(egui::Shape::convex_polygon(pts.clone(), fill, Stroke::NONE));
            let mut closed = pts;
            closed.push(closed[0]);
            draw_stroked_polyline(painter, &closed, stroke, node.style.border_style, dash_offset);
            painter.line_segment(
                [Pos2::new(srect.max.x - fold, srect.min.y), Pos2::new(srect.max.x - fold, srect.min.y + fold)],
                stroke,
            );
            painter.line_segment(
                [Pos2::new(srect.max.x - fold, srect.min.y + fold), Pos2::new(srect.max.x, srect.min.y + fold)],
                stroke,
            );
        }
        ShapeKind::Hexagon => {
            let inset = srect.width() * 0.18;
            let cy = srect.center().y;
            let pts = vec![
                Pos2::new(srect.min.x + inset, srect.min.y),
                Pos2::new(srect.max.x - inset, srect.min.y),
                Pos2::new(srect.max.x, cy),
                Pos2::new(srect.max.x - inset, srect.max.y),
                Pos2::new(srect.min.x + inset, srect.max.y),
                Pos2::new(srect.min.x, cy),
            ];
            painter.add(egui::Shape::convex_polygon(pts.clone(), fill, Stroke::NONE));
            let mut closed = pts;
            closed.push(closed[0]);
            draw_stroked_polyline(painter, &closed, stroke, node.style.border_style, dash_offset);
        }
        ShapeKind::Parallelogram => {
            let skew = srect.width() * 0.16;
            let pts = vec![
                Pos2::new(srect.min.x + skew, srect.min.y),
                Pos2::new(srect.max.x, srect.min.y),
                Pos2::new(srect.max.x - skew, srect.max.y),
                Pos2::new(srect.min.x, srect.max.y),
            ];
            painter.add(egui::Shape::convex_polygon(pts.clone(), fill, Stroke::NONE));
            let mut closed = pts;
            closed.push(closed[0]);
            draw_stroked_polyline(painter, &closed, stroke, node.style.border_style, dash_offset);
        }
        ShapeKind::BrowserWindow => {
            painter.rect_filled(srect, CornerRadius::same(4), fill);
            let hh = (22.0 * app.bridge.viewport.zoom).clamp(14.0, 28.0);
            let hrect = EguiRect::from_min_size(srect.min, Vec2::new(srect.width(), hh));
            painter.rect_filled(hrect, CornerRadius::same(4), Color32::from_rgb(212, 218, 232));
            draw_rect_border(painter, srect, 4.0, stroke, node.style.border_style, dash_offset);
            painter.line_segment([hrect.left_bottom(), hrect.right_bottom()], stroke);
            for (i, c) in [
                Color32::from_rgb(239, 68, 68),
                Color32::from_rgb(245, 158, 11),
                Color32::from_rgb(34, 197, 94),
            ]
            .iter()
            .enumerate()
            {
                painter.circle_filled(
                    Pos2::new(srect.min.x + 10.0 + i as f32 * 10.0, srect.min.y + hh * 0.5),
                    3.2,
                    *c,
                );
            }
        }
        ShapeKind::MobileScreen => {
            painter.rect_filled(srect, CornerRadius::same(12), fill);
            draw_rect_border(painter, srect, 12.0, stroke, node.style.border_style, dash_offset);
            let notch = EguiRect::from_center_size(
                Pos2::new(srect.center().x, srect.min.y + 8.0),
                Vec2::new(srect.width() * 0.34, 4.0),
            );
            painter.rect_filled(notch, 2.0, stroke_col);
            painter.circle_stroke(Pos2::new(srect.center().x, srect.max.y - 10.0), 4.0, stroke);
        }
        ShapeKind::Section => {
            painter.rect_filled(srect, CornerRadius::same(corner as u8), fill);
            let header_h = (24.0 * app.bridge.viewport.zoom).clamp(16.0, 32.0);
            let header_rect = EguiRect::from_min_size(srect.min, Vec2::new(srect.width(), header_h));
            painter.rect_filled(
                header_rect,
                CornerRadius::same(corner as u8),
                Color32::from_rgba_unmultiplied(185, 198, 224, (150.0 * opacity) as u8),
            );
            draw_rect_border(painter, srect, corner, stroke, node.style.border_style, dash_offset);
            painter.text(
                Pos2::new(srect.min.x + 10.0, srect.min.y + header_h * 0.5),
                Align2::LEFT_CENTER,
                &node.title,
                FontId::proportional((12.5 * app.bridge.viewport.zoom).clamp(9.0, 20.0)),
                to_color32(node.text_style.color),
            );
        }
        ShapeKind::FolderTree => {
            painter.rect_filled(srect, CornerRadius::same(corner as u8), fill);
            let header_h = (24.0 * app.bridge.viewport.zoom).clamp(16.0, 30.0);
            let header_rect = EguiRect::from_min_size(srect.min, Vec2::new(srect.width(), header_h));
            painter.rect_filled(header_rect, CornerRadius::same(4), Color32::from_rgb(218, 224, 236));
            draw_rect_border(painter, srect, corner, stroke, node.style.border_style, dash_offset);
            painter.text(
                Pos2::new(srect.min.x + 8.0, srect.min.y + header_h * 0.5),
                Align2::LEFT_CENTER,
                &node.title,
                FontId::proportional((12.0 * app.bridge.viewport.zoom).clamp(9.0, 18.0)),
                to_color32(node.text_style.color),
            );
            let row_h = (20.0 * app.bridge.viewport.zoom).clamp(14.0, 26.0);
            let mut y = srect.min.y + header_h + 4.0;
            let mut hidden_depth: Option<u8> = None;
            for item in &node.tree_items {
                if let Some(hd) = hidden_depth {
                    if item.depth > hd {
                        continue;
                    } else {
                        hidden_depth = None;
                    }
                }
                if y + row_h > srect.max.y - 2.0 {
                    break;
                }
                let indent = item.depth as f32 * 14.0 * app.bridge.viewport.zoom;
                let prefix = if item.is_folder {
                    if item.expanded { "v [D] " } else { "> [D] " }
                } else {
                    "  [F] "
                };
                painter.text(
                    Pos2::new(srect.min.x + 8.0 + indent, y + row_h * 0.5),
                    Align2::LEFT_CENTER,
                    format!("{prefix}{}", item.label),
                    FontId::proportional((11.5 * app.bridge.viewport.zoom).clamp(8.5, 16.0)),
                    to_color32(node.text_style.color),
                );
                if item.is_folder && !item.expanded {
                    hidden_depth = Some(item.depth);
                }
                y += row_h;
            }
        }
        ShapeKind::Collapsible => {
            painter.rect_filled(srect, CornerRadius::same(corner as u8), fill);
            let header_h = (30.0 * app.bridge.viewport.zoom).clamp(20.0, 36.0);
            let header_rect = EguiRect::from_min_size(srect.min, Vec2::new(srect.width(), header_h));
            painter.rect_filled(header_rect, CornerRadius::same(4), Color32::from_rgb(220, 226, 238));
            draw_rect_border(painter, srect, corner, stroke, node.style.border_style, dash_offset);
            let chev = if node.collapsed { ">" } else { "v" };
            painter.text(
                Pos2::new(srect.min.x + 10.0, srect.min.y + header_h * 0.5),
                Align2::LEFT_CENTER,
                format!("{chev} {}", node.title),
                FontId::proportional((12.5 * app.bridge.viewport.zoom).clamp(9.0, 18.0)),
                to_color32(node.text_style.color),
            );
            if !node.collapsed && !node.body.is_empty() {
                painter.text(
                    Pos2::new(srect.min.x + 10.0, srect.min.y + header_h + 10.0),
                    Align2::LEFT_TOP,
                    &node.body,
                    FontId::proportional((11.0 * app.bridge.viewport.zoom).clamp(8.0, 16.0)),
                    to_color32(node.text_style.color),
                );
            }
        }
        ShapeKind::ImageNode => {
            painter.rect_filled(srect, CornerRadius::same(corner as u8), fill);
            draw_rect_border(painter, srect, corner, stroke, node.style.border_style, dash_offset);
            let inner = srect.shrink(5.0);
            if let Some(tex) = app.image_textures.get(&node.id) {
                let uv = EguiRect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0));
                painter.image(tex.id(), inner, uv, Color32::WHITE);
            } else {
                painter.rect_filled(inner, CornerRadius::same(4), Color32::from_rgb(230, 236, 245));
                painter.text(
                    inner.center(),
                    Align2::CENTER_CENTER,
                    format!("[IMG] Drop or Upload\n{}", node.title),
                    FontId::proportional((11.5 * app.bridge.viewport.zoom).clamp(8.5, 18.0)),
                    Color32::from_rgb(70, 82, 104),
                );
            }
        }
        _ => {
            painter.rect_filled(srect, CornerRadius::same(corner as u8), fill);
            draw_rect_border(painter, srect, corner, stroke, node.style.border_style, dash_offset);
        }
    }
    }

    if !matches!(
        node.shape,
        ShapeKind::Section | ShapeKind::FolderTree | ShapeKind::Collapsible | ShapeKind::ImageNode
    ) {
        let font_size = (node.text_style.font_size * app.bridge.viewport.zoom).clamp(8.5, 28.0);
        let max_chars = ((srect.width() / (font_size * 0.56)).floor() as usize).max(4);
        let label = if node.title.chars().count() > max_chars {
            let truncated: String = node.title.chars().take(max_chars.saturating_sub(1)).collect();
            format!("{truncated}...")
        } else {
            node.title.clone()
        };
        let has_body = node_supports_body(&node.shape) && !node.body.trim().is_empty();
        if let Some(icon) = node.icon {
            let icon_sz = (srect.width().min(srect.height() * if has_body { 0.50 } else { 0.68 }))
                .clamp(22.0, 72.0);
            let icon_center_y = srect.min.y + icon_sz * 0.5 + (4.0 * app.bridge.viewport.zoom).clamp(2.0, 8.0);
            let icon_rect = EguiRect::from_center_size(
                Pos2::new(srect.center().x, icon_center_y),
                Vec2::splat(icon_sz),
            );
            draw_symbol_icon(painter, icon_rect, icon);
            let title_y = icon_rect.max.y + (font_size * 0.68) + 3.0;
            painter.text(
                Pos2::new(srect.center().x, title_y),
                Align2::CENTER_CENTER,
                label,
                FontId::proportional(font_size),
                to_color32(node.text_style.color),
            );
            if has_body {
                let body_font = (font_size * 0.80).clamp(8.0, 18.0);
                let body_max_chars = ((srect.width() - 14.0) / (body_font * 0.54)).floor().max(6.0) as usize;
                let mut by = title_y + body_font * 0.8 + 4.0;
                let line_h = body_font + 2.5;
                for raw_line in node.body.lines() {
                    if by > srect.max.y - 5.0 {
                        break;
                    }
                    let clipped: String = if raw_line.chars().count() > body_max_chars {
                        format!("{}...", raw_line.chars().take(body_max_chars.saturating_sub(3)).collect::<String>())
                    } else {
                        raw_line.to_string()
                    };
                    painter.text(
                        Pos2::new(srect.center().x, by),
                        Align2::CENTER_TOP,
                        clipped,
                        FontId::proportional(body_font),
                        Color32::from_rgba_unmultiplied(
                            to_color32(node.text_style.color).r(),
                            to_color32(node.text_style.color).g(),
                            to_color32(node.text_style.color).b(),
                            210,
                        ),
                    );
                    by += line_h;
                }
            }
        } else if has_body {
            let title_y = srect.min.y + (16.0 * app.bridge.viewport.zoom).clamp(12.0, 26.0);
            painter.text(
                Pos2::new(srect.center().x, title_y),
                Align2::CENTER_CENTER,
                label,
                FontId::proportional(font_size),
                to_color32(node.text_style.color),
            );
            let body_font = (font_size * 0.82).clamp(8.0, 20.0);
            let body_max_chars = ((srect.width() - 16.0) / (body_font * 0.54)).floor().max(6.0) as usize;
            let mut by = title_y + body_font + 6.0;
            let line_h = body_font + 3.0;
            for raw_line in node.body.lines() {
                if by > srect.max.y - 6.0 {
                    break;
                }
                let clipped: String = if raw_line.chars().count() > body_max_chars {
                    format!("{}...", raw_line.chars().take(body_max_chars.saturating_sub(3)).collect::<String>())
                } else {
                    raw_line.to_string()
                };
                painter.text(
                    Pos2::new(srect.center().x, by),
                    Align2::CENTER_TOP,
                    clipped,
                    FontId::proportional(body_font),
                    Color32::from_rgba_unmultiplied(
                        to_color32(node.text_style.color).r(),
                        to_color32(node.text_style.color).g(),
                        to_color32(node.text_style.color).b(),
                        215,
                    ),
                );
                by += line_h;
            }
        } else {
            let text_pos = if matches!(node.shape, ShapeKind::Actor) {
                Pos2::new(srect.center().x, srect.max.y - 8.0)
            } else {
                srect.center()
            };
            painter.text(
                text_pos,
                Align2::CENTER_CENTER,
                label,
                FontId::proportional(font_size),
                to_color32(node.text_style.color),
            );
        }
    }

    if let Some(owner) = app.collab.locked_by_other(node.id) {
        let badge_pos = Pos2::new(srect.min.x + 4.0, srect.min.y - 16.0);
        painter.text(
            badge_pos,
            Align2::LEFT_TOP,
            format!("LOCKED: {owner} (click to duel)"),
            FontId::proportional(10.5),
            Color32::from_rgb(220, 38, 38),
        );
    }

    if is_selected {
        let sel_col = Color32::from_rgb(96, 165, 250);
        let sel_stroke = Stroke::new(1.6_f32, sel_col);
        let box_rect = srect.expand(3.0);
        draw_rect_border(
            painter,
            box_rect,
            3.0,
            sel_stroke,
            BorderStyle::Dotted,
            0.0,
        );
        let show_mid = box_rect.width() >= 48.0 && box_rect.height() >= 40.0;
        for (kind, pos) in resize_handle_positions(box_rect) {
            if !show_mid
                && matches!(
                    kind,
                    ResizeHandle::N | ResizeHandle::E | ResizeHandle::S | ResizeHandle::W
                )
            {
                continue;
            }
            let r = if matches!(
                kind,
                ResizeHandle::NW | ResizeHandle::NE | ResizeHandle::SE | ResizeHandle::SW
            ) {
                3.8
            } else {
                3.0
            };
            painter.circle_filled(pos, r, Color32::WHITE);
            painter.circle_stroke(pos, r, Stroke::new(1.4_f32, sel_col));
        }
    }

    if !busy_dragging {
        let reveal_margin = if app.connecting_from.is_some() {
            36.0_f32
        } else {
            20.0_f32
        };
        let show_all_ports_on_node = pointer_pos
            .is_some_and(|p| srect.expand(reveal_margin).contains(p))
            || hovered_port.is_some_and(|(hnid, _)| hnid == node.id)
            || app.connecting_from.is_some_and(|(snid, _)| snid == node.id);

        if show_all_ports_on_node {
            for port in &node.ports {
                let pt_w = port.world_position(&draw_bounds);
                let pt_s = world_to_screen(canvas_rect, app, pt_w);
                let is_src = app.connecting_from == Some((node.id, port.id));
                let is_hot = hovered_port == Some((node.id, port.id));
                if is_hot || is_src {
                    painter.circle_filled(pt_s, 5.0, Color32::from_rgb(16, 185, 129));
                    painter.circle_stroke(pt_s, 5.0, Stroke::new(1.5_f32, Color32::WHITE));
                } else {
                    painter.circle_filled(pt_s, 3.2, Color32::WHITE);
                    painter.circle_stroke(
                        pt_s,
                        3.2,
                        Stroke::new(1.25_f32, Color32::from_rgb(0, 0, 140)),
                    );
                }
            }
        }
    }
}

pub fn draw_simple_layer_shape(
    painter: &Painter,
    r: EguiRect,
    shape: &ShapeKind,
    fill: Color32,
    stroke: Stroke,
) {
    match shape {
        ShapeKind::Diamond => {
            let c = r.center();
            let pts = vec![
                Pos2::new(c.x, r.min.y),
                Pos2::new(r.max.x, c.y),
                Pos2::new(c.x, r.max.y),
                Pos2::new(r.min.x, c.y),
            ];
            painter.add(egui::Shape::convex_polygon(pts, fill, stroke));
        }
        ShapeKind::Ellipse => {
            let c = r.center();
            let rx = r.width() * 0.5;
            let ry = r.height() * 0.5;
            let segs = 24;
            let mut pts = Vec::with_capacity(segs);
            for i in 0..segs {
                let a = (i as f32 / segs as f32) * std::f32::consts::TAU;
                pts.push(Pos2::new(c.x + a.cos() * rx, c.y + a.sin() * ry));
            }
            painter.add(egui::Shape::convex_polygon(pts, fill, stroke));
        }
        ShapeKind::Hexagon => {
            let inset = r.width() * 0.2;
            let cy = r.center().y;
            let pts = vec![
                Pos2::new(r.min.x + inset, r.min.y),
                Pos2::new(r.max.x - inset, r.min.y),
                Pos2::new(r.max.x, cy),
                Pos2::new(r.max.x - inset, r.max.y),
                Pos2::new(r.min.x + inset, r.max.y),
                Pos2::new(r.min.x, cy),
            ];
            painter.add(egui::Shape::convex_polygon(pts, fill, stroke));
        }
        ShapeKind::Shield => {
            let cx = r.center().x;
            let pts = vec![
                Pos2::new(cx, r.min.y),
                Pos2::new(r.max.x, r.min.y + r.height() * 0.2),
                Pos2::new(r.max.x - r.width() * 0.14, r.max.y - r.height() * 0.25),
                Pos2::new(cx, r.max.y),
                Pos2::new(r.min.x + r.width() * 0.14, r.max.y - r.height() * 0.25),
                Pos2::new(r.min.x, r.min.y + r.height() * 0.2),
            ];
            painter.add(egui::Shape::convex_polygon(pts, fill, stroke));
        }
        ShapeKind::RoundedRect => {
            painter.rect_filled(r, 6.0, fill);
            painter.rect_stroke(r, 6.0, stroke, egui::StrokeKind::Inside);
        }
        _ => {
            painter.rect_filled(r, 1.5, fill);
            painter.rect_stroke(r, 1.5, stroke, egui::StrokeKind::Inside);
        }
    }
}

pub(super) fn find_end_tangent(pts: &[Pos2]) -> Option<(Pos2, Pos2)> {
    let tip = *pts.last()?;
    for &p in pts.iter().rev().skip(1) {
        if tip.distance(p) >= 1.5 {
            return Some((tip, p));
        }
    }
    None
}

fn find_start_tangent(pts: &[Pos2]) -> Option<(Pos2, Pos2)> {
    let tip = *pts.first()?;
    for &p in pts.iter().skip(1) {
        if tip.distance(p) >= 1.5 {
            return Some((tip, p));
        }
    }
    None
}

pub(super) fn draw_arrowhead(painter: &Painter, tip: Pos2, prev: Pos2, color: Color32, style: EdgeArrowStyle, zoom: f32) {
    let dir = (tip - prev).normalized();
    if dir.length_sq() < 0.1 {
        return;
    }
    let perp = Vec2::new(-dir.y, dir.x);
    let size = (9.5 * zoom).clamp(6.0, 15.0);
    match style {
        EdgeArrowStyle::None => {}
        EdgeArrowStyle::OneWay | EdgeArrowStyle::TwoWay => {
            let arrow = vec![
                tip,
                tip - dir * size + perp * (size * 0.52),
                tip - dir * size - perp * (size * 0.52),
            ];
            painter.add(egui::Shape::convex_polygon(arrow, color, Stroke::NONE));
        }
        EdgeArrowStyle::Open => {
            let s = Stroke::new(1.8_f32, color);
            painter.line_segment([tip, tip - dir * size + perp * (size * 0.55)], s);
            painter.line_segment([tip, tip - dir * size - perp * (size * 0.55)], s);
        }
        EdgeArrowStyle::Diamond => {
            let mid = tip - dir * (size * 0.6);
            let back = tip - dir * (size * 1.2);
            let pts = vec![
                tip,
                mid + perp * (size * 0.42),
                back,
                mid - perp * (size * 0.42),
            ];
            painter.add(egui::Shape::convex_polygon(pts, color, Stroke::NONE));
        }
    }
}

pub(super) fn draw_edge(
    painter: &Painter,
    canvas_rect: EguiRect,
    app: &mut BohatDrawApp,
    edge_idx: usize,
    now_sec: f32,
    placed_labels: &mut Vec<Rect>,
) {
    let edge = &app.bridge.scene.edges()[edge_idx];
    let world_pts = tessellate_edge_path(&app.bridge.scene, edge);
    if world_pts.len() < 2 {
        return;
    }
    let screen_pts: Vec<Pos2> = world_pts
        .iter()
        .map(|&p| world_to_screen(canvas_rect, app, p))
        .collect();
    let is_selected = app.bridge.selection.selected_edges.contains(&edge.id);
    let color = if is_selected {
        Color32::from_rgb(0, 0, 180)
    } else {
        to_color32(edge.stroke)
    };
    let width = ((edge.stroke_width + if is_selected { 1.2 } else { 0.0 }) * app.bridge.viewport.zoom)
        .clamp(1.2, 8.0);
    let stroke = Stroke::new(width, color);

    let h = edge.id.wrapping_mul(0x9E37_79B9_7F4A_7C15);
    let edge_phase = ((h >> 32) as u32 as f32) / (u32::MAX as f32);
    let speed_factor = 0.86 + (((h & 0xFFFF) as u32 as f32) / 65535.0) * 0.28;

    match &edge.animation {
        EdgeAnimation::MarchingAnts { speed, reverse, .. } => {
            let dir = if *reverse { 1.0 } else { -1.0 };
            draw_stroked_polyline(
                painter,
                &screen_pts,
                stroke,
                BorderStyle::Dashed,
                (now_sec * speed * speed_factor * 0.9 + edge_phase * 24.0) * dir,
            );
        }
        EdgeAnimation::PulseParticles {
            speed,
            particle_count,
            radius,
            color: pcol,
        } => {
            draw_stroked_polyline(painter, &screen_pts, stroke, BorderStyle::Solid, 0.0);
            let count = (*particle_count).clamp(1, 12);
            let path_len: f32 = world_pts
                .windows(2)
                .map(|w| w[0].distance_to(w[1]))
                .sum::<f32>()
                .max(48.0);
            let vel_px =
                (if *speed > 4.0 { *speed * 1.65 } else { *speed * 145.0 }) * speed_factor;
            let rate = (vel_px / path_len).clamp(0.12, 1.35);
            let base_col = to_color32(*pcol);
            let base_r = (*radius * app.bridge.viewport.zoom).clamp(2.5, 9.0);
            for i in 0..count {
                let t = (now_sec * rate + edge_phase + i as f32 / count as f32).rem_euclid(1.0);
                let env = if t < 0.08 {
                    (t / 0.08).clamp(0.15, 1.0)
                } else if t > 0.92 {
                    ((1.0 - t) / 0.08).clamp(0.15, 1.0)
                } else {
                    1.0
                };
                if let Some(pt_w) = rustflow_renderer::sample_polyline_at(&world_pts, t) {
                    let pt_s = world_to_screen(canvas_rect, app, pt_w);
                    let r = base_r * (0.68 + 0.32 * env);
                    let halo_a = (68.0 * env).round() as u8;
                    let core_a = (255.0 * env).round() as u8;
                    painter.circle_filled(
                        pt_s,
                        r * 1.45,
                        Color32::from_rgba_unmultiplied(
                            base_col.r(),
                            base_col.g(),
                            base_col.b(),
                            halo_a,
                        ),
                    );
                    painter.circle_filled(
                        pt_s,
                        r,
                        Color32::from_rgba_unmultiplied(
                            base_col.r(),
                            base_col.g(),
                            base_col.b(),
                            core_a,
                        ),
                    );
                }
            }
        }
        EdgeAnimation::None => {
            draw_stroked_polyline(painter, &screen_pts, stroke, BorderStyle::Solid, 0.0);
        }
    }

    if let Some((tip, prev)) = find_end_tangent(&screen_pts) {
        draw_arrowhead(
            painter,
            tip,
            prev,
            color,
            edge.arrow,
            app.bridge.viewport.zoom,
        );
    }
    if matches!(edge.arrow, EdgeArrowStyle::TwoWay) {
        if let Some((tip, next)) = find_start_tangent(&screen_pts) {
            draw_arrowhead(
                painter,
                tip,
                next,
                color,
                EdgeArrowStyle::OneWay,
                app.bridge.viewport.zoom,
            );
        }
    }

    if !edge.label.is_empty() {
        let ls = edge.label_style.clone();
        let label_text = edge.label.clone();
        let zoom = app.bridge.viewport.zoom.max(0.1);
        let font_px = (ls.font_size * zoom).clamp(8.5, 28.0);
        if matches!(ls.placement, rustflow_core::EdgeLabelPlacement::AlongPath) {
            let fast = app.is_interacting();
            draw_curved_edge_label(
                painter,
                &mut app.glyph_cache,
                &screen_pts,
                &label_text,
                font_px,
                ls.position.clamp(0.08, 0.92),
                ls.offset_y * zoom,
                to_color32(ls.text_color),
                to_color32(ls.bg_color),
                ls.show_bg,
                fast,
            );
        } else {
            let galley = painter.layout_no_wrap(
                label_text,
                FontId::proportional(font_px),
                to_color32(ls.text_color),
            );
            let gsize = galley.size();
            let pill_size = gsize + Vec2::new(14.0, 7.0);
            if let Some(pose) = rustflow_renderer::resolve_smart_edge_label_pose(
                &app.bridge.scene,
                &world_pts,
                ls.placement,
                ls.position,
                ls.offset_y,
                pill_size.x / zoom,
                pill_size.y / zoom,
                placed_labels,
            ) {
                let mid = world_to_screen(canvas_rect, app, pose.world_center);
                let pill = EguiRect::from_center_size(mid, pill_size);
                if ls.show_bg {
                    painter.rect_filled(pill, 4.0, to_color32(ls.bg_color));
                    painter.rect_stroke(pill, 4.0, Stroke::new(1.0_f32, color), egui::StrokeKind::Inside);
                }
                painter.galley(
                    Pos2::new(mid.x - gsize.x * 0.5, mid.y - gsize.y * 0.5),
                    galley,
                    to_color32(ls.text_color),
                );
            }
        }
    }

    if is_selected {
        let (wp_handles, mid_handles) = edge_control_handles(app, canvas_rect, edge_idx, &world_pts);
        for (_, _, mpos) in mid_handles {
            painter.circle_filled(mpos, 4.5, Color32::from_rgba_unmultiplied(14, 165, 233, 195));
            painter.circle_stroke(mpos, 4.5, Stroke::new(1.1_f32, Color32::WHITE));
        }
        for (_, wpos) in wp_handles {
            painter.circle_filled(wpos, 5.5, Color32::WHITE);
            painter.circle_stroke(wpos, 5.5, Stroke::new(1.8_f32, Color32::from_rgb(0, 0, 160)));
        }
    }
}

fn edge_control_handles(
    app: &BohatDrawApp,
    canvas_rect: EguiRect,
    edge_idx: usize,
    world_pts: &[Point],
) -> (Vec<(usize, Pos2)>, Vec<(usize, Point, Pos2)>) {
    let mut wp_handles = Vec::new();
    let mut mid_handles = Vec::new();
    let Some(edge) = app.bridge.scene.edges().get(edge_idx) else {
        return (wp_handles, mid_handles);
    };
    let Some((start, end, ..)) = app.bridge.scene.edge_endpoints(edge) else {
        return (wp_handles, mid_handles);
    };
    if edge.waypoints.is_empty() {
        if let Some(mid_w) = rustflow_renderer::sample_polyline_at(world_pts, 0.5) {
            mid_handles.push((0, mid_w, world_to_screen(canvas_rect, app, mid_w)));
        }
    } else {
        for (i, &pt) in edge.waypoints.iter().enumerate() {
            wp_handles.push((i, world_to_screen(canvas_rect, app, pt)));
        }
        let mut chain = Vec::with_capacity(edge.waypoints.len() + 2);
        chain.push(start);
        chain.extend_from_slice(&edge.waypoints);
        chain.push(end);
        for i in 0..(chain.len() - 1) {
            let a = chain[i];
            let b = chain[i + 1];
            if a.distance_to(b) >= 28.0 {
                let mid_w = Point::new((a.x + b.x) * 0.5, (a.y + b.y) * 0.5);
                mid_handles.push((i, mid_w, world_to_screen(canvas_rect, app, mid_w)));
            }
        }
    }
    (wp_handles, mid_handles)
}

pub(super) enum EdgeHandleHit {
    Waypoint(EdgeId, usize),
    Midpoint(EdgeId, usize, Point),
}

pub(super) fn hit_test_selected_edge_handle(
    app: &BohatDrawApp,
    canvas_rect: EguiRect,
    screen_pos: Pos2,
) -> Option<EdgeHandleHit> {
    for &eid in &app.bridge.selection.selected_edges {
        if let Some(edge_idx) = app.bridge.scene.edges().iter().position(|e| e.id == eid) {
            let world_pts = tessellate_edge_path(&app.bridge.scene, &app.bridge.scene.edges()[edge_idx]);
            let (wp_handles, mid_handles) = edge_control_handles(app, canvas_rect, edge_idx, &world_pts);
            for (i, pos) in wp_handles {
                if pos.distance(screen_pos) <= 10.0 {
                    return Some(EdgeHandleHit::Waypoint(eid, i));
                }
            }
            for (insert_idx, mid_w, pos) in mid_handles {
                if pos.distance(screen_pos) <= 9.5 {
                    return Some(EdgeHandleHit::Midpoint(eid, insert_idx, mid_w));
                }
            }
        }
    }
    None
}

fn sample_screen_path_at_dist(pts: &[Pos2], dist: f32) -> (Pos2, f32) {
    let mut acc = 0.0_f32;
    for w in pts.windows(2) {
        let seg = w[1] - w[0];
        let len = seg.length();
        if len < 1e-4 {
            continue;
        }
        if acc + len >= dist {
            let frac = ((dist - acc) / len).clamp(0.0, 1.0);
            let pos = w[0] + seg * frac;
            let angle = seg.y.atan2(seg.x);
            return (pos, angle);
        }
        acc += len;
    }
    let last = *pts.last().unwrap_or(&Pos2::ZERO);
    let prev = pts.get(pts.len().saturating_sub(2)).copied().unwrap_or(last);
    let seg = last - prev;
    (last, seg.y.atan2(seg.x))
}

#[allow(clippy::too_many_arguments)]
fn draw_curved_edge_label(
    painter: &Painter,
    glyph_cache: &mut std::collections::HashMap<(char, u16, Color32), (std::sync::Arc<egui::Galley>, f32)>,
    screen_pts: &[Pos2],
    label: &str,
    font_px: f32,
    t_pos: f32,
    offset_y: f32,
    text_color: Color32,
    bg_color: Color32,
    show_bg: bool,
    fast_mode: bool,
) {
    if screen_pts.len() < 2 || label.is_empty() {
        return;
    }
    let mut path: Vec<Pos2> = screen_pts.to_vec();
    let first = path[0];
    let last = *path.last().unwrap();
    let reversed = last.x < first.x - 1.0 || ((last.x - first.x).abs() <= 1.0 && last.y > first.y);
    if reversed {
        path.reverse();
    }
    let effective_t = if reversed { 1.0 - t_pos } else { t_pos };

    let total_len: f32 = path.windows(2).map(|w| w[0].distance(w[1])).sum();
    if total_len < 12.0 {
        return;
    }

    let font_key = (font_px * 2.0).round().clamp(16.0, 120.0) as u16;
    let snapped_px = font_key as f32 * 0.5;
    let font_id = FontId::proportional(snapped_px);
    let mid_dist = effective_t * total_len;
    let est_half = (label.len() as f32 * snapped_px * 0.28).min(total_len * 0.45);
    let (_, ang_a) = sample_screen_path_at_dist(&path, (mid_dist - est_half).max(0.0));
    let (_, ang_b) = sample_screen_path_at_dist(&path, (mid_dist + est_half).min(total_len));
    let is_nearly_flat = (ang_b - ang_a).abs() < 0.05;

    if fast_mode || is_nearly_flat {
        let galley = painter.layout_no_wrap(label.to_string(), font_id, text_color);
        let gs = galley.size();
        let lift = gs.y * 0.55 + 5.0 - offset_y;
        let (p, ang) = sample_screen_path_at_dist(&path, mid_dist);
        let nx = ang.sin();
        let ny = -ang.cos();
        let center = Pos2::new(p.x + nx * lift, p.y + ny * lift);
        let (sin_a, cos_a) = ang.sin_cos();
        if show_bg {
            let half_w = gs.x * 0.5 + 5.0;
            let p0 = Pos2::new(center.x - cos_a * half_w, center.y - sin_a * half_w);
            let p1 = Pos2::new(center.x + cos_a * half_w, center.y + sin_a * half_w);
            painter.line_segment([p0, p1], Stroke::new(gs.y + 4.0, bg_color));
        }
        let dx = -gs.x * 0.5;
        let dy = -gs.y * 0.5;
        let text_pos = Pos2::new(
            center.x + dx * cos_a - dy * sin_a,
            center.y + dx * sin_a + dy * cos_a,
        );
        let tshape = egui::epaint::TextShape::new(text_pos, galley, text_color).with_angle(ang);
        painter.add(egui::Shape::Text(tshape));
        return;
    }

    if glyph_cache.len() > 2048 {
        glyph_cache.clear();
    }
    let mut glyphs = Vec::with_capacity(label.len());
    let mut total_w = 0.0_f32;
    let mut max_h = snapped_px;
    for ch in label.chars() {
        let entry = glyph_cache.entry((ch, font_key, text_color)).or_insert_with(|| {
            let g = painter.layout_no_wrap(ch.to_string(), font_id.clone(), text_color);
            let w = g.size().x.max(snapped_px * 0.32);
            (g, w)
        });
        max_h = max_h.max(entry.0.size().y);
        glyphs.push((entry.0.clone(), entry.1));
        total_w += entry.1;
    }

    let start_dist = (effective_t * total_len - total_w * 0.5)
        .clamp(6.0, (total_len - total_w - 6.0).max(0.0));
    let lift = max_h * 0.55 + 5.0 - offset_y;

    if show_bg {
        let ribbon_steps = ((total_w / 10.0).ceil() as usize).clamp(3, 18);
        let mut ribbon_pts = Vec::with_capacity(ribbon_steps + 1);
        for i in 0..=ribbon_steps {
            let d = (start_dist - 5.0 + (total_w + 10.0) * (i as f32 / ribbon_steps as f32))
                .clamp(0.0, total_len);
            let (p, ang) = sample_screen_path_at_dist(&path, d);
            let nx = ang.sin();
            let ny = -ang.cos();
            ribbon_pts.push(Pos2::new(p.x + nx * lift, p.y + ny * lift));
        }
        painter.add(egui::Shape::line(
            ribbon_pts,
            Stroke::new(max_h + 4.0, bg_color),
        ));
    }

    let mut cursor = 0.0_f32;
    for (galley, cw) in glyphs {
        let d = (start_dist + cursor + cw * 0.5).clamp(0.0, total_len);
        let (p, ang) = sample_screen_path_at_dist(&path, d);
        let nx = ang.sin();
        let ny = -ang.cos();
        let center = Pos2::new(p.x + nx * lift, p.y + ny * lift);
        let (sin_a, cos_a) = ang.sin_cos();
        let gs = galley.size();
        let dx = -gs.x * 0.5;
        let dy = -gs.y * 0.5;
        let text_pos = Pos2::new(
            center.x + dx * cos_a - dy * sin_a,
            center.y + dx * sin_a + dy * cos_a,
        );
        let tshape = egui::epaint::TextShape::new(text_pos, galley, text_color).with_angle(ang);
        painter.add(egui::Shape::Text(tshape));
        cursor += cw;
    }
}

pub(super) fn draw_remote_cursors(painter: &Painter, canvas_rect: EguiRect, app: &BohatDrawApp) {
    for peer in &app.collab.peers {
        if app.collab.client_id.as_deref() == Some(peer.id.as_str()) {
            continue;
        }
        if let Some([wx, wy]) = peer.cursor {
            let pos = world_to_screen(canvas_rect, app, Point::new(wx, wy));
            let col = Color32::from_rgb(239, 68, 68);
            painter.circle_filled(pos, 4.5, col);
            painter.text(
                pos + Vec2::new(8.0, -4.0),
                Align2::LEFT_CENTER,
                &peer.name,
                FontId::proportional(11.0),
                col,
            );
        }
    }
}

