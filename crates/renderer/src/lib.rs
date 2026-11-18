//! Hardware-accelerated wgpu/WGSL shader descriptors, Bezier/Orthogonal tessellation,
//! GPU instance buffer generation, and headless rasterizer for RustFlow Draw.

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

/// Tessellate an `Edge` into a world-space polyline according to its `EdgeKind`.
pub fn tessellate_edge_path(scene: &SceneGraph, edge: &Edge) -> Vec<Point> {
    let Some((start, end, start_side, end_side)) = scene.edge_endpoints(edge) else {
        return Vec::new();
    };
    match edge.kind {
        EdgeKind::Straight => {
            let mut pts = Vec::with_capacity(edge.waypoints.len() + 2);
            pts.push(start);
            pts.extend_from_slice(&edge.waypoints);
            pts.push(end);
            pts
        }
        EdgeKind::Bezier => {
            if let Some(&via) = edge.waypoints.first() {
                let mut pts = Vec::with_capacity(25);
                for i in 0..=24 {
                    let t = i as f32 / 24.0;
                    pts.push(eval_cubic_bezier(start, via, via, end, t));
                }
                pts
            } else {
                tessellate_cubic_bezier(start, end, start_side, end_side, 24)
            }
        }
        EdgeKind::SmoothStep => {
            if !edge.waypoints.is_empty() {
                route_orthogonal(start, end, &edge.waypoints, &[])
            } else {
                tessellate_smooth_step(start, end, 12.0)
            }
        }
        EdgeKind::Orthogonal => {
            let obstacles: Vec<Rect> = scene
                .nodes()
                .iter()
                .filter(|n| n.id != edge.source_node && n.id != edge.target_node)
                .map(|n| n.bounds)
                .collect();
            route_orthogonal(start, end, &edge.waypoints, &obstacles)
        }
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
        // 1. Render pan-zoom dot grid
        for dot in compute_visible_grid_dots(vp) {
            self.set_pixel(
                dot.x.round() as i32,
                dot.y.round() as i32,
                [199, 206, 219, 255],
            );
        }

        let frame = RenderFrame::build(scene, vp, time_seconds);

        // 2. Render React Flow node cards (filled rect + border)
        for node in &frame.node_instances {
            let x0 = node.screen_rect[0].round() as i32;
            let y0 = node.screen_rect[1].round() as i32;
            let x1 = (node.screen_rect[0] + node.screen_rect[2]).round() as i32;
            let y1 = (node.screen_rect[1] + node.screen_rect[3]).round() as i32;
            let fill = [
                (node.fill_color[0] * 255.0) as u8,
                (node.fill_color[1] * 255.0) as u8,
                (node.fill_color[2] * 255.0) as u8,
                255,
            ];
            let border = [
                (node.stroke_color[0] * 255.0) as u8,
                (node.stroke_color[1] * 255.0) as u8,
                (node.stroke_color[2] * 255.0) as u8,
                255,
            ];
            for y in y0..=y1 {
                for x in x0..=x1 {
                    let on_border = x == x0 || x == x1 || y == y0 || y == y1;
                    self.set_pixel(x, y, if on_border { border } else { fill });
                }
            }
        }

        // 3. Render edge polylines
        let frustum = vp.visible_world_rect();
        for edge in scene.query_visible_edges(&frustum) {
            let pts = tessellate_edge_path(scene, edge);
            let color = [
                (edge.stroke.r * 255.0) as u8,
                (edge.stroke.g * 255.0) as u8,
                (edge.stroke.b * 255.0) as u8,
                255,
            ];
            for seg in pts.windows(2) {
                let s0 = vp.world_to_screen(seg[0]);
                let s1 = vp.world_to_screen(seg[1]);
                self.draw_line(
                    s0.x.round() as i32,
                    s0.y.round() as i32,
                    s1.x.round() as i32,
                    s1.y.round() as i32,
                    color,
                );
            }
        }

        // 4. Render pulse particles
        for p in &frame.pulse_particles {
            let cx = p.screen_center[0].round() as i32;
            let cy = p.screen_center[1].round() as i32;
            let r = (p.radius.round() as i32).max(1);
            let c = [
                (p.color[0] * 255.0) as u8,
                (p.color[1] * 255.0) as u8,
                (p.color[2] * 255.0) as u8,
                255,
            ];
            for dy in -r..=r {
                for dx in -r..=r {
                    if dx * dx + dy * dy <= r * r {
                        self.set_pixel(cx + dx, cy + dy, c);
                    }
                }
            }
        }
    }

    fn draw_line(&mut self, mut x0: i32, mut y0: i32, x1: i32, y1: i32, color: [u8; 4]) {
        let dx = (x1 - x0).abs();
        let sx = if x0 < x1 { 1 } else { -1 };
        let dy = -(y1 - y0).abs();
        let sy = if y0 < y1 { 1 } else { -1 };
        let mut err = dx + dy;
        loop {
            self.set_pixel(x0, y0, color);
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
        assert_eq!(frame.port_instances.len(), 8);
        assert!(!frame.edge_vertices.is_empty());
        assert_eq!(frame.pulse_particles.len(), 2);
        assert!(NODE_CARD_SHADER_WGSL.contains("fs_node_card"));
        assert!(EDGE_BEZIER_SHADER_WGSL.contains("fs_edge"));
    }
}
