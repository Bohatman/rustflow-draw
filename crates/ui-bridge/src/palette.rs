//! Draw.io + React Flow stencil shape palette catalog and searchable accordion categories.

use rustflow_core::{Node, NodeId, Point, Rect, ShapeKind};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StencilCategory {
    Flowchart,
    Uml,
    Bpmn,
    CloudArchitecture,
    Network,
    Wireframes,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StencilItem {
    pub id: String,
    pub label: String,
    pub category: StencilCategory,
    pub shape: ShapeKind,
    pub default_width: f32,
    pub default_height: f32,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct PaletteCatalog {
    items: Vec<StencilItem>,
}

impl Default for PaletteCatalog {
    fn default() -> Self {
        Self::built_in()
    }
}

impl PaletteCatalog {
    pub fn built_in() -> Self {
        let items = vec![
            // Flowchart
            StencilItem {
                id: "flowchart.process".into(),
                label: "Process Card".into(),
                category: StencilCategory::Flowchart,
                shape: ShapeKind::ReactFlowCard,
                default_width: 160.0,
                default_height: 76.0,
                tags: vec!["process".into(), "step".into(), "card".into()],
            },
            StencilItem {
                id: "flowchart.rounded".into(),
                label: "Start / Terminator".into(),
                category: StencilCategory::Flowchart,
                shape: ShapeKind::RoundedRect,
                default_width: 140.0,
                default_height: 60.0,
                tags: vec!["start".into(), "end".into(), "terminator".into()],
            },
            StencilItem {
                id: "flowchart.decision".into(),
                label: "Decision Diamond".into(),
                category: StencilCategory::Flowchart,
                shape: ShapeKind::Diamond,
                default_width: 120.0,
                default_height: 100.0,
                tags: vec!["decision".into(), "branch".into(), "if".into()],
            },
            StencilItem {
                id: "flowchart.database".into(),
                label: "Database Cylinder".into(),
                category: StencilCategory::Flowchart,
                shape: ShapeKind::Cylinder,
                default_width: 120.0,
                default_height: 100.0,
                tags: vec!["db".into(), "storage".into(), "sql".into()],
            },
            StencilItem {
                id: "flowchart.data_io".into(),
                label: "Data Input / Output".into(),
                category: StencilCategory::Flowchart,
                shape: ShapeKind::Parallelogram,
                default_width: 150.0,
                default_height: 74.0,
                tags: vec![
                    "io".into(),
                    "input".into(),
                    "output".into(),
                    "parallelogram".into(),
                ],
            },
            StencilItem {
                id: "flowchart.document".into(),
                label: "Document".into(),
                category: StencilCategory::Flowchart,
                shape: ShapeKind::Document,
                default_width: 150.0,
                default_height: 84.0,
                tags: vec!["doc".into(), "file".into(), "report".into()],
            },
            StencilItem {
                id: "flowchart.hexagon".into(),
                label: "Preparation Hexagon".into(),
                category: StencilCategory::Flowchart,
                shape: ShapeKind::Hexagon,
                default_width: 150.0,
                default_height: 80.0,
                tags: vec!["hex".into(), "init".into(), "setup".into()],
            },
            StencilItem {
                id: "flowchart.note".into(),
                label: "Sticky Note / Callout".into(),
                category: StencilCategory::Flowchart,
                shape: ShapeKind::Note,
                default_width: 150.0,
                default_height: 90.0,
                tags: vec!["note".into(), "comment".into(), "annotation".into()],
            },
            // UML
            StencilItem {
                id: "uml.class".into(),
                label: "UML Class".into(),
                category: StencilCategory::Uml,
                shape: ShapeKind::ReactFlowCard,
                default_width: 180.0,
                default_height: 110.0,
                tags: vec!["uml".into(), "class".into(), "object".into()],
            },
            StencilItem {
                id: "uml.interface".into(),
                label: "UML Interface «trait»".into(),
                category: StencilCategory::Uml,
                shape: ShapeKind::Interface,
                default_width: 175.0,
                default_height: 90.0,
                tags: vec!["uml".into(), "interface".into(), "trait".into()],
            },
            StencilItem {
                id: "uml.component".into(),
                label: "UML Component".into(),
                category: StencilCategory::Uml,
                shape: ShapeKind::Component,
                default_width: 170.0,
                default_height: 86.0,
                tags: vec!["uml".into(), "component".into(), "module".into()],
            },
            StencilItem {
                id: "uml.package".into(),
                label: "UML Package / Crate".into(),
                category: StencilCategory::Uml,
                shape: ShapeKind::Package,
                default_width: 180.0,
                default_height: 100.0,
                tags: vec!["uml".into(), "package".into(), "crate".into()],
            },
            StencilItem {
                id: "uml.actor".into(),
                label: "UML Actor".into(),
                category: StencilCategory::Uml,
                shape: ShapeKind::Actor,
                default_width: 80.0,
                default_height: 110.0,
                tags: vec!["uml".into(), "actor".into(), "user".into()],
            },
            // BPMN
            StencilItem {
                id: "bpmn.task".into(),
                label: "BPMN Task".into(),
                category: StencilCategory::Bpmn,
                shape: ShapeKind::RoundedRect,
                default_width: 150.0,
                default_height: 80.0,
                tags: vec!["bpmn".into(), "task".into(), "workflow".into()],
            },
            StencilItem {
                id: "bpmn.gateway.xor".into(),
                label: "BPMN Exclusive Gateway (×)".into(),
                category: StencilCategory::Bpmn,
                shape: ShapeKind::GatewayXor,
                default_width: 90.0,
                default_height: 90.0,
                tags: vec!["bpmn".into(), "gateway".into(), "xor".into()],
            },
            StencilItem {
                id: "bpmn.gateway.parallel".into(),
                label: "BPMN Parallel Gateway (+)".into(),
                category: StencilCategory::Bpmn,
                shape: ShapeKind::GatewayParallel,
                default_width: 90.0,
                default_height: 90.0,
                tags: vec![
                    "bpmn".into(),
                    "gateway".into(),
                    "parallel".into(),
                    "and".into(),
                ],
            },
            StencilItem {
                id: "bpmn.timer".into(),
                label: "BPMN Timer Event (⏱)".into(),
                category: StencilCategory::Bpmn,
                shape: ShapeKind::TimerEvent,
                default_width: 84.0,
                default_height: 84.0,
                tags: vec!["bpmn".into(), "timer".into(), "cron".into(), "event".into()],
            },
            // Cloud Architecture
            StencilItem {
                id: "cloud.aws.lambda".into(),
                label: "AWS Lambda Function".into(),
                category: StencilCategory::CloudArchitecture,
                shape: ShapeKind::Hexagon,
                default_width: 150.0,
                default_height: 84.0,
                tags: vec![
                    "aws".into(),
                    "cloud".into(),
                    "serverless".into(),
                    "lambda".into(),
                ],
            },
            StencilItem {
                id: "cloud.aws.s3".into(),
                label: "AWS S3 Bucket".into(),
                category: StencilCategory::CloudArchitecture,
                shape: ShapeKind::Cylinder,
                default_width: 140.0,
                default_height: 92.0,
                tags: vec!["aws".into(), "s3".into(), "bucket".into(), "object".into()],
            },
            StencilItem {
                id: "cloud.gcp.cloudrun".into(),
                label: "GCP Cloud Run".into(),
                category: StencilCategory::CloudArchitecture,
                shape: ShapeKind::Cloud,
                default_width: 160.0,
                default_height: 92.0,
                tags: vec!["gcp".into(), "cloud".into(), "container".into()],
            },
            StencilItem {
                id: "cloud.azure.aks".into(),
                label: "Azure Kubernetes (AKS)".into(),
                category: StencilCategory::CloudArchitecture,
                shape: ShapeKind::Hexagon,
                default_width: 160.0,
                default_height: 90.0,
                tags: vec!["azure".into(), "cloud".into(), "k8s".into(), "aks".into()],
            },
            // Network & Infra
            StencilItem {
                id: "network.router".into(),
                label: "Load Balancer / Router".into(),
                category: StencilCategory::Network,
                shape: ShapeKind::Ellipse,
                default_width: 110.0,
                default_height: 110.0,
                tags: vec!["network".into(), "router".into(), "lb".into()],
            },
            StencilItem {
                id: "network.queue".into(),
                label: "Message Queue (Kafka)".into(),
                category: StencilCategory::Network,
                shape: ShapeKind::Queue,
                default_width: 165.0,
                default_height: 76.0,
                tags: vec![
                    "queue".into(),
                    "kafka".into(),
                    "rabbitmq".into(),
                    "pubsub".into(),
                ],
            },
            StencilItem {
                id: "network.firewall".into(),
                label: "WAF / Firewall".into(),
                category: StencilCategory::Network,
                shape: ShapeKind::Firewall,
                default_width: 150.0,
                default_height: 82.0,
                tags: vec!["firewall".into(), "waf".into(), "security".into()],
            },
            StencilItem {
                id: "network.shield".into(),
                label: "Auth / Zero-Trust Shield".into(),
                category: StencilCategory::Network,
                shape: ShapeKind::Shield,
                default_width: 120.0,
                default_height: 100.0,
                tags: vec!["auth".into(), "oauth".into(), "shield".into(), "jwt".into()],
            },
            // Wireframes
            StencilItem {
                id: "wireframe.modal".into(),
                label: "Browser Window".into(),
                category: StencilCategory::Wireframes,
                shape: ShapeKind::BrowserWindow,
                default_width: 220.0,
                default_height: 140.0,
                tags: vec![
                    "wireframe".into(),
                    "ui".into(),
                    "window".into(),
                    "browser".into(),
                ],
            },
            StencilItem {
                id: "wireframe.mobile".into(),
                label: "Mobile App Screen".into(),
                category: StencilCategory::Wireframes,
                shape: ShapeKind::MobileScreen,
                default_width: 120.0,
                default_height: 190.0,
                tags: vec![
                    "wireframe".into(),
                    "mobile".into(),
                    "ios".into(),
                    "android".into(),
                ],
            },
        ];
        Self { items }
    }

    pub fn items(&self) -> &[StencilItem] {
        &self.items
    }

    pub fn search(&self, query: &str) -> Vec<&StencilItem> {
        let q = query.trim().to_ascii_lowercase();
        if q.is_empty() {
            return self.items.iter().collect();
        }
        self.items
            .iter()
            .filter(|item| {
                item.label.to_ascii_lowercase().contains(&q)
                    || item.id.to_ascii_lowercase().contains(&q)
                    || item
                        .tags
                        .iter()
                        .any(|t| t.to_ascii_lowercase().contains(&q))
            })
            .collect()
    }

    pub fn by_category(&self, category: StencilCategory) -> Vec<&StencilItem> {
        self.items
            .iter()
            .filter(|i| i.category == category)
            .collect()
    }

    pub fn instantiate(
        &self,
        stencil_id: &str,
        node_id: NodeId,
        drop_world: Point,
    ) -> Option<Node> {
        let stencil = self.items.iter().find(|i| i.id == stencil_id)?;
        let bounds = Rect::new(
            drop_world.x,
            drop_world.y,
            stencil.default_width,
            stencil.default_height,
        );
        let mut node = Node::new_react_flow_card(node_id, stencil.label.clone(), bounds);
        node.shape = stencil.shape.clone();
        Some(node)
    }
}
