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
                default_width: 110.0,
                default_height: 120.0,
                tags: vec!["db".into(), "storage".into(), "sql".into()],
            },
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
                id: "uml.actor".into(),
                label: "UML Actor".into(),
                category: StencilCategory::Uml,
                shape: ShapeKind::Actor,
                default_width: 60.0,
                default_height: 100.0,
                tags: vec!["uml".into(), "actor".into(), "user".into()],
            },
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
                id: "bpmn.gateway".into(),
                label: "BPMN Exclusive Gateway".into(),
                category: StencilCategory::Bpmn,
                shape: ShapeKind::Diamond,
                default_width: 80.0,
                default_height: 80.0,
                tags: vec!["bpmn".into(), "gateway".into(), "xor".into()],
            },
            StencilItem {
                id: "cloud.aws.lambda".into(),
                label: "AWS Lambda Function".into(),
                category: StencilCategory::CloudArchitecture,
                shape: ShapeKind::Hexagon,
                default_width: 130.0,
                default_height: 90.0,
                tags: vec![
                    "aws".into(),
                    "cloud".into(),
                    "serverless".into(),
                    "lambda".into(),
                ],
            },
            StencilItem {
                id: "cloud.gcp.cloudrun".into(),
                label: "GCP Cloud Run".into(),
                category: StencilCategory::CloudArchitecture,
                shape: ShapeKind::Cloud,
                default_width: 160.0,
                default_height: 96.0,
                tags: vec!["gcp".into(), "cloud".into(), "container".into()],
            },
            StencilItem {
                id: "cloud.azure.aks".into(),
                label: "Azure Kubernetes Service".into(),
                category: StencilCategory::CloudArchitecture,
                shape: ShapeKind::Hexagon,
                default_width: 160.0,
                default_height: 96.0,
                tags: vec!["azure".into(), "cloud".into(), "k8s".into(), "aks".into()],
            },
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
                id: "wireframe.modal".into(),
                label: "Browser Window / Modal".into(),
                category: StencilCategory::Wireframes,
                shape: ShapeKind::ReactFlowCard,
                default_width: 240.0,
                default_height: 160.0,
                tags: vec!["wireframe".into(), "ui".into(), "window".into()],
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
