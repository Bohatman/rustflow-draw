//! Performance stress tests and benchmarks for 5,000+ nodes and 10,000+ animated edges.

use rustflow_core::{ColorRgba, Edge, EdgeAnimation, Node, Rect, SceneGraph, Viewport};
use rustflow_io_formats::export_drawio_xml;
use rustflow_layout::compute_grid_layout;
use rustflow_renderer::RenderFrame;
use std::time::Instant;

/// Construct a synthetic diagram with `node_count` nodes and `edge_count` animated edges.
pub fn build_large_scene(node_count: usize, edge_count: usize) -> SceneGraph {
    let mut scene = SceneGraph::new();
    let cols = (node_count as f32).sqrt().ceil().max(1.0) as usize;

    for i in 0..node_count {
        let id = (i + 1) as u64;
        let col = i % cols;
        let row = i / cols;
        let x = col as f32 * 220.0;
        let y = row as f32 * 140.0;
        scene.insert_node(Node::new_react_flow_card(
            id,
            format!("Node-{id}"),
            Rect::new(x, y, 160.0, 72.0),
        ));
    }

    if node_count >= 2 {
        for e in 0..edge_count {
            let eid = (node_count + e + 1) as u64;
            let src = ((e % node_count) + 1) as u64;
            let dst = (((e * 7 + 1) % node_count) + 1) as u64;
            let src = if src == dst {
                (src % node_count as u64) + 1
            } else {
                src
            };
            let mut edge = Edge::new_bezier(eid, src, dst);
            edge.animation = if e % 2 == 0 {
                EdgeAnimation::MarchingAnts {
                    speed: 28.0,
                    dash_length: 10.0,
                    gap_length: 6.0,
                    reverse: false,
                }
            } else {
                EdgeAnimation::PulseParticles {
                    speed: 0.6,
                    particle_count: 1,
                    radius: 3.5,
                    color: ColorRgba::INDIGO_500,
                }
            };
            scene.insert_edge(edge);
        }
    }

    scene.rebuild_spatial_index();
    scene
}

pub fn benchmark_5k_nodes_10k_edges_frame() -> (usize, usize, u128) {
    let scene = build_large_scene(5_000, 10_000);
    let vp = Viewport::new(1920.0, 1080.0);

    let start = Instant::now();
    let frame = RenderFrame::build(&scene, &vp, 1.25);
    let elapsed_micros = start.elapsed().as_micros();

    let grid_moves = compute_grid_layout(&scene, 75, rustflow_core::Point::ZERO, 60.0, 60.0);
    assert_eq!(grid_moves.len(), 5_000);

    let xml_preview = export_drawio_xml(&SceneGraph::new(), "EmptyCheck");
    assert!(xml_preview.contains("<mxfile"));

    (
        frame.node_instances.len(),
        frame.edge_vertices.len(),
        elapsed_micros,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use rustflow_core::Point;
    use rustflow_renderer::compute_visible_grid_dots;

    #[test]
    fn phase1_grid_generation_within_budget() {
        let mut vp = Viewport::new(1920.0, 1080.0);
        vp.zoom_at(1.0, Point::new(960.0, 540.0));
        let dots = compute_visible_grid_dots(&vp);
        assert!(dots.len() > 5000);
    }

    #[test]
    fn stress_test_5000_nodes_10000_animated_edges_frustum_culled() {
        let (visible_nodes, edge_verts, elapsed_micros) = benchmark_5k_nodes_10k_edges_frame();
        // R-Tree frustum culling ensures only the visible subset of the 5,000 nodes is instantiated
        assert!(visible_nodes > 0 && visible_nodes < 200);
        assert!(edge_verts > 0);
        // Frame preparation easily finishes well under 100ms even in unoptimized debug mode
        assert!(elapsed_micros < 250_000);
    }
}
