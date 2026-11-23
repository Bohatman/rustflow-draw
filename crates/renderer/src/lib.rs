//! Hardware-accelerated wgpu/WGSL shader descriptors, Bezier/Orthogonal tessellation,
//! GPU instance buffer generation, and headless rasterizer for RustFlow Draw.

pub mod raster;
pub use raster::ExportBgStyle;

use rustflow_core::{
    Edge, EdgeAnimation, EdgeKind, GridStyle, Node, Point, PortSide, Rect, SceneGraph, Viewport,
};

pub const GRID_SHADER_WGSL: &str = r#"
struct GridUniforms {
    pan: vec2<f32>,
    screen_size: vec2<f32>,
    zoom: f32,
    grid_size: f32,
    style: u32,
};

@group(0) @binding(0)
var<uniform> grid: GridUniforms;

@vertex
fn vs_main(@builtin(vertex_index) idx: u32) -> @builtin(position) vec4<f32> {
    var pos = array<vec2<f32>, 3>(
        vec2<f32>(-1.0, -3.0),
        vec2<f32>(-1.0,  1.0),
        vec2<f32>( 3.0,  1.0)
    );
    return vec4<f32>(pos[idx], 0.0, 1.0);
}

@fragment
fn fs_main(@builtin(position) frag_coord: vec4<f32>) -> @location(0) vec4<f32> {
    let world = (frag_coord.xy - grid.pan) / grid.zoom;
    let cell = abs(fract(world / grid.grid_size - 0.5) - 0.5) * grid.grid_size * grid.zoom;
    let d = length(cell);
    let alpha = smoothstep(1.8, 0.8, d);
    return vec4<f32>(0.78, 0.81, 0.86, alpha);
}
"#;

pub const NODE_CARD_SHADER_WGSL: &str = r#"
struct NodeCardInstance {
    rect: vec4<f32>,
    fill_color: vec4<f32>,
    stroke_color: vec4<f32>,
    corner_radius: f32,
    stroke_width: f32,
    shadow_blur: f32,
    opacity: f32,
};

@fragment
fn fs_node_card(
    @location(0) local_pos: vec2<f32>,
    @location(1) half_size: vec2<f32>,
    @location(2) fill_color: vec4<f32>,
    @location(3) stroke_color: vec4<f32>,
    @location(4) params: vec4<f32>,
) -> @location(0) vec4<f32> {
    let q = abs(local_pos) - half_size + vec2<f32>(params.x);
    let sdf = min(max(q.x, q.y), 0.0) + length(max(q, vec2<f32>(0.0))) - params.x;
    let shadow_alpha = smoothstep(params.z, 0.0, sdf) * 0.18;
    let border_mask = smoothstep(0.5, -0.5, abs(sdf) - params.y * 0.5);
    let fill_mask = smoothstep(0.5, -0.5, sdf);
    let base = mix(vec4<f32>(0.0, 0.0, 0.0, shadow_alpha), fill_color, fill_mask);
    return mix(base, stroke_color, border_mask) * params.w;
}
"#;

pub const EDGE_BEZIER_SHADER_WGSL: &str = r#"
struct EdgeFragmentInput {
    @location(0) arc_length: f32,
    @location(1) perp_dist: f32,
    @location(2) color: vec4<f32>,
    @location(3) anim_params: vec4<f32>, // (speed, dash_len, gap_len, time)
};

@fragment
fn fs_edge(in: EdgeFragmentInput) -> @location(0) vec4<f32> {
    let edge_alpha = smoothstep(1.0, 0.2, abs(in.perp_dist));
    let period = in.anim_params.y + in.anim_params.z;
    if (period > 0.001) {
        let phase = fract((in.arc_length - in.anim_params.w * in.anim_params.x) / period) * period;
        let dash_mask = step(phase, in.anim_params.y);
        return vec4<f32>(in.color.rgb, in.color.a * edge_alpha * dash_mask);
    }
    return vec4<f32>(in.color.rgb, in.color.a * edge_alpha);
}
"#;

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GridUniforms {
    pub pan: [f32; 2],
    pub screen_size: [f32; 2],
    pub zoom: f32,
    pub grid_size: f32,
    pub style: u32,
}

impl GridUniforms {
    pub fn from_viewport(vp: &Viewport) -> Self {
        let style = match vp.grid_style {
            GridStyle::Dots => 1,
            GridStyle::Lines => 2,
            GridStyle::Hidden => 0,
        };
        Self {
            pan: [vp.pan.x, vp.pan.y],
            screen_size: [vp.screen_width, vp.screen_height],
            zoom: vp.zoom,
            grid_size: vp.grid_size,
            style,
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NodeInstance {
    pub node_id: u64,
    pub screen_rect: [f32; 4],
    pub fill_color: [f32; 4],
    pub stroke_color: [f32; 4],
    pub corner_radius: f32,
    pub stroke_width: f32,
    pub shadow_blur: f32,
    pub opacity: f32,
    pub rotation_deg: f32,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PortHandleInstance {
    pub node_id: u64,
    pub port_id: u64,
    pub screen_center: [f32; 2],
    pub radius: f32,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EdgeVertex {
    pub position: [f32; 2],
    pub arc_length: f32,
    pub perp_dist: f32,
    pub color: [f32; 4],
    pub anim_params: [f32; 4],
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PulseParticleInstance {
    pub edge_id: u64,
    pub world_center: [f32; 2],
    pub screen_center: [f32; 2],
    pub radius: f32,
    pub color: [f32; 4],
}

/// Compute visible grid dot screen positions for CPU verification and instanced rendering.
pub fn compute_visible_grid_dots(vp: &Viewport) -> Vec<Point> {
    if vp.grid_style == GridStyle::Hidden || vp.grid_size <= 0.0 {
        return Vec::new();
    }
    let world = vp.visible_world_rect();
    let start_x = (world.x / vp.grid_size).floor() as i32;
    let end_x = (world.right() / vp.grid_size).ceil() as i32;
    let start_y = (world.y / vp.grid_size).floor() as i32;
    let end_y = (world.bottom() / vp.grid_size).ceil() as i32;

    let mut dots = Vec::new();
    for gx in start_x..=end_x {
        for gy in start_y..=end_y {
            let world_pt = Point::new(gx as f32 * vp.grid_size, gy as f32 * vp.grid_size);
            let screen_pt = vp.world_to_screen(world_pt);
            if screen_pt.x >= 0.0
                && screen_pt.x <= vp.screen_width
                && screen_pt.y >= 0.0
                && screen_pt.y <= vp.screen_height
            {
                dots.push(screen_pt);
            }
        }
    }
    dots
}

fn port_tangent(side: PortSide, offset: f32) -> Point {
    match side {
        PortSide::Top => Point::new(0.0, -offset),
        PortSide::Right => Point::new(offset, 0.0),
        PortSide::Bottom => Point::new(0.0, offset),
        PortSide::Left => Point::new(-offset, 0.0),
    }
}

/// Evaluate a cubic Bezier curve `B(t)` for `t in 0.0..=1.0`.
pub fn eval_cubic_bezier(p0: Point, p1: Point, p2: Point, p3: Point, t: f32) -> Point {
    let u = 1.0 - t;
    let tt = t * t;
    let uu = u * u;
    let uuu = uu * u;
    let ttt = tt * t;
    Point::new(
        uuu * p0.x + 3.0 * uu * t * p1.x + 3.0 * u * tt * p2.x + ttt * p3.x,
        uuu * p0.y + 3.0 * uu * t * p1.y + 3.0 * u * tt * p2.y + ttt * p3.y,
    )
}

/// Tessellate a cubic Bezier edge between two ports with React Flow control point curvature.
pub fn tessellate_cubic_bezier(
    start: Point,
    end: Point,
    start_side: PortSide,
    end_side: PortSide,
    segments: usize,
) -> Vec<Point> {
    let dist = start.distance_to(end);
    let ctrl_dist = (dist * 0.4).clamp(40.0, 180.0);
    let t1 = port_tangent(start_side, ctrl_dist);
    let t2 = port_tangent(end_side, ctrl_dist);
    let c1 = Point::new(start.x + t1.x, start.y + t1.y);
    let c2 = Point::new(end.x + t2.x, end.y + t2.y);

    let steps = segments.max(4);
    let mut pts = Vec::with_capacity(steps + 1);
    for i in 0..=steps {
        let t = i as f32 / steps as f32;
        pts.push(eval_cubic_bezier(start, c1, c2, end, t));
    }
    pts
}

/// Tessellate a React Flow SmoothStep (rounded orthogonal) path.
pub fn tessellate_smooth_step(start: Point, end: Point, corner_radius: f32) -> Vec<Point> {
    let mid_x = (start.x + end.x) * 0.5;
    let r = corner_radius
        .min((mid_x - start.x).abs() * 0.5)
        .min((end.y - start.y).abs() * 0.5);
    if r < 1.0 {
        return vec![
            start,
            Point::new(mid_x, start.y),
            Point::new(mid_x, end.y),
            end,
        ];
    }
    let dir_x = if end.x >= start.x { 1.0 } else { -1.0 };
    let dir_y = if end.y >= start.y { 1.0 } else { -1.0 };
    vec![
        start,
        Point::new(mid_x - dir_x * r, start.y),
        Point::new(mid_x, start.y + dir_y * r),
        Point::new(mid_x, end.y - dir_y * r),
        Point::new(mid_x + dir_x * r, end.y),
        end,
    ]
}

/// Route an orthogonal edge avoiding obstacle bounding boxes or following manual waypoints.
pub fn route_orthogonal(
    start: Point,
    end: Point,
    waypoints: &[Point],
    obstacles: &[Rect],
) -> Vec<Point> {
    if !waypoints.is_empty() {
        let mut poly = Vec::with_capacity(waypoints.len() + 2);
        poly.push(start);
        for wp in waypoints {
            let prev = *poly.last().unwrap();
            if (prev.x - wp.x).abs() > 1e-3 && (prev.y - wp.y).abs() > 1e-3 {
                poly.push(Point::new(wp.x, prev.y));
            }
            poly.push(*wp);
        }
        let prev = *poly.last().unwrap();
        if (prev.x - end.x).abs() > 1e-3 && (prev.y - end.y).abs() > 1e-3 {
            poly.push(Point::new(end.x, prev.y));
        }
        poly.push(end);
        return poly;
    }

    let mid_x = (start.x + end.x) * 0.5;
    let vertical_seg_bounds = Rect::new(
        mid_x - 2.0,
        start.y.min(end.y),
        4.0,
        (end.y - start.y).abs().max(1.0),
    );
    let blocked = obstacles
        .iter()
        .any(|obs| obs.expand(8.0).intersects(&vertical_seg_bounds));
    if blocked {
        let detour_y = obstacles
            .iter()
            .fold(start.y.min(end.y), |acc, o| acc.min(o.y - 28.0));
        vec![
            start,
            Point::new(start.x, detour_y),
            Point::new(end.x, detour_y),
            end,
        ]
    } else {
        vec![
            start,
            Point::new(mid_x, start.y),
            Point::new(mid_x, end.y),
            end,
        ]
    }
}

fn port_normal(side: PortSide) -> Point {
    match side {
        PortSide::Top => Point::new(0.0, -1.0),
        PortSide::Right => Point::new(1.0, 0.0),
        PortSide::Bottom => Point::new(0.0, 1.0),
        PortSide::Left => Point::new(-1.0, 0.0),
    }
}

fn dedup_polyline(raw: Vec<Point>) -> Vec<Point> {
    let mut out: Vec<Point> = Vec::with_capacity(raw.len());
    for pt in raw {
        if out.last().is_none_or(|prev| prev.distance_to(pt) > 0.5) {
            out.push(pt);
        }
    }
    let mut simplified: Vec<Point> = Vec::with_capacity(out.len());
    for pt in out {
        while simplified.len() >= 2 {
            let a = simplified[simplified.len() - 2];
            let b = simplified[simplified.len() - 1];
            let collinear_x = (a.x - b.x).abs() < 0.5 && (b.x - pt.x).abs() < 0.5;
            let collinear_y = (a.y - b.y).abs() < 0.5 && (b.y - pt.y).abs() < 0.5;
            if collinear_x || collinear_y {
                simplified.pop();
            } else {
                break;
            }
        }
        simplified.push(pt);
    }
    simplified
}

fn round_polyline_corners(pts: &[Point], radius: f32) -> Vec<Point> {
    if pts.len() < 3 || radius <= 0.5 {
        return pts.to_vec();
    }
    let mut out = Vec::with_capacity(pts.len() * 4);
    out.push(pts[0]);
    for i in 1..(pts.len() - 1) {
        let prev = pts[i - 1];
        let curr = pts[i];
        let next = pts[i + 1];
        let d1 = prev.distance_to(curr);
        let d2 = curr.distance_to(next);
        let r = radius.min(d1 * 0.45).min(d2 * 0.45);
        if r < 1.0 {
            out.push(curr);
            continue;
        }
        let p_in = Point::new(
            curr.x + (prev.x - curr.x) * (r / d1),
            curr.y + (prev.y - curr.y) * (r / d1),
        );
        let p_out = Point::new(
            curr.x + (next.x - curr.x) * (r / d2),
            curr.y + (next.y - curr.y) * (r / d2),
        );
        out.push(p_in);
        for step in 1..=4 {
            let t = step as f32 / 4.0;
            let u = 1.0 - t;
            let q = Point::new(
                u * u * p_in.x + 2.0 * u * t * curr.x + t * t * p_out.x,
                u * u * p_in.y + 2.0 * u * t * curr.y + t * t * p_out.y,
            );
            out.push(q);
        }
    }
    out.push(*pts.last().unwrap());
    out
}

fn rect_ray_exit(bounds: &Rect, target: Point) -> Point {
    let c = bounds.center();
    let dx = target.x - c.x;
    let dy = target.y - c.y;
    if dx.abs() < 1e-4 && dy.abs() < 1e-4 {
        return c;
    }
    let hw = (bounds.width * 0.5).max(1.0);
    let hh = (bounds.height * 0.5).max(1.0);
    let tx = if dx.abs() > 1e-4 { hw / dx.abs() } else { f32::INFINITY };
    let ty = if dy.abs() > 1e-4 { hh / dy.abs() } else { f32::INFINITY };
    let t = tx.min(ty);
    Point::new(c.x + dx * t, c.y + dy * t)
}

pub fn tessellate_smooth_step_oriented(
    start: Point,
    end: Point,
    start_side: PortSide,
    end_side: PortSide,
) -> Vec<Point> {
    let stub = 26.0_f32;
    let sn = port_normal(start_side);
    let en = port_normal(end_side);
    let p1 = Point::new(start.x + sn.x * stub, start.y + sn.y * stub);
    let p2 = Point::new(end.x + en.x * stub, end.y + en.y * stub);

    let start_horiz = matches!(start_side, PortSide::Left | PortSide::Right);
    let end_horiz = matches!(end_side, PortSide::Left | PortSide::Right);

    let mut pts = vec![start, p1];
    match (start_horiz, end_horiz) {
        (true, true) => {
            if start_side == end_side {
                let outer_x = if sn.x > 0.0 { p1.x.max(p2.x) } else { p1.x.min(p2.x) };
                pts.push(Point::new(outer_x, p1.y));
                pts.push(Point::new(outer_x, p2.y));
            } else {
                let facing = (sn.x > 0.0 && p2.x >= p1.x) || (sn.x < 0.0 && p2.x <= p1.x);
                if facing {
                    let mid_x = (p1.x + p2.x) * 0.5;
                    pts.push(Point::new(mid_x, p1.y));
                    pts.push(Point::new(mid_x, p2.y));
                } else {
                    let mid_y = (p1.y + p2.y) * 0.5;
                    pts.push(Point::new(p1.x, mid_y));
                    pts.push(Point::new(p2.x, mid_y));
                }
            }
        }
        (false, false) => {
            if start_side == end_side {
                let outer_y = if sn.y > 0.0 { p1.y.max(p2.y) } else { p1.y.min(p2.y) };
                pts.push(Point::new(p1.x, outer_y));
                pts.push(Point::new(p2.x, outer_y));
            } else {
                let facing = (sn.y > 0.0 && p2.y >= p1.y) || (sn.y < 0.0 && p2.y <= p1.y);
                if facing {
                    let mid_y = (p1.y + p2.y) * 0.5;
                    pts.push(Point::new(p1.x, mid_y));
                    pts.push(Point::new(p2.x, mid_y));
                } else {
                    let mid_x = (p1.x + p2.x) * 0.5;
                    pts.push(Point::new(mid_x, p1.y));
                    pts.push(Point::new(mid_x, p2.y));
                }
            }
        }
        (true, false) => {
            let clear_x = (sn.x > 0.0 && p2.x >= p1.x) || (sn.x < 0.0 && p2.x <= p1.x);
            let clear_y = (en.y > 0.0 && p1.y >= p2.y) || (en.y < 0.0 && p1.y <= p2.y);
            if clear_x && clear_y {
                pts.push(Point::new(p2.x, p1.y));
            } else {
                let mid_x = (p1.x + p2.x) * 0.5;
                let mid_y = (p1.y + p2.y) * 0.5;
                pts.push(Point::new(p1.x, mid_y));
                pts.push(Point::new(mid_x, mid_y));
                pts.push(Point::new(mid_x, p2.y));
            }
        }
        (false, true) => {
            let clear_y = (sn.y > 0.0 && p2.y >= p1.y) || (sn.y < 0.0 && p2.y <= p1.y);
            let clear_x = (en.x > 0.0 && p1.x >= p2.x) || (en.x < 0.0 && p1.x <= p2.x);
            if clear_y && clear_x {
                pts.push(Point::new(p1.x, p2.y));
            } else {
                let mid_x = (p1.x + p2.x) * 0.5;
                let mid_y = (p1.y + p2.y) * 0.5;
                pts.push(Point::new(mid_x, p1.y));
                pts.push(Point::new(mid_x, mid_y));
                pts.push(Point::new(p2.x, mid_y));
            }
        }
    }
    pts.push(p2);
    pts.push(end);
    dedup_polyline(pts)
}

/// Tessellate an `Edge` into a world-space polyline according to its `EdgeKind`.
pub fn tessellate_edge_path(scene: &SceneGraph, edge: &Edge) -> Vec<Point> {
    let Some((start, end, start_side, end_side)) = scene.edge_endpoints(edge) else {
        return Vec::new();
    };
    match edge.kind {
        EdgeKind::Straight => {
            let s = if edge.source_port.is_some() {
                start
            } else if let (Some(sn), Some(dn)) = (
                scene.get_node(edge.source_node),
                scene.get_node(edge.target_node),
            ) {
                rect_ray_exit(&sn.bounds, dn.bounds.center())
            } else {
                start
            };
            let e = if edge.target_port.is_some() {
                end
            } else if let (Some(sn), Some(dn)) = (
                scene.get_node(edge.source_node),
                scene.get_node(edge.target_node),
            ) {
                rect_ray_exit(&dn.bounds, sn.bounds.center())
            } else {
                end
            };
            let mut pts = Vec::with_capacity(edge.waypoints.len() + 2);
            pts.push(s);
            pts.extend_from_slice(&edge.waypoints);
            pts.push(e);
            dedup_polyline(pts)
        }
        EdgeKind::Bezier => {
            if !edge.waypoints.is_empty() {
                let mut ctrl = Vec::with_capacity(edge.waypoints.len() + 2);
                ctrl.push(start);
                ctrl.extend_from_slice(&edge.waypoints);
                ctrl.push(end);
                if ctrl.len() == 3 {
                    let p0 = ctrl[0];
                    let mid = ctrl[1];
                    let p2 = ctrl[2];
                    let cp = Point::new(
                        2.0 * mid.x - 0.5 * (p0.x + p2.x),
                        2.0 * mid.y - 0.5 * (p0.y + p2.y),
                    );
                    let mut pts = Vec::with_capacity(25);
                    for i in 0..=24 {
                        let t = i as f32 / 24.0;
                        let u = 1.0 - t;
                        pts.push(Point::new(
                            u * u * p0.x + 2.0 * u * t * cp.x + t * t * p2.x,
                            u * u * p0.y + 2.0 * u * t * cp.y + t * t * p2.y,
                        ));
                    }
                    dedup_polyline(pts)
                } else {
                    let mut pts = Vec::with_capacity(ctrl.len() * 12);
                    for i in 0..(ctrl.len() - 1) {
                        let p0 = if i == 0 { ctrl[0] } else { ctrl[i - 1] };
                        let p1 = ctrl[i];
                        let p2 = ctrl[i + 1];
                        let p3 = if i + 2 < ctrl.len() {
                            ctrl[i + 2]
                        } else {
                            ctrl[ctrl.len() - 1]
                        };
                        let steps = 12usize;
                        let start_s = if i == 0 { 0 } else { 1 };
                        for s in start_s..=steps {
                            let t = s as f32 / steps as f32;
                            let t2 = t * t;
                            let t3 = t2 * t;
                            let x = 0.5
                                * ((2.0 * p1.x)
                                    + (-p0.x + p2.x) * t
                                    + (2.0 * p0.x - 5.0 * p1.x + 4.0 * p2.x - p3.x) * t2
                                    + (-p0.x + 3.0 * p1.x - 3.0 * p2.x + p3.x) * t3);
                            let y = 0.5
                                * ((2.0 * p1.y)
                                    + (-p0.y + p2.y) * t
                                    + (2.0 * p0.y - 5.0 * p1.y + 4.0 * p2.y - p3.y) * t2
                                    + (-p0.y + 3.0 * p1.y - 3.0 * p2.y + p3.y) * t3);
                            pts.push(Point::new(x, y));
                        }
                    }
                    dedup_polyline(pts)
                }
            } else {
                dedup_polyline(tessellate_cubic_bezier(start, end, start_side, end_side, 24))
            }
        }
        EdgeKind::SmoothStep => {
            let base = if !edge.waypoints.is_empty() {
                dedup_polyline(route_orthogonal(start, end, &edge.waypoints, &[]))
            } else {
                tessellate_smooth_step_oriented(start, end, start_side, end_side)
            };
            round_polyline_corners(&base, 14.0)
        }
        EdgeKind::Orthogonal => {
            if !edge.waypoints.is_empty() {
                dedup_polyline(route_orthogonal(start, end, &edge.waypoints, &[]))
            } else {
                tessellate_smooth_step_oriented(start, end, start_side, end_side)
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct EdgeLabelPose {
    pub world_center: Point,
    pub angle_rad: f32,
}

pub fn resolve_smart_edge_label_pose(
    scene: &SceneGraph,
    pts: &[Point],
    placement: rustflow_core::EdgeLabelPlacement,
    preferred_t: f32,
    offset_y: f32,
    label_world_w: f32,
    label_world_h: f32,
    placed_labels: &mut Vec<Rect>,
) -> Option<EdgeLabelPose> {
    if pts.len() < 2 {
        return None;
    }

    let mut total_len = 0.0_f32;
    let mut best_seg_t = 0.5_f32;
    let mut best_seg_len = -1.0_f32;
    for w in pts.windows(2) {
        total_len += w[0].distance_to(w[1]);
    }
    if total_len > 1.0 {
        let mut acc = 0.0_f32;
        for w in pts.windows(2) {
            let sl = w[0].distance_to(w[1]);
            if sl > best_seg_len {
                best_seg_len = sl;
                best_seg_t = ((acc + sl * 0.5) / total_len).clamp(0.12, 0.88);
            }
            acc += sl;
        }
    }

    let pref = preferred_t.clamp(0.08, 0.92);
    let mut candidate_ts = vec![pref];
    if (pref - 0.5).abs() < 0.06 {
        candidate_ts.push(best_seg_t);
    }
    for t in [0.5, 0.36, 0.64, 0.24, 0.76, 0.16, 0.84] {
        if candidate_ts.iter().all(|c| (c - t).abs() > 0.04) {
            candidate_ts.push(t);
        }
    }

    let obstacles: Vec<Rect> = scene
        .nodes()
        .iter()
        .filter(|n| n.visible && !n.is_section)
        .map(|n| n.bounds.expand(4.0))
        .collect();

    let mut best_pose: Option<(f32, EdgeLabelPose, Rect)> = None;

    for &t in &candidate_ts {
        let Some(base_pt) = sample_polyline_at(pts, t) else {
            continue;
        };
        let p_prev = sample_polyline_at(pts, (t - 0.025).max(0.0)).unwrap_or(base_pt);
        let p_next = sample_polyline_at(pts, (t + 0.025).min(1.0)).unwrap_or(base_pt);
        let dx = p_next.x - p_prev.x;
        let dy = p_next.y - p_prev.y;
        let dlen = (dx * dx + dy * dy).sqrt().max(1e-4);
        let mut tx = dx / dlen;
        let mut ty = dy / dlen;
        if tx < 0.0 || (tx.abs() < 0.05 && ty > 0.0) {
            tx = -tx;
            ty = -ty;
        }
        let angle = ty.atan2(tx);
        let nx = ty;
        let ny = -tx;

        let p_before = sample_polyline_at(pts, (t - 0.06).max(0.0)).unwrap_or(p_prev);
        let p_after = sample_polyline_at(pts, (t + 0.06).min(1.0)).unwrap_or(p_next);
        let straight_dist = p_before.distance_to(p_after);
        let curve_dist = p_before.distance_to(base_pt) + base_pt.distance_to(p_after);
        let bend_penalty = ((curve_dist - straight_dist).max(0.0) * 3.0).min(80.0);

        let side_signs: &[f32] = match placement {
            rustflow_core::EdgeLabelPlacement::Center => &[0.0, 1.0, -1.0],
            rustflow_core::EdgeLabelPlacement::Above => &[1.0, -1.0],
            rustflow_core::EdgeLabelPlacement::Below => &[-1.0, 1.0],
            rustflow_core::EdgeLabelPlacement::AlongPath => &[1.0, -1.0],
        };

        for (s_idx, &sign) in side_signs.iter().enumerate() {
            let perp_dist = if sign == 0.0 {
                0.0
            } else {
                let half_proj = (label_world_w * 0.5 * nx.abs()) + (label_world_h * 0.5 * ny.abs());
                half_proj + 7.0
            };
            let cx = base_pt.x + nx * perp_dist * sign;
            let cy = base_pt.y + ny * perp_dist * sign + offset_y;
            let lrect = Rect::new(
                cx - label_world_w * 0.5,
                cy - label_world_h * 0.5,
                label_world_w,
                label_world_h,
            );

            let mut cost = (t - pref).abs() * 35.0 + bend_penalty + (s_idx as f32) * 18.0;
            for obs in &obstacles {
                if lrect.intersects(obs) {
                    cost += 1200.0;
                }
            }
            for other in placed_labels.iter() {
                if lrect.expand(3.0).intersects(other) {
                    cost += 900.0;
                }
            }

            if best_pose.is_none_or(|(bc, ..)| cost < bc) {
                best_pose = Some((
                    cost,
                    EdgeLabelPose {
                        world_center: Point::new(cx, cy),
                        angle_rad: angle,
                    },
                    lrect,
                ));
            }
        }
    }

    if let Some((_, pose, lrect)) = best_pose {
        placed_labels.push(lrect);
        Some(pose)
    } else {
        None
    }
}

/// Sample a point at normalized arc-length `t in 0.0..=1.0` along a polyline.
pub fn sample_polyline_at(points: &[Point], t: f32) -> Option<Point> {
    if points.is_empty() {
        return None;
    }
    if points.len() == 1 {
        return Some(points[0]);
    }
    let mut lengths = Vec::with_capacity(points.len() - 1);
    let mut total = 0.0;
    for pair in points.windows(2) {
        let seg = pair[0].distance_to(pair[1]);
        lengths.push(seg);
        total += seg;
    }
    if total <= 1e-5 {
        return Some(points[0]);
    }
    let target = t.clamp(0.0, 1.0) * total;
    let mut acc = 0.0;
    for (i, &seg_len) in lengths.iter().enumerate() {
        if acc + seg_len >= target || i + 1 == lengths.len() {
            let local_t = if seg_len > 1e-5 {
                ((target - acc) / seg_len).clamp(0.0, 1.0)
            } else {
                0.0
            };
            return Some(points[i].lerp(points[i + 1], local_t));
        }
        acc += seg_len;
    }
    points.last().copied()
}

/// Expand a polyline into a GPU triangle-strip vertex buffer carrying arc-length and animation uniforms.
pub fn build_edge_strip_vertices(
    points: &[Point],
    vp: &Viewport,
    edge: &Edge,
    time_seconds: f32,
) -> Vec<EdgeVertex> {
    if points.len() < 2 {
        return Vec::new();
    }
    let half_w = (edge.stroke_width * vp.zoom * 0.5).max(1.0);
    let anim_params = match edge.animation {
        EdgeAnimation::MarchingAnts {
            speed,
            dash_length,
            gap_length,
            reverse,
        } => {
            let dir_speed = if reverse { -speed } else { speed };
            [dir_speed, dash_length, gap_length, time_seconds]
        }
        _ => [0.0, 0.0, 0.0, time_seconds],
    };

    let mut vertices = Vec::with_capacity(points.len() * 2);
    let mut arc = 0.0;
    for i in 0..points.len() {
        if i > 0 {
            arc += points[i - 1].distance_to(points[i]);
        }
        let p_screen = vp.world_to_screen(points[i]);
        let next_idx = if i + 1 < points.len() { i + 1 } else { i };
        let prev_idx = if i > 0 { i - 1 } else { 0 };
        let p_prev = vp.world_to_screen(points[prev_idx]);
        let p_next = vp.world_to_screen(points[next_idx]);
        let dx = p_next.x - p_prev.x;
        let dy = p_next.y - p_prev.y;
        let len = (dx * dx + dy * dy).sqrt().max(1e-4);
        let nx = -dy / len;
        let ny = dx / len;

        vertices.push(EdgeVertex {
            position: [p_screen.x + nx * half_w, p_screen.y + ny * half_w],
            arc_length: arc,
            perp_dist: 1.0,
            color: edge.stroke.to_array(),
            anim_params,
        });
        vertices.push(EdgeVertex {
            position: [p_screen.x - nx * half_w, p_screen.y - ny * half_w],
            arc_length: arc,
            perp_dist: -1.0,
            color: edge.stroke.to_array(),
            anim_params,
        });
    }
    vertices
}

/// Complete GPU render batch for one frame after R-Tree frustum culling.
#[derive(Debug, Clone)]
pub struct RenderFrame {
    pub grid: GridUniforms,
    pub node_instances: Vec<NodeInstance>,
    pub port_instances: Vec<PortHandleInstance>,
    pub edge_vertices: Vec<EdgeVertex>,
    pub pulse_particles: Vec<PulseParticleInstance>,
}

impl RenderFrame {
    pub fn build(scene: &SceneGraph, vp: &Viewport, time_seconds: f32) -> Self {
        let frustum = vp.visible_world_rect();
        let visible_nodes = scene.query_visible_nodes(&frustum);
        let visible_edges = scene.query_visible_edges(&frustum);

        let mut node_instances = Vec::with_capacity(visible_nodes.len());
        let mut port_instances = Vec::with_capacity(visible_nodes.len() * 4);
        for node in visible_nodes {
            node_instances.push(Self::node_to_instance(node, vp));
            for port in &node.ports {
                let world_pt = port.world_position(&node.bounds);
                let screen_pt = vp.world_to_screen(world_pt);
                port_instances.push(PortHandleInstance {
                    node_id: node.id,
                    port_id: port.id,
                    screen_center: [screen_pt.x, screen_pt.y],
                    radius: (4.5 * vp.zoom).clamp(3.0, 10.0),
                });
            }
        }

        let mut edge_vertices = Vec::new();
        let mut pulse_particles = Vec::new();
        for edge in visible_edges {
            let pts = tessellate_edge_path(scene, edge);
            edge_vertices.extend(build_edge_strip_vertices(&pts, vp, edge, time_seconds));

            if let EdgeAnimation::PulseParticles {
                speed,
                particle_count,
                radius,
                color,
            } = edge.animation
            {
                let count = particle_count.max(1);
                for idx in 0..count {
                    let phase_offset = idx as f32 / count as f32;
                    let t = (time_seconds * speed + phase_offset).rem_euclid(1.0);
                    if let Some(world_pt) = sample_polyline_at(&pts, t) {
                        let screen_pt = vp.world_to_screen(world_pt);
                        pulse_particles.push(PulseParticleInstance {
                            edge_id: edge.id,
                            world_center: [world_pt.x, world_pt.y],
                            screen_center: [screen_pt.x, screen_pt.y],
                            radius: radius * vp.zoom,
                            color: color.to_array(),
                        });
                    }
                }
            }
        }

        Self {
            grid: GridUniforms::from_viewport(vp),
            node_instances,
            port_instances,
            edge_vertices,
            pulse_particles,
        }
    }

    fn node_to_instance(node: &Node, vp: &Viewport) -> NodeInstance {
        let top_left = vp.world_to_screen(Point::new(node.bounds.x, node.bounds.y));
        NodeInstance {
            node_id: node.id,
            screen_rect: [
                top_left.x,
                top_left.y,
                node.bounds.width * vp.zoom,
                node.bounds.height * vp.zoom,
            ],
            fill_color: node.style.fill.to_array(),
            stroke_color: node.style.stroke.to_array(),
            corner_radius: node.style.corner_radius * vp.zoom,
            stroke_width: node.style.stroke_width * vp.zoom,
            shadow_blur: node.style.shadow_blur * vp.zoom,
            opacity: node.style.opacity,
            rotation_deg: node.rotation_deg,
        }
    }
}

/// Deterministic RGBA8 headless framebuffer for Visual Regression Testing (VRT) and raster export.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HeadlessFramebuffer {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<[u8; 4]>,
}

impl HeadlessFramebuffer {
    pub fn new(width: u32, height: u32, clear_rgba: [u8; 4]) -> Self {
        Self {
            width,
            height,
            pixels: vec![clear_rgba; (width * height) as usize],
        }
    }

    pub fn set_pixel(&mut self, x: i32, y: i32, color: [u8; 4]) {
        if x >= 0 && y >= 0 && (x as u32) < self.width && (y as u32) < self.height {
            let idx = (y as u32 * self.width + x as u32) as usize;
            self.pixels[idx] = color;
        }
    }

    pub fn get_pixel(&self, x: u32, y: u32) -> [u8; 4] {
        self.pixels[(y * self.width + x) as usize]
    }

    pub fn render_scene(&mut self, scene: &SceneGraph, vp: &Viewport, time_seconds: f32) {
        self.render_scene_with_bg(scene, vp, time_seconds, ExportBgStyle::Dots);
    }

    /// FNV-1a 64-bit deterministic checksum for golden snapshot tests.
    pub fn checksum(&self) -> u64 {
        let mut hash: u64 = 0xcbf29ce484222325;
        for px in &self.pixels {
            for &b in px {
                hash ^= b as u64;
                hash = hash.wrapping_mul(0x100000001b3);
            }
        }
        hash
    }

    /// Compute normalized pixel difference ratio (`0.0..=1.0`) against a golden reference.
    pub fn pixel_diff_ratio(&self, other: &HeadlessFramebuffer) -> f32 {
        if self.width != other.width || self.height != other.height || self.pixels.is_empty() {
            return 1.0;
        }
        let diff_count = self
            .pixels
            .iter()
            .zip(other.pixels.iter())
            .filter(|(a, b)| a != b)
            .count();
        diff_count as f32 / self.pixels.len() as f32
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rustflow_core::ColorRgba;

    #[test]
    fn milestone1_bezier_tessellation_and_render_frame() {
        let mut scene = SceneGraph::new();
        scene.insert_node(Node::new_react_flow_card(
            1,
            "Source Node",
            Rect::new(40.0, 40.0, 120.0, 60.0),
        ));
        scene.insert_node(Node::new_react_flow_card(
            2,
            "Target Node",
            Rect::new(260.0, 120.0, 120.0, 60.0),
        ));
        let mut edge = Edge::new_bezier(10, 1, 2);
        edge.animation = EdgeAnimation::PulseParticles {
            speed: 0.5,
            particle_count: 2,
            radius: 4.0,
            color: ColorRgba::INDIGO_500,
        };
        scene.insert_edge(edge);

        let vp = Viewport::new(640.0, 360.0);
        let frame = RenderFrame::build(&scene, &vp, 0.25);
        assert_eq!(frame.node_instances.len(), 2);
        assert!(frame.port_instances.len() >= 8);
        assert!(!frame.edge_vertices.is_empty());
        assert_eq!(frame.pulse_particles.len(), 2);
        assert!(NODE_CARD_SHADER_WGSL.contains("fs_node_card"));
        assert!(EDGE_BEZIER_SHADER_WGSL.contains("fs_edge"));
    }
}
