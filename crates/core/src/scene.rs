//! Scene graph entities: React Flow styled Node cards, Ports/Handles, and animated Edges.

use crate::spatial::{RTree, SpatialEntry};
use crate::{Point, Rect};

pub type NodeId = u64;
pub type EdgeId = u64;
pub type PortId = u64;
pub type GroupId = u64;
pub type PageId = u64;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ColorRgba {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl ColorRgba {
    pub const fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    pub const WHITE: Self = Self::new(1.0, 1.0, 1.0, 1.0);
    pub const SLATE_900: Self = Self::new(0.06, 0.09, 0.16, 1.0);
    pub const INDIGO_500: Self = Self::new(0.39, 0.40, 0.95, 1.0);
    pub const SLATE_400: Self = Self::new(0.58, 0.64, 0.72, 1.0);

    pub fn to_array(self) -> [f32; 4] {
        [self.r, self.g, self.b, self.a]
    }

    pub fn to_css_hex(self) -> String {
        let r = (self.r.clamp(0.0, 1.0) * 255.0).round() as u8;
        let g = (self.g.clamp(0.0, 1.0) * 255.0).round() as u8;
        let b = (self.b.clamp(0.0, 1.0) * 255.0).round() as u8;
        format!("#{:02X}{:02X}{:02X}", r, g, b)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemeMode {
    Light,
    Dark,
    HighContrast,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ShapeKind {
    ReactFlowCard,
    Rectangle,
    RoundedRect,
    Diamond,
    Ellipse,
    Cylinder,
    Parallelogram,
    Hexagon,
    Cloud,
    Actor,
    Document,
    CustomSvg { view_box: Rect, path_data: String },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PortDirection {
    Input,
    Output,
    Bidirectional,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PortSide {
    Top,
    Right,
    Bottom,
    Left,
}

/// Explicit React Flow connection Handle / Port on a node boundary.
#[derive(Debug, Clone, PartialEq)]
pub struct Port {
    pub id: PortId,
    pub name: String,
    pub direction: PortDirection,
    pub side: PortSide,
    /// Normalized offset along the side (`0.0..=1.0`, default `0.5` for centered).
    pub offset: f32,
}

impl Port {
    pub fn new(
        id: PortId,
        name: impl Into<String>,
        direction: PortDirection,
        side: PortSide,
    ) -> Self {
        Self {
            id,
            name: name.into(),
            direction,
            side,
            offset: 0.5,
        }
    }

    pub fn world_position(&self, node_bounds: &Rect) -> Point {
        let t = self.offset.clamp(0.0, 1.0);
        match self.side {
            PortSide::Top => Point::new(node_bounds.x + node_bounds.width * t, node_bounds.y),
            PortSide::Right => {
                Point::new(node_bounds.right(), node_bounds.y + node_bounds.height * t)
            }
            PortSide::Bottom => {
                Point::new(node_bounds.x + node_bounds.width * t, node_bounds.bottom())
            }
            PortSide::Left => Point::new(node_bounds.x, node_bounds.y + node_bounds.height * t),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NodeStyle {
    pub fill: ColorRgba,
    pub gradient_to: Option<ColorRgba>,
    pub stroke: ColorRgba,
    pub stroke_width: f32,
    pub opacity: f32,
    pub corner_radius: f32,
    pub shadow_blur: f32,
    pub shadow_elevation: f32,
    pub header_fill: Option<ColorRgba>,
}

impl Default for NodeStyle {
    fn default() -> Self {
        Self {
            fill: ColorRgba::WHITE,
            gradient_to: None,
            stroke: ColorRgba::new(0.82, 0.85, 0.90, 1.0),
            stroke_width: 1.5,
            opacity: 1.0,
            corner_radius: 10.0,
            shadow_blur: 14.0,
            shadow_elevation: 4.0,
            header_fill: Some(ColorRgba::new(0.95, 0.96, 0.99, 1.0)),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextAlign {
    Left,
    Center,
    Right,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TextStyle {
    pub font_family: String,
    pub font_weight: u16,
    pub font_size: f32,
    pub align: TextAlign,
    pub color: ColorRgba,
    pub padding: f32,
}

impl Default for TextStyle {
    fn default() -> Self {
        Self {
            font_family: "Inter".to_string(),
            font_weight: 600,
            font_size: 14.0,
            align: TextAlign::Center,
            color: ColorRgba::SLATE_900,
            padding: 12.0,
        }
    }
}

/// A diagram node combining React Flow card styling + Draw.io stencil shapes.
#[derive(Debug, Clone, PartialEq)]
pub struct Node {
    pub id: NodeId,
    pub title: String,
    pub body: String,
    pub bounds: Rect,
    pub rotation_deg: f32,
    pub shape: ShapeKind,
    pub style: NodeStyle,
    pub text_style: TextStyle,
    pub ports: Vec<Port>,
    pub z_index: i32,
    pub locked: bool,
    pub group_id: Option<GroupId>,
    pub parent_id: Option<NodeId>,
}

impl Node {
    pub fn new_react_flow_card(id: NodeId, title: impl Into<String>, bounds: Rect) -> Self {
        let base_port = id * 10;
        Self {
            id,
            title: title.into(),
            body: String::new(),
            bounds,
            rotation_deg: 0.0,
            shape: ShapeKind::ReactFlowCard,
            style: NodeStyle::default(),
            text_style: TextStyle::default(),
            ports: vec![
                Port::new(base_port + 1, "in-top", PortDirection::Input, PortSide::Top),
                Port::new(
                    base_port + 2,
                    "out-right",
                    PortDirection::Output,
                    PortSide::Right,
                ),
                Port::new(
                    base_port + 3,
                    "out-bottom",
                    PortDirection::Output,
                    PortSide::Bottom,
                ),
                Port::new(
                    base_port + 4,
                    "in-left",
                    PortDirection::Input,
                    PortSide::Left,
                ),
            ],
            z_index: 0,
            locked: false,
            group_id: None,
            parent_id: None,
        }
    }

    pub fn port_position(&self, port_id: PortId) -> Option<Point> {
        self.ports
            .iter()
            .find(|p| p.id == port_id)
            .map(|p| p.world_position(&self.bounds))
    }

    pub fn port_side(&self, port_id: PortId) -> Option<PortSide> {
        self.ports.iter().find(|p| p.id == port_id).map(|p| p.side)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EdgeKind {
    Bezier,
    SmoothStep,
    Straight,
    Orthogonal,
}

#[derive(Debug, Clone, PartialEq)]
pub enum EdgeAnimation {
    None,
    MarchingAnts {
        speed: f32,
        dash_length: f32,
        gap_length: f32,
        reverse: bool,
    },
    PulseParticles {
        speed: f32,
        particle_count: u32,
        radius: f32,
        color: ColorRgba,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct Edge {
    pub id: EdgeId,
    pub source_node: NodeId,
    pub source_port: Option<PortId>,
    pub target_node: NodeId,
    pub target_port: Option<PortId>,
    pub waypoints: Vec<Point>,
    pub kind: EdgeKind,
    pub label: String,
    pub stroke: ColorRgba,
    pub stroke_width: f32,
    pub animation: EdgeAnimation,
    pub z_index: i32,
}

impl Edge {
    pub fn new_bezier(id: EdgeId, source_node: NodeId, target_node: NodeId) -> Self {
        Self {
            id,
            source_node,
            source_port: None,
            target_node,
            target_port: None,
            waypoints: Vec::new(),
            kind: EdgeKind::Bezier,
            label: String::new(),
            stroke: ColorRgba::INDIGO_500,
            stroke_width: 2.0,
            animation: EdgeAnimation::None,
            z_index: 0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QuickConnectDirection {
    Up,
    Right,
    Down,
    Left,
}

/// Scene graph storing nodes, edges, and R-Tree spatial indices.
#[derive(Debug, Clone, Default)]
pub struct SceneGraph {
    nodes: Vec<Node>,
    edges: Vec<Edge>,
    node_rtree: RTree<NodeId>,
    edge_rtree: RTree<EdgeId>,
    next_id: u64,
}

impl SceneGraph {
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            edges: Vec::new(),
            node_rtree: RTree::new(16),
            edge_rtree: RTree::new(16),
            next_id: 1,
        }
    }

    pub fn allocate_id(&mut self) -> u64 {
        let id = self.next_id.max(1);
        self.next_id = id + 1;
        id
    }

    pub fn nodes(&self) -> &[Node] {
        &self.nodes
    }

    pub fn edges(&self) -> &[Edge] {
        &self.edges
    }

    pub fn get_node(&self, id: NodeId) -> Option<&Node> {
        self.nodes.iter().find(|n| n.id == id)
    }

    pub fn get_node_mut(&mut self, id: NodeId) -> Option<&mut Node> {
        self.nodes.iter_mut().find(|n| n.id == id)
    }

    pub fn get_edge(&self, id: EdgeId) -> Option<&Edge> {
        self.edges.iter().find(|e| e.id == id)
    }

    pub fn get_edge_mut(&mut self, id: EdgeId) -> Option<&mut Edge> {
        self.edges.iter_mut().find(|e| e.id == id)
    }

    pub fn insert_node(&mut self, node: Node) {
        self.next_id = self.next_id.max(node.id + 1);
        self.node_rtree.upsert(node.id, node.bounds);
        if let Some(existing) = self.nodes.iter_mut().find(|n| n.id == node.id) {
            *existing = node;
        } else {
            self.nodes.push(node);
        }
    }

    pub fn remove_node(&mut self, id: NodeId) -> Option<Node> {
        let idx = self.nodes.iter().position(|n| n.id == id)?;
        let removed = self.nodes.remove(idx);
        self.node_rtree.remove(id);
        let removed_edges: Vec<EdgeId> = self
            .edges
            .iter()
            .filter(|e| e.source_node == id || e.target_node == id)
            .map(|e| e.id)
            .collect();
        for eid in removed_edges {
            self.remove_edge(eid);
        }
        Some(removed)
    }

    pub fn move_node_to(&mut self, id: NodeId, new_origin: Point) -> bool {
        let Some(node) = self.nodes.iter_mut().find(|n| n.id == id) else {
            return false;
        };
        if node.locked {
            return false;
        }
        node.bounds.x = new_origin.x;
        node.bounds.y = new_origin.y;
        let bounds = node.bounds;
        self.node_rtree.upsert(id, bounds);
        self.refresh_incident_edge_bounds(id);
        true
    }

    pub fn resize_node(&mut self, id: NodeId, new_bounds: Rect, lock_aspect: bool) -> bool {
        let Some(node) = self.nodes.iter_mut().find(|n| n.id == id) else {
            return false;
        };
        if node.locked {
            return false;
        }
        let mut target = new_bounds;
        target.width = target.width.max(20.0);
        target.height = target.height.max(20.0);
        if lock_aspect && node.bounds.height > 0.0 {
            let aspect = node.bounds.width / node.bounds.height;
            target.height = (target.width / aspect).max(20.0);
        }
        node.bounds = target;
        self.node_rtree.upsert(id, target);
        self.refresh_incident_edge_bounds(id);
        true
    }

    pub fn insert_edge(&mut self, edge: Edge) {
        self.next_id = self.next_id.max(edge.id + 1);
        let bounds = self.compute_edge_bounds(&edge);
        self.edge_rtree.upsert(edge.id, bounds);
        if let Some(existing) = self.edges.iter_mut().find(|e| e.id == edge.id) {
            *existing = edge;
        } else {
            self.edges.push(edge);
        }
    }

    pub fn remove_edge(&mut self, id: EdgeId) -> Option<Edge> {
        let idx = self.edges.iter().position(|e| e.id == id)?;
        let removed = self.edges.remove(idx);
        self.edge_rtree.remove(id);
        Some(removed)
    }

    pub fn rebuild_spatial_index(&mut self) {
        let node_entries: Vec<SpatialEntry<NodeId>> = self
            .nodes
            .iter()
            .map(|n| SpatialEntry {
                id: n.id,
                bounds: n.bounds,
            })
            .collect();
        self.node_rtree.bulk_load(node_entries);

        let edge_entries: Vec<SpatialEntry<EdgeId>> = self
            .edges
            .iter()
            .map(|e| SpatialEntry {
                id: e.id,
                bounds: self.compute_edge_bounds(e),
            })
            .collect();
        self.edge_rtree.bulk_load(edge_entries);
    }

    pub fn edge_endpoints(&self, edge: &Edge) -> Option<(Point, Point, PortSide, PortSide)> {
        let src_node = self.get_node(edge.source_node)?;
        let dst_node = self.get_node(edge.target_node)?;

        let (src_pt, src_side) = if let Some(pid) = edge.source_port {
            let pt = src_node
                .port_position(pid)
                .unwrap_or_else(|| src_node.bounds.center());
            let side = src_node.port_side(pid).unwrap_or(PortSide::Right);
            (pt, side)
        } else {
            let c1 = src_node.bounds.center();
            let c2 = dst_node.bounds.center();
            if (c2.x - c1.x).abs() >= (c2.y - c1.y).abs() {
                if c2.x >= c1.x {
                    (Point::new(src_node.bounds.right(), c1.y), PortSide::Right)
                } else {
                    (Point::new(src_node.bounds.x, c1.y), PortSide::Left)
                }
            } else if c2.y >= c1.y {
                (Point::new(c1.x, src_node.bounds.bottom()), PortSide::Bottom)
            } else {
                (Point::new(c1.x, src_node.bounds.y), PortSide::Top)
            }
        };

        let (dst_pt, dst_side) = if let Some(pid) = edge.target_port {
            let pt = dst_node
                .port_position(pid)
                .unwrap_or_else(|| dst_node.bounds.center());
            let side = dst_node.port_side(pid).unwrap_or(PortSide::Left);
            (pt, side)
        } else {
            let c1 = src_node.bounds.center();
            let c2 = dst_node.bounds.center();
            if (c2.x - c1.x).abs() >= (c2.y - c1.y).abs() {
                if c2.x >= c1.x {
                    (Point::new(dst_node.bounds.x, c2.y), PortSide::Left)
                } else {
                    (Point::new(dst_node.bounds.right(), c2.y), PortSide::Right)
                }
            } else if c2.y >= c1.y {
                (Point::new(c2.x, dst_node.bounds.y), PortSide::Top)
            } else {
                (Point::new(c2.x, dst_node.bounds.bottom()), PortSide::Bottom)
            }
        };

        Some((src_pt, dst_pt, src_side, dst_side))
    }

    fn compute_edge_bounds(&self, edge: &Edge) -> Rect {
        let Some((p0, p1, _, _)) = self.edge_endpoints(edge) else {
            return Rect::new(0.0, 0.0, 1.0, 1.0);
        };
        let mut rect = Rect::new(
            p0.x.min(p1.x),
            p0.y.min(p1.y),
            (p0.x - p1.x).abs(),
            (p0.y - p1.y).abs(),
        );
        for wp in &edge.waypoints {
            let wp_rect = Rect::new(wp.x, wp.y, 0.0, 0.0);
            rect = rect.union(&wp_rect);
        }
        rect.expand(24.0)
    }

    fn refresh_incident_edge_bounds(&mut self, node_id: NodeId) {
        let updates: Vec<(EdgeId, Rect)> = self
            .edges
            .iter()
            .filter(|e| e.source_node == node_id || e.target_node == node_id)
            .map(|e| (e.id, self.compute_edge_bounds(e)))
            .collect();
        for (eid, bounds) in updates {
            self.edge_rtree.upsert(eid, bounds);
        }
    }

    /// Frustum query returning visible nodes sorted by `z_index`.
    pub fn query_visible_nodes(&self, frustum: &Rect) -> Vec<&Node> {
        let ids = self.node_rtree.query_rect(frustum);
        let mut visible: Vec<&Node> = self.nodes.iter().filter(|n| ids.contains(&n.id)).collect();
        visible.sort_by_key(|n| (n.z_index, n.id));
        visible
    }

    /// Frustum query returning visible edges sorted by `z_index`.
    pub fn query_visible_edges(&self, frustum: &Rect) -> Vec<&Edge> {
        let ids = self.edge_rtree.query_rect(frustum);
        let mut visible: Vec<&Edge> = self.edges.iter().filter(|e| ids.contains(&e.id)).collect();
        visible.sort_by_key(|e| (e.z_index, e.id));
        visible
    }

    /// Topmost node under a world point.
    pub fn hit_test_node(&self, world_pt: Point) -> Option<NodeId> {
        let candidates = self.node_rtree.query_point(world_pt);
        self.nodes
            .iter()
            .filter(|n| candidates.contains(&n.id) && n.bounds.contains_point(world_pt))
            .max_by_key(|n| (n.z_index, n.id))
            .map(|n| n.id)
    }

    /// Magnetic port snapping: find nearest port within `snap_radius` world units.
    pub fn find_nearest_port(
        &self,
        world_pt: Point,
        snap_radius: f32,
        exclude_node: Option<NodeId>,
    ) -> Option<(NodeId, PortId, Point)> {
        let search_box = Rect::new(
            world_pt.x - snap_radius,
            world_pt.y - snap_radius,
            snap_radius * 2.0,
            snap_radius * 2.0,
        );
        let candidate_ids = self.node_rtree.query_rect(&search_box);
        let mut best: Option<(f32, NodeId, PortId, Point)> = None;
        for node in self.nodes.iter().filter(|n| candidate_ids.contains(&n.id)) {
            if Some(node.id) == exclude_node {
                continue;
            }
            for port in &node.ports {
                let pos = port.world_position(&node.bounds);
                let dist = pos.distance_to(world_pt);
                if dist <= snap_radius && best.is_none_or(|(best_d, ..)| dist < best_d) {
                    best = Some((dist, node.id, port.id, pos));
                }
            }
        }
        best.map(|(_, nid, pid, pt)| (nid, pid, pt))
    }

    /// Draw.io quick-add directional arrow action: clones `source_id` in `dir` and connects with an edge.
    pub fn quick_clone_and_connect(
        &mut self,
        source_id: NodeId,
        dir: QuickConnectDirection,
        gap: f32,
    ) -> Option<(NodeId, EdgeId)> {
        let src = self.get_node(source_id)?.clone();
        let new_node_id = self.allocate_id();
        let new_edge_id = self.allocate_id();

        let new_bounds = match dir {
            QuickConnectDirection::Right => Rect::new(
                src.bounds.right() + gap,
                src.bounds.y,
                src.bounds.width,
                src.bounds.height,
            ),
            QuickConnectDirection::Left => Rect::new(
                src.bounds.x - src.bounds.width - gap,
                src.bounds.y,
                src.bounds.width,
                src.bounds.height,
            ),
            QuickConnectDirection::Down => Rect::new(
                src.bounds.x,
                src.bounds.bottom() + gap,
                src.bounds.width,
                src.bounds.height,
            ),
            QuickConnectDirection::Up => Rect::new(
                src.bounds.x,
                src.bounds.y - src.bounds.height - gap,
                src.bounds.width,
                src.bounds.height,
            ),
        };

        let mut cloned = Node::new_react_flow_card(new_node_id, src.title.clone(), new_bounds);
        cloned.shape = src.shape.clone();
        cloned.style = src.style.clone();
        cloned.text_style = src.text_style.clone();

        self.insert_node(cloned);
        let edge = Edge::new_bezier(new_edge_id, source_id, new_node_id);
        self.insert_edge(edge);
        Some((new_node_id, new_edge_id))
    }

    pub fn world_bounds(&self) -> Option<Rect> {
        self.node_rtree.total_bounds()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scene_graph_node_cards_ports_and_quick_connect() {
        let mut graph = SceneGraph::new();
        let n1 = Node::new_react_flow_card(1, "API Gateway", Rect::new(100.0, 100.0, 180.0, 80.0));
        graph.insert_node(n1);

        let (n2_id, e1_id) = graph
            .quick_clone_and_connect(1, QuickConnectDirection::Right, 80.0)
            .expect("should clone and connect");
        assert_eq!(graph.nodes().len(), 2);
        assert_eq!(graph.edges().len(), 1);
        assert_eq!(graph.get_edge(e1_id).unwrap().target_node, n2_id);

        // Magnetic port snapping near right handle of node 1 (280, 140)
        let snap = graph
            .find_nearest_port(Point::new(284.0, 138.0), 12.0, None)
            .expect("should snap to port");
        assert_eq!(snap.0, 1);
        assert!((snap.2.x - 280.0).abs() < 1e-3);
        assert!((snap.2.y - 140.0).abs() < 1e-3);
    }
}
