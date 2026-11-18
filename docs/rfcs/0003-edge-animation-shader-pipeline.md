# RFC 0003: Edge Animation Shader Pipeline

- **Status:** Accepted
- **Phase:** Phase 1 (Architecture & RFC)

## 1. Context
React Flow aesthetics require smooth anti-aliased Bezier/SmoothStep edges with GPU-driven animations:
1. **Marching Ants:** Animated dashed strokes with configurable speed, dash length, gap length, and direction.
2. **Data-Packet Pulses:** Glowing particles traveling along edge splines at configurable speeds and intervals.

## 2. Decision: Parametric Arc-Length UV + Instanced Pulse Quads
- **Spline Tessellation:** Edges are tessellated into adaptive polyline strips carrying cumulative arc-length `u` and signed perpendicular distance `v` in vertex attributes (`EdgeVertex`).
- **Marching Ants in Fragment Shader:** The WGSL fragment shader computes `fract((in.arc_length - uniforms.time * in.speed) / period)` so dash animation runs entirely on the GPU via a single frame timestamp uniform—zero vertex buffer rebuilds per frame when nodes are static.
- **Pulse Particles:** Sampled along the parametric curve `B(t)` using normalized phase `t = fract(time * speed + phase_offset)` and emitted as GPU-instanced radial-glow quads.
