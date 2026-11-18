# RFC 0001: Cargo Workspace & Engine Architecture

- **Status:** Accepted
- **Phase:** Phase 1 (Architecture & RFC)

## 1. Context & Problem Statement
RustFlow Draw targets both Web (WASM + WebGPU/WebGL2) and Desktop (Tauri v2 + native Vulkan/Metal/DX12) from a single Rust codebase while maintaining 60–120 FPS with 5,000+ nodes and 10,000+ animated edges.

## 2. Decision: Strict Layer Separation in a Cargo Workspace
We partition the repository into five platform-agnostic library crates, two test suites, and two platform shells:

1. `crates/core` (`rustflow-core`):
   - Pure Rust scene graph (`Document`, `Page`, `Node`, `Edge`, `Port`).
   - Spatial indexing (`RTree`) for frustum culling, hit-testing, and smart alignment guides.
   - Deterministic `CommandHistory` (Command pattern for Undo/Redo).
   - `Viewport` math (infinite pan and cursor-centered zoom clamped to `0.10..=8.00`).
2. `crates/renderer` (`rustflow-renderer`):
   - WGSL shader pipeline definitions (`grid.wgsl`, `node_card.wgsl`, `edge_bezier.wgsl`).
   - Curve tessellation (Cubic Bezier, SmoothStep, Straight, Orthogonal routing).
   - GPU instance buffer builder (`RenderFrame`) consuming visible entities from `rustflow-core`.
   - Deterministic headless software rasterizer for visual regression tests.
3. `crates/layout` (`rustflow-layout`):
   - Graph auto-layout algorithms (Sugiyama Hierarchical, Force-Directed, Grid) and spring-physics transition interpolation.
4. `crates/io-formats` (`rustflow-io-formats`):
   - Versioned native JSON format, bidirectional `.drawio` (`mxfile`) XML parser/serializer, and animated SVG/PDF/raster exporters.
5. `crates/ui-bridge` (`rustflow-ui-bridge`):
   - Shared state bindings, Draw.io stencil palette catalog, Format Inspector state (`Style`, `Text`, `Arrange`), and keyboard shortcut dispatcher for Web (`wasm-bindgen`) and Desktop (Tauri commands).

## 3. Consequences
- Core logic compiles without GPU or DOM dependencies and runs in standard `cargo test` in sub-second time.
- Platform shells (`apps/web`, `apps/desktop`) remain thin wrappers around `rustflow-ui-bridge`.
