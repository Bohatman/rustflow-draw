//! Scene graph entities: React Flow styled Node cards, Ports/Handles, and animated Edges.

use crate::spatial::{RTree, SpatialEntry};
use crate::{Point, Rect};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

pub type NodeId = u64;
pub type EdgeId = u64;
pub type PortId = u64;
pub type GroupId = u64;
pub type PageId = u64;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ThemeMode {
    Light,
    Dark,
    HighContrast,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum BorderStyle {
    #[default]
    Solid,
    Dashed,
    Dotted,
    DashDot,
    Double,
    None,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum PortDensity {
    #[default]
    Auto,
    Fixed(u8),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SymbolLayer {
    pub shape: ShapeKind,
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    pub fill: ColorRgba,
    pub stroke: ColorRgba,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FolderTreeItem {
    pub label: String,
    pub depth: u8,
    pub is_folder: bool,
    pub expanded: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LayerRow {
    pub node_id: NodeId,
    pub depth: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SymbolIcon {
    // AWS Compute & Containers
    AwsEc2,
    AwsLambda,
    AwsEcs,
    AwsEks,
    AwsEcr,
    AwsFargate,
    AwsAppRunner,
    AwsElasticBeanstalk,
    AwsLightsail,
    AwsBatch,
    AwsOutposts,
    // AWS Storage
    AwsS3,
    AwsEbs,
    AwsEfs,
    AwsGlacier,
    AwsBackup,
    // AWS Database
    AwsRds,
    AwsDynamoDb,
    AwsAurora,
    AwsElastiCache,
    AwsRedshift,
    AwsDocumentDb,
    AwsNeptune,
    AwsTimestream,
    AwsMemoryDb,
    AwsKeyspaces,
    // AWS Networking
    AwsVpc,
    AwsApiGateway,
    AwsCloudFront,
    AwsRoute53,
    AwsElb,
    AwsDirectConnect,
    AwsTransitGateway,
    AwsPrivateLink,
    AwsGlobalAccelerator,
    AwsNatGateway,
    // AWS Messaging & App Integration
    AwsSqs,
    AwsSns,
    AwsEventBridge,
    AwsStepFunctions,
    AwsAppSync,
    AwsSes,
    AwsMq,
    AwsAmplify,
    // AWS Analytics & Streaming
    AwsMsk,
    AwsFlink,
    AwsKinesis,
    AwsDataFirehose,
    AwsAthena,
    AwsGlue,
    AwsEmr,
    AwsOpenSearch,
    AwsQuickSight,
    AwsLakeFormation,
    // AWS Security, Governance & DevOps
    AwsIam,
    AwsCognito,
    AwsKms,
    AwsWaf,
    AwsShield,
    AwsGuardDuty,
    AwsSecretsManager,
    AwsCloudWatch,
    AwsCloudTrail,
    AwsConfig,
    AwsOrganizations,
    AwsCodePipeline,
    AwsCodeBuild,
    AwsXRay,
    AwsSsm,
    AwsCloudFormation,
    // AWS AI / ML
    AwsBedrock,
    AwsSageMaker,
    AwsRekognition,
    AwsTextract,
    AwsPolly,
    AwsQ,
    // Azure Compute
    AzureVm,
    AzureAks,
    AzureFunctions,
    AzureAppService,
    AzureContainerApps,
    AzureContainerInstances,
    AzureSpringApps,
    // Azure Storage
    AzureBlob,
    AzureFiles,
    AzureDisk,
    AzureDataLake,
    // Azure Database
    AzureSql,
    AzureCosmosDb,
    AzurePostgres,
    AzureRedis,
    AzureSynapse,
    // Azure Networking
    AzureVnet,
    AzureFrontDoor,
    AzureAppGateway,
    AzureLoadBalancer,
    AzureDns,
    AzureFirewall,
    AzureApiManagement,
    // Azure Messaging & Integration
    AzureServiceBus,
    AzureEventHubs,
    AzureEventGrid,
    AzureLogicApps,
    // Azure Security & Ops
    AzureEntraId,
    AzureKeyVault,
    AzureMonitor,
    AzureDevOps,
    // Azure AI / ML
    AzureOpenAi,
    AzureAiSearch,
    // GCP
    GcpCloudRun,
    GcpGke,
    GcpBigQuery,
    GcpPubSub,
    GcpCloudStorage,
    GcpCloudSql,
    // Infra & DevOps
    Kubernetes,
    Docker,
    Postgres,
    Redis,
    Kafka,
    Nginx,
    LoadBalancer,
    Cdn,
    // Standard Icons
    User,
    Users,
    Globe,
    Lock,
    Key,
    Api,
    Webhook,
    Monitor,
    Terminal,
    AiModel,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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
    Note,
    Callout,
    Queue,
    Server,
    Firewall,
    Shield,
    GatewayXor,
    GatewayParallel,
    TimerEvent,
    Package,
    Component,
    Interface,
    BrowserWindow,
    MobileScreen,
    Section,
    ImageNode,
    FolderTree,
    Collapsible,
    Subprocess,
    Delay,
    ManualInput,
    OffPage,
    Triangle,
    CustomSvg { view_box: Rect, path_data: String },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PortDirection {
    Input,
    Output,
    Bidirectional,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PortSide {
    Top,
    Right,
    Bottom,
    Left,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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
    #[serde(default)]
    pub border_style: BorderStyle,
    #[serde(default)]
    pub border_anim: bool,
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
            header_fill: None,
            border_style: BorderStyle::Solid,
            border_anim: false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TextAlign {
    Left,
    Center,
    Right,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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
    #[serde(default = "default_true")]
    pub visible: bool,
    #[serde(default)]
    pub is_section: bool,
    #[serde(default)]
    pub port_density: PortDensity,
    #[serde(default)]
    pub collapsed: bool,
    #[serde(default)]
    pub tree_items: Vec<FolderTreeItem>,
    #[serde(default)]
    pub image_data: Option<Vec<u8>>,
    #[serde(default)]
    pub custom_layers: Vec<SymbolLayer>,
    #[serde(default)]
    pub icon: Option<SymbolIcon>,
}

impl Node {
    pub fn new_react_flow_card(id: NodeId, title: impl Into<String>, bounds: Rect) -> Self {
        let mut node = Self {
            id,
            title: title.into(),
            body: String::new(),
            bounds,
            rotation_deg: 0.0,
            shape: ShapeKind::ReactFlowCard,
            style: NodeStyle::default(),
            text_style: TextStyle::default(),
            ports: Vec::new(),
            z_index: 0,
            locked: false,
            group_id: None,
            parent_id: None,
            visible: true,
            is_section: false,
            port_density: PortDensity::Auto,
            collapsed: false,
            tree_items: Vec::new(),
            image_data: None,
            custom_layers: Vec::new(),
            icon: None,
        };
        node.rebuild_ports();
        node
    }

    pub fn new_shape(
        id: NodeId,
        title: impl Into<String>,
        bounds: Rect,
        shape: ShapeKind,
    ) -> Self {
        let mut node = Self::new_react_flow_card(id, title, bounds);
        node.is_section = matches!(shape, ShapeKind::Section);
        if matches!(shape, ShapeKind::Section) {
            node.style.fill = ColorRgba::new(0.92, 0.94, 0.98, 0.55);
            node.style.stroke = ColorRgba::new(0.39, 0.45, 0.62, 1.0);
            node.style.border_style = BorderStyle::Dashed;
            node.style.stroke_width = 1.5;
            node.style.corner_radius = 8.0;
        } else if matches!(shape, ShapeKind::FolderTree) {
            node.tree_items = vec![
                FolderTreeItem { label: "src".into(), depth: 0, is_folder: true, expanded: true },
                FolderTreeItem { label: "main.rs".into(), depth: 1, is_folder: false, expanded: false },
                FolderTreeItem { label: "canvas.rs".into(), depth: 1, is_folder: false, expanded: false },
                FolderTreeItem { label: "Cargo.toml".into(), depth: 0, is_folder: false, expanded: false },
            ];
        } else if matches!(shape, ShapeKind::Collapsible) {
            node.body = "Click header chevron to collapse or expand details.".into();
        }
        node.shape = shape;
        node.rebuild_ports();
        node
    }

    pub fn rebuild_ports(&mut self) {
        let auto_count = |len: f32| -> u8 {
            if len >= 340.0 {
                7
            } else if len >= 220.0 {
                5
            } else if len >= 72.0 {
                3
            } else {
                1
            }
        };
        let horiz_count = match self.port_density {
            PortDensity::Auto => auto_count(self.bounds.width),
            PortDensity::Fixed(n) => n.clamp(1, 7),
        };
        let vert_count = match self.port_density {
            PortDensity::Auto => auto_count(self.bounds.height),
            PortDensity::Fixed(n) => n.clamp(1, 7),
        };
        let base_port = self.id * 100;
        let mut ports = Vec::with_capacity((horiz_count as usize + vert_count as usize) * 2);
        for (side_idx, (side, dir, prefix, count)) in [
            (PortSide::Top, PortDirection::Input, "top", horiz_count),
            (PortSide::Right, PortDirection::Output, "right", vert_count),
            (PortSide::Bottom, PortDirection::Output, "bottom", horiz_count),
            (PortSide::Left, PortDirection::Input, "left", vert_count),
        ]
        .into_iter()
        .enumerate()
        {
            for i in 0..count {
                let t = (i as f32 + 1.0) / (count as f32 + 1.0);
                ports.push(Port {
                    id: base_port + (side_idx as u64) * 16 + (i as u64) + 1,
                    name: format!("{prefix}-{i}"),
                    direction: dir,
                    side,
                    offset: t,
                });
            }
        }
        self.ports = ports;
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EdgeKind {
    Bezier,
    SmoothStep,
    Straight,
    Orthogonal,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum EdgeArrowStyle {
    #[default]
    OneWay,
    TwoWay,
    None,
    Open,
    Diamond,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum EdgeLabelPlacement {
    Center,
    Above,
    Below,
    #[default]
    AlongPath,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EdgeLabelStyle {
    pub font_size: f32,
    pub text_color: ColorRgba,
    pub bg_color: ColorRgba,
    pub show_bg: bool,
    pub position: f32,
    pub offset_y: f32,
    #[serde(default)]
    pub placement: EdgeLabelPlacement,
}

impl Default for EdgeLabelStyle {
    fn default() -> Self {
        Self {
            font_size: 11.5,
            text_color: ColorRgba::SLATE_900,
            bg_color: ColorRgba::new(1.0, 1.0, 1.0, 0.94),
            show_bg: false,
            position: 0.5,
            offset_y: 0.0,
            placement: EdgeLabelPlacement::AlongPath,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Edge {
    pub id: EdgeId,
    pub source_node: NodeId,
    pub source_port: Option<PortId>,
    pub target_node: NodeId,
    pub target_port: Option<PortId>,
    pub waypoints: Vec<Point>,
    pub kind: EdgeKind,
    #[serde(default)]
    pub arrow: EdgeArrowStyle,
    pub label: String,
    #[serde(default)]
    pub label_style: EdgeLabelStyle,
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
            arrow: EdgeArrowStyle::OneWay,
            label: String::new(),
            label_style: EdgeLabelStyle::default(),
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
        let is_self_loop = edge.source_node == edge.target_node;

        let (src_pt, src_side) = if let Some(pid) = edge.source_port {
            let pt = src_node
                .port_position(pid)
                .unwrap_or_else(|| src_node.bounds.center());
            let side = src_node.port_side(pid).unwrap_or(PortSide::Right);
            (pt, side)
        } else if is_self_loop {
            let c1 = src_node.bounds.center();
            (Point::new(src_node.bounds.right(), c1.y), PortSide::Right)
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
        } else if is_self_loop {
            let c1 = src_node.bounds.center();
            match src_side {
                PortSide::Right => (Point::new(c1.x, src_node.bounds.y), PortSide::Top),
                PortSide::Top => (Point::new(src_node.bounds.x, c1.y), PortSide::Left),
                PortSide::Left => (Point::new(c1.x, src_node.bounds.bottom()), PortSide::Bottom),
                PortSide::Bottom => (Point::new(src_node.bounds.right(), c1.y), PortSide::Right),
            }
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
        let pad = if edge.source_node == edge.target_node {
            72.0
        } else {
            24.0
        };
        rect.expand(pad)
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

    pub fn query_visible_nodes(&self, frustum: &Rect) -> Vec<&Node> {
        let ids = self.node_rtree.query_rect(frustum);
        let mut visible: Vec<&Node> = self
            .nodes
            .iter()
            .filter(|n| n.visible && ids.contains(&n.id))
            .collect();
        visible.sort_by_key(|n| (n.z_index, n.id));
        visible
    }

    pub fn query_visible_edges(&self, frustum: &Rect) -> Vec<&Edge> {
        let ids = self.edge_rtree.query_rect(frustum);
        let mut visible: Vec<&Edge> = self.edges.iter().filter(|e| ids.contains(&e.id)).collect();
        visible.sort_by_key(|e| (e.z_index, e.id));
        visible
    }

    pub fn hit_test_node(&self, world_pt: Point) -> Option<NodeId> {
        let candidates = self.node_rtree.query_point(world_pt);
        self.nodes
            .iter()
            .filter(|n| n.visible && candidates.contains(&n.id) && n.bounds.contains_point(world_pt))
            .max_by_key(|n| (n.z_index, n.id))
            .map(|n| n.id)
    }

    pub fn normalize_layer_order(&mut self) {
        let mut order: Vec<(i32, NodeId)> = self.nodes.iter().map(|n| (n.z_index, n.id)).collect();
        order.sort_by_key(|&(z, id)| (z, id));
        let mut z_map: HashMap<NodeId, i32> = HashMap::new();
        for (idx, &(_, id)) in order.iter().enumerate() {
            z_map.insert(id, idx as i32);
        }
        let parent_of: HashMap<NodeId, Option<NodeId>> =
            self.nodes.iter().map(|n| (n.id, n.parent_id)).collect();
        for _ in 0..self.nodes.len().max(1) {
            let mut changed = false;
            for (&nid, &pid_opt) in &parent_of {
                if let Some(pid) = pid_opt {
                    let cz = z_map.get(&nid).copied().unwrap_or(0);
                    let pz = z_map.get(&pid).copied().unwrap_or(0);
                    if cz <= pz {
                        z_map.insert(nid, pz + 1);
                        changed = true;
                    }
                }
            }
            if !changed {
                break;
            }
        }
        for node in &mut self.nodes {
            if let Some(&z) = z_map.get(&node.id) {
                node.z_index = z;
            }
        }
    }

    pub fn hierarchical_layers_for_panel(&self) -> Vec<LayerRow> {
        let mut children_map: HashMap<Option<NodeId>, Vec<&Node>> = HashMap::new();
        let existing: HashSet<NodeId> = self.nodes.iter().map(|n| n.id).collect();
        for node in &self.nodes {
            let parent = node.parent_id.filter(|pid| existing.contains(pid));
            children_map.entry(parent).or_default().push(node);
        }
        for list in children_map.values_mut() {
            list.sort_by(|a, b| b.z_index.cmp(&a.z_index).then_with(|| b.id.cmp(&a.id)));
        }
        let mut out = Vec::with_capacity(self.nodes.len());
        fn dfs(
            parent: Option<NodeId>,
            depth: usize,
            map: &HashMap<Option<NodeId>, Vec<&Node>>,
            visited: &mut HashSet<NodeId>,
            out: &mut Vec<LayerRow>,
        ) {
            if let Some(list) = map.get(&parent) {
                for node in list {
                    if visited.insert(node.id) {
                        out.push(LayerRow {
                            node_id: node.id,
                            depth,
                        });
                        dfs(Some(node.id), depth + 1, map, visited, out);
                    }
                }
            }
        }
        let mut visited = HashSet::new();
        dfs(None, 0, &children_map, &mut visited, &mut out);
        out
    }

    pub fn bring_to_front(&mut self, ids: &[NodeId]) {
        let max_z = self.nodes.iter().map(|n| n.z_index).max().unwrap_or(0);
        let set: HashSet<NodeId> = ids.iter().copied().collect();
        let mut next = max_z + 1;
        for node in &mut self.nodes {
            if set.contains(&node.id) {
                node.z_index = next;
                next += 1;
            }
        }
        self.normalize_layer_order();
    }

    pub fn send_to_back(&mut self, ids: &[NodeId]) {
        let min_z = self.nodes.iter().map(|n| n.z_index).min().unwrap_or(0);
        let set: HashSet<NodeId> = ids.iter().copied().collect();
        let mut next = min_z - (ids.len() as i32) - 1;
        for node in &mut self.nodes {
            if set.contains(&node.id) {
                node.z_index = next;
                next += 1;
            }
        }
        self.normalize_layer_order();
    }

    pub fn bring_forward(&mut self, ids: &[NodeId]) {
        let set: HashSet<NodeId> = ids.iter().copied().collect();
        for node in &mut self.nodes {
            if set.contains(&node.id) {
                node.z_index += 2;
            }
        }
        self.normalize_layer_order();
    }

    pub fn send_backward(&mut self, ids: &[NodeId]) {
        let set: HashSet<NodeId> = ids.iter().copied().collect();
        for node in &mut self.nodes {
            if set.contains(&node.id) {
                node.z_index -= 2;
            }
        }
        self.normalize_layer_order();
    }

    pub fn descendant_ids(&self, root: NodeId) -> Vec<NodeId> {
        let mut out = Vec::new();
        let mut stack = vec![root];
        let mut seen = HashSet::new();
        seen.insert(root);
        while let Some(curr) = stack.pop() {
            for node in &self.nodes {
                if node.parent_id == Some(curr) && seen.insert(node.id) {
                    out.push(node.id);
                    stack.push(node.id);
                }
            }
        }
        out
    }

    pub fn move_node_with_descendants(&mut self, id: NodeId, dx: f32, dy: f32) {
        let mut targets = vec![id];
        targets.extend(self.descendant_ids(id));
        for tid in targets {
            if let Some(n) = self.get_node(tid) {
                if !n.locked {
                    let pt = Point::new(n.bounds.x + dx, n.bounds.y + dy);
                    self.move_node_to(tid, pt);
                }
            }
        }
    }

    pub fn group_into_section(&mut self, ids: &[NodeId], title: impl Into<String>) -> Option<NodeId> {
        let mut union_rect: Option<Rect> = None;
        for &id in ids {
            if let Some(n) = self.get_node(id) {
                union_rect = Some(match union_rect {
                    Some(acc) => acc.union(&n.bounds),
                    None => n.bounds,
                });
            }
        }
        let bounds = union_rect?.expand(28.0);
        let section_bounds = Rect::new(bounds.x, bounds.y - 22.0, bounds.width, bounds.height + 22.0);
        let sid = self.allocate_id();
        let mut section = Node::new_shape(sid, title, section_bounds, ShapeKind::Section);
        section.z_index = self.nodes.iter().map(|n| n.z_index).min().unwrap_or(0) - 1;
        self.insert_node(section);
        let id_set: HashSet<NodeId> = ids.iter().copied().collect();
        for node in &mut self.nodes {
            if id_set.contains(&node.id) && node.id != sid {
                node.parent_id = Some(sid);
            }
        }
        self.normalize_layer_order();
        Some(sid)
    }

    pub fn fit_section_to_children(&mut self, section_id: NodeId) -> bool {
        let mut union_rect: Option<Rect> = None;
        for n in &self.nodes {
            if n.parent_id == Some(section_id) {
                union_rect = Some(match union_rect {
                    Some(acc) => acc.union(&n.bounds),
                    None => n.bounds,
                });
            }
        }
        let Some(bounds) = union_rect else {
            return false;
        };
        let padded = Rect::new(
            bounds.x - 24.0,
            bounds.y - 44.0,
            bounds.width + 48.0,
            bounds.height + 68.0,
        );
        self.resize_node(section_id, padded, false)
    }

    pub fn detach_from_parent(&mut self, ids: &[NodeId]) {
        let set: HashSet<NodeId> = ids.iter().copied().collect();
        for node in &mut self.nodes {
            if set.contains(&node.id) {
                node.parent_id = None;
            }
        }
        self.normalize_layer_order();
    }

    pub fn auto_attach_to_section(&mut self, node_id: NodeId) {
        let Some(node) = self.get_node(node_id) else {
            return;
        };
        if node.is_section {
            return;
        }
        let center = node.bounds.center();
        let found = self
            .nodes
            .iter()
            .filter(|s| s.is_section && s.id != node_id && s.visible && s.bounds.contains_point(center))
            .max_by_key(|s| (s.z_index, s.id))
            .map(|s| s.id);
        if let Some(n) = self.get_node_mut(node_id) {
            n.parent_id = found;
        }
        self.normalize_layer_order();
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

    /// Resolve a drag-from-port drop point, supporting both inter-node connections and self-loops.
    pub fn resolve_connection_target_from_port(
        &self,
        drop_world_pt: Point,
        snap_radius: f32,
        src_node: NodeId,
        src_port: PortId,
    ) -> Option<(NodeId, Option<PortId>, Point)> {
        let search_box = Rect::new(
            drop_world_pt.x - snap_radius,
            drop_world_pt.y - snap_radius,
            snap_radius * 2.0,
            snap_radius * 2.0,
        );
        let candidate_ids = self.node_rtree.query_rect(&search_box);
        let mut best_port: Option<(f32, NodeId, PortId, Point)> = None;
        for node in self.nodes.iter().filter(|n| n.visible && candidate_ids.contains(&n.id)) {
            for port in &node.ports {
                if node.id == src_node && port.id == src_port {
                    continue;
                }
                let pos = port.world_position(&node.bounds);
                let dist = pos.distance_to(drop_world_pt);
                if dist <= snap_radius && best_port.is_none_or(|(bd, ..)| dist < bd) {
                    best_port = Some((dist, node.id, port.id, pos));
                }
            }
        }
        if let Some((_, nid, pid, pt)) = best_port {
            return Some((nid, Some(pid), pt));
        }

        if let Some(target_node) = self
            .nodes
            .iter()
            .filter(|n| {
                n.visible
                    && !n.is_section
                    && candidate_ids.contains(&n.id)
                    && n.bounds.expand(12.0).contains_point(drop_world_pt)
            })
            .max_by_key(|n| (n.z_index, n.id))
        {
            if let Some((pid, pt)) = target_node
                .ports
                .iter()
                .filter(|p| !(target_node.id == src_node && p.id == src_port))
                .map(|p| (p.id, p.world_position(&target_node.bounds)))
                .min_by(|a, b| {
                    a.1.distance_to(drop_world_pt)
                        .partial_cmp(&b.1.distance_to(drop_world_pt))
                        .unwrap_or(std::cmp::Ordering::Equal)
                })
            {
                return Some((target_node.id, Some(pid), pt));
            }
            if target_node.id == src_node {
                if let Some(pt) = target_node.port_position(src_port) {
                    return Some((src_node, Some(src_port), pt));
                }
            }
            return Some((target_node.id, None, target_node.bounds.center()));
        }
        None
    }

    /// Resolve a drag-from-port drop point: snaps to any port within `snap_radius`, or if dropped
    /// anywhere on a target node's body, snaps to the closest port on that node.
    pub fn resolve_connection_target(
        &self,
        drop_world_pt: Point,
        snap_radius: f32,
        exclude_node: NodeId,
    ) -> Option<(NodeId, Option<PortId>, Point)> {
        if let Some((nid, pid, pt)) =
            self.find_nearest_port(drop_world_pt, snap_radius, Some(exclude_node))
        {
            return Some((nid, Some(pid), pt));
        }
        // Fallback: if dropped anywhere inside or within margin of a node body, pick its nearest port
        let expanded = Rect::new(
            drop_world_pt.x - snap_radius,
            drop_world_pt.y - snap_radius,
            snap_radius * 2.0,
            snap_radius * 2.0,
        );
        let candidates = self.node_rtree.query_rect(&expanded);
        let target_node = self
            .nodes
            .iter()
            .filter(|n| {
                n.id != exclude_node
                    && candidates.contains(&n.id)
                    && n.bounds.expand(12.0).contains_point(drop_world_pt)
            })
            .max_by_key(|n| (n.z_index, n.id))?;

        if let Some((pid, pt)) = target_node
            .ports
            .iter()
            .map(|p| (p.id, p.world_position(&target_node.bounds)))
            .min_by(|a, b| {
                a.1.distance_to(drop_world_pt)
                    .partial_cmp(&b.1.distance_to(drop_world_pt))
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
        {
            Some((target_node.id, Some(pid), pt))
        } else {
            Some((target_node.id, None, target_node.bounds.center()))
        }
    }

    /// Replace an edge's custom routing waypoints and update its spatial index envelope.
    pub fn set_edge_waypoints(&mut self, edge_id: EdgeId, waypoints: Vec<Point>) -> bool {
        let Some(edge) = self.edges.iter_mut().find(|e| e.id == edge_id) else {
            return false;
        };
        edge.waypoints = waypoints;
        let updated = edge.clone();
        let bounds = self.compute_edge_bounds(&updated);
        self.edge_rtree.upsert(edge_id, bounds);
        true
    }

    /// Move a single waypoint `index` on `edge_id` to `new_pos` (or insert if empty).
    pub fn move_edge_waypoint(&mut self, edge_id: EdgeId, index: usize, new_pos: Point) -> bool {
        let Some(edge) = self.edges.iter_mut().find(|e| e.id == edge_id) else {
            return false;
        };
        if index < edge.waypoints.len() {
            edge.waypoints[index] = new_pos;
        } else {
            edge.waypoints.push(new_pos);
        }
        let updated = edge.clone();
        let bounds = self.compute_edge_bounds(&updated);
        self.edge_rtree.upsert(edge_id, bounds);
        true
    }

    /// Reconnect an edge's source and/or target node & port handles.
    pub fn reconnect_edge(
        &mut self,
        edge_id: EdgeId,
        new_source: (NodeId, Option<PortId>),
        new_target: (NodeId, Option<PortId>),
    ) -> bool {
        let Some(edge) = self.edges.iter_mut().find(|e| e.id == edge_id) else {
            return false;
        };
        edge.source_node = new_source.0;
        edge.source_port = new_source.1;
        edge.target_node = new_target.0;
        edge.target_port = new_target.1;
        let updated = edge.clone();
        let bounds = self.compute_edge_bounds(&updated);
        self.edge_rtree.upsert(edge_id, bounds);
        true
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

        // Dropping inside node 2 body snaps to its nearest port
        let target_hit = graph
            .resolve_connection_target(Point::new(370.0, 140.0), 16.0, 1)
            .expect("should resolve target node and port");
        assert_eq!(target_hit.0, n2_id);
        assert!(target_hit.1.is_some());

        // Adjusting edge waypoints
        assert!(graph.move_edge_waypoint(e1_id, 0, Point::new(320.0, 60.0)));
        assert_eq!(
            graph.get_edge(e1_id).unwrap().waypoints,
            vec![Point::new(320.0, 60.0)]
        );
    }
}
