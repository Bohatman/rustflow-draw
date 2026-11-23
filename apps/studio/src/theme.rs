use egui::{Align2, Color32, CornerRadius, FontId, Frame, Margin, Painter, Pos2, Rect, Stroke, Style, Vec2, Visuals};
use rustflow_core::{ColorRgba, ShapeKind, SymbolIcon, ThemeMode};

pub const COLOR_SWATCHES: &[[u8; 3]] = &[
    [255, 255, 255],
    [241, 245, 249],
    [219, 234, 254],
    [220, 252, 231],
    [254, 249, 195],
    [254, 226, 226],
    [243, 232, 255],
    [224, 231, 255],
    [0, 0, 128],
    [37, 99, 235],
    [5, 150, 105],
    [217, 119, 6],
    [220, 38, 38],
    [124, 58, 237],
    [15, 23, 42],
    [100, 116, 139],
];

pub fn apply_theme(ctx: &egui::Context, mode: ThemeMode) {
    let mut style = Style::default();
    let mut visuals = match mode {
        ThemeMode::Dark => Visuals::dark(),
        ThemeMode::Light | ThemeMode::HighContrast => Visuals::light(),
    };

    match mode {
        ThemeMode::Light => {
            visuals.panel_fill = Color32::from_rgb(206, 206, 210);
            visuals.window_fill = Color32::from_rgb(220, 220, 224);
            visuals.faint_bg_color = Color32::from_rgb(194, 194, 200);
            visuals.extreme_bg_color = Color32::from_rgb(255, 255, 255);
            visuals.slider_trailing_fill = true;
            visuals.widgets.noninteractive.bg_fill = Color32::from_rgb(208, 208, 212);
            visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0_f32, Color32::from_rgb(148, 152, 166));
            visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0_f32, Color32::from_rgb(16, 16, 20));
            visuals.widgets.inactive.bg_fill = Color32::from_rgb(142, 150, 172);
            visuals.widgets.inactive.weak_bg_fill = Color32::from_rgb(232, 234, 240);
            visuals.widgets.inactive.bg_stroke = Stroke::new(1.2_f32, Color32::from_rgb(36, 40, 56));
            visuals.widgets.inactive.fg_stroke = Stroke::new(1.5_f32, Color32::from_rgb(14, 18, 32));
            visuals.widgets.hovered.bg_fill = Color32::from_rgb(236, 240, 255);
            visuals.widgets.hovered.bg_stroke = Stroke::new(1.5_f32, Color32::from_rgb(0, 0, 140));
            visuals.widgets.active.bg_fill = Color32::from_rgb(0, 0, 128);
            visuals.widgets.active.bg_stroke = Stroke::new(1.2_f32, Color32::from_rgb(12, 12, 16));
            visuals.widgets.active.fg_stroke = Stroke::new(1.5_f32, Color32::WHITE);
            visuals.selection.bg_fill = Color32::from_rgb(28, 72, 196);
            visuals.selection.stroke = Stroke::new(1.0_f32, Color32::WHITE);
            visuals.window_corner_radius = CornerRadius::same(1);
            visuals.widgets.inactive.corner_radius = CornerRadius::same(2);
            visuals.widgets.hovered.corner_radius = CornerRadius::same(2);
            visuals.widgets.active.corner_radius = CornerRadius::same(2);
        }
        ThemeMode::Dark => {
            visuals.panel_fill = Color32::from_rgb(28, 30, 38);
            visuals.window_fill = Color32::from_rgb(34, 37, 46);
            visuals.extreme_bg_color = Color32::from_rgb(18, 20, 26);
            visuals.slider_trailing_fill = true;
            visuals.widgets.inactive.bg_fill = Color32::from_rgb(84, 92, 114);
            visuals.widgets.inactive.weak_bg_fill = Color32::from_rgb(52, 56, 70);
            visuals.widgets.inactive.bg_stroke = Stroke::new(1.2_f32, Color32::from_rgb(140, 152, 180));
            visuals.selection.bg_fill = Color32::from_rgb(76, 118, 224);
            visuals.window_corner_radius = CornerRadius::same(2);
        }
        ThemeMode::HighContrast => {
            visuals.panel_fill = Color32::from_rgb(245, 245, 245);
            visuals.window_fill = Color32::WHITE;
            visuals.slider_trailing_fill = true;
            visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.5_f32, Color32::BLACK);
            visuals.widgets.inactive.bg_fill = Color32::from_rgb(140, 140, 140);
            visuals.widgets.inactive.weak_bg_fill = Color32::from_rgb(235, 235, 235);
            visuals.widgets.inactive.bg_stroke = Stroke::new(1.5_f32, Color32::BLACK);
            visuals.selection.bg_fill = Color32::from_rgb(0, 0, 160);
        }
    }

    style.visuals = visuals;
    style.animation_time = 0.0;
    style.spacing.item_spacing = egui::vec2(5.0, 4.0);
    style.spacing.button_padding = egui::vec2(6.0, 2.5);
    style.spacing.slider_rail_height = 10.0;
    let mut scroll = egui::style::ScrollStyle::solid();
    scroll.bar_width = 5.0;
    scroll.bar_inner_margin = 2.0;
    scroll.bar_outer_margin = 1.0;
    scroll.handle_min_length = 24.0;
    scroll.foreground_color = false;
    style.spacing.scroll = scroll;
    ctx.set_style(style);
}

pub fn desktop_bg_color(mode: ThemeMode) -> Color32 {
    match mode {
        ThemeMode::Light => Color32::from_rgb(154, 154, 160),
        ThemeMode::Dark => Color32::from_rgb(16, 18, 24),
        ThemeMode::HighContrast => Color32::from_rgb(40, 40, 40),
    }
}

pub fn canvas_bg_color(mode: ThemeMode) -> Color32 {
    match mode {
        ThemeMode::Light => Color32::from_rgb(246, 247, 250),
        ThemeMode::Dark => Color32::from_rgb(20, 22, 28),
        ThemeMode::HighContrast => Color32::WHITE,
    }
}

pub fn canvas_clear_rgba(mode: ThemeMode) -> [u8; 4] {
    let c = canvas_bg_color(mode);
    [c.r(), c.g(), c.b(), 255]
}

pub fn platinum_window_frame(mode: ThemeMode) -> Frame {
    let (fill, border) = match mode {
        ThemeMode::Light => (Color32::from_rgb(208, 208, 212), Color32::from_rgb(108, 112, 126)),
        ThemeMode::Dark => (Color32::from_rgb(30, 33, 42), Color32::from_rgb(76, 84, 104)),
        ThemeMode::HighContrast => (Color32::WHITE, Color32::BLACK),
    };
    Frame::new()
        .fill(fill)
        .stroke(Stroke::new(1.0_f32, border))
        .inner_margin(Margin::same(0))
}

pub fn draw_platinum_titlebar(
    ui: &mut egui::Ui,
    mode: ThemeMode,
    title: &str,
    right_badge: Option<&str>,
) -> bool {
    let h = 22.0_f32;
    let (rect, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), h), egui::Sense::hover());
    let box_rect = Rect::from_min_size(Pos2::new(rect.min.x + 5.0, rect.min.y + 5.0), Vec2::splat(12.0));
    let close_resp = ui.interact(box_rect.expand(3.0), ui.id().with(("titlebar_close", title)), egui::Sense::click());
    let painter = ui.painter();

    let (bg, stripe_col, text_col, border_col) = match mode {
        ThemeMode::Light => (
            Color32::from_rgb(212, 212, 218),
            Color32::from_rgb(156, 156, 168),
            Color32::from_rgb(18, 18, 26),
            Color32::from_rgb(112, 112, 124),
        ),
        ThemeMode::Dark => (
            Color32::from_rgb(42, 46, 58),
            Color32::from_rgb(64, 70, 88),
            Color32::from_rgb(232, 236, 248),
            Color32::from_rgb(76, 84, 104),
        ),
        ThemeMode::HighContrast => (
            Color32::from_rgb(235, 235, 235),
            Color32::BLACK,
            Color32::BLACK,
            Color32::BLACK,
        ),
    };

    painter.rect_filled(rect, 0.0, bg);
    painter.line_segment(
        [rect.left_bottom(), rect.right_bottom()],
        Stroke::new(1.0_f32, border_col),
    );

    for i in 0..5 {
        let y = rect.min.y + 4.0 + i as f32 * 3.0;
        painter.line_segment(
            [Pos2::new(rect.min.x + 22.0, y), Pos2::new(rect.max.x - 8.0, y)],
            Stroke::new(1.0_f32, stripe_col),
        );
    }

    let box_fill = if close_resp.hovered() {
        Color32::from_rgb(220, 38, 38)
    } else {
        bg
    };
    let x_col = if close_resp.hovered() {
        Color32::WHITE
    } else {
        border_col
    };
    painter.rect_filled(box_rect, 1.0, box_fill);
    painter.rect_stroke(box_rect, 1.0, Stroke::new(1.0_f32, border_col), egui::StrokeKind::Inside);
    let bc = box_rect.center();
    painter.line_segment([Pos2::new(bc.x - 2.6, bc.y - 2.6), Pos2::new(bc.x + 2.6, bc.y + 2.6)], Stroke::new(1.2_f32, x_col));
    painter.line_segment([Pos2::new(bc.x + 2.6, bc.y - 2.6), Pos2::new(bc.x - 2.6, bc.y + 2.6)], Stroke::new(1.2_f32, x_col));

    let title_w = (title.len() as f32 * 6.8 + 16.0).min((rect.width() - 48.0).max(24.0));
    let title_bg = Rect::from_center_size(rect.center(), Vec2::new(title_w, h - 4.0));
    painter.rect_filled(title_bg, 0.0, bg);
    painter.text(
        rect.center(),
        Align2::CENTER_CENTER,
        title,
        FontId::proportional(11.5),
        text_col,
    );

    if let Some(badge) = right_badge {
        let bw = badge.len() as f32 * 6.2 + 12.0;
        if rect.width() > title_w + bw + 36.0 {
            let brect = Rect::from_min_size(
                Pos2::new(rect.max.x - bw - 6.0, rect.min.y + 3.0),
                Vec2::new(bw, h - 6.0),
            );
            painter.rect_filled(brect, 2.0, bg);
            painter.text(
                brect.center(),
                Align2::CENTER_CENTER,
                badge,
                FontId::monospace(10.0),
                Color32::from_rgb(0, 0, 140),
            );
        }
    }
    close_resp.clicked()
}

pub fn draw_prism_badge(ui: &mut egui::Ui) {
    let (rect, _) = ui.allocate_exact_size(Vec2::splat(14.0), egui::Sense::hover());
    let c = rect.center();
    let pts = vec![
        Pos2::new(c.x, rect.min.y + 1.0),
        Pos2::new(rect.max.x - 1.0, c.y),
        Pos2::new(c.x, rect.max.y - 1.0),
        Pos2::new(rect.min.x + 1.0, c.y),
    ];
    ui.painter().add(egui::Shape::convex_polygon(
        pts,
        Color32::from_rgb(0, 0, 140),
        Stroke::new(1.0_f32, Color32::WHITE),
    ));
}

pub fn draw_eye_toggle(ui: &mut egui::Ui, visible: bool) -> egui::Response {
    let (rect, resp) = ui.allocate_exact_size(Vec2::new(20.0, 18.0), egui::Sense::click());
    let bg = if resp.hovered() {
        Color32::from_rgb(236, 240, 255)
    } else {
        Color32::from_rgb(228, 228, 232)
    };
    ui.painter().rect_filled(rect, 2.0, bg);
    ui.painter().rect_stroke(
        rect,
        2.0,
        Stroke::new(1.0_f32, Color32::from_rgb(60, 60, 70)),
        egui::StrokeKind::Inside,
    );
    let c = rect.center();
    let col = if visible {
        Color32::from_rgb(0, 0, 140)
    } else {
        Color32::from_rgb(130, 130, 140)
    };
    let s = Stroke::new(1.3_f32, col);
    ui.painter().line_segment([Pos2::new(c.x - 6.0, c.y), Pos2::new(c.x, c.y - 4.0)], s);
    ui.painter().line_segment([Pos2::new(c.x, c.y - 4.0), Pos2::new(c.x + 6.0, c.y)], s);
    ui.painter().line_segment([Pos2::new(c.x - 6.0, c.y), Pos2::new(c.x, c.y + 4.0)], s);
    ui.painter().line_segment([Pos2::new(c.x, c.y + 4.0), Pos2::new(c.x + 6.0, c.y)], s);
    if visible {
        ui.painter().circle_filled(c, 2.2, col);
    } else {
        ui.painter().line_segment(
            [Pos2::new(c.x - 5.0, c.y + 5.0), Pos2::new(c.x + 5.0, c.y - 5.0)],
            Stroke::new(1.5_f32, Color32::from_rgb(180, 40, 40)),
        );
    }
    resp
}

pub fn draw_lock_toggle(ui: &mut egui::Ui, locked: bool) -> egui::Response {
    let (rect, resp) = ui.allocate_exact_size(Vec2::new(20.0, 18.0), egui::Sense::click());
    let bg = if resp.hovered() {
        Color32::from_rgb(236, 240, 255)
    } else {
        Color32::from_rgb(228, 228, 232)
    };
    ui.painter().rect_filled(rect, 2.0, bg);
    ui.painter().rect_stroke(
        rect,
        2.0,
        Stroke::new(1.0_f32, Color32::from_rgb(60, 60, 70)),
        egui::StrokeKind::Inside,
    );
    let c = rect.center();
    let col = if locked {
        Color32::from_rgb(190, 40, 40)
    } else {
        Color32::from_rgb(70, 74, 86)
    };
    let body = Rect::from_min_size(Pos2::new(c.x - 4.5, c.y - 1.0), Vec2::new(9.0, 6.5));
    ui.painter().rect_filled(body, 1.0, if locked { col } else { Color32::TRANSPARENT });
    ui.painter().rect_stroke(body, 1.0, Stroke::new(1.2_f32, col), egui::StrokeKind::Inside);
    let shackle_x = if locked { c.x - 2.5 } else { c.x - 1.0 };
    let shackle = Rect::from_min_size(Pos2::new(shackle_x, c.y - 5.5), Vec2::new(5.0, 4.5));
    ui.painter().rect_stroke(shackle, 2.0, Stroke::new(1.2_f32, col), egui::StrokeKind::Inside);
    resp
}

pub fn draw_stencil_preview(painter: &Painter, rect: Rect, shape: &ShapeKind) {
    let fill = Color32::WHITE;
    let stroke = Stroke::new(1.25_f32, Color32::from_rgb(0, 0, 128));
    let r = rect.shrink(3.0);
    match shape {
        ShapeKind::Diamond | ShapeKind::GatewayXor | ShapeKind::GatewayParallel => {
            let c = r.center();
            let pts = vec![
                Pos2::new(c.x, r.min.y),
                Pos2::new(r.max.x, c.y),
                Pos2::new(c.x, r.max.y),
                Pos2::new(r.min.x, c.y),
            ];
            painter.add(egui::Shape::convex_polygon(pts, fill, stroke));
            if matches!(shape, ShapeKind::GatewayXor) {
                painter.line_segment([Pos2::new(c.x - 3.5, c.y - 3.5), Pos2::new(c.x + 3.5, c.y + 3.5)], stroke);
                painter.line_segment([Pos2::new(c.x + 3.5, c.y - 3.5), Pos2::new(c.x - 3.5, c.y + 3.5)], stroke);
            } else if matches!(shape, ShapeKind::GatewayParallel) {
                painter.line_segment([Pos2::new(c.x - 4.0, c.y), Pos2::new(c.x + 4.0, c.y)], stroke);
                painter.line_segment([Pos2::new(c.x, c.y - 4.0), Pos2::new(c.x, c.y + 4.0)], stroke);
            }
        }
        ShapeKind::Ellipse | ShapeKind::TimerEvent => {
            let rad = r.height() * 0.46;
            painter.circle_filled(r.center(), rad, fill);
            painter.circle_stroke(r.center(), rad, stroke);
            if matches!(shape, ShapeKind::TimerEvent) {
                painter.line_segment([r.center(), Pos2::new(r.center().x, r.center().y - rad * 0.65)], stroke);
                painter.line_segment([r.center(), Pos2::new(r.center().x + rad * 0.5, r.center().y)], stroke);
            }
        }
        ShapeKind::Cylinder => {
            let b = r.shrink2(Vec2::new(4.0, 1.0));
            let ry = (b.height() * 0.16).clamp(2.5, 4.5);
            let rx = b.width() * 0.5;
            let top_c = Pos2::new(b.center().x, b.min.y + ry);
            let bot_c = Pos2::new(b.center().x, b.max.y - ry);
            let body = Rect::from_min_max(
                Pos2::new(b.min.x, top_c.y),
                Pos2::new(b.max.x, bot_c.y),
            );
            painter.rect_filled(body, 0.0, fill);
            let segs = 16usize;
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
            painter.add(egui::Shape::line(top_pts, stroke));
            painter.add(egui::Shape::line(bot_arc, stroke));
            painter.line_segment([body.left_top(), body.left_bottom()], stroke);
            painter.line_segment([body.right_top(), body.right_bottom()], stroke);
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
        ShapeKind::Parallelogram => {
            let sk = r.width() * 0.18;
            let pts = vec![
                Pos2::new(r.min.x + sk, r.min.y),
                Pos2::new(r.max.x, r.min.y),
                Pos2::new(r.max.x - sk, r.max.y),
                Pos2::new(r.min.x, r.max.y),
            ];
            painter.add(egui::Shape::convex_polygon(pts, fill, stroke));
        }
        ShapeKind::Cloud => {
            let c = r.center();
            let r1 = r.height() * 0.34;
            let r2 = r.height() * 0.28;
            painter.circle_filled(Pos2::new(c.x - r.width() * 0.22, c.y + 1.5), r2, fill);
            painter.circle_filled(Pos2::new(c.x + r.width() * 0.22, c.y + 1.5), r2, fill);
            painter.circle_filled(Pos2::new(c.x, c.y - 1.5), r1, fill);
            painter.circle_stroke(Pos2::new(c.x - r.width() * 0.22, c.y + 1.5), r2, stroke);
            painter.circle_stroke(Pos2::new(c.x + r.width() * 0.22, c.y + 1.5), r2, stroke);
            painter.circle_stroke(Pos2::new(c.x, c.y - 1.5), r1, stroke);
            painter.rect_filled(
                Rect::from_center_size(Pos2::new(c.x, c.y + 1.5), Vec2::new(r.width() * 0.56, r1 * 1.1)),
                2.0,
                fill,
            );
        }
        ShapeKind::Actor => {
            let cx = r.center().x;
            let head_r = 3.2;
            let head_c = Pos2::new(cx, r.min.y + head_r + 1.0);
            painter.circle_filled(head_c, head_r, fill);
            painter.circle_stroke(head_c, head_r, stroke);
            let neck = head_c.y + head_r;
            let hip = r.max.y - 6.0;
            painter.line_segment([Pos2::new(cx, neck), Pos2::new(cx, hip)], stroke);
            let arm_y = neck + (hip - neck) * 0.35;
            painter.line_segment([Pos2::new(cx - 5.5, arm_y), Pos2::new(cx + 5.5, arm_y)], stroke);
            painter.line_segment([Pos2::new(cx, hip), Pos2::new(cx - 4.5, r.max.y)], stroke);
            painter.line_segment([Pos2::new(cx, hip), Pos2::new(cx + 4.5, r.max.y)], stroke);
        }
        ShapeKind::Document => {
            let wave = r.height() * 0.16;
            let pts = vec![
                r.left_top(),
                r.right_top(),
                Pos2::new(r.max.x, r.max.y - wave),
                Pos2::new(r.min.x + r.width() * 0.72, r.max.y - wave * 1.4),
                Pos2::new(r.min.x + r.width() * 0.3, r.max.y),
                Pos2::new(r.min.x, r.max.y - wave * 0.5),
            ];
            painter.add(egui::Shape::convex_polygon(pts.clone(), fill, Stroke::NONE));
            let mut closed = pts;
            closed.push(closed[0]);
            painter.add(egui::Shape::line(closed, stroke));
        }
        ShapeKind::Callout => {
            let tail_h = 5.0;
            let body = Rect::from_min_max(r.min, Pos2::new(r.max.x, r.max.y - tail_h));
            painter.rect_filled(body, 3.0, fill);
            painter.rect_stroke(body, 3.0, stroke, egui::StrokeKind::Inside);
            let tail = vec![
                Pos2::new(r.min.x + r.width() * 0.24, body.max.y - 0.8),
                Pos2::new(r.min.x + r.width() * 0.18, r.max.y),
                Pos2::new(r.min.x + r.width() * 0.42, body.max.y - 0.8),
            ];
            painter.add(egui::Shape::convex_polygon(tail.clone(), fill, Stroke::NONE));
            painter.line_segment([tail[0], tail[1]], stroke);
            painter.line_segment([tail[1], tail[2]], stroke);
        }
        ShapeKind::Note => {
            let fold = 5.5;
            let pts = vec![
                r.left_top(),
                Pos2::new(r.max.x - fold, r.min.y),
                Pos2::new(r.max.x, r.min.y + fold),
                r.right_bottom(),
                r.left_bottom(),
            ];
            painter.add(egui::Shape::convex_polygon(pts, fill, stroke));
            painter.line_segment([Pos2::new(r.max.x - fold, r.min.y), Pos2::new(r.max.x - fold, r.min.y + fold)], stroke);
            painter.line_segment([Pos2::new(r.max.x - fold, r.min.y + fold), Pos2::new(r.max.x, r.min.y + fold)], stroke);
        }
        ShapeKind::Package => {
            let tab = Rect::from_min_size(r.min, Vec2::new(r.width() * 0.42, 5.5));
            let body = Rect::from_min_max(Pos2::new(r.min.x, r.min.y + 4.8), r.max);
            painter.rect_filled(tab, 1.0, Color32::from_rgb(214, 222, 240));
            painter.rect_stroke(tab, 1.0, stroke, egui::StrokeKind::Inside);
            painter.rect_filled(body, 1.0, fill);
            painter.rect_stroke(body, 1.0, stroke, egui::StrokeKind::Inside);
        }
        ShapeKind::Component => {
            let body = Rect::from_min_max(Pos2::new(r.min.x + 3.5, r.min.y), r.max);
            painter.rect_filled(body, 1.0, fill);
            painter.rect_stroke(body, 1.0, stroke, egui::StrokeKind::Inside);
            for frac in [0.28, 0.68] {
                let ny = r.min.y + r.height() * frac - 2.2;
                let nub = Rect::from_min_size(Pos2::new(r.min.x, ny), Vec2::new(7.0, 4.4));
                painter.rect_filled(nub, 0.5, fill);
                painter.rect_stroke(nub, 0.5, stroke, egui::StrokeKind::Inside);
            }
        }
        ShapeKind::Interface => {
            let cy = r.center().y;
            let rad = 5.5;
            let cx = r.min.x + rad + 2.0;
            painter.circle_filled(Pos2::new(cx, cy), rad, fill);
            painter.circle_stroke(Pos2::new(cx, cy), rad, stroke);
            painter.line_segment([Pos2::new(cx + rad, cy), Pos2::new(r.max.x - 1.5, cy)], stroke);
        }
        ShapeKind::Queue => {
            let qr = r.shrink2(Vec2::new(0.5, 2.5));
            let cr = (qr.height() * 0.35).min(8.0);
            painter.rect_filled(qr, cr, fill);
            painter.rect_stroke(qr, cr, stroke, egui::StrokeKind::Inside);
            for t in [0.28, 0.52, 0.76] {
                let x = qr.min.x + qr.width() * t;
                painter.line_segment([Pos2::new(x, qr.min.y + 1.5), Pos2::new(x, qr.max.y - 1.5)], stroke);
            }
        }
        ShapeKind::Server => {
            painter.rect_filled(r, 2.0, fill);
            painter.rect_stroke(r, 2.0, stroke, egui::StrokeKind::Inside);
            for t in [0.33, 0.66] {
                let y = r.min.y + r.height() * t;
                painter.line_segment([Pos2::new(r.min.x, y), Pos2::new(r.max.x, y)], stroke);
            }
            for t in [0.16, 0.5, 0.83] {
                let y = r.min.y + r.height() * t;
                painter.circle_filled(Pos2::new(r.max.x - 4.5, y), 1.3, Color32::from_rgb(16, 185, 129));
            }
        }
        ShapeKind::Firewall => {
            painter.rect_filled(r, 1.0, fill);
            painter.rect_stroke(r, 1.0, stroke, egui::StrokeKind::Inside);
            for row in 1..3 {
                let y = r.min.y + r.height() * (row as f32 / 3.0);
                painter.line_segment([Pos2::new(r.min.x, y), Pos2::new(r.max.x, y)], stroke);
            }
            for row in 0..3 {
                let y0 = r.min.y + r.height() * (row as f32 / 3.0);
                let y1 = r.min.y + r.height() * ((row + 1) as f32 / 3.0);
                if row % 2 == 0 {
                    let x = r.center().x;
                    painter.line_segment([Pos2::new(x, y0), Pos2::new(x, y1)], stroke);
                } else {
                    painter.line_segment([Pos2::new(r.min.x + r.width() * 0.25, y0), Pos2::new(r.min.x + r.width() * 0.25, y1)], stroke);
                    painter.line_segment([Pos2::new(r.min.x + r.width() * 0.75, y0), Pos2::new(r.min.x + r.width() * 0.75, y1)], stroke);
                }
            }
        }
        ShapeKind::Shield => {
            let cx = r.center().x;
            let pts = vec![
                Pos2::new(cx, r.min.y),
                Pos2::new(r.max.x - 1.5, r.min.y + r.height() * 0.18),
                Pos2::new(r.max.x - 3.5, r.max.y - r.height() * 0.24),
                Pos2::new(cx, r.max.y),
                Pos2::new(r.min.x + 3.5, r.max.y - r.height() * 0.24),
                Pos2::new(r.min.x + 1.5, r.min.y + r.height() * 0.18),
            ];
            painter.add(egui::Shape::convex_polygon(pts, fill, stroke));
        }
        ShapeKind::BrowserWindow => {
            painter.rect_filled(r, 2.0, fill);
            let hh = 6.0;
            painter.rect_filled(Rect::from_min_size(r.min, Vec2::new(r.width(), hh)), 2.0, Color32::from_rgb(212, 218, 232));
            painter.rect_stroke(r, 2.0, stroke, egui::StrokeKind::Inside);
            painter.line_segment([Pos2::new(r.min.x, r.min.y + hh), Pos2::new(r.max.x, r.min.y + hh)], stroke);
            for (i, c) in [
                Color32::from_rgb(239, 68, 68),
                Color32::from_rgb(245, 158, 11),
                Color32::from_rgb(34, 197, 94),
            ]
            .iter()
            .enumerate()
            {
                painter.circle_filled(Pos2::new(r.min.x + 3.5 + i as f32 * 3.2, r.min.y + hh * 0.5), 1.1, *c);
            }
        }
        ShapeKind::MobileScreen => {
            let mr = Rect::from_center_size(r.center(), Vec2::new(14.0, r.height()));
            painter.rect_filled(mr, 3.0, fill);
            painter.rect_stroke(mr, 3.0, stroke, egui::StrokeKind::Inside);
            let notch = Rect::from_center_size(Pos2::new(mr.center().x, mr.min.y + 2.8), Vec2::new(5.0, 1.4));
            painter.rect_filled(notch, 0.5, Color32::from_rgb(0, 0, 128));
            painter.circle_stroke(Pos2::new(mr.center().x, mr.max.y - 3.0), 1.4, stroke);
        }
        ShapeKind::Section => {
            painter.rect_filled(r, 2.0, fill);
            painter.rect_filled(
                Rect::from_min_size(r.min, Vec2::new(r.width(), 6.0)),
                2.0,
                Color32::from_rgba_unmultiplied(185, 198, 224, 180),
            );
            painter.rect_stroke(r, 2.0, stroke, egui::StrokeKind::Inside);
        }
        ShapeKind::FolderTree => {
            painter.rect_filled(r, 2.0, fill);
            painter.rect_filled(Rect::from_min_size(r.min, Vec2::new(r.width(), 5.5)), 2.0, Color32::from_rgb(218, 224, 236));
            painter.rect_stroke(r, 2.0, stroke, egui::StrokeKind::Inside);
            for i in 0..3 {
                let y = r.min.y + 9.0 + i as f32 * 4.2;
                let ind = if i == 0 { 4.0 } else { 9.0 };
                painter.line_segment([Pos2::new(r.min.x + ind, y), Pos2::new(r.max.x - 4.0, y)], stroke);
            }
        }
        ShapeKind::Collapsible => {
            painter.rect_filled(r, 2.0, fill);
            painter.rect_filled(Rect::from_min_size(r.min, Vec2::new(r.width(), 7.0)), 2.0, Color32::from_rgb(220, 226, 238));
            painter.rect_stroke(r, 2.0, stroke, egui::StrokeKind::Inside);
            let cx = r.min.x + 4.5;
            let cy = r.min.y + 3.5;
            painter.line_segment([Pos2::new(cx - 1.8, cy - 1.0), Pos2::new(cx, cy + 1.2)], stroke);
            painter.line_segment([Pos2::new(cx, cy + 1.2), Pos2::new(cx + 1.8, cy - 1.0)], stroke);
        }
        ShapeKind::Subprocess => {
            painter.rect_filled(r, 1.0, fill);
            painter.rect_stroke(r, 1.0, stroke, egui::StrokeKind::Inside);
            painter.line_segment([Pos2::new(r.min.x + 4.5, r.min.y), Pos2::new(r.min.x + 4.5, r.max.y)], stroke);
            painter.line_segment([Pos2::new(r.max.x - 4.5, r.min.y), Pos2::new(r.max.x - 4.5, r.max.y)], stroke);
        }
        ShapeKind::ManualInput => {
            let pts = vec![
                Pos2::new(r.min.x, r.min.y + r.height() * 0.26),
                r.right_top(),
                r.right_bottom(),
                r.left_bottom(),
            ];
            painter.add(egui::Shape::convex_polygon(pts, fill, stroke));
        }
        ShapeKind::Delay => {
            let rad = r.height() * 0.5;
            let cx = (r.max.x - rad).max(r.min.x + 4.0);
            let cy = r.center().y;
            let mut pts = vec![r.left_top(), Pos2::new(cx, r.min.y)];
            let segs = 12usize;
            for i in 1..=segs {
                let a = -std::f32::consts::FRAC_PI_2 + (i as f32 / segs as f32) * std::f32::consts::PI;
                pts.push(Pos2::new(cx + a.cos() * rad, cy + a.sin() * rad));
            }
            pts.push(r.left_bottom());
            painter.add(egui::Shape::convex_polygon(pts.clone(), fill, Stroke::NONE));
            let mut closed = pts;
            closed.push(closed[0]);
            painter.add(egui::Shape::line(closed, stroke));
        }
        ShapeKind::OffPage => {
            let tip = r.height() * 0.28;
            let pts = vec![
                r.left_top(),
                r.right_top(),
                Pos2::new(r.max.x, r.max.y - tip),
                Pos2::new(r.center().x, r.max.y),
                Pos2::new(r.min.x, r.max.y - tip),
            ];
            painter.add(egui::Shape::convex_polygon(pts, fill, stroke));
        }
        ShapeKind::Triangle => {
            let pts = vec![
                Pos2::new(r.center().x, r.min.y),
                r.right_bottom(),
                r.left_bottom(),
            ];
            painter.add(egui::Shape::convex_polygon(pts, fill, stroke));
        }
        ShapeKind::ImageNode => {
            painter.rect_filled(r, 2.0, fill);
            painter.rect_stroke(r, 2.0, stroke, egui::StrokeKind::Inside);
            let inner = r.shrink(2.5);
            painter.rect_filled(inner, 1.0, Color32::from_rgb(230, 236, 245));
            painter.circle_filled(Pos2::new(inner.max.x - 4.0, inner.min.y + 4.0), 1.8, Color32::from_rgb(245, 158, 11));
            let mtn = vec![
                Pos2::new(inner.min.x + 1.5, inner.max.y - 1.0),
                Pos2::new(inner.min.x + inner.width() * 0.42, inner.min.y + 4.5),
                Pos2::new(inner.max.x - 1.5, inner.max.y - 1.0),
            ];
            painter.add(egui::Shape::convex_polygon(mtn, Color32::from_rgb(100, 128, 178), Stroke::NONE));
        }
        _ => {
            let cr = if matches!(shape, ShapeKind::RoundedRect | ShapeKind::ReactFlowCard) { 4.0 } else { 1.0 };
            painter.rect_filled(r, cr, fill);
            painter.rect_stroke(r, cr, stroke, egui::StrokeKind::Inside);
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolIcon {
    NewFile,
    OpenFile,
    Undo,
    Redo,
    Copy,
    Paste,
    Duplicate,
    Delete,
    ZoomOut,
    ZoomIn,
    FitView,
    Image,
    Symbol,
    Png,
    Gif,
    Layout,
    Share,
    Help,
    Search,
    AlignLeft,
    AlignCenterH,
    AlignRight,
    AlignTop,
    AlignCenterV,
    AlignBottom,
    DistH,
    DistV,
    BringFront,
    BringForward,
    SendBackward,
    SendBack,
}

pub fn draw_tool_icon(painter: &Painter, rect: Rect, icon: ToolIcon, col: Color32) {
    let s = Stroke::new(1.4_f32, col);
    let r = rect.shrink(2.0);
    let c = r.center();
    match icon {
        ToolIcon::NewFile => {
            let fold = 4.0;
            let pts = [
                r.left_top(),
                Pos2::new(r.max.x - fold, r.min.y),
                Pos2::new(r.max.x, r.min.y + fold),
                r.right_bottom(),
                r.left_bottom(),
                r.left_top(),
            ];
            for w in pts.windows(2) {
                painter.line_segment([w[0], w[1]], s);
            }
        }
        ToolIcon::OpenFile => {
            let body = Rect::from_min_max(Pos2::new(r.min.x, r.min.y + 3.0), r.max);
            painter.rect_stroke(body, 1.0, s, egui::StrokeKind::Inside);
            painter.line_segment([Pos2::new(r.min.x, r.min.y + 3.0), Pos2::new(r.min.x + 2.0, r.min.y + 1.0)], s);
            painter.line_segment([Pos2::new(r.min.x + 2.0, r.min.y + 1.0), Pos2::new(r.min.x + 6.0, r.min.y + 1.0)], s);
        }
        ToolIcon::Undo => {
            painter.line_segment([Pos2::new(r.min.x + 1.0, c.y - 1.0), Pos2::new(r.max.x - 2.0, c.y - 1.0)], s);
            painter.line_segment([Pos2::new(r.max.x - 2.0, c.y - 1.0), Pos2::new(r.max.x - 2.0, r.max.y - 1.0)], s);
            painter.line_segment([Pos2::new(r.min.x + 1.0, c.y - 1.0), Pos2::new(r.min.x + 4.5, c.y - 4.5)], s);
            painter.line_segment([Pos2::new(r.min.x + 1.0, c.y - 1.0), Pos2::new(r.min.x + 4.5, c.y + 2.5)], s);
        }
        ToolIcon::Redo => {
            painter.line_segment([Pos2::new(r.max.x - 1.0, c.y - 1.0), Pos2::new(r.min.x + 2.0, c.y - 1.0)], s);
            painter.line_segment([Pos2::new(r.min.x + 2.0, c.y - 1.0), Pos2::new(r.min.x + 2.0, r.max.y - 1.0)], s);
            painter.line_segment([Pos2::new(r.max.x - 1.0, c.y - 1.0), Pos2::new(r.max.x - 4.5, c.y - 4.5)], s);
            painter.line_segment([Pos2::new(r.max.x - 1.0, c.y - 1.0), Pos2::new(r.max.x - 4.5, c.y + 2.5)], s);
        }
        ToolIcon::Copy => {
            let b1 = Rect::from_min_size(r.min, Vec2::new(r.width() * 0.68, r.height() * 0.72));
            let b2 = Rect::from_min_size(r.min + Vec2::new(3.5, 3.0), Vec2::new(r.width() * 0.68, r.height() * 0.72));
            painter.rect_stroke(b1, 1.0, s, egui::StrokeKind::Inside);
            painter.rect_filled(b2, 1.0, Color32::from_rgb(232, 232, 236));
            painter.rect_stroke(b2, 1.0, s, egui::StrokeKind::Inside);
        }
        ToolIcon::Paste => {
            let body = Rect::from_min_max(Pos2::new(r.min.x + 1.0, r.min.y + 2.5), Pos2::new(r.max.x - 1.0, r.max.y));
            let clip = Rect::from_center_size(Pos2::new(c.x, r.min.y + 2.5), Vec2::new(5.5, 3.0));
            painter.rect_stroke(body, 1.0, s, egui::StrokeKind::Inside);
            painter.rect_filled(clip, 0.5, col);
        }
        ToolIcon::Duplicate => {
            let b1 = Rect::from_min_size(r.min, Vec2::new(7.5, 7.5));
            let b2 = Rect::from_min_size(r.min + Vec2::new(3.5, 3.5), Vec2::new(7.5, 7.5));
            painter.rect_stroke(b1, 1.0, s, egui::StrokeKind::Inside);
            painter.rect_stroke(b2, 1.0, s, egui::StrokeKind::Inside);
        }
        ToolIcon::Delete => {
            let ds = Stroke::new(1.4_f32, Color32::from_rgb(185, 28, 28));
            painter.line_segment([Pos2::new(r.min.x + 1.0, r.min.y + 2.5), Pos2::new(r.max.x - 1.0, r.min.y + 2.5)], ds);
            let bin = Rect::from_min_max(Pos2::new(r.min.x + 2.5, r.min.y + 2.5), Pos2::new(r.max.x - 2.5, r.max.y));
            painter.rect_stroke(bin, 1.0, ds, egui::StrokeKind::Inside);
            painter.line_segment([Pos2::new(c.x, r.min.y + 5.0), Pos2::new(c.x, r.max.y - 2.0)], ds);
        }
        ToolIcon::ZoomOut => {
            painter.line_segment([Pos2::new(r.min.x + 2.0, c.y), Pos2::new(r.max.x - 2.0, c.y)], s);
        }
        ToolIcon::ZoomIn => {
            painter.line_segment([Pos2::new(r.min.x + 2.0, c.y), Pos2::new(r.max.x - 2.0, c.y)], s);
            painter.line_segment([Pos2::new(c.x, r.min.y + 2.0), Pos2::new(c.x, r.max.y - 2.0)], s);
        }
        ToolIcon::FitView => {
            let d = 3.5;
            painter.line_segment([r.left_top(), r.left_top() + Vec2::new(d, 0.0)], s);
            painter.line_segment([r.left_top(), r.left_top() + Vec2::new(0.0, d)], s);
            painter.line_segment([r.right_top(), r.right_top() + Vec2::new(-d, 0.0)], s);
            painter.line_segment([r.right_top(), r.right_top() + Vec2::new(0.0, d)], s);
            painter.line_segment([r.left_bottom(), r.left_bottom() + Vec2::new(d, 0.0)], s);
            painter.line_segment([r.left_bottom(), r.left_bottom() + Vec2::new(0.0, -d)], s);
            painter.line_segment([r.right_bottom(), r.right_bottom() + Vec2::new(-d, 0.0)], s);
            painter.line_segment([r.right_bottom(), r.right_bottom() + Vec2::new(0.0, -d)], s);
        }
        ToolIcon::Image | ToolIcon::Png => {
            painter.rect_stroke(r, 1.0, s, egui::StrokeKind::Inside);
            painter.line_segment([Pos2::new(r.min.x + 1.5, r.max.y - 1.5), Pos2::new(c.x - 1.0, c.y)], s);
            painter.line_segment([Pos2::new(c.x - 1.0, c.y), Pos2::new(r.max.x - 1.5, r.max.y - 1.5)], s);
            painter.circle_filled(Pos2::new(r.max.x - 3.5, r.min.y + 3.5), 1.4, col);
        }
        ToolIcon::Symbol => {
            let pts = vec![
                Pos2::new(c.x, r.min.y),
                Pos2::new(r.max.x, c.y),
                Pos2::new(c.x, r.max.y),
                Pos2::new(r.min.x, c.y),
            ];
            painter.add(egui::Shape::convex_polygon(pts, Color32::TRANSPARENT, s));
        }
        ToolIcon::Gif => {
            painter.rect_stroke(r, 1.0, s, egui::StrokeKind::Inside);
            let tri = vec![
                Pos2::new(c.x - 2.0, c.y - 3.5),
                Pos2::new(c.x + 3.5, c.y),
                Pos2::new(c.x - 2.0, c.y + 3.5),
            ];
            painter.add(egui::Shape::convex_polygon(tri, col, Stroke::NONE));
        }
        ToolIcon::Layout => {
            let top = Rect::from_center_size(Pos2::new(c.x, r.min.y + 2.5), Vec2::new(5.0, 3.5));
            let bl = Rect::from_center_size(Pos2::new(r.min.x + 3.0, r.max.y - 2.5), Vec2::new(4.5, 3.5));
            let br = Rect::from_center_size(Pos2::new(r.max.x - 3.0, r.max.y - 2.5), Vec2::new(4.5, 3.5));
            painter.rect_filled(top, 0.5, col);
            painter.rect_filled(bl, 0.5, col);
            painter.rect_filled(br, 0.5, col);
            painter.line_segment([Pos2::new(c.x, r.min.y + 4.0), Pos2::new(c.x, c.y + 1.0)], s);
            painter.line_segment([Pos2::new(r.min.x + 3.0, c.y + 1.0), Pos2::new(r.max.x - 3.0, c.y + 1.0)], s);
        }
        ToolIcon::Share => {
            let p1 = Pos2::new(r.min.x + 2.5, c.y);
            let p2 = Pos2::new(r.max.x - 2.5, r.min.y + 2.5);
            let p3 = Pos2::new(r.max.x - 2.5, r.max.y - 2.5);
            painter.line_segment([p1, p2], s);
            painter.line_segment([p1, p3], s);
            painter.circle_filled(p1, 2.0, col);
            painter.circle_filled(p2, 2.0, col);
            painter.circle_filled(p3, 2.0, col);
        }
        ToolIcon::Help => {
            painter.circle_stroke(c, r.width() * 0.45, s);
            painter.text(c, Align2::CENTER_CENTER, "?", FontId::proportional(10.0), col);
        }
        ToolIcon::Search => {
            let lens = Pos2::new(c.x - 1.5, c.y - 1.5);
            painter.circle_stroke(lens, 3.8, s);
            painter.line_segment([lens + Vec2::splat(2.8), r.right_bottom()], s);
        }
        ToolIcon::AlignLeft => {
            painter.line_segment([r.left_top(), r.left_bottom()], s);
            painter.rect_filled(Rect::from_min_size(Pos2::new(r.min.x + 2.5, r.min.y + 2.0), Vec2::new(8.0, 3.0)), 0.5, col);
            painter.rect_filled(Rect::from_min_size(Pos2::new(r.min.x + 2.5, r.max.y - 5.0), Vec2::new(5.0, 3.0)), 0.5, col);
        }
        ToolIcon::AlignCenterH => {
            painter.line_segment([Pos2::new(c.x, r.min.y), Pos2::new(c.x, r.max.y)], s);
            painter.rect_filled(Rect::from_center_size(Pos2::new(c.x, r.min.y + 3.5), Vec2::new(9.0, 3.0)), 0.5, col);
            painter.rect_filled(Rect::from_center_size(Pos2::new(c.x, r.max.y - 3.5), Vec2::new(5.5, 3.0)), 0.5, col);
        }
        ToolIcon::AlignRight => {
            painter.line_segment([r.right_top(), r.right_bottom()], s);
            painter.rect_filled(Rect::from_min_size(Pos2::new(r.max.x - 10.5, r.min.y + 2.0), Vec2::new(8.0, 3.0)), 0.5, col);
            painter.rect_filled(Rect::from_min_size(Pos2::new(r.max.x - 7.5, r.max.y - 5.0), Vec2::new(5.0, 3.0)), 0.5, col);
        }
        ToolIcon::AlignTop => {
            painter.line_segment([r.left_top(), r.right_top()], s);
            painter.rect_filled(Rect::from_min_size(Pos2::new(r.min.x + 2.0, r.min.y + 2.5), Vec2::new(3.0, 8.0)), 0.5, col);
            painter.rect_filled(Rect::from_min_size(Pos2::new(r.max.x - 5.0, r.min.y + 2.5), Vec2::new(3.0, 5.0)), 0.5, col);
        }
        ToolIcon::AlignCenterV => {
            painter.line_segment([Pos2::new(r.min.x, c.y), Pos2::new(r.max.x, c.y)], s);
            painter.rect_filled(Rect::from_center_size(Pos2::new(r.min.x + 3.5, c.y), Vec2::new(3.0, 9.0)), 0.5, col);
            painter.rect_filled(Rect::from_center_size(Pos2::new(r.max.x - 3.5, c.y), Vec2::new(3.0, 5.5)), 0.5, col);
        }
        ToolIcon::AlignBottom => {
            painter.line_segment([r.left_bottom(), r.right_bottom()], s);
            painter.rect_filled(Rect::from_min_size(Pos2::new(r.min.x + 2.0, r.max.y - 10.5), Vec2::new(3.0, 8.0)), 0.5, col);
            painter.rect_filled(Rect::from_min_size(Pos2::new(r.max.x - 5.0, r.max.y - 7.5), Vec2::new(3.0, 5.0)), 0.5, col);
        }
        ToolIcon::DistH => {
            painter.line_segment([r.left_top(), r.left_bottom()], s);
            painter.line_segment([r.right_top(), r.right_bottom()], s);
            painter.rect_filled(Rect::from_center_size(c, Vec2::new(3.5, 7.5)), 0.5, col);
        }
        ToolIcon::DistV => {
            painter.line_segment([r.left_top(), r.right_top()], s);
            painter.line_segment([r.left_bottom(), r.right_bottom()], s);
            painter.rect_filled(Rect::from_center_size(c, Vec2::new(7.5, 3.5)), 0.5, col);
        }
        ToolIcon::BringFront => {
            let b1 = Rect::from_min_size(r.min, Vec2::splat(7.0));
            let b2 = Rect::from_min_size(r.min + Vec2::splat(4.0), Vec2::splat(7.0));
            painter.rect_stroke(b1, 0.5, s, egui::StrokeKind::Inside);
            painter.rect_filled(b2, 0.5, col);
        }
        ToolIcon::BringForward => {
            painter.line_segment([Pos2::new(c.x, r.max.y), Pos2::new(c.x, r.min.y + 1.0)], s);
            painter.line_segment([Pos2::new(c.x, r.min.y + 1.0), Pos2::new(c.x - 3.5, r.min.y + 4.5)], s);
            painter.line_segment([Pos2::new(c.x, r.min.y + 1.0), Pos2::new(c.x + 3.5, r.min.y + 4.5)], s);
        }
        ToolIcon::SendBackward => {
            painter.line_segment([Pos2::new(c.x, r.min.y), Pos2::new(c.x, r.max.y - 1.0)], s);
            painter.line_segment([Pos2::new(c.x, r.max.y - 1.0), Pos2::new(c.x - 3.5, r.max.y - 4.5)], s);
            painter.line_segment([Pos2::new(c.x, r.max.y - 1.0), Pos2::new(c.x + 3.5, r.max.y - 4.5)], s);
        }
        ToolIcon::SendBack => {
            let b1 = Rect::from_min_size(r.min, Vec2::splat(7.0));
            let b2 = Rect::from_min_size(r.min + Vec2::splat(4.0), Vec2::splat(7.0));
            painter.rect_filled(b1, 0.5, col);
            painter.rect_filled(b2, 0.5, Color32::WHITE);
            painter.rect_stroke(b2, 0.5, s, egui::StrokeKind::Inside);
        }
    }
}

pub fn icon_button(
    ui: &mut egui::Ui,
    icon: ToolIcon,
    label: Option<&str>,
    tooltip: &str,
    enabled: bool,
    primary: bool,
) -> egui::Response {
    let text_w = label.map(|l| l.len() as f32 * 6.4 + 4.0).unwrap_or(0.0);
    let size = Vec2::new(22.0 + text_w, 22.0);
    let sense = if enabled { egui::Sense::click() } else { egui::Sense::hover() };
    let (rect, resp) = ui.allocate_exact_size(size, sense);
    let bg = if !enabled {
        Color32::from_rgb(206, 206, 210)
    } else if primary {
        if resp.hovered() {
            Color32::from_rgb(20, 40, 165)
        } else {
            Color32::from_rgb(0, 0, 128)
        }
    } else if resp.hovered() {
        Color32::from_rgb(244, 246, 255)
    } else {
        Color32::from_rgb(228, 228, 232)
    };
    let fg = if !enabled {
        Color32::from_rgb(138, 138, 146)
    } else if primary {
        Color32::WHITE
    } else {
        Color32::from_rgb(18, 18, 26)
    };
    let border = if resp.hovered() && enabled {
        Color32::from_rgb(0, 0, 128)
    } else {
        Color32::from_rgb(36, 36, 44)
    };
    ui.painter().rect_filled(rect, 1.5, bg);
    ui.painter().rect_stroke(rect, 1.5, Stroke::new(1.0_f32, border), egui::StrokeKind::Inside);
    let icon_rect = Rect::from_min_size(Pos2::new(rect.min.x + 3.0, rect.min.y + 3.0), Vec2::splat(16.0));
    draw_tool_icon(ui.painter(), icon_rect, icon, fg);
    if let Some(lbl) = label {
        ui.painter().text(
            Pos2::new(rect.min.x + 21.0, rect.center().y),
            Align2::LEFT_CENTER,
            lbl,
            FontId::proportional(11.0),
            fg,
        );
    }
    resp.on_hover_text(tooltip)
}

pub fn to_color32(c: ColorRgba) -> Color32 {
    Color32::from_rgba_unmultiplied(
        (c.r.clamp(0.0, 1.0) * 255.0).round() as u8,
        (c.g.clamp(0.0, 1.0) * 255.0).round() as u8,
        (c.b.clamp(0.0, 1.0) * 255.0).round() as u8,
        (c.a.clamp(0.0, 1.0) * 255.0).round() as u8,
    )
}

pub fn from_color32(c: Color32) -> ColorRgba {
    ColorRgba::new(
        c.r() as f32 / 255.0,
        c.g() as f32 / 255.0,
        c.b() as f32 / 255.0,
        c.a() as f32 / 255.0,
    )
}

pub fn symbol_icon_accent(icon: SymbolIcon) -> Color32 {
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
        | SymbolIcon::AwsOutposts => Color32::from_rgb(237, 113, 0),
        SymbolIcon::AwsS3
        | SymbolIcon::AwsEbs
        | SymbolIcon::AwsEfs
        | SymbolIcon::AwsGlacier
        | SymbolIcon::AwsBackup => Color32::from_rgb(63, 134, 36),
        SymbolIcon::AwsRds
        | SymbolIcon::AwsDynamoDb
        | SymbolIcon::AwsAurora
        | SymbolIcon::AwsElastiCache
        | SymbolIcon::AwsRedshift
        | SymbolIcon::AwsDocumentDb
        | SymbolIcon::AwsNeptune
        | SymbolIcon::AwsTimestream
        | SymbolIcon::AwsMemoryDb
        | SymbolIcon::AwsKeyspaces => Color32::from_rgb(59, 72, 204),
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
        | SymbolIcon::AwsLakeFormation => Color32::from_rgb(140, 79, 255),
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
        | SymbolIcon::AwsCloudFormation => Color32::from_rgb(231, 21, 123),
        SymbolIcon::AwsIam
        | SymbolIcon::AwsCognito
        | SymbolIcon::AwsKms
        | SymbolIcon::AwsWaf
        | SymbolIcon::AwsShield
        | SymbolIcon::AwsGuardDuty
        | SymbolIcon::AwsSecretsManager => Color32::from_rgb(221, 52, 76),
        SymbolIcon::AwsBedrock
        | SymbolIcon::AwsSageMaker
        | SymbolIcon::AwsRekognition
        | SymbolIcon::AwsTextract
        | SymbolIcon::AwsPolly
        | SymbolIcon::AwsQ => Color32::from_rgb(1, 168, 141),
        SymbolIcon::AzureVm
        | SymbolIcon::AzureAks
        | SymbolIcon::AzureFunctions
        | SymbolIcon::AzureAppService
        | SymbolIcon::AzureContainerApps
        | SymbolIcon::AzureContainerInstances
        | SymbolIcon::AzureSpringApps => Color32::from_rgb(0, 120, 212),
        SymbolIcon::AzureBlob
        | SymbolIcon::AzureFiles
        | SymbolIcon::AzureDisk
        | SymbolIcon::AzureDataLake => Color32::from_rgb(0, 148, 198),
        SymbolIcon::AzureSql
        | SymbolIcon::AzureCosmosDb
        | SymbolIcon::AzurePostgres
        | SymbolIcon::AzureRedis
        | SymbolIcon::AzureSynapse => Color32::from_rgb(14, 90, 178),
        SymbolIcon::AzureVnet
        | SymbolIcon::AzureFrontDoor
        | SymbolIcon::AzureAppGateway
        | SymbolIcon::AzureLoadBalancer
        | SymbolIcon::AzureDns
        | SymbolIcon::AzureFirewall
        | SymbolIcon::AzureApiManagement => Color32::from_rgb(2, 132, 199),
        SymbolIcon::AzureServiceBus
        | SymbolIcon::AzureEventHubs
        | SymbolIcon::AzureEventGrid
        | SymbolIcon::AzureLogicApps => Color32::from_rgb(67, 56, 202),
        SymbolIcon::AzureEntraId
        | SymbolIcon::AzureKeyVault
        | SymbolIcon::AzureMonitor
        | SymbolIcon::AzureDevOps => Color32::from_rgb(3, 105, 161),
        SymbolIcon::AzureOpenAi | SymbolIcon::AzureAiSearch => Color32::from_rgb(109, 40, 217),
        SymbolIcon::GcpCloudRun
        | SymbolIcon::GcpGke
        | SymbolIcon::GcpBigQuery
        | SymbolIcon::GcpPubSub
        | SymbolIcon::GcpCloudStorage
        | SymbolIcon::GcpCloudSql => Color32::from_rgb(66, 133, 244),
        SymbolIcon::Kubernetes => Color32::from_rgb(50, 108, 229),
        SymbolIcon::Docker => Color32::from_rgb(36, 150, 237),
        SymbolIcon::Postgres => Color32::from_rgb(51, 103, 145),
        SymbolIcon::Redis => Color32::from_rgb(216, 44, 32),
        SymbolIcon::Kafka => Color32::from_rgb(35, 31, 32),
        SymbolIcon::Nginx => Color32::from_rgb(0, 150, 57),
        SymbolIcon::LoadBalancer | SymbolIcon::Cdn => Color32::from_rgb(79, 70, 229),
        SymbolIcon::User | SymbolIcon::Users => Color32::from_rgb(30, 64, 175),
        SymbolIcon::Globe => Color32::from_rgb(14, 116, 144),
        SymbolIcon::Lock | SymbolIcon::Key => Color32::from_rgb(180, 83, 9),
        SymbolIcon::Api | SymbolIcon::Webhook => Color32::from_rgb(15, 118, 110),
        SymbolIcon::Monitor | SymbolIcon::Terminal => Color32::from_rgb(51, 65, 85),
        SymbolIcon::AiModel => Color32::from_rgb(124, 58, 237),
    }
}

pub fn draw_symbol_icon(painter: &Painter, rect: Rect, icon: SymbolIcon) {
    let side = rect.width().min(rect.height()).max(14.0);
    let badge = Rect::from_center_size(rect.center(), Vec2::splat(side));
    let accent = symbol_icon_accent(icon);
    let cr = (side * 0.22).clamp(3.0, 9.0);
    painter.rect_filled(badge, cr, accent);
    painter.rect_stroke(
        badge,
        cr,
        Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(0, 0, 0, 55)),
        egui::StrokeKind::Inside,
    );

    let c = badge.center();
    let u = side * 0.30;
    let sw = (side * 0.075).clamp(1.2, 2.4);
    let white = Color32::WHITE;
    let st = Stroke::new(sw, white);
    let thin = Stroke::new((sw * 0.85).max(1.0), white);

    match icon {
        SymbolIcon::AwsLambda => {
            painter.line_segment([Pos2::new(c.x - u * 0.45, c.y - u * 0.9), Pos2::new(c.x + u * 0.75, c.y + u * 0.9)], st);
            painter.line_segment([Pos2::new(c.x + u * 0.05, c.y - u * 0.1), Pos2::new(c.x - u * 0.75, c.y + u * 0.9)], st);
            painter.line_segment([Pos2::new(c.x - u * 0.75, c.y - u * 0.9), Pos2::new(c.x - u * 0.35, c.y - u * 0.9)], st);
        }
        SymbolIcon::AwsS3
        | SymbolIcon::AwsGlacier
        | SymbolIcon::AwsBackup
        | SymbolIcon::GcpCloudStorage
        | SymbolIcon::AzureBlob
        | SymbolIcon::AzureDataLake => {
            let pts = vec![
                Pos2::new(c.x - u * 0.78, c.y - u * 0.55),
                Pos2::new(c.x + u * 0.78, c.y - u * 0.55),
                Pos2::new(c.x + u * 0.56, c.y + u * 0.78),
                Pos2::new(c.x - u * 0.56, c.y + u * 0.78),
            ];
            painter.add(egui::Shape::closed_line(pts, st));
            painter.line_segment([Pos2::new(c.x - u * 0.65, c.y - u * 0.15), Pos2::new(c.x + u * 0.65, c.y - u * 0.15)], thin);
            painter.line_segment([Pos2::new(c.x - u * 0.45, c.y - u * 0.85), Pos2::new(c.x + u * 0.45, c.y - u * 0.85)], thin);
        }
        SymbolIcon::AwsEbs | SymbolIcon::AwsEfs | SymbolIcon::AzureFiles | SymbolIcon::AzureDisk => {
            let folder = Rect::from_center_size(c, Vec2::new(u * 1.6, u * 1.25));
            painter.rect_stroke(folder, 2.0, st, egui::StrokeKind::Inside);
            painter.line_segment([Pos2::new(folder.min.x, c.y - u * 0.1), Pos2::new(folder.max.x, c.y - u * 0.1)], thin);
            painter.line_segment([Pos2::new(folder.min.x, c.y + u * 0.3), Pos2::new(folder.max.x, c.y + u * 0.3)], thin);
        }
        SymbolIcon::AwsEc2
        | SymbolIcon::AwsLightsail
        | SymbolIcon::AwsBatch
        | SymbolIcon::AwsOutposts
        | SymbolIcon::AzureVm
        | SymbolIcon::AzureAppService
        | SymbolIcon::AzureSpringApps => {
            let chip = Rect::from_center_size(c, Vec2::splat(u * 1.25));
            painter.rect_stroke(chip, 1.5, st, egui::StrokeKind::Inside);
            for t in [-0.35_f32, 0.35] {
                painter.line_segment([Pos2::new(c.x + u * t, chip.min.y - u * 0.35), Pos2::new(c.x + u * t, chip.min.y)], thin);
                painter.line_segment([Pos2::new(c.x + u * t, chip.max.y), Pos2::new(c.x + u * t, chip.max.y + u * 0.35)], thin);
                painter.line_segment([Pos2::new(chip.min.x - u * 0.35, c.y + u * t), Pos2::new(chip.min.x, c.y + u * t)], thin);
                painter.line_segment([Pos2::new(chip.max.x, c.y + u * t), Pos2::new(chip.max.x + u * 0.35, c.y + u * t)], thin);
            }
        }
        SymbolIcon::AwsEcs
        | SymbolIcon::AwsEcr
        | SymbolIcon::AwsFargate
        | SymbolIcon::AwsAppRunner
        | SymbolIcon::AwsElasticBeanstalk
        | SymbolIcon::AzureContainerApps
        | SymbolIcon::AzureContainerInstances => {
            for (dx, dy) in [(-0.42_f32, -0.35), (0.42, -0.35), (-0.42, 0.35), (0.42, 0.35)] {
                let r = Rect::from_center_size(Pos2::new(c.x + u * dx, c.y + u * dy), Vec2::splat(u * 0.62));
                painter.rect_stroke(r, 1.2, st, egui::StrokeKind::Inside);
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
            let db = Rect::from_center_size(c, Vec2::new(u * 1.45, u * 1.55));
            painter.rect_stroke(db, u * 0.35, st, egui::StrokeKind::Inside);
            painter.line_segment([Pos2::new(db.min.x + 1.0, c.y - u * 0.25), Pos2::new(db.max.x - 1.0, c.y - u * 0.25)], thin);
            painter.line_segment([Pos2::new(db.min.x + 1.0, c.y + u * 0.25), Pos2::new(db.max.x - 1.0, c.y + u * 0.25)], thin);
        }
        SymbolIcon::AwsDynamoDb | SymbolIcon::AzureFunctions => {
            let bolt = vec![
                Pos2::new(c.x + u * 0.15, c.y - u * 0.92),
                Pos2::new(c.x - u * 0.58, c.y + u * 0.08),
                Pos2::new(c.x + u * 0.05, c.y + u * 0.08),
                Pos2::new(c.x - u * 0.18, c.y + u * 0.92),
                Pos2::new(c.x + u * 0.60, c.y - u * 0.12),
                Pos2::new(c.x - u * 0.02, c.y - u * 0.12),
            ];
            painter.add(egui::Shape::convex_polygon(bolt, white, Stroke::NONE));
        }
        SymbolIcon::AwsApiGateway
        | SymbolIcon::AwsAppSync
        | SymbolIcon::AwsAmplify
        | SymbolIcon::AzureApiManagement
        | SymbolIcon::Api => {
            painter.line_segment([Pos2::new(c.x - u * 0.85, c.y - u * 0.45), Pos2::new(c.x - u * 0.35, c.y - u * 0.85)], st);
            painter.line_segment([Pos2::new(c.x - u * 0.85, c.y - u * 0.45), Pos2::new(c.x - u * 0.35, c.y)], st);
            painter.line_segment([Pos2::new(c.x + u * 0.85, c.y + u * 0.45), Pos2::new(c.x + u * 0.35, c.y)], st);
            painter.line_segment([Pos2::new(c.x + u * 0.85, c.y + u * 0.45), Pos2::new(c.x + u * 0.35, c.y + u * 0.85)], st);
            painter.line_segment([Pos2::new(c.x - u * 0.2, c.y + u * 0.7), Pos2::new(c.x + u * 0.2, c.y - u * 0.7)], st);
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
            for (dx, dy) in [(-0.55_f32, -0.55), (0.55, -0.55), (-0.55, 0.55), (0.55, 0.55)] {
                let p = Pos2::new(c.x + u * dx, c.y + u * dy);
                painter.line_segment([c, p], thin);
                painter.circle_filled(p, (u * 0.24).max(1.8), white);
            }
            painter.circle_filled(c, (u * 0.30).max(2.2), white);
        }
        SymbolIcon::AwsStepFunctions
        | SymbolIcon::AwsCodePipeline
        | SymbolIcon::AwsCodeBuild
        | SymbolIcon::AwsCloudFormation
        | SymbolIcon::AwsSsm
        | SymbolIcon::AwsOrganizations
        | SymbolIcon::AzureLogicApps
        | SymbolIcon::AzureDevOps => {
            let b0 = Rect::from_center_size(Pos2::new(c.x - u * 0.52, c.y - u * 0.45), Vec2::splat(u * 0.55));
            let b1 = Rect::from_center_size(Pos2::new(c.x + u * 0.52, c.y), Vec2::splat(u * 0.55));
            let b2 = Rect::from_center_size(Pos2::new(c.x - u * 0.52, c.y + u * 0.45), Vec2::splat(u * 0.55));
            painter.rect_stroke(b0, 1.0, st, egui::StrokeKind::Inside);
            painter.rect_stroke(b1, 1.0, st, egui::StrokeKind::Inside);
            painter.rect_stroke(b2, 1.0, st, egui::StrokeKind::Inside);
            painter.line_segment([b0.right_center(), b1.left_center()], thin);
            painter.line_segment([b1.left_center(), b2.right_center()], thin);
        }
        SymbolIcon::AwsCloudFront
        | SymbolIcon::AwsRoute53
        | SymbolIcon::AwsGlobalAccelerator
        | SymbolIcon::AzureFrontDoor
        | SymbolIcon::AzureDns
        | SymbolIcon::Cdn
        | SymbolIcon::Globe => {
            let rad = u * 0.88;
            painter.circle_stroke(c, rad, st);
            painter.line_segment([Pos2::new(c.x - rad, c.y), Pos2::new(c.x + rad, c.y)], thin);
            painter.line_segment([Pos2::new(c.x, c.y - rad), Pos2::new(c.x, c.y + rad)], thin);
            painter.circle_stroke(c, rad * 0.48, thin);
        }
        SymbolIcon::AwsVpc
        | SymbolIcon::AwsDirectConnect
        | SymbolIcon::AwsTransitGateway
        | SymbolIcon::AwsPrivateLink
        | SymbolIcon::AwsNatGateway
        | SymbolIcon::AzureVnet => {
            let outer = Rect::from_center_size(c, Vec2::new(u * 1.75, u * 1.35));
            painter.rect_stroke(outer, 2.5, st, egui::StrokeKind::Inside);
            painter.line_segment([Pos2::new(c.x, outer.min.y), Pos2::new(c.x, outer.max.y)], thin);
            painter.circle_filled(Pos2::new(c.x - u * 0.42, c.y), (u * 0.18).max(1.5), white);
            painter.circle_filled(Pos2::new(c.x + u * 0.42, c.y), (u * 0.18).max(1.5), white);
        }
        SymbolIcon::AwsWaf | SymbolIcon::AwsShield | SymbolIcon::AwsGuardDuty | SymbolIcon::AzureFirewall => {
            let sh = vec![
                Pos2::new(c.x, c.y - u * 0.88),
                Pos2::new(c.x + u * 0.72, c.y - u * 0.52),
                Pos2::new(c.x + u * 0.58, c.y + u * 0.42),
                Pos2::new(c.x, c.y + u * 0.88),
                Pos2::new(c.x - u * 0.58, c.y + u * 0.42),
                Pos2::new(c.x - u * 0.72, c.y - u * 0.52),
            ];
            painter.add(egui::Shape::closed_line(sh, st));
            painter.line_segment([Pos2::new(c.x - u * 0.35, c.y), Pos2::new(c.x + u * 0.35, c.y)], thin);
        }
        SymbolIcon::GcpCloudRun => {
            let pts = vec![
                Pos2::new(c.x - u * 0.55, c.y - u * 0.78),
                Pos2::new(c.x + u * 0.75, c.y),
                Pos2::new(c.x - u * 0.55, c.y + u * 0.78),
                Pos2::new(c.x - u * 0.12, c.y),
            ];
            painter.add(egui::Shape::convex_polygon(pts, white, Stroke::NONE));
        }
        SymbolIcon::AwsEks | SymbolIcon::GcpGke | SymbolIcon::AzureAks | SymbolIcon::Kubernetes => {
            let n = 7usize;
            let rad = u * 0.85;
            let mut pts = Vec::with_capacity(n);
            for i in 0..n {
                let a = (i as f32 / n as f32) * std::f32::consts::TAU - std::f32::consts::FRAC_PI_2;
                let p = Pos2::new(c.x + a.cos() * rad, c.y + a.sin() * rad);
                pts.push(p);
                painter.line_segment([c, p], thin);
            }
            painter.add(egui::Shape::closed_line(pts, st));
            painter.circle_filled(c, (u * 0.22).max(1.8), white);
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
            for (i, h) in [0.45_f32, 0.85, 0.62].iter().enumerate() {
                let bx = c.x - u * 0.55 + i as f32 * u * 0.55;
                let bar = Rect::from_min_max(
                    Pos2::new(bx - u * 0.18, c.y + u * 0.65 - u * 1.3 * h),
                    Pos2::new(bx + u * 0.18, c.y + u * 0.65),
                );
                painter.rect_filled(bar, 1.0, white);
            }
        }
        SymbolIcon::AzureCosmosDb => {
            painter.circle_filled(c, (u * 0.34).max(2.2), white);
            painter.circle_stroke(c, u * 0.82, st);
            painter.line_segment([Pos2::new(c.x - u * 0.85, c.y - u * 0.35), Pos2::new(c.x + u * 0.85, c.y + u * 0.35)], thin);
        }
        SymbolIcon::AwsKms | SymbolIcon::AwsSecretsManager | SymbolIcon::AzureKeyVault | SymbolIcon::Key => {
            let head = Pos2::new(c.x - u * 0.35, c.y);
            painter.circle_stroke(head, u * 0.42, st);
            painter.line_segment([Pos2::new(head.x + u * 0.42, c.y), Pos2::new(c.x + u * 0.88, c.y)], st);
            painter.line_segment([Pos2::new(c.x + u * 0.55, c.y), Pos2::new(c.x + u * 0.55, c.y + u * 0.35)], st);
            painter.line_segment([Pos2::new(c.x + u * 0.82, c.y), Pos2::new(c.x + u * 0.82, c.y + u * 0.35)], st);
        }
        SymbolIcon::Docker => {
            let box_s = u * 0.42;
            for (gx, gy) in [(-0.55_f32, -0.25), (0.0, -0.25), (0.55, -0.25), (-0.28, -0.72), (0.28, -0.72)] {
                let r = Rect::from_center_size(Pos2::new(c.x + u * gx, c.y + u * gy), Vec2::splat(box_s));
                painter.rect_filled(r, 0.8, white);
            }
            let hull = Rect::from_min_max(
                Pos2::new(c.x - u * 0.88, c.y + u * 0.1),
                Pos2::new(c.x + u * 0.88, c.y + u * 0.68),
            );
            painter.rect_filled(hull, u * 0.25, white);
        }
        SymbolIcon::AwsElastiCache | SymbolIcon::AwsMemoryDb | SymbolIcon::AzureRedis | SymbolIcon::Redis => {
            for dy in [-0.45_f32, 0.0, 0.45] {
                let pts = vec![
                    Pos2::new(c.x, c.y + u * (dy - 0.25)),
                    Pos2::new(c.x + u * 0.82, c.y + u * dy),
                    Pos2::new(c.x, c.y + u * (dy + 0.25)),
                    Pos2::new(c.x - u * 0.82, c.y + u * dy),
                ];
                painter.add(egui::Shape::closed_line(pts, st));
            }
        }
        SymbolIcon::Nginx => {
            painter.line_segment([Pos2::new(c.x - u * 0.58, c.y + u * 0.68), Pos2::new(c.x - u * 0.58, c.y - u * 0.68)], st);
            painter.line_segment([Pos2::new(c.x - u * 0.58, c.y - u * 0.68), Pos2::new(c.x + u * 0.58, c.y + u * 0.68)], st);
            painter.line_segment([Pos2::new(c.x + u * 0.58, c.y + u * 0.68), Pos2::new(c.x + u * 0.58, c.y - u * 0.68)], st);
        }
        SymbolIcon::AwsElb | SymbolIcon::AzureAppGateway | SymbolIcon::AzureLoadBalancer | SymbolIcon::LoadBalancer => {
            let left = Pos2::new(c.x - u * 0.78, c.y);
            painter.circle_filled(left, (u * 0.24).max(1.8), white);
            for dy in [-0.62_f32, 0.0, 0.62] {
                let right = Pos2::new(c.x + u * 0.72, c.y + u * dy);
                painter.line_segment([left, right], thin);
                painter.circle_filled(right, (u * 0.22).max(1.8), white);
            }
        }
        SymbolIcon::AwsCognito | SymbolIcon::AzureEntraId | SymbolIcon::User => {
            painter.circle_filled(Pos2::new(c.x, c.y - u * 0.38), (u * 0.36).max(2.2), white);
            let body = Rect::from_min_max(
                Pos2::new(c.x - u * 0.65, c.y + u * 0.12),
                Pos2::new(c.x + u * 0.65, c.y + u * 0.82),
            );
            painter.rect_filled(body, u * 0.35, white);
        }
        SymbolIcon::Users => {
            for (dx, s) in [(-0.36_f32, 0.82_f32), (0.36, 0.82)] {
                let cx = c.x + u * dx;
                painter.circle_filled(Pos2::new(cx, c.y - u * 0.32), (u * 0.28 * s).max(1.8), white);
                let body = Rect::from_min_max(
                    Pos2::new(cx - u * 0.42, c.y + u * 0.12),
                    Pos2::new(cx + u * 0.42, c.y + u * 0.74),
                );
                painter.rect_filled(body, u * 0.25, white);
            }
        }
        SymbolIcon::AwsIam | SymbolIcon::Lock => {
            let body = Rect::from_min_max(
                Pos2::new(c.x - u * 0.62, c.y - u * 0.08),
                Pos2::new(c.x + u * 0.62, c.y + u * 0.78),
            );
            painter.rect_filled(body, 2.0, white);
            let shackle = Rect::from_min_max(
                Pos2::new(c.x - u * 0.38, c.y - u * 0.72),
                Pos2::new(c.x + u * 0.38, c.y),
            );
            painter.rect_stroke(shackle, u * 0.38, st, egui::StrokeKind::Inside);
        }
        SymbolIcon::Webhook => {
            let p0 = Pos2::new(c.x, c.y - u * 0.62);
            let p1 = Pos2::new(c.x - u * 0.58, c.y + u * 0.48);
            let p2 = Pos2::new(c.x + u * 0.58, c.y + u * 0.48);
            painter.line_segment([p0, p1], st);
            painter.line_segment([p1, p2], st);
            painter.line_segment([p2, p0], st);
            for p in [p0, p1, p2] {
                painter.circle_filled(p, (u * 0.24).max(2.0), white);
            }
        }
        SymbolIcon::Terminal => {
            painter.line_segment([Pos2::new(c.x - u * 0.68, c.y - u * 0.45), Pos2::new(c.x - u * 0.12, c.y)], st);
            painter.line_segment([Pos2::new(c.x - u * 0.12, c.y), Pos2::new(c.x - u * 0.68, c.y + u * 0.45)], st);
            painter.line_segment([Pos2::new(c.x + u * 0.05, c.y + u * 0.45), Pos2::new(c.x + u * 0.72, c.y + u * 0.45)], st);
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
                Pos2::new(c.x, c.y - u * 0.88),
                Pos2::new(c.x + u * 0.26, c.y - u * 0.26),
                Pos2::new(c.x + u * 0.88, c.y),
                Pos2::new(c.x + u * 0.26, c.y + u * 0.26),
                Pos2::new(c.x, c.y + u * 0.88),
                Pos2::new(c.x - u * 0.26, c.y + u * 0.26),
                Pos2::new(c.x - u * 0.88, c.y),
                Pos2::new(c.x - u * 0.26, c.y - u * 0.26),
            ];
            painter.add(egui::Shape::closed_line(star, st));
            painter.circle_filled(c, (u * 0.24).max(2.0), white);
        }
    }
}
