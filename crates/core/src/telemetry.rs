//! Opt-in frame-time histograms, telemetry diagnostics, and dogfood architecture scene builder.

use crate::scene::{ColorRgba, Edge, EdgeAnimation, Node, SceneGraph};
use crate::Rect;

#[derive(Debug, Clone, PartialEq)]
pub struct DiagnosticEvent {
    pub severity: &'static str,
    pub subsystem: &'static str,
    pub message: String,
}

/// Opt-in frame-time histogram and diagnostic ring buffer for catching GPU/driver quirks.
#[derive(Debug, Clone)]
pub struct TelemetryCollector {
    pub opt_in_enabled: bool,
    samples_micros: Vec<u64>,
    dropped_frames_60fps: u64,
    dropped_frames_120fps: u64,
    events: Vec<DiagnosticEvent>,
    max_samples: usize,
}

impl Default for TelemetryCollector {
    fn default() -> Self {
        Self::new(true, 1024)
    }
}

impl TelemetryCollector {
    pub const BUDGET_60FPS_MICROS: u64 = 16_667;
    pub const BUDGET_120FPS_MICROS: u64 = 8_333;

    pub fn new(opt_in_enabled: bool, max_samples: usize) -> Self {
        Self {
            opt_in_enabled,
            samples_micros: Vec::new(),
            dropped_frames_60fps: 0,
            dropped_frames_120fps: 0,
            events: Vec::new(),
            max_samples: max_samples.max(32),
        }
    }

    pub fn record_frame_micros(&mut self, duration_micros: u64) {
        if !self.opt_in_enabled {
            return;
        }
        if duration_micros > Self::BUDGET_60FPS_MICROS {
            self.dropped_frames_60fps += 1;
        }
        if duration_micros > Self::BUDGET_120FPS_MICROS {
            self.dropped_frames_120fps += 1;
        }
        if self.samples_micros.len() >= self.max_samples {
            self.samples_micros.remove(0);
        }
        self.samples_micros.push(duration_micros);
    }

    pub fn record_event(
        &mut self,
        severity: &'static str,
        subsystem: &'static str,
        message: impl Into<String>,
    ) {
        if !self.opt_in_enabled {
            return;
        }
        if self.events.len() >= 128 {
            self.events.remove(0);
        }
        self.events.push(DiagnosticEvent {
            severity,
            subsystem,
            message: message.into(),
        });
    }

    pub fn percentile_micros(&self, percentile: f32) -> u64 {
        if self.samples_micros.is_empty() {
            return 0;
        }
        let mut sorted = self.samples_micros.clone();
        sorted.sort_unstable();
        let idx =
            ((percentile.clamp(0.0, 100.0) / 100.0) * (sorted.len() - 1) as f32).round() as usize;
        sorted[idx]
    }

    pub fn dropped_frames_60fps(&self) -> u64 {
        self.dropped_frames_60fps
    }

    pub fn dropped_frames_120fps(&self) -> u64 {
        self.dropped_frames_120fps
    }

    pub fn events(&self) -> &[DiagnosticEvent] {
        &self.events
    }

    pub fn summary_report(&self) -> String {
        format!(
            "samples={}, p50={}us, p95={}us, p99={}us, dropped_60fps={}, events={}",
            self.samples_micros.len(),
            self.percentile_micros(50.0),
            self.percentile_micros(95.0),
            self.percentile_micros(99.0),
            self.dropped_frames_60fps,
            self.events.len()
        )
    }
}

/// Dogfooding: Build RustFlow Draw's own crate architecture diagram using the core engine.
pub fn build_dogfood_architecture_scene() -> SceneGraph {
    let mut scene = SceneGraph::new();
    scene.insert_node(Node::new_react_flow_card(
        1,
        "rustflow-core",
        Rect::new(80.0, 160.0, 180.0, 80.0),
    ));
    scene.insert_node(Node::new_react_flow_card(
        2,
        "rustflow-renderer",
        Rect::new(360.0, 60.0, 180.0, 80.0),
    ));
    scene.insert_node(Node::new_react_flow_card(
        3,
        "rustflow-layout",
        Rect::new(360.0, 160.0, 180.0, 80.0),
    ));
    scene.insert_node(Node::new_react_flow_card(
        4,
        "rustflow-io-formats",
        Rect::new(360.0, 260.0, 180.0, 80.0),
    ));
    scene.insert_node(Node::new_react_flow_card(
        5,
        "rustflow-ui-bridge",
        Rect::new(640.0, 160.0, 180.0, 80.0),
    ));

    for (eid, src, dst, label) in [
        (101, 1, 2, "R-Tree Frustum"),
        (102, 1, 3, "Scene Graph"),
        (103, 1, 4, "AST Serialization"),
        (104, 2, 5, "wgpu Frame"),
        (105, 3, 5, "Spring Layout"),
        (106, 4, 5, ".drawio & SVG"),
    ] {
        let mut edge = Edge::new_bezier(eid, src, dst);
        edge.label = label.to_string();
        edge.animation = EdgeAnimation::PulseParticles {
            speed: 0.6,
            particle_count: 2,
            radius: 4.0,
            color: ColorRgba::INDIGO_500,
        };
        scene.insert_edge(edge);
    }
    scene
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn telemetry_histogram_percentiles_and_dogfood_diagram() {
        let mut telemetry = TelemetryCollector::new(true, 128);
        for us in [4_000, 5_200, 6_100, 7_800, 18_500] {
            telemetry.record_frame_micros(us);
        }
        telemetry.record_event("INFO", "renderer", "Initialized WebGPU adapter");
        assert_eq!(telemetry.percentile_micros(50.0), 6_100);
        assert_eq!(telemetry.dropped_frames_60fps(), 1);
        assert_eq!(telemetry.events().len(), 1);

        let dogfood = build_dogfood_architecture_scene();
        assert_eq!(dogfood.nodes().len(), 5);
        assert_eq!(dogfood.edges().len(), 6);
    }
}
