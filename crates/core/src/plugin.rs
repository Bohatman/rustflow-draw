//! Phase 6: SemVer version negotiation and Plugin Ecosystem API for custom stencil packs,
//! custom auto-layout engines, and custom WGSL edge animation shaders.

use crate::scene::{Node, NodeId, SceneGraph, ShapeKind};
use crate::{Point, Rect};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct SemVer {
    pub major: u16,
    pub minor: u16,
    pub patch: u16,
}

impl SemVer {
    pub const V1_0_0: Self = Self::new(1, 0, 0);

    pub const fn new(major: u16, minor: u16, patch: u16) -> Self {
        Self {
            major,
            minor,
            patch,
        }
    }

    pub fn parse(raw: &str) -> Option<Self> {
        let trimmed = raw.trim().strip_prefix('v').unwrap_or(raw.trim());
        let mut parts = trimmed.split('.');
        let major = parts.next()?.parse().ok()?;
        let minor = parts.next()?.parse().ok()?;
        let patch = parts.next()?.parse().ok()?;
        Some(Self {
            major,
            minor,
            patch,
        })
    }

    pub fn is_backward_compatible_with(&self, file_version: &SemVer) -> bool {
        if self.major == file_version.major {
            self >= file_version
        } else {
            // Engine v1.x maintains backwards compatibility with pre-release 0.x files
            self.major == 1 && file_version.major == 0
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PluginStencilDescriptor {
    pub id: String,
    pub label: String,
    pub width: f32,
    pub height: f32,
    pub shape: ShapeKind,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EdgeShaderPlugin {
    pub id: String,
    pub name: String,
    pub wgsl_entry_point: String,
    pub wgsl_source: String,
}

pub trait CustomLayoutPlugin: std::fmt::Debug {
    fn id(&self) -> &str;
    fn name(&self) -> &str;
    fn compute_positions(&self, scene: &SceneGraph, origin: Point) -> Vec<(NodeId, Point, Point)>;
}

/// Built-in example plugin: Radial / Circular graph layout.
#[derive(Debug, Clone)]
pub struct RadialLayoutPlugin {
    pub radius: f32,
}

impl Default for RadialLayoutPlugin {
    fn default() -> Self {
        Self { radius: 220.0 }
    }
}

impl CustomLayoutPlugin for RadialLayoutPlugin {
    fn id(&self) -> &str {
        "plugin.layout.radial"
    }

    fn name(&self) -> &str {
        "Radial Ring Layout"
    }

    fn compute_positions(&self, scene: &SceneGraph, center: Point) -> Vec<(NodeId, Point, Point)> {
        let count = scene.nodes().len().max(1);
        scene
            .nodes()
            .iter()
            .enumerate()
            .map(|(i, n)| {
                let angle = (i as f32 / count as f32) * std::f32::consts::TAU;
                let to = Point::new(
                    center.x + self.radius * angle.cos() - n.bounds.width * 0.5,
                    center.y + self.radius * angle.sin() - n.bounds.height * 0.5,
                );
                (n.id, Point::new(n.bounds.x, n.bounds.y), to)
            })
            .collect()
    }
}

/// Extensible plugin registry for custom stencil packs, shaders, and layout algorithms.
#[derive(Debug, Default)]
pub struct PluginRegistry {
    stencils: Vec<PluginStencilDescriptor>,
    shaders: Vec<EdgeShaderPlugin>,
    layouts: Vec<Box<dyn CustomLayoutPlugin>>,
}

impl PluginRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register_stencil(&mut self, stencil: PluginStencilDescriptor) {
        if let Some(existing) = self.stencils.iter_mut().find(|s| s.id == stencil.id) {
            *existing = stencil;
        } else {
            self.stencils.push(stencil);
        }
    }

    pub fn register_edge_shader(&mut self, shader: EdgeShaderPlugin) {
        if let Some(existing) = self.shaders.iter_mut().find(|s| s.id == shader.id) {
            *existing = shader;
        } else {
            self.shaders.push(shader);
        }
    }

    pub fn register_layout(&mut self, layout: Box<dyn CustomLayoutPlugin>) {
        self.layouts.retain(|l| l.id() != layout.id());
        self.layouts.push(layout);
    }

    pub fn stencils(&self) -> &[PluginStencilDescriptor] {
        &self.stencils
    }

    pub fn shaders(&self) -> &[EdgeShaderPlugin] {
        &self.shaders
    }

    pub fn instantiate_stencil(
        &self,
        plugin_stencil_id: &str,
        node_id: NodeId,
        origin: Point,
    ) -> Option<Node> {
        let desc = self.stencils.iter().find(|s| s.id == plugin_stencil_id)?;
        let mut node = Node::new_react_flow_card(
            node_id,
            desc.label.clone(),
            Rect::new(origin.x, origin.y, desc.width, desc.height),
        );
        node.shape = desc.shape.clone();
        Some(node)
    }

    pub fn run_custom_layout(
        &self,
        layout_id: &str,
        scene: &mut SceneGraph,
        center: Point,
    ) -> bool {
        let Some(plugin) = self.layouts.iter().find(|l| l.id() == layout_id) else {
            return false;
        };
        for (nid, _from, to) in plugin.compute_positions(scene, center) {
            scene.move_node_to(nid, to);
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn semver_compatibility_and_plugin_ecosystem_registry() {
        let engine_ver = SemVer::V1_0_0;
        let legacy_v0 = SemVer::parse("0.1.0").unwrap();
        let current_v1 = SemVer::parse("v1.0.0").unwrap();
        let future_v2 = SemVer::parse("2.0.0").unwrap();

        assert!(engine_ver.is_backward_compatible_with(&legacy_v0));
        assert!(engine_ver.is_backward_compatible_with(&current_v1));
        assert!(!engine_ver.is_backward_compatible_with(&future_v2));

        let mut registry = PluginRegistry::new();
        registry.register_stencil(PluginStencilDescriptor {
            id: "k8s.ingress".into(),
            label: "K8s Ingress Controller".into(),
            width: 180.0,
            height: 80.0,
            shape: ShapeKind::Hexagon,
        });
        registry.register_edge_shader(EdgeShaderPlugin {
            id: "shader.neon_beam".into(),
            name: "Neon Beam Shader".into(),
            wgsl_entry_point: "fs_neon_beam".into(),
            wgsl_source: "@fragment fn fs_neon_beam() -> @location(0) vec4<f32> { return vec4<f32>(0.0, 1.0, 1.0, 1.0); }".into(),
        });
        registry.register_layout(Box::new(RadialLayoutPlugin { radius: 200.0 }));

        let mut scene = SceneGraph::new();
        for id in 1..=4 {
            let node = registry
                .instantiate_stencil("k8s.ingress", id, Point::ZERO)
                .unwrap();
            scene.insert_node(node);
        }

        assert!(registry.run_custom_layout(
            "plugin.layout.radial",
            &mut scene,
            Point::new(300.0, 300.0)
        ));
        assert_ne!(
            scene.get_node(1).unwrap().bounds.x,
            scene.get_node(2).unwrap().bounds.x
        );
    }
}
