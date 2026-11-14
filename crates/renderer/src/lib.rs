//! Hardware-accelerated wgpu/WGSL shader descriptors and headless pan-zoom grid renderer.

use rustflow_core::{GridStyle, Point, Viewport};

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

/// Compute visible grid dot screen positions for both CPU verification and instanced rendering.
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grid_uniforms_and_visible_dots_work() {
        let vp = Viewport::new(200.0, 100.0);
        let uniforms = GridUniforms::from_viewport(&vp);
        assert_eq!(uniforms.zoom, 1.0);
        assert_eq!(uniforms.style, 1);
        let dots = compute_visible_grid_dots(&vp);
        assert!(!dots.is_empty());
        assert!(GRID_SHADER_WGSL.contains("@fragment"));
    }
}
