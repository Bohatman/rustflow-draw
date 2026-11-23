use crate::{
    compute_visible_grid_dots, sample_polyline_at, tessellate_edge_path, HeadlessFramebuffer,
};
use ab_glyph::{Font, FontRef, PxScale, ScaleFont};
use rustflow_core::{
    BorderStyle, EdgeAnimation, EdgeArrowStyle, Node, Point, SceneGraph, ShapeKind, Viewport,
};
use std::sync::OnceLock;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ExportBgStyle {
    #[default]
    Dots,
    Grid,
    Crosshair,
    Isometric,
    None,
}

fn ui_font() -> Option<&'static FontRef<'static>> {
    static FONT: OnceLock<Option<FontRef<'static>>> = OnceLock::new();
    FONT.get_or_init(|| FontRef::try_from_slice(epaint_default_fonts::UBUNTU_LIGHT).ok())
        .as_ref()
}

fn rgba_u8(r: f32, g: f32, b: f32, a: f32) -> [u8; 4] {
    [
        (r.clamp(0.0, 1.0) * 255.0).round() as u8,
        (g.clamp(0.0, 1.0) * 255.0).round() as u8,
        (b.clamp(0.0, 1.0) * 255.0).round() as u8,
        (a.clamp(0.0, 1.0) * 255.0).round() as u8,
    ]
}

fn rounded_rect_polygon(x: f32, y: f32, w: f32, h: f32, radius: f32) -> Vec<Point> {
    let r = radius.clamp(0.0, (w * 0.5).min(h * 0.5));
    if r < 1.5 {
        return vec![
            Point::new(x, y),
            Point::new(x + w, y),
            Point::new(x + w, y + h),
            Point::new(x, y + h),
        ];
    }
    let segs = 6usize;
    let mut pts = Vec::with_capacity((segs + 1) * 4);
    let corners = [
        (x + w - r, y + r, -std::f32::consts::FRAC_PI_2, 0.0),
        (x + w - r, y + h - r, 0.0, std::f32::consts::FRAC_PI_2),
        (x + r, y + h - r, std::f32::consts::FRAC_PI_2, std::f32::consts::PI),
        (x + r, y + r, std::f32::consts::PI, std::f32::consts::PI * 1.5),
    ];
    for (cx, cy, a0, a1) in corners {
        for i in 0..=segs {
            let a = a0 + (a1 - a0) * (i as f32 / segs as f32);
            pts.push(Point::new(cx + a.cos() * r, cy + a.sin() * r));
        }
    }
    pts
}

impl HeadlessFramebuffer {
    pub fn blend_pixel(&mut self, x: i32, y: i32, src: [u8; 4]) {
        if x < 0 || y < 0 || (x as u32) >= self.width || (y as u32) >= self.height || src[3] == 0 {
            return;
        }
        let idx = (y as u32 * self.width + x as u32) as usize;
        if src[3] == 255 {
            self.pixels[idx] = src;
            return;
        }
        let dst = self.pixels[idx];
        let a = src[3] as f32 / 255.0;
        let inv = 1.0 - a;
        self.pixels[idx] = [
            (src[0] as f32 * a + dst[0] as f32 * inv).round() as u8,
            (src[1] as f32 * a + dst[1] as f32 * inv).round() as u8,
            (src[2] as f32 * a + dst[2] as f32 * inv).round() as u8,
            255,
        ];
    }

    pub fn render_scene_with_bg(
        &mut self,
        scene: &SceneGraph,
        vp: &Viewport,
        time_seconds: f32,
        bg_style: ExportBgStyle,
    ) {
        self.render_background(vp, bg_style);

        let frustum = vp.visible_world_rect();
        let mut placed_labels: Vec<rustflow_core::Rect> = Vec::new();

        for edge in scene.query_visible_edges(&frustum) {
            let pts = tessellate_edge_path(scene, edge);
            if pts.len() < 2 {
                continue;
            }
            let screen_pts: Vec<Point> = pts.iter().map(|&p| vp.world_to_screen(p)).collect();
            let color = rgba_u8(edge.stroke.r, edge.stroke.g, edge.stroke.b, 1.0);
            let thickness = ((edge.stroke_width * vp.zoom).round() as i32).clamp(1, 5);

            match edge.animation {
                EdgeAnimation::MarchingAnts {
                    speed,
                    dash_length,
                    gap_length,
                    reverse,
                } => {
                    let dir = if reverse { 1.0 } else { -1.0 };
                    let dash = (dash_length * vp.zoom).max(4.0);
                    let gap = (gap_length * vp.zoom).max(3.0);
                    let offset = time_seconds * speed * 0.9 * dir;
                    self.draw_dashed_polyline_aa(&screen_pts, color, thickness, dash, gap, offset);
                }
                EdgeAnimation::PulseParticles {
                    speed,
                    particle_count,
                    radius,
                    color: pcol,
                } => {
                    for seg in screen_pts.windows(2) {
                        self.draw_thick_line_aa(seg[0], seg[1], color, thickness);
                    }
                    let count = particle_count.clamp(1, 12);
                    let pr = ((radius * vp.zoom).round() as i32).clamp(2, 9);
                    let pc = rgba_u8(pcol.r, pcol.g, pcol.b, 1.0);
                    let rate = if speed > 4.0 { speed * 0.004 } else { speed };
                    for idx in 0..count {
                        let t = (time_seconds * rate + idx as f32 / count as f32).rem_euclid(1.0);
                        if let Some(world_pt) = sample_polyline_at(&pts, t) {
                            let sp = vp.world_to_screen(world_pt);
                            self.fill_circle_aa(sp.x.round() as i32, sp.y.round() as i32, pr, pc);
                        }
                    }
                }
                EdgeAnimation::None => {
                    for seg in screen_pts.windows(2) {
                        self.draw_thick_line_aa(seg[0], seg[1], color, thickness);
                    }
                }
            }

            if !matches!(edge.arrow, EdgeArrowStyle::None) {
                let tip = *screen_pts.last().unwrap();
                if let Some(&prev) = screen_pts
                    .iter()
                    .rev()
                    .skip(1)
                    .find(|p| tip.distance_to(**p) >= 1.5)
                {
                    self.draw_arrow_styled(
                        tip,
                        prev,
                        color,
                        edge.arrow,
                        (9.5 * vp.zoom).clamp(6.0, 15.0),
                    );
                }
                if matches!(edge.arrow, EdgeArrowStyle::TwoWay) {
                    let start_tip = screen_pts[0];
                    if let Some(&next) = screen_pts
                        .iter()
                        .skip(1)
                        .find(|p| start_tip.distance_to(**p) >= 1.5)
                    {
                        self.draw_arrow_styled(
                            start_tip,
                            next,
                            color,
                            EdgeArrowStyle::OneWay,
                            (9.5 * vp.zoom).clamp(6.0, 15.0),
                        );
                    }
                }
            }

            if !edge.label.is_empty() {
                let ls = &edge.label_style;
                let zoom = vp.zoom.max(0.1);
                let font_px = (ls.font_size * zoom).clamp(9.0, 28.0);
                let tc = rgba_u8(ls.text_color.r, ls.text_color.g, ls.text_color.b, 1.0);
                let bg = rgba_u8(ls.bg_color.r, ls.bg_color.g, ls.bg_color.b, ls.bg_color.a);
                if matches!(ls.placement, rustflow_core::EdgeLabelPlacement::AlongPath) {
                    self.draw_text_along_path_aa(
                        &screen_pts,
                        &edge.label,
                        font_px,
                        ls.position.clamp(0.08, 0.92),
                        ls.offset_y * zoom,
                        tc,
                        bg,
                        ls.show_bg,
                    );
                } else {
                    let tw = self.measure_text_width(&edge.label, font_px) + 14.0;
                    let th = font_px + 8.0;
                    if let Some(pose) = crate::resolve_smart_edge_label_pose(
                        scene,
                        &pts,
                        ls.placement,
                        ls.position,
                        ls.offset_y,
                        tw / zoom,
                        th / zoom,
                        &mut placed_labels,
                    ) {
                        let mid = vp.world_to_screen(pose.world_center);
                        if ls.show_bg {
                            let bx0 = (mid.x - tw * 0.5).round() as i32;
                            let by0 = (mid.y - th * 0.5).round() as i32;
                            let bx1 = (mid.x + tw * 0.5).round() as i32;
                            let by1 = (mid.y + th * 0.5).round() as i32;
                            self.fill_rect_aa(bx0, by0, bx1, by1, bg);
                            self.stroke_rect_aa(bx0, by0, bx1, by1, color, 1);
                        }
                        self.draw_text_aa(mid.x, mid.y, &edge.label, font_px, tc, true);
                    }
                }
            }
        }

        for node in scene.query_visible_nodes(&frustum) {
            self.render_node_aa(node, vp, time_seconds);
        }
    }

    fn render_background(&mut self, vp: &Viewport, bg_style: ExportBgStyle) {
        let step = (vp.grid_size * vp.zoom).clamp(14.0, 80.0);
        let ox = vp.pan.x.rem_euclid(step);
        let oy = vp.pan.y.rem_euclid(step);
        let dot_col = [110, 122, 150, 74];
        let minor_col = [136, 148, 174, 42];
        let major_col = [112, 124, 154, 72];

        match bg_style {
            ExportBgStyle::None => {}
            ExportBgStyle::Dots => {
                for dot in compute_visible_grid_dots(vp) {
                    self.fill_circle_aa(dot.x.round() as i32, dot.y.round() as i32, 1, dot_col);
                }
            }
            ExportBgStyle::Grid => {
                let mut ix = 0i32;
                let mut x = ox;
                while x < self.width as f32 {
                    let c = if ix % 5 == 0 { major_col } else { minor_col };
                    self.draw_thick_line_aa(
                        Point::new(x, 0.0),
                        Point::new(x, self.height as f32),
                        c,
                        1,
                    );
                    x += step;
                    ix += 1;
                }
                let mut iy = 0i32;
                let mut y = oy;
                while y < self.height as f32 {
                    let c = if iy % 5 == 0 { major_col } else { minor_col };
                    self.draw_thick_line_aa(
                        Point::new(0.0, y),
                        Point::new(self.width as f32, y),
                        c,
                        1,
                    );
                    y += step;
                    iy += 1;
                }
            }
            ExportBgStyle::Crosshair => {
                let arm = 4.0;
                let mut y = oy;
                while y < self.height as f32 {
                    let mut x = ox;
                    while x < self.width as f32 {
                        self.draw_thick_line_aa(
                            Point::new(x - arm, y),
                            Point::new(x + arm, y),
                            major_col,
                            1,
                        );
                        self.draw_thick_line_aa(
                            Point::new(x, y - arm),
                            Point::new(x, y + arm),
                            major_col,
                            1,
                        );
                        x += step;
                    }
                    y += step;
                }
            }
            ExportBgStyle::Isometric => {
                let iso_step = step * 1.6;
                let h = self.height as f32;
                let w = self.width as f32;
                let mut d = -h * 1.8 + vp.pan.x.rem_euclid(iso_step);
                while d < w + h * 2.0 {
                    self.draw_thick_line_aa(
                        Point::new(d, 0.0),
                        Point::new(d + h * 1.732, h),
                        minor_col,
                        1,
                    );
                    self.draw_thick_line_aa(
                        Point::new(d, 0.0),
                        Point::new(d - h * 1.732, h),
                        minor_col,
                        1,
                    );
                    d += iso_step;
                }
            }
        }
    }

    fn render_node_aa(&mut self, node: &Node, vp: &Viewport, time_seconds: f32) {
        let mut bounds = node.bounds;
        if matches!(node.shape, ShapeKind::Collapsible) && node.collapsed {
            bounds.height = 34.0;
        }
        let tl = vp.world_to_screen(Point::new(bounds.x, bounds.y));
        let br = vp.world_to_screen(Point::new(bounds.right(), bounds.bottom()));
        let w = (br.x - tl.x).max(4.0);
        let h = (br.y - tl.y).max(4.0);
        let cx = tl.x + w * 0.5;
        let cy = tl.y + h * 0.5;
        let opacity = node.style.opacity.clamp(0.08, 1.0);
        let fill = rgba_u8(
            node.style.fill.r,
            node.style.fill.g,
            node.style.fill.b,
            node.style.fill.a * opacity,
        );
        let stroke = rgba_u8(node.style.stroke.r, node.style.stroke.g, node.style.stroke.b, 1.0);
        let text_col = rgba_u8(
            node.text_style.color.r,
            node.text_style.color.g,
            node.text_style.color.b,
            1.0,
        );
        let bw = ((node.style.stroke_width * vp.zoom).round() as i32).clamp(1, 5);
        let corner_r = match node.shape {
            ShapeKind::MobileScreen => 12.0 * vp.zoom,
            ShapeKind::Queue => (h * 0.35).min(24.0 * vp.zoom),
            ShapeKind::BrowserWindow | ShapeKind::Server => 4.0 * vp.zoom,
            ShapeKind::Rectangle | ShapeKind::Firewall | ShapeKind::Subprocess => 2.0,
            _ => (node.style.corner_radius * vp.zoom).clamp(0.0, 24.0),
        };
        let dash_offset = if node.style.border_anim {
            -time_seconds * 36.0
        } else {
            0.0
        };
        let bs = node.style.border_style;

        if !node.custom_layers.is_empty() {
            for layer in &node.custom_layers {
                let lx = tl.x + layer.x * w;
                let ly = tl.y + layer.y * h;
                let lw = (layer.w * w).max(4.0);
                let lh = (layer.h * h).max(4.0);
                let lf = rgba_u8(layer.fill.r, layer.fill.g, layer.fill.b, layer.fill.a * opacity);
                let ls = rgba_u8(layer.stroke.r, layer.stroke.g, layer.stroke.b, 1.0);
                let lcr = if matches!(layer.shape, ShapeKind::RoundedRect) { 6.0 * vp.zoom } else { 1.5 };
                self.render_shape_geometry(
                    &layer.shape,
                    lx,
                    ly,
                    lw,
                    lh,
                    lf,
                    ls,
                    bw,
                    lcr,
                    BorderStyle::Solid,
                    0.0,
                );
            }
        } else if node.icon.is_none() {
            let is_box = matches!(
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
            if is_box {
                let shadow_poly = rounded_rect_polygon(tl.x + 3.0, tl.y + 4.0, w, h, corner_r);
                self.fill_polygon(&shadow_poly, [15, 23, 42, (28.0 * opacity) as u8]);
            }

            self.render_shape_geometry(
                &node.shape,
                tl.x,
                tl.y,
                w,
                h,
                fill,
                stroke,
                bw,
                corner_r,
                bs,
                dash_offset,
            );

            match &node.shape {
                ShapeKind::BrowserWindow => {
                    let hh = (22.0 * vp.zoom).clamp(14.0, 28.0);
                    self.fill_rect_aa(
                        tl.x as i32 + bw,
                        tl.y as i32 + bw,
                        br.x as i32 - bw,
                        (tl.y + hh) as i32,
                        [212, 218, 232, 255],
                    );
                    self.draw_thick_line_aa(
                        Point::new(tl.x, tl.y + hh),
                        Point::new(br.x, tl.y + hh),
                        stroke,
                        bw,
                    );
                    for (i, c) in [
                        [239, 68, 68, 255],
                        [245, 158, 11, 255],
                        [34, 197, 94, 255],
                    ]
                    .iter()
                    .enumerate()
                    {
                        self.fill_circle_aa(
                            (tl.x + 10.0 + i as f32 * 10.0) as i32,
                            (tl.y + hh * 0.5) as i32,
                            3,
                            *c,
                        );
                    }
                }
                ShapeKind::Section => {
                    let hh = (24.0 * vp.zoom).clamp(16.0, 32.0);
                    self.fill_rect_aa(
                        tl.x as i32 + bw,
                        tl.y as i32 + bw,
                        br.x as i32 - bw,
                        (tl.y + hh) as i32,
                        [185, 198, 224, (150.0 * opacity) as u8],
                    );
                    let fpx = (12.5 * vp.zoom).clamp(10.0, 22.0);
                    self.draw_text_aa(tl.x + 10.0, tl.y + hh * 0.5, &node.title, fpx, text_col, false);
                }
                ShapeKind::FolderTree => {
                    let hh = (24.0 * vp.zoom).clamp(16.0, 30.0);
                    self.fill_rect_aa(
                        tl.x as i32 + bw,
                        tl.y as i32 + bw,
                        br.x as i32 - bw,
                        (tl.y + hh) as i32,
                        [218, 224, 236, 255],
                    );
                    let fpx = (12.0 * vp.zoom).clamp(10.0, 20.0);
                    self.draw_text_aa(tl.x + 8.0, tl.y + hh * 0.5, &node.title, fpx, text_col, false);
                    let row_h = (20.0 * vp.zoom).clamp(14.0, 26.0);
                    let mut y = tl.y + hh + 4.0;
                    let mut hidden_depth: Option<u8> = None;
                    for item in &node.tree_items {
                        if let Some(hd) = hidden_depth {
                            if item.depth > hd {
                                continue;
                            } else {
                                hidden_depth = None;
                            }
                        }
                        if y + row_h > br.y - 2.0 {
                            break;
                        }
                        let indent = item.depth as f32 * 14.0 * vp.zoom;
                        let prefix = if item.is_folder {
                            if item.expanded { "v [D] " } else { "> [D] " }
                        } else {
                            "  [F] "
                        };
                        self.draw_text_aa(
                            tl.x + 8.0 + indent,
                            y + row_h * 0.5,
                            &format!("{prefix}{}", item.label),
                            (11.5 * vp.zoom).clamp(9.0, 18.0),
                            text_col,
                            false,
                        );
                        if item.is_folder && !item.expanded {
                            hidden_depth = Some(item.depth);
                        }
                        y += row_h;
                    }
                }
                ShapeKind::Collapsible => {
                    let hh = (30.0 * vp.zoom).clamp(20.0, 36.0);
                    self.fill_rect_aa(
                        tl.x as i32 + bw,
                        tl.y as i32 + bw,
                        br.x as i32 - bw,
                        (tl.y + hh) as i32,
                        [220, 226, 238, 255],
                    );
                    let chev = if node.collapsed { ">" } else { "v" };
                    let fpx = (12.5 * vp.zoom).clamp(10.0, 20.0);
                    self.draw_text_aa(
                        tl.x + 10.0,
                        tl.y + hh * 0.5,
                        &format!("{chev} {}", node.title),
                        fpx,
                        text_col,
                        false,
                    );
                    if !node.collapsed && !node.body.is_empty() {
                        self.draw_text_aa(
                            tl.x + 10.0,
                            tl.y + hh + 14.0,
                            node.body.lines().next().unwrap_or(""),
                            (11.0 * vp.zoom).clamp(9.0, 18.0),
                            text_col,
                            false,
                        );
                    }
                }
                ShapeKind::ImageNode => {
                    self.fill_rect_aa(
                        tl.x as i32 + 6,
                        tl.y as i32 + 6,
                        br.x as i32 - 6,
                        br.y as i32 - 6,
                        [230, 236, 245, 255],
                    );
                    self.draw_text_aa(
                        cx,
                        cy,
                        &format!("[IMG] {}", node.title),
                        (12.0 * vp.zoom).clamp(10.0, 20.0),
                        [70, 82, 104, 255],
                        true,
                    );
                }
                _ => {}
            }
        }

        if !matches!(
            node.shape,
            ShapeKind::Section
                | ShapeKind::FolderTree
                | ShapeKind::Collapsible
                | ShapeKind::ImageNode
        ) {
            let font_px = (node.text_style.font_size * vp.zoom).clamp(10.0, 32.0);
            if let Some(icon) = node.icon {
                let has_body = !node.body.trim().is_empty();
                let icon_sz = (w.min(h * if has_body { 0.50 } else { 0.68 })).clamp(22.0, 72.0);
                let iy = tl.y + (4.0 * vp.zoom).clamp(2.0, 8.0);
                let ix = cx - icon_sz * 0.5;
                let badge_col = raster_icon_accent(icon);
                let badge_poly = rounded_rect_polygon(ix, iy, icon_sz, icon_sz, (icon_sz * 0.22).clamp(3.0, 9.0));
                self.fill_polygon(&badge_poly, badge_col);
                let mut badge_border = badge_poly;
                if let Some(&first) = badge_border.first() {
                    badge_border.push(first);
                }
                self.draw_polyline_aa(&badge_border, [0, 0, 0, 55], 1);
                self.draw_symbol_icon_glyph(ix, iy, icon_sz, icon);
                let title_y = iy + icon_sz + font_px * 0.68 + 3.0;
                self.draw_text_aa(cx, title_y, &node.title, font_px, text_col, true);
                if has_body {
                    let body_px = (font_px * 0.80).clamp(9.0, 20.0);
                    let mut by = title_y + body_px + 4.0;
                    for line in node.body.lines() {
                        if by > br.y - 6.0 {
                            break;
                        }
                        self.draw_text_aa(cx, by, line, body_px, text_col, true);
                        by += body_px + 3.0;
                    }
                }
            } else if !node.body.trim().is_empty() {
                let title_y = tl.y + (16.0 * vp.zoom).clamp(12.0, 26.0);
                self.draw_text_aa(cx, title_y, &node.title, font_px, text_col, true);
                let body_px = (font_px * 0.82).clamp(9.0, 22.0);
                let mut by = title_y + body_px + 6.0;
                for line in node.body.lines() {
                    if by > br.y - 6.0 {
                        break;
                    }
                    self.draw_text_aa(cx, by, line, body_px, text_col, true);
                    by += body_px + 4.0;
                }
            } else {
                let ty = if matches!(node.shape, ShapeKind::Actor) {
                    br.y - 8.0
                } else {
                    cy
                };
                self.draw_text_aa(cx, ty, &node.title, font_px, text_col, true);
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn render_shape_geometry(
        &mut self,
        shape: &ShapeKind,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        fill: [u8; 4],
        stroke: [u8; 4],
        bw: i32,
        corner_r: f32,
        bs: BorderStyle,
        dash_offset: f32,
    ) {
        let x2 = x + w;
        let y2 = y + h;
        let cx = x + w * 0.5;
        let cy = y + h * 0.5;
        match shape {
            ShapeKind::Diamond | ShapeKind::GatewayXor | ShapeKind::GatewayParallel => {
                let pts = vec![
                    Point::new(cx, y),
                    Point::new(x2, cy),
                    Point::new(cx, y2),
                    Point::new(x, cy),
                ];
                self.fill_polygon(&pts, fill);
                let mut closed = pts;
                closed.push(closed[0]);
                self.draw_styled_polyline(&closed, stroke, bw, bs, dash_offset);
                if matches!(shape, ShapeKind::GatewayXor) {
                    let d = w.min(h) * 0.16;
                    self.draw_thick_line_aa(Point::new(cx - d, cy - d), Point::new(cx + d, cy + d), stroke, bw);
                    self.draw_thick_line_aa(Point::new(cx + d, cy - d), Point::new(cx - d, cy + d), stroke, bw);
                } else if matches!(shape, ShapeKind::GatewayParallel) {
                    let d = w.min(h) * 0.18;
                    self.draw_thick_line_aa(Point::new(cx - d, cy), Point::new(cx + d, cy), stroke, bw);
                    self.draw_thick_line_aa(Point::new(cx, cy - d), Point::new(cx, cy + d), stroke, bw);
                }
            }
            ShapeKind::Ellipse | ShapeKind::TimerEvent => {
                let segs = 32;
                let rx = w * 0.5;
                let ry = h * 0.5;
                let mut pts = Vec::with_capacity(segs + 1);
                for i in 0..=segs {
                    let a = (i as f32 / segs as f32) * std::f32::consts::TAU;
                    pts.push(Point::new(cx + a.cos() * rx, cy + a.sin() * ry));
                }
                self.fill_polygon(&pts[..segs], fill);
                self.draw_styled_polyline(&pts, stroke, bw, bs, dash_offset);
                if matches!(shape, ShapeKind::TimerEvent) {
                    self.draw_thick_line_aa(Point::new(cx, cy), Point::new(cx, cy - ry * 0.55), stroke, bw);
                    self.draw_thick_line_aa(Point::new(cx, cy), Point::new(cx + rx * 0.4, cy), stroke, bw);
                }
            }
            ShapeKind::Hexagon => {
                let inset = w * 0.18;
                let pts = vec![
                    Point::new(x + inset, y),
                    Point::new(x2 - inset, y),
                    Point::new(x2, cy),
                    Point::new(x2 - inset, y2),
                    Point::new(x + inset, y2),
                    Point::new(x, cy),
                ];
                self.fill_polygon(&pts, fill);
                let mut closed = pts;
                closed.push(closed[0]);
                self.draw_styled_polyline(&closed, stroke, bw, bs, dash_offset);
            }
            ShapeKind::Parallelogram => {
                let sk = w * 0.16;
                let pts = vec![
                    Point::new(x + sk, y),
                    Point::new(x2, y),
                    Point::new(x2 - sk, y2),
                    Point::new(x, y2),
                ];
                self.fill_polygon(&pts, fill);
                let mut closed = pts;
                closed.push(closed[0]);
                self.draw_styled_polyline(&closed, stroke, bw, bs, dash_offset);
            }
            ShapeKind::ManualInput => {
                let slope = (h * 0.24).clamp(8.0, 22.0);
                let pts = vec![
                    Point::new(x, y + slope),
                    Point::new(x2, y),
                    Point::new(x2, y2),
                    Point::new(x, y2),
                ];
                self.fill_polygon(&pts, fill);
                let mut closed = pts;
                closed.push(closed[0]);
                self.draw_styled_polyline(&closed, stroke, bw, bs, dash_offset);
            }
            ShapeKind::OffPage => {
                let tip = (h * 0.28).clamp(10.0, 24.0);
                let pts = vec![
                    Point::new(x, y),
                    Point::new(x2, y),
                    Point::new(x2, y2 - tip),
                    Point::new(cx, y2),
                    Point::new(x, y2 - tip),
                ];
                self.fill_polygon(&pts, fill);
                let mut closed = pts;
                closed.push(closed[0]);
                self.draw_styled_polyline(&closed, stroke, bw, bs, dash_offset);
            }
            ShapeKind::Triangle => {
                let pts = vec![Point::new(cx, y), Point::new(x2, y2), Point::new(x, y2)];
                self.fill_polygon(&pts, fill);
                let mut closed = pts;
                closed.push(closed[0]);
                self.draw_styled_polyline(&closed, stroke, bw, bs, dash_offset);
            }
            ShapeKind::Delay => {
                let r = h * 0.5;
                let dcx = (x2 - r).max(x + 8.0);
                let mut pts = vec![Point::new(x, y), Point::new(dcx, y)];
                for i in 1..=16 {
                    let a = -std::f32::consts::FRAC_PI_2 + (i as f32 / 16.0) * std::f32::consts::PI;
                    pts.push(Point::new(dcx + a.cos() * r, cy + a.sin() * r));
                }
                pts.push(Point::new(x, y2));
                self.fill_polygon(&pts, fill);
                let mut closed = pts;
                closed.push(closed[0]);
                self.draw_styled_polyline(&closed, stroke, bw, bs, dash_offset);
            }
            ShapeKind::Shield => {
                let pts = vec![
                    Point::new(cx, y),
                    Point::new(x2, y + h * 0.18),
                    Point::new(x2 - w * 0.12, y2 - h * 0.24),
                    Point::new(cx, y2),
                    Point::new(x + w * 0.12, y2 - h * 0.24),
                    Point::new(x, y + h * 0.18),
                ];
                self.fill_polygon(&pts, fill);
                let mut closed = pts;
                closed.push(closed[0]);
                self.draw_styled_polyline(&closed, stroke, bw, bs, dash_offset);
            }
            ShapeKind::Document => {
                let wave = (h * 0.14).clamp(6.0, 18.0);
                let pts = vec![
                    Point::new(x, y),
                    Point::new(x2, y),
                    Point::new(x2, y2 - wave),
                    Point::new(x + w * 0.72, y2 - wave * 1.4),
                    Point::new(x + w * 0.3, y2),
                    Point::new(x, y2 - wave * 0.5),
                ];
                self.fill_polygon(&pts, fill);
                let mut closed = pts;
                closed.push(closed[0]);
                self.draw_styled_polyline(&closed, stroke, bw, bs, dash_offset);
            }
            ShapeKind::Note => {
                let fold = (w.min(h) * 0.2).clamp(10.0, 22.0);
                let pts = vec![
                    Point::new(x, y),
                    Point::new(x2 - fold, y),
                    Point::new(x2, y + fold),
                    Point::new(x2, y2),
                    Point::new(x, y2),
                ];
                self.fill_polygon(&pts, fill);
                let mut closed = pts;
                closed.push(closed[0]);
                self.draw_styled_polyline(&closed, stroke, bw, bs, dash_offset);
                self.draw_thick_line_aa(Point::new(x2 - fold, y), Point::new(x2 - fold, y + fold), stroke, bw);
                self.draw_thick_line_aa(Point::new(x2 - fold, y + fold), Point::new(x2, y + fold), stroke, bw);
            }
            ShapeKind::Cylinder => {
                let cap_h = (h * 0.18).clamp(8.0, 22.0);
                self.fill_rect_aa(x as i32, (y + cap_h * 0.5) as i32, x2 as i32, (y2 - cap_h * 0.5) as i32, fill);
                let rx = w * 0.5;
                let ry = cap_h * 0.5;
                let mut top = Vec::with_capacity(25);
                let mut bot = Vec::with_capacity(13);
                for i in 0..=24 {
                    let a = (i as f32 / 24.0) * std::f32::consts::TAU;
                    top.push(Point::new(cx + a.cos() * rx, y + ry + a.sin() * ry));
                }
                for i in 0..=12 {
                    let a = (i as f32 / 12.0) * std::f32::consts::PI;
                    bot.push(Point::new(cx + a.cos() * rx, y2 - ry + a.sin() * ry));
                }
                self.fill_polygon(&top[..24], fill);
                self.fill_polygon(&bot, fill);
                self.draw_styled_polyline(&top, stroke, bw, bs, dash_offset);
                self.draw_styled_polyline(&bot, stroke, bw, bs, dash_offset);
                self.draw_thick_line_aa(Point::new(x, y + ry), Point::new(x, y2 - ry), stroke, bw);
                self.draw_thick_line_aa(Point::new(x2, y + ry), Point::new(x2, y2 - ry), stroke, bw);
            }
            ShapeKind::Cloud => {
                let r1 = (h * 0.34) as i32;
                let r2 = (h * 0.28) as i32;
                self.fill_circle_aa((cx - w * 0.22) as i32, (cy + 4.0) as i32, r2, fill);
                self.fill_circle_aa((cx + w * 0.22) as i32, (cy + 4.0) as i32, r2, fill);
                self.fill_circle_aa(cx as i32, (cy - 4.0) as i32, r1, fill);
                self.stroke_circle_aa((cx - w * 0.22) as i32, (cy + 4.0) as i32, r2, stroke, bw);
                self.stroke_circle_aa((cx + w * 0.22) as i32, (cy + 4.0) as i32, r2, stroke, bw);
                self.stroke_circle_aa(cx as i32, (cy - 4.0) as i32, r1, stroke, bw);
            }
            ShapeKind::Actor => {
                let hr = (h * 0.14).clamp(6.0, 18.0);
                let hc = Point::new(cx, y + hr + 6.0);
                self.fill_circle_aa(hc.x as i32, hc.y as i32, hr as i32, fill);
                self.stroke_circle_aa(hc.x as i32, hc.y as i32, hr as i32, stroke, bw);
                let neck = hc.y + hr;
                let hip = y2 - h * 0.34;
                self.draw_thick_line_aa(Point::new(cx, neck), Point::new(cx, hip), stroke, bw);
                let arm_y = neck + (hip - neck) * 0.32;
                let arm_w = w * 0.32;
                self.draw_thick_line_aa(Point::new(cx - arm_w, arm_y), Point::new(cx + arm_w, arm_y), stroke, bw);
                self.draw_thick_line_aa(Point::new(cx, hip), Point::new(cx - arm_w * 0.85, y2 - 18.0), stroke, bw);
                self.draw_thick_line_aa(Point::new(cx, hip), Point::new(cx + arm_w * 0.85, y2 - 18.0), stroke, bw);
            }
            ShapeKind::Subprocess => {
                self.fill_rect_aa(x as i32, y as i32, x2 as i32, y2 as i32, fill);
                let rect_pts = [
                    Point::new(x, y),
                    Point::new(x2, y),
                    Point::new(x2, y2),
                    Point::new(x, y2),
                    Point::new(x, y),
                ];
                self.draw_styled_polyline(&rect_pts, stroke, bw, bs, dash_offset);
                let bar = (w * 0.12).clamp(8.0, 20.0);
                self.draw_thick_line_aa(Point::new(x + bar, y), Point::new(x + bar, y2), stroke, bw);
                self.draw_thick_line_aa(Point::new(x2 - bar, y), Point::new(x2 - bar, y2), stroke, bw);
            }
            ShapeKind::Package => {
                let tab_h = (h * 0.22).clamp(12.0, 22.0);
                let tab_w = (w * 0.42).clamp(32.0, 120.0);
                self.fill_rect_aa(x as i32, y as i32, (x + tab_w) as i32, (y + tab_h) as i32, [214, 222, 240, 255]);
                self.stroke_rect_aa(x as i32, y as i32, (x + tab_w) as i32, (y + tab_h) as i32, stroke, bw);
                self.fill_rect_aa(x as i32, (y + tab_h) as i32, x2 as i32, y2 as i32, fill);
                self.stroke_rect_aa(x as i32, (y + tab_h) as i32, x2 as i32, y2 as i32, stroke, bw);
            }
            ShapeKind::Component => {
                let nw = 12.0;
                self.fill_rect_aa((x + nw * 0.5) as i32, y as i32, x2 as i32, y2 as i32, fill);
                self.stroke_rect_aa((x + nw * 0.5) as i32, y as i32, x2 as i32, y2 as i32, stroke, bw);
                for frac in [0.28, 0.68] {
                    let ny = y + h * frac - 5.0;
                    self.fill_rect_aa(x as i32, ny as i32, (x + nw) as i32, (ny + 10.0) as i32, fill);
                    self.stroke_rect_aa(x as i32, ny as i32, (x + nw) as i32, (ny + 10.0) as i32, stroke, bw);
                }
            }
            ShapeKind::Interface => {
                let r = (h * 0.28).clamp(10.0, 24.0);
                let icx = x + r + 8.0;
                self.fill_circle_aa(icx as i32, cy as i32, r as i32, fill);
                self.stroke_circle_aa(icx as i32, cy as i32, r as i32, stroke, bw);
                self.draw_thick_line_aa(Point::new(icx + r, cy), Point::new(x2 - 4.0, cy), stroke, bw);
            }
            ShapeKind::Server => {
                self.fill_rect_aa(x as i32, y as i32, x2 as i32, y2 as i32, fill);
                self.stroke_rect_aa(x as i32, y as i32, x2 as i32, y2 as i32, stroke, bw);
                for frac in [0.33, 0.66] {
                    self.draw_thick_line_aa(Point::new(x, y + h * frac), Point::new(x2, y + h * frac), stroke, bw);
                }
                for frac in [0.16, 0.5, 0.83] {
                    self.fill_circle_aa((x2 - 12.0) as i32, (y + h * frac) as i32, 3, [16, 185, 129, 255]);
                }
            }
            ShapeKind::Firewall => {
                self.fill_rect_aa(x as i32, y as i32, x2 as i32, y2 as i32, fill);
                self.stroke_rect_aa(x as i32, y as i32, x2 as i32, y2 as i32, stroke, bw);
                for r in 1..3 {
                    let ly = y + h * (r as f32 / 3.0);
                    self.draw_thick_line_aa(Point::new(x, ly), Point::new(x2, ly), stroke, bw);
                }
            }
            ShapeKind::Queue => {
                let pts = rounded_rect_polygon(x, y, w, h, corner_r);
                self.fill_polygon(&pts, fill);
                let mut closed = pts;
                closed.push(closed[0]);
                self.draw_styled_polyline(&closed, stroke, bw, bs, dash_offset);
                for frac in [0.28, 0.52, 0.76] {
                    self.draw_thick_line_aa(Point::new(x + w * frac, y + 2.0), Point::new(x + w * frac, y2 - 2.0), stroke, bw);
                }
            }
            _ => {
                let pts = rounded_rect_polygon(x, y, w, h, corner_r);
                self.fill_polygon(&pts, fill);
                let mut closed = pts;
                closed.push(closed[0]);
                self.draw_styled_polyline(&closed, stroke, bw, bs, dash_offset);
            }
        }
    }

    fn fill_polygon(&mut self, pts: &[Point], color: [u8; 4]) {
        if pts.len() < 3 || color[3] == 0 {
            return;
        }
        let min_y = pts.iter().map(|p| p.y).fold(f32::INFINITY, f32::min).floor() as i32;
        let max_y = pts.iter().map(|p| p.y).fold(f32::NEG_INFINITY, f32::max).ceil() as i32;
        let mut xs = Vec::with_capacity(pts.len());
        for y in min_y.max(0)..=max_y.min(self.height as i32 - 1) {
            let fy = y as f32 + 0.5;
            xs.clear();
            let n = pts.len();
            for i in 0..n {
                let a = pts[i];
                let b = pts[(i + 1) % n];
                if (a.y <= fy && b.y > fy) || (b.y <= fy && a.y > fy) {
                    let t = (fy - a.y) / (b.y - a.y);
                    xs.push(a.x + t * (b.x - a.x));
                }
            }
            xs.sort_by(|a, b| a.total_cmp(b));
            for pair in xs.chunks_exact(2) {
                let x0 = pair[0].floor().max(0.0) as i32;
                let x1 = pair[1].ceil().min(self.width as f32 - 1.0) as i32;
                for x in x0..=x1 {
                    self.blend_pixel(x, y, color);
                }
            }
        }
    }

    fn fill_rect_aa(&mut self, x0: i32, y0: i32, x1: i32, y1: i32, color: [u8; 4]) {
        for y in y0.max(0)..=y1.min(self.height as i32 - 1) {
            for x in x0.max(0)..=x1.min(self.width as i32 - 1) {
                self.blend_pixel(x, y, color);
            }
        }
    }

    fn stroke_rect_aa(&mut self, x0: i32, y0: i32, x1: i32, y1: i32, color: [u8; 4], bw: i32) {
        let pts = [
            Point::new(x0 as f32, y0 as f32),
            Point::new(x1 as f32, y0 as f32),
            Point::new(x1 as f32, y1 as f32),
            Point::new(x0 as f32, y1 as f32),
            Point::new(x0 as f32, y0 as f32),
        ];
        for seg in pts.windows(2) {
            self.draw_thick_line_aa(seg[0], seg[1], color, bw);
        }
    }

    fn fill_circle_aa(&mut self, cx: i32, cy: i32, r: i32, color: [u8; 4]) {
        for dy in -r..=r {
            for dx in -r..=r {
                if dx * dx + dy * dy <= r * r {
                    self.blend_pixel(cx + dx, cy + dy, color);
                }
            }
        }
    }

    fn stroke_circle_aa(&mut self, cx: i32, cy: i32, r: i32, color: [u8; 4], bw: i32) {
        let segs = 28;
        let mut pts = Vec::with_capacity(segs + 1);
        for i in 0..=segs {
            let a = (i as f32 / segs as f32) * std::f32::consts::TAU;
            pts.push(Point::new(cx as f32 + a.cos() * r as f32, cy as f32 + a.sin() * r as f32));
        }
        for seg in pts.windows(2) {
            self.draw_thick_line_aa(seg[0], seg[1], color, bw);
        }
    }

    fn draw_thick_line_aa(&mut self, a: Point, b: Point, color: [u8; 4], thickness: i32) {
        let half = (thickness - 1) / 2;
        let mut x0 = a.x.round() as i32;
        let mut y0 = a.y.round() as i32;
        let x1 = b.x.round() as i32;
        let y1 = b.y.round() as i32;
        let dx = (x1 - x0).abs();
        let sx = if x0 < x1 { 1 } else { -1 };
        let dy = -(y1 - y0).abs();
        let sy = if y0 < y1 { 1 } else { -1 };
        let mut err = dx + dy;
        loop {
            for oy in -half..=half {
                for ox in -half..=half {
                    self.blend_pixel(x0 + ox, y0 + oy, color);
                }
            }
            if x0 == x1 && y0 == y1 {
                break;
            }
            let e2 = 2 * err;
            if e2 >= dy {
                err += dy;
                x0 += sx;
            }
            if e2 <= dx {
                err += dx;
                y0 += sy;
            }
        }
    }

    fn draw_dashed_polyline_aa(
        &mut self,
        pts: &[Point],
        color: [u8; 4],
        thickness: i32,
        dash: f32,
        gap: f32,
        offset: f32,
    ) {
        let period = (dash + gap).max(2.0);
        let mut acc = offset.rem_euclid(period);
        for seg in pts.windows(2) {
            let a = seg[0];
            let b = seg[1];
            let len = a.distance_to(b);
            if len < 0.1 {
                continue;
            }
            let ux = (b.x - a.x) / len;
            let uy = (b.y - a.y) / len;
            let mut pos = 0.0;
            while pos < len {
                let phase = (acc + pos).rem_euclid(period);
                if phase < dash {
                    let step = (dash - phase).min(len - pos);
                    let p0 = Point::new(a.x + ux * pos, a.y + uy * pos);
                    let p1 = Point::new(a.x + ux * (pos + step), a.y + uy * (pos + step));
                    self.draw_thick_line_aa(p0, p1, color, thickness);
                    pos += step;
                } else {
                    pos += (period - phase).min(len - pos);
                }
            }
            acc += len;
        }
    }

    fn draw_styled_polyline(
        &mut self,
        pts: &[Point],
        color: [u8; 4],
        bw: i32,
        style: BorderStyle,
        dash_offset: f32,
    ) {
        match style {
            BorderStyle::None => {}
            BorderStyle::Solid => {
                for seg in pts.windows(2) {
                    self.draw_thick_line_aa(seg[0], seg[1], color, bw);
                }
            }
            BorderStyle::Double => {
                for seg in pts.windows(2) {
                    self.draw_thick_line_aa(seg[0], seg[1], color, bw + 2);
                    self.draw_thick_line_aa(seg[0], seg[1], [255, 255, 255, 255], bw.max(1));
                }
            }
            BorderStyle::Dashed => self.draw_dashed_polyline_aa(pts, color, bw, 8.0, 5.0, dash_offset),
            BorderStyle::Dotted => self.draw_dashed_polyline_aa(pts, color, bw, 2.5, 4.0, dash_offset),
            BorderStyle::DashDot => self.draw_dashed_polyline_aa(pts, color, bw, 9.0, 4.0, dash_offset),
        }
    }

    fn draw_arrow_styled(
        &mut self,
        tip: Point,
        prev: Point,
        color: [u8; 4],
        style: EdgeArrowStyle,
        size: f32,
    ) {
        let len = tip.distance_to(prev);
        if len < 0.1 {
            return;
        }
        let dx = (tip.x - prev.x) / len;
        let dy = (tip.y - prev.y) / len;
        let px = -dy;
        let py = dx;
        match style {
            EdgeArrowStyle::None => {}
            EdgeArrowStyle::OneWay | EdgeArrowStyle::TwoWay => {
                let p1 = Point::new(tip.x - dx * size + px * size * 0.52, tip.y - dy * size + py * size * 0.52);
                let p2 = Point::new(tip.x - dx * size - px * size * 0.52, tip.y - dy * size - py * size * 0.52);
                self.fill_polygon(&[tip, p1, p2], color);
            }
            EdgeArrowStyle::Open => {
                let p1 = Point::new(tip.x - dx * size + px * size * 0.55, tip.y - dy * size + py * size * 0.55);
                let p2 = Point::new(tip.x - dx * size - px * size * 0.55, tip.y - dy * size - py * size * 0.55);
                self.draw_thick_line_aa(tip, p1, color, 2);
                self.draw_thick_line_aa(tip, p2, color, 2);
            }
            EdgeArrowStyle::Diamond => {
                let mid = Point::new(tip.x - dx * size * 0.6, tip.y - dy * size * 0.6);
                let back = Point::new(tip.x - dx * size * 1.2, tip.y - dy * size * 1.2);
                let p1 = Point::new(mid.x + px * size * 0.42, mid.y + py * size * 0.42);
                let p2 = Point::new(mid.x - px * size * 0.42, mid.y - py * size * 0.42);
                self.fill_polygon(&[tip, p1, back, p2], color);
            }
        }
    }

    fn measure_text_width(&self, text: &str, font_px: f32) -> f32 {
        let Some(font) = ui_font() else {
            return text.len() as f32 * font_px * 0.55;
        };
        let scaled = font.as_scaled(PxScale::from(font_px));
        let mut w = 0.0_f32;
        for ch in text.chars() {
            w += scaled.h_advance(scaled.glyph_id(ch));
        }
        w
    }

    fn draw_text_aa(
        &mut self,
        anchor_x: f32,
        anchor_y: f32,
        text: &str,
        font_px: f32,
        color: [u8; 4],
        centered: bool,
    ) {
        let Some(font) = ui_font() else {
            return;
        };
        let scale = PxScale::from(font_px);
        let scaled = font.as_scaled(scale);
        let total_w = self.measure_text_width(text, font_px);
        let mut cursor_x = if centered {
            anchor_x - total_w * 0.5
        } else {
            anchor_x
        };
        let baseline_y = anchor_y + (scaled.ascent() + scaled.descent()) * 0.5;
        for ch in text.chars() {
            let gid = scaled.glyph_id(ch);
            let glyph = gid.with_scale_and_position(scale, ab_glyph::point(cursor_x, baseline_y));
            if let Some(outlined) = font.outline_glyph(glyph) {
                let bb = outlined.px_bounds();
                outlined.draw(|gx, gy, cov| {
                    let alpha = (cov.clamp(0.0, 1.0) * color[3] as f32).round() as u8;
                    if alpha > 0 {
                        self.blend_pixel(
                            bb.min.x as i32 + gx as i32,
                            bb.min.y as i32 + gy as i32,
                            [color[0], color[1], color[2], alpha],
                        );
                    }
                });
            }
            cursor_x += scaled.h_advance(gid);
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn draw_text_along_path_aa(
        &mut self,
        screen_pts: &[Point],
        text: &str,
        font_px: f32,
        t_pos: f32,
        offset_y: f32,
        color: [u8; 4],
        bg_color: [u8; 4],
        show_bg: bool,
    ) {
        let Some(font) = ui_font() else {
            return;
        };
        if screen_pts.len() < 2 || text.is_empty() {
            return;
        }
        let mut path = screen_pts.to_vec();
        let first = path[0];
        let last = *path.last().unwrap();
        let reversed =
            last.x < first.x - 1.0 || ((last.x - first.x).abs() <= 1.0 && last.y > first.y);
        if reversed {
            path.reverse();
        }
        let eff_t = if reversed { 1.0 - t_pos } else { t_pos };
        let total_len: f32 = path.windows(2).map(|w| w[0].distance_to(w[1])).sum();
        if total_len < 12.0 {
            return;
        }
        let sample_at = |dist: f32| -> (Point, f32) {
            let mut acc = 0.0_f32;
            for w in path.windows(2) {
                let len = w[0].distance_to(w[1]);
                if len < 1e-4 {
                    continue;
                }
                if acc + len >= dist {
                    let frac = ((dist - acc) / len).clamp(0.0, 1.0);
                    let pos = w[0].lerp(w[1], frac);
                    let ang = (w[1].y - w[0].y).atan2(w[1].x - w[0].x);
                    return (pos, ang);
                }
                acc += len;
            }
            let p1 = *path.last().unwrap();
            let p0 = path[path.len().saturating_sub(2)];
            (p1, (p1.y - p0.y).atan2(p1.x - p0.x))
        };

        let scale = PxScale::from(font_px);
        let scaled = font.as_scaled(scale);
        let total_w = self.measure_text_width(text, font_px);
        let start_dist =
            (eff_t * total_len - total_w * 0.5).clamp(6.0, (total_len - total_w - 6.0).max(0.0));
        let lift = font_px * 0.65 + 5.0 - offset_y;

        if show_bg {
            let steps = ((total_w / 6.0).ceil() as usize).clamp(4, 36);
            let mut rpts = Vec::with_capacity(steps + 1);
            for i in 0..=steps {
                let d = (start_dist - 4.0 + (total_w + 8.0) * (i as f32 / steps as f32))
                    .clamp(0.0, total_len);
                let (p, ang) = sample_at(d);
                rpts.push(Point::new(p.x + ang.sin() * lift, p.y - ang.cos() * lift));
            }
            let thick = (font_px + 4.0).round() as i32;
            for w in rpts.windows(2) {
                self.draw_thick_line_aa(w[0], w[1], bg_color, thick);
            }
        }

        let baseline_y = (scaled.ascent() + scaled.descent()) * 0.5;
        let mut cursor = 0.0_f32;
        for ch in text.chars() {
            let gid = scaled.glyph_id(ch);
            let adv = scaled.h_advance(gid);
            let d = (start_dist + cursor + adv * 0.5).clamp(0.0, total_len);
            let (p, ang) = sample_at(d);
            let cx = p.x + ang.sin() * lift;
            let cy = p.y - ang.cos() * lift;
            let (sin_a, cos_a) = ang.sin_cos();
            let glyph = gid.with_scale_and_position(scale, ab_glyph::point(-adv * 0.5, baseline_y));
            if let Some(outlined) = font.outline_glyph(glyph) {
                let bb = outlined.px_bounds();
                outlined.draw(|gx, gy, cov| {
                    let alpha = (cov.clamp(0.0, 1.0) * color[3] as f32).round() as u8;
                    if alpha > 0 {
                        let lx = bb.min.x + gx as f32;
                        let ly = bb.min.y + gy as f32;
                        let rx = cx + lx * cos_a - ly * sin_a;
                        let ry = cy + lx * sin_a + ly * cos_a;
                        self.blend_pixel(
                            rx.round() as i32,
                            ry.round() as i32,
                            [color[0], color[1], color[2], alpha],
                        );
                    }
                });
            }
            cursor += adv;
        }
    }

    fn draw_polyline_aa(&mut self, pts: &[Point], color: [u8; 4], width: i32) {
        for w in pts.windows(2) {
            self.draw_thick_line_aa(w[0], w[1], color, width);
        }
    }

    fn draw_symbol_icon_glyph(&mut self, ix: f32, iy: f32, side: f32, icon: rustflow_core::SymbolIcon) {
        use rustflow_core::SymbolIcon;
        let cx = ix + side * 0.5;
        let cy = iy + side * 0.5;
        let u = side * 0.30;
        let sw = (side * 0.075).round().clamp(1.0, 4.0) as i32;
        let thin = (sw - 1).max(1);
        let white = [255, 255, 255, 255];

        let stroke_rect = |fb: &mut Self, rx: f32, ry: f32, rw: f32, rh: f32, cr: f32, w: i32| {
            let mut poly = rounded_rect_polygon(rx, ry, rw, rh, cr);
            if let Some(&first) = poly.first() {
                poly.push(first);
            }
            fb.draw_polyline_aa(&poly, white, w);
        };
        let stroke_circle = |fb: &mut Self, ccx: f32, ccy: f32, rad: f32, w: i32| {
            let segs = 20usize;
            let mut pts = Vec::with_capacity(segs + 1);
            for i in 0..=segs {
                let a = (i as f32 / segs as f32) * std::f32::consts::TAU;
                pts.push(Point::new(ccx + a.cos() * rad, ccy + a.sin() * rad));
            }
            fb.draw_polyline_aa(&pts, white, w);
        };

        match icon {
            SymbolIcon::AwsLambda => {
                self.draw_thick_line_aa(Point::new(cx - u * 0.45, cy - u * 0.9), Point::new(cx + u * 0.75, cy + u * 0.9), white, sw);
                self.draw_thick_line_aa(Point::new(cx + u * 0.05, cy - u * 0.1), Point::new(cx - u * 0.75, cy + u * 0.9), white, sw);
                self.draw_thick_line_aa(Point::new(cx - u * 0.75, cy - u * 0.9), Point::new(cx - u * 0.35, cy - u * 0.9), white, sw);
            }
            SymbolIcon::AwsS3
            | SymbolIcon::AwsGlacier
            | SymbolIcon::AwsBackup
            | SymbolIcon::GcpCloudStorage
            | SymbolIcon::AzureBlob
            | SymbolIcon::AzureDataLake => {
                let pts = vec![
                    Point::new(cx - u * 0.78, cy - u * 0.55),
                    Point::new(cx + u * 0.78, cy - u * 0.55),
                    Point::new(cx + u * 0.56, cy + u * 0.78),
                    Point::new(cx - u * 0.56, cy + u * 0.78),
                    Point::new(cx - u * 0.78, cy - u * 0.55),
                ];
                self.draw_polyline_aa(&pts, white, sw);
                self.draw_thick_line_aa(Point::new(cx - u * 0.65, cy - u * 0.15), Point::new(cx + u * 0.65, cy - u * 0.15), white, thin);
                self.draw_thick_line_aa(Point::new(cx - u * 0.45, cy - u * 0.85), Point::new(cx + u * 0.45, cy - u * 0.85), white, thin);
            }
            SymbolIcon::AwsEbs | SymbolIcon::AwsEfs | SymbolIcon::AzureFiles | SymbolIcon::AzureDisk => {
                let rw = u * 1.6;
                let rh = u * 1.25;
                stroke_rect(self, cx - rw * 0.5, cy - rh * 0.5, rw, rh, 2.0, sw);
                self.draw_thick_line_aa(Point::new(cx - rw * 0.5, cy - u * 0.1), Point::new(cx + rw * 0.5, cy - u * 0.1), white, thin);
                self.draw_thick_line_aa(Point::new(cx - rw * 0.5, cy + u * 0.3), Point::new(cx + rw * 0.5, cy + u * 0.3), white, thin);
            }
            SymbolIcon::AwsEc2
            | SymbolIcon::AwsLightsail
            | SymbolIcon::AwsBatch
            | SymbolIcon::AwsOutposts
            | SymbolIcon::AzureVm
            | SymbolIcon::AzureAppService
            | SymbolIcon::AzureSpringApps => {
                let s = u * 1.25;
                let x0 = cx - s * 0.5;
                let y0 = cy - s * 0.5;
                stroke_rect(self, x0, y0, s, s, 2.0, sw);
                for t in [-0.35_f32, 0.35] {
                    self.draw_thick_line_aa(Point::new(cx + u * t, y0 - u * 0.35), Point::new(cx + u * t, y0), white, thin);
                    self.draw_thick_line_aa(Point::new(cx + u * t, y0 + s), Point::new(cx + u * t, y0 + s + u * 0.35), white, thin);
                    self.draw_thick_line_aa(Point::new(x0 - u * 0.35, cy + u * t), Point::new(x0, cy + u * t), white, thin);
                    self.draw_thick_line_aa(Point::new(x0 + s, cy + u * t), Point::new(x0 + s + u * 0.35, cy + u * t), white, thin);
                }
            }
            SymbolIcon::AwsEcs
            | SymbolIcon::AwsEcr
            | SymbolIcon::AwsFargate
            | SymbolIcon::AwsAppRunner
            | SymbolIcon::AwsElasticBeanstalk
            | SymbolIcon::AzureContainerApps
            | SymbolIcon::AzureContainerInstances => {
                let bs = u * 0.62;
                for (dx, dy) in [(-0.42_f32, -0.35), (0.42, -0.35), (-0.42, 0.35), (0.42, 0.35)] {
                    stroke_rect(self, cx + u * dx - bs * 0.5, cy + u * dy - bs * 0.5, bs, bs, 1.5, sw);
                }
            }
            SymbolIcon::AwsRds
            | SymbolIcon::AwsAurora
            | SymbolIcon::AwsRedshift
            | SymbolIcon::AwsDocumentDb
            | SymbolIcon::AwsNeptune
            | SymbolIcon::AwsTimestream
            | SymbolIcon::AwsKeyspaces
            | SymbolIcon::AwsLakeFormation
            | SymbolIcon::AzureSql
            | SymbolIcon::AzurePostgres
            | SymbolIcon::AzureSynapse
            | SymbolIcon::GcpCloudSql
            | SymbolIcon::Postgres => {
                let rw = u * 1.45;
                let rh = u * 1.55;
                stroke_rect(self, cx - rw * 0.5, cy - rh * 0.5, rw, rh, u * 0.35, sw);
                self.draw_thick_line_aa(Point::new(cx - rw * 0.45, cy - u * 0.25), Point::new(cx + rw * 0.45, cy - u * 0.25), white, thin);
                self.draw_thick_line_aa(Point::new(cx - rw * 0.45, cy + u * 0.25), Point::new(cx + rw * 0.45, cy + u * 0.25), white, thin);
            }
            SymbolIcon::AwsDynamoDb | SymbolIcon::AzureFunctions => {
                let bolt = vec![
                    Point::new(cx + u * 0.15, cy - u * 0.92),
                    Point::new(cx - u * 0.58, cy + u * 0.08),
                    Point::new(cx + u * 0.05, cy + u * 0.08),
                    Point::new(cx - u * 0.18, cy + u * 0.92),
                    Point::new(cx + u * 0.60, cy - u * 0.12),
                    Point::new(cx - u * 0.02, cy - u * 0.12),
                ];
                self.fill_polygon(&bolt, white);
            }
            SymbolIcon::AwsApiGateway
            | SymbolIcon::AwsAppSync
            | SymbolIcon::AwsAmplify
            | SymbolIcon::AzureApiManagement
            | SymbolIcon::Api => {
                self.draw_thick_line_aa(Point::new(cx - u * 0.85, cy - u * 0.45), Point::new(cx - u * 0.35, cy - u * 0.85), white, sw);
                self.draw_thick_line_aa(Point::new(cx - u * 0.85, cy - u * 0.45), Point::new(cx - u * 0.35, cy), white, sw);
                self.draw_thick_line_aa(Point::new(cx + u * 0.85, cy + u * 0.45), Point::new(cx + u * 0.35, cy), white, sw);
                self.draw_thick_line_aa(Point::new(cx + u * 0.85, cy + u * 0.45), Point::new(cx + u * 0.35, cy + u * 0.85), white, sw);
                self.draw_thick_line_aa(Point::new(cx - u * 0.2, cy + u * 0.7), Point::new(cx + u * 0.2, cy - u * 0.7), white, sw);
            }
            SymbolIcon::AwsSqs
            | SymbolIcon::AwsSns
            | SymbolIcon::AwsEventBridge
            | SymbolIcon::AwsSes
            | SymbolIcon::AwsMq
            | SymbolIcon::AwsMsk
            | SymbolIcon::AwsFlink
            | SymbolIcon::AwsKinesis
            | SymbolIcon::AwsDataFirehose
            | SymbolIcon::AwsGlue
            | SymbolIcon::AwsEmr
            | SymbolIcon::AzureServiceBus
            | SymbolIcon::AzureEventHubs
            | SymbolIcon::AzureEventGrid
            | SymbolIcon::GcpPubSub
            | SymbolIcon::Kafka => {
                let r_sm = (u * 0.24).round().max(2.0) as i32;
                let r_mid = (u * 0.30).round().max(2.0) as i32;
                for (dx, dy) in [(-0.55_f32, -0.55), (0.55, -0.55), (-0.55, 0.55), (0.55, 0.55)] {
                    let p = Point::new(cx + u * dx, cy + u * dy);
                    self.draw_thick_line_aa(Point::new(cx, cy), p, white, thin);
                    self.fill_circle_aa(p.x.round() as i32, p.y.round() as i32, r_sm, white);
                }
                self.fill_circle_aa(cx.round() as i32, cy.round() as i32, r_mid, white);
            }
            SymbolIcon::AwsStepFunctions
            | SymbolIcon::AwsCodePipeline
            | SymbolIcon::AwsCodeBuild
            | SymbolIcon::AwsCloudFormation
            | SymbolIcon::AwsSsm
            | SymbolIcon::AwsOrganizations
            | SymbolIcon::AzureLogicApps
            | SymbolIcon::AzureDevOps => {
                let bs = u * 0.55;
                let p0 = Point::new(cx - u * 0.52, cy - u * 0.45);
                let p1 = Point::new(cx + u * 0.52, cy);
                let p2 = Point::new(cx - u * 0.52, cy + u * 0.45);
                for p in [p0, p1, p2] {
                    stroke_rect(self, p.x - bs * 0.5, p.y - bs * 0.5, bs, bs, 1.5, sw);
                }
                self.draw_thick_line_aa(Point::new(p0.x + bs * 0.5, p0.y), Point::new(p1.x - bs * 0.5, p1.y), white, thin);
                self.draw_thick_line_aa(Point::new(p1.x - bs * 0.5, p1.y), Point::new(p2.x + bs * 0.5, p2.y), white, thin);
            }
            SymbolIcon::AwsCloudFront
            | SymbolIcon::AwsRoute53
            | SymbolIcon::AwsGlobalAccelerator
            | SymbolIcon::AzureFrontDoor
            | SymbolIcon::AzureDns
            | SymbolIcon::Cdn
            | SymbolIcon::Globe => {
                let rad = u * 0.88;
                stroke_circle(self, cx, cy, rad, sw);
                self.draw_thick_line_aa(Point::new(cx - rad, cy), Point::new(cx + rad, cy), white, thin);
                self.draw_thick_line_aa(Point::new(cx, cy - rad), Point::new(cx, cy + rad), white, thin);
                stroke_circle(self, cx, cy, rad * 0.48, thin);
            }
            SymbolIcon::AwsVpc
            | SymbolIcon::AwsDirectConnect
            | SymbolIcon::AwsTransitGateway
            | SymbolIcon::AwsPrivateLink
            | SymbolIcon::AwsNatGateway
            | SymbolIcon::AzureVnet => {
                let rw = u * 1.75;
                let rh = u * 1.35;
                stroke_rect(self, cx - rw * 0.5, cy - rh * 0.5, rw, rh, 2.5, sw);
                self.draw_thick_line_aa(Point::new(cx, cy - rh * 0.5), Point::new(cx, cy + rh * 0.5), white, thin);
                let r = (u * 0.18).round().max(2.0) as i32;
                self.fill_circle_aa((cx - u * 0.42).round() as i32, cy.round() as i32, r, white);
                self.fill_circle_aa((cx + u * 0.42).round() as i32, cy.round() as i32, r, white);
            }
            SymbolIcon::AwsWaf | SymbolIcon::AwsShield | SymbolIcon::AwsGuardDuty | SymbolIcon::AzureFirewall => {
                let sh = vec![
                    Point::new(cx, cy - u * 0.88),
                    Point::new(cx + u * 0.72, cy - u * 0.52),
                    Point::new(cx + u * 0.58, cy + u * 0.42),
                    Point::new(cx, cy + u * 0.88),
                    Point::new(cx - u * 0.58, cy + u * 0.42),
                    Point::new(cx - u * 0.72, cy - u * 0.52),
                    Point::new(cx, cy - u * 0.88),
                ];
                self.draw_polyline_aa(&sh, white, sw);
                self.draw_thick_line_aa(Point::new(cx - u * 0.35, cy), Point::new(cx + u * 0.35, cy), white, thin);
            }
            SymbolIcon::GcpCloudRun => {
                let pts = vec![
                    Point::new(cx - u * 0.55, cy - u * 0.78),
                    Point::new(cx + u * 0.75, cy),
                    Point::new(cx - u * 0.55, cy + u * 0.78),
                    Point::new(cx - u * 0.12, cy),
                ];
                self.fill_polygon(&pts, white);
            }
            SymbolIcon::AwsEks | SymbolIcon::GcpGke | SymbolIcon::AzureAks | SymbolIcon::Kubernetes => {
                let n = 7usize;
                let rad = u * 0.85;
                let mut pts = Vec::with_capacity(n + 1);
                for i in 0..n {
                    let a = (i as f32 / n as f32) * std::f32::consts::TAU - std::f32::consts::FRAC_PI_2;
                    let p = Point::new(cx + a.cos() * rad, cy + a.sin() * rad);
                    pts.push(p);
                    self.draw_thick_line_aa(Point::new(cx, cy), p, white, thin);
                }
                pts.push(pts[0]);
                self.draw_polyline_aa(&pts, white, sw);
                self.fill_circle_aa(cx.round() as i32, cy.round() as i32, (u * 0.22).round().max(2.0) as i32, white);
            }
            SymbolIcon::AwsCloudWatch
            | SymbolIcon::AwsCloudTrail
            | SymbolIcon::AwsConfig
            | SymbolIcon::AwsXRay
            | SymbolIcon::AwsAthena
            | SymbolIcon::AwsOpenSearch
            | SymbolIcon::AwsQuickSight
            | SymbolIcon::AzureMonitor
            | SymbolIcon::GcpBigQuery
            | SymbolIcon::Monitor => {
                for (i, bh) in [0.45_f32, 0.85, 0.62].iter().enumerate() {
                    let bx = cx - u * 0.55 + i as f32 * u * 0.55;
                    self.fill_rect_aa(
                        (bx - u * 0.18).round() as i32,
                        (cy + u * 0.65 - u * 1.3 * bh).round() as i32,
                        (bx + u * 0.18).round() as i32,
                        (cy + u * 0.65).round() as i32,
                        white,
                    );
                }
            }
            SymbolIcon::AzureCosmosDb => {
                self.fill_circle_aa(cx.round() as i32, cy.round() as i32, (u * 0.34).round().max(2.0) as i32, white);
                stroke_circle(self, cx, cy, u * 0.82, sw);
                self.draw_thick_line_aa(Point::new(cx - u * 0.85, cy - u * 0.35), Point::new(cx + u * 0.85, cy + u * 0.35), white, thin);
            }
            SymbolIcon::AwsKms | SymbolIcon::AwsSecretsManager | SymbolIcon::AzureKeyVault | SymbolIcon::Key => {
                let hx = cx - u * 0.35;
                stroke_circle(self, hx, cy, u * 0.42, sw);
                self.draw_thick_line_aa(Point::new(hx + u * 0.42, cy), Point::new(cx + u * 0.88, cy), white, sw);
                self.draw_thick_line_aa(Point::new(cx + u * 0.55, cy), Point::new(cx + u * 0.55, cy + u * 0.35), white, sw);
                self.draw_thick_line_aa(Point::new(cx + u * 0.82, cy), Point::new(cx + u * 0.82, cy + u * 0.35), white, sw);
            }
            SymbolIcon::Docker => {
                let bs = u * 0.21;
                for (gx, gy) in [(-0.55_f32, -0.25), (0.0, -0.25), (0.55, -0.25), (-0.28, -0.72), (0.28, -0.72)] {
                    self.fill_rect_aa(
                        (cx + u * gx - bs).round() as i32,
                        (cy + u * gy - bs).round() as i32,
                        (cx + u * gx + bs).round() as i32,
                        (cy + u * gy + bs).round() as i32,
                        white,
                    );
                }
                let hull = rounded_rect_polygon(cx - u * 0.88, cy + u * 0.1, u * 1.76, u * 0.58, u * 0.22);
                self.fill_polygon(&hull, white);
            }
            SymbolIcon::AwsElastiCache | SymbolIcon::AwsMemoryDb | SymbolIcon::AzureRedis | SymbolIcon::Redis => {
                for dy in [-0.45_f32, 0.0, 0.45] {
                    let pts = vec![
                        Point::new(cx, cy + u * (dy - 0.25)),
                        Point::new(cx + u * 0.82, cy + u * dy),
                        Point::new(cx, cy + u * (dy + 0.25)),
                        Point::new(cx - u * 0.82, cy + u * dy),
                        Point::new(cx, cy + u * (dy - 0.25)),
                    ];
                    self.draw_polyline_aa(&pts, white, sw);
                }
            }
            SymbolIcon::Nginx => {
                self.draw_thick_line_aa(Point::new(cx - u * 0.58, cy + u * 0.68), Point::new(cx - u * 0.58, cy - u * 0.68), white, sw);
                self.draw_thick_line_aa(Point::new(cx - u * 0.58, cy - u * 0.68), Point::new(cx + u * 0.58, cy + u * 0.68), white, sw);
                self.draw_thick_line_aa(Point::new(cx + u * 0.58, cy + u * 0.68), Point::new(cx + u * 0.58, cy - u * 0.68), white, sw);
            }
            SymbolIcon::AwsElb | SymbolIcon::AzureAppGateway | SymbolIcon::AzureLoadBalancer | SymbolIcon::LoadBalancer => {
                let left = Point::new(cx - u * 0.78, cy);
                self.fill_circle_aa(left.x.round() as i32, left.y.round() as i32, (u * 0.24).round().max(2.0) as i32, white);
                for dy in [-0.62_f32, 0.0, 0.62] {
                    let right = Point::new(cx + u * 0.72, cy + u * dy);
                    self.draw_thick_line_aa(left, right, white, thin);
                    self.fill_circle_aa(right.x.round() as i32, right.y.round() as i32, (u * 0.22).round().max(2.0) as i32, white);
                }
            }
            SymbolIcon::AwsCognito | SymbolIcon::AzureEntraId | SymbolIcon::User => {
                self.fill_circle_aa(cx.round() as i32, (cy - u * 0.38).round() as i32, (u * 0.36).round().max(2.0) as i32, white);
                let body = rounded_rect_polygon(cx - u * 0.65, cy + u * 0.12, u * 1.3, u * 0.70, u * 0.32);
                self.fill_polygon(&body, white);
            }
            SymbolIcon::Users => {
                for dx in [-0.36_f32, 0.36] {
                    let ux = cx + u * dx;
                    self.fill_circle_aa(ux.round() as i32, (cy - u * 0.32).round() as i32, (u * 0.24).round().max(2.0) as i32, white);
                    let body = rounded_rect_polygon(ux - u * 0.42, cy + u * 0.12, u * 0.84, u * 0.62, u * 0.22);
                    self.fill_polygon(&body, white);
                }
            }
            SymbolIcon::AwsIam | SymbolIcon::Lock => {
                let body = rounded_rect_polygon(cx - u * 0.62, cy - u * 0.08, u * 1.24, u * 0.86, 2.0);
                self.fill_polygon(&body, white);
                stroke_rect(self, cx - u * 0.38, cy - u * 0.72, u * 0.76, u * 0.72, u * 0.36, sw);
            }
            SymbolIcon::Webhook => {
                let p0 = Point::new(cx, cy - u * 0.62);
                let p1 = Point::new(cx - u * 0.58, cy + u * 0.48);
                let p2 = Point::new(cx + u * 0.58, cy + u * 0.48);
                self.draw_thick_line_aa(p0, p1, white, sw);
                self.draw_thick_line_aa(p1, p2, white, sw);
                self.draw_thick_line_aa(p2, p0, white, sw);
                let r = (u * 0.24).round().max(2.0) as i32;
                for p in [p0, p1, p2] {
                    self.fill_circle_aa(p.x.round() as i32, p.y.round() as i32, r, white);
                }
            }
            SymbolIcon::Terminal => {
                self.draw_thick_line_aa(Point::new(cx - u * 0.68, cy - u * 0.45), Point::new(cx - u * 0.12, cy), white, sw);
                self.draw_thick_line_aa(Point::new(cx - u * 0.12, cy), Point::new(cx - u * 0.68, cy + u * 0.45), white, sw);
                self.draw_thick_line_aa(Point::new(cx + u * 0.05, cy + u * 0.45), Point::new(cx + u * 0.72, cy + u * 0.45), white, sw);
            }
            SymbolIcon::AwsBedrock
            | SymbolIcon::AwsSageMaker
            | SymbolIcon::AwsRekognition
            | SymbolIcon::AwsTextract
            | SymbolIcon::AwsPolly
            | SymbolIcon::AwsQ
            | SymbolIcon::AzureOpenAi
            | SymbolIcon::AzureAiSearch
            | SymbolIcon::AiModel => {
                let star = vec![
                    Point::new(cx, cy - u * 0.88),
                    Point::new(cx + u * 0.26, cy - u * 0.26),
                    Point::new(cx + u * 0.88, cy),
                    Point::new(cx + u * 0.26, cy + u * 0.26),
                    Point::new(cx, cy + u * 0.88),
                    Point::new(cx - u * 0.26, cy + u * 0.26),
                    Point::new(cx - u * 0.88, cy),
                    Point::new(cx - u * 0.26, cy - u * 0.26),
                    Point::new(cx, cy - u * 0.88),
                ];
                self.draw_polyline_aa(&star, white, sw);
                self.fill_circle_aa(cx.round() as i32, cy.round() as i32, (u * 0.24).round().max(2.0) as i32, white);
            }
        }
    }
}

fn raster_icon_accent(icon: rustflow_core::SymbolIcon) -> [u8; 4] {
    use rustflow_core::SymbolIcon;
    match icon {
        SymbolIcon::AwsEc2
        | SymbolIcon::AwsLambda
        | SymbolIcon::AwsEcs
        | SymbolIcon::AwsEks
        | SymbolIcon::AwsEcr
        | SymbolIcon::AwsFargate
        | SymbolIcon::AwsAppRunner
        | SymbolIcon::AwsElasticBeanstalk
        | SymbolIcon::AwsLightsail
        | SymbolIcon::AwsBatch
        | SymbolIcon::AwsOutposts => [237, 113, 0, 255],
        SymbolIcon::AwsS3
        | SymbolIcon::AwsEbs
        | SymbolIcon::AwsEfs
        | SymbolIcon::AwsGlacier
        | SymbolIcon::AwsBackup => [63, 134, 36, 255],
        SymbolIcon::AwsRds
        | SymbolIcon::AwsDynamoDb
        | SymbolIcon::AwsAurora
        | SymbolIcon::AwsElastiCache
        | SymbolIcon::AwsRedshift
        | SymbolIcon::AwsDocumentDb
        | SymbolIcon::AwsNeptune
        | SymbolIcon::AwsTimestream
        | SymbolIcon::AwsMemoryDb
        | SymbolIcon::AwsKeyspaces => [59, 72, 204, 255],
        SymbolIcon::AwsVpc
        | SymbolIcon::AwsApiGateway
        | SymbolIcon::AwsCloudFront
        | SymbolIcon::AwsRoute53
        | SymbolIcon::AwsElb
        | SymbolIcon::AwsDirectConnect
        | SymbolIcon::AwsTransitGateway
        | SymbolIcon::AwsPrivateLink
        | SymbolIcon::AwsGlobalAccelerator
        | SymbolIcon::AwsNatGateway
        | SymbolIcon::AwsMsk
        | SymbolIcon::AwsFlink
        | SymbolIcon::AwsKinesis
        | SymbolIcon::AwsDataFirehose
        | SymbolIcon::AwsAthena
        | SymbolIcon::AwsGlue
        | SymbolIcon::AwsEmr
        | SymbolIcon::AwsOpenSearch
        | SymbolIcon::AwsQuickSight
        | SymbolIcon::AwsLakeFormation => [140, 79, 255, 255],
        SymbolIcon::AwsSqs
        | SymbolIcon::AwsSns
        | SymbolIcon::AwsEventBridge
        | SymbolIcon::AwsStepFunctions
        | SymbolIcon::AwsAppSync
        | SymbolIcon::AwsSes
        | SymbolIcon::AwsMq
        | SymbolIcon::AwsAmplify
        | SymbolIcon::AwsCloudWatch
        | SymbolIcon::AwsCloudTrail
        | SymbolIcon::AwsConfig
        | SymbolIcon::AwsOrganizations
        | SymbolIcon::AwsCodePipeline
        | SymbolIcon::AwsCodeBuild
        | SymbolIcon::AwsXRay
        | SymbolIcon::AwsSsm
        | SymbolIcon::AwsCloudFormation => [231, 21, 123, 255],
        SymbolIcon::AwsIam
        | SymbolIcon::AwsCognito
        | SymbolIcon::AwsKms
        | SymbolIcon::AwsWaf
        | SymbolIcon::AwsShield
        | SymbolIcon::AwsGuardDuty
        | SymbolIcon::AwsSecretsManager => [221, 52, 76, 255],
        SymbolIcon::AwsBedrock
        | SymbolIcon::AwsSageMaker
        | SymbolIcon::AwsRekognition
        | SymbolIcon::AwsTextract
        | SymbolIcon::AwsPolly
        | SymbolIcon::AwsQ => [1, 168, 141, 255],
        SymbolIcon::AzureVm
        | SymbolIcon::AzureAks
        | SymbolIcon::AzureFunctions
        | SymbolIcon::AzureAppService
        | SymbolIcon::AzureContainerApps
        | SymbolIcon::AzureContainerInstances
        | SymbolIcon::AzureSpringApps
        | SymbolIcon::AzureBlob
        | SymbolIcon::AzureFiles
        | SymbolIcon::AzureDisk
        | SymbolIcon::AzureDataLake
        | SymbolIcon::AzureSql
        | SymbolIcon::AzureCosmosDb
        | SymbolIcon::AzurePostgres
        | SymbolIcon::AzureRedis
        | SymbolIcon::AzureSynapse
        | SymbolIcon::AzureVnet
        | SymbolIcon::AzureFrontDoor
        | SymbolIcon::AzureAppGateway
        | SymbolIcon::AzureLoadBalancer
        | SymbolIcon::AzureDns
        | SymbolIcon::AzureFirewall
        | SymbolIcon::AzureApiManagement
        | SymbolIcon::AzureServiceBus
        | SymbolIcon::AzureEventHubs
        | SymbolIcon::AzureEventGrid
        | SymbolIcon::AzureLogicApps
        | SymbolIcon::AzureEntraId
        | SymbolIcon::AzureKeyVault
        | SymbolIcon::AzureMonitor
        | SymbolIcon::AzureDevOps
        | SymbolIcon::AzureOpenAi
        | SymbolIcon::AzureAiSearch => [0, 120, 212, 255],
        SymbolIcon::GcpCloudRun
        | SymbolIcon::GcpGke
        | SymbolIcon::GcpBigQuery
        | SymbolIcon::GcpPubSub
        | SymbolIcon::GcpCloudStorage
        | SymbolIcon::GcpCloudSql => [66, 133, 244, 255],
        SymbolIcon::Kubernetes => [50, 108, 229, 255],
        SymbolIcon::Docker => [36, 150, 237, 255],
        SymbolIcon::Postgres => [51, 103, 145, 255],
        SymbolIcon::Redis => [216, 44, 32, 255],
        SymbolIcon::Kafka => [35, 31, 32, 255],
        SymbolIcon::Nginx => [0, 150, 57, 255],
        SymbolIcon::LoadBalancer | SymbolIcon::Cdn => [79, 70, 229, 255],
        SymbolIcon::User | SymbolIcon::Users => [30, 64, 175, 255],
        SymbolIcon::Globe => [14, 116, 144, 255],
        SymbolIcon::Lock | SymbolIcon::Key => [180, 83, 9, 255],
        SymbolIcon::Api | SymbolIcon::Webhook => [15, 118, 110, 255],
        SymbolIcon::Monitor | SymbolIcon::Terminal => [51, 65, 85, 255],
        SymbolIcon::AiModel => [124, 58, 237, 255],
    }
}
