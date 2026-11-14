# Software Requirements Specification (SRS) & SDLC Way of Work
**Project Name:** RustFlow Draw (Cross-Platform Diagramming Engine)  
**Concept:** Draw.io Functional UX + React Flow Visual Aesthetic & Animations + Rust Core Performance  
**Target Platforms:** Web (WASM), Desktop (Windows, macOS, Linux via Tauri/Native), Mobile-Ready  

---

## Part 1: Software Requirements Specification (SRS)

### 1. Executive Summary
This project aims to build a next-generation, cross-platform diagramming application. It marries the comprehensive, battle-tested user experience (UX) and stencil ecosystem of **Draw.io (diagrams.net)** with the modern, node-based visual aesthetic, crisp UI, and smooth edge animations of **React Flow**, all powered by a high-performance, memory-safe **Rust** core engine.

### 2. Core Architecture & Tech Stack
* **Core Engine (`rust-core`):** Pure Rust library handling the scene graph, spatial indexing (R-Tree/Quadtree), command pattern (Undo/Redo), graph layout algorithms, collision detection, and serialization.
* **Graphics & Rendering Pipeline:** Hardware-accelerated 2D vector rendering using **`wgpu`** (compiling seamlessly to WebGPU/WebGL2 on Web, Metal on macOS, Vulkan/DX12 on Windows/Linux) paired with **`Vello`** or **`Lyon`** for anti-aliased Bezier curves, shadows, and GPU-instanced shapes.
* **UI Shell & Cross-Platform Targets:**
  * **Web:** Compiled to **WebAssembly (`wasm32-unknown-unknown`)** with thin TypeScript/HTML overlays for accessible text inputs and context menus.
  * **Desktop:** **Tauri v2** (Rust backend + lightweight native webview) for native file system dialogs, OS menus, and small binary footprints (~10–15 MB).
* **State & Collaboration:** Local-first storage with optional real-time multiplayer via **CRDTs** (`yrs` / `automerge-rs`).

---

### 3. Functional Requirements

#### 3.1 Canvas & Viewport (Draw.io UX)
* **Infinite Canvas:** Hardware-accelerated pan (`Space + Drag`, Middle-click, two-finger trackpad) and cursor-centered smooth zoom (`10%` to `800%`).
* **Grid & Smart Snapping:** Toggleable dot/line grid (React Flow aesthetic) with snap-to-grid, dynamic alignment guides (center/edge alignment), and live distance/spacing indicators.
* **Navigation Aids:** Interactive floating Minimap (bottom-right) with viewport frustum box, plus an Outline/Layers tree panel.
* **Multi-Page Workspaces:** Tabbed pages at the bottom of the window matching Draw.io's multi-tab document structure.

#### 3.2 Nodes & Shape Libraries (Hybrid Model)
* **React Flow Visual Styling:** Rounded card containers, subtle multi-layered drop shadows, header/body slots, theme tokens (Light/Dark/High-Contrast), and explicit input/output **Handles (Ports)**.
* **Draw.io Rich Shape Ecosystem:** Built-in vector shape palettes (Flowchart, UML, BPMN, Cloud Architecture—AWS/GCP/Azure, Network, Wireframes) plus custom SVG/image drop support.
* **Node Manipulation:**
  * Multi-select via marquee drag or `Shift + Click`.
  * Grouping/Ungrouping, container nesting (sub-flows), locking, and Z-index layering (`Bring to Front`, `Send to Back`).
  * Inline rich-text editing (double-click any node or edge to edit formatted labels).
  * 8-point resize handles with aspect-ratio locking (`Shift`) and rotation controls.

#### 3.3 Edges & Animations (React Flow Style)
* **Routing Algorithms:** Bezier, SmoothStep (rounded orthogonal), Straight, and Draw.io-style smart obstacle-avoiding orthogonal routing with draggable manual waypoints.
* **Connection UX:** Drag-from-port connection creation with magnetic target snapping, plus Draw.io's quick-add directional hover arrows (click to clone and connect in 4 directions).
* **GPU-Driven Animations:**
  * Animated dashed flows ("marching ants") with configurable speed, dash length, and direction.
  * Data-packet pulse animations (glowing dots/particles traveling along edge splines).
  * Spring-physics or cubic-bezier transitions when auto-layout runs or sub-flows expand/collapse.

#### 3.4 Toolbars, Panels & Draw.io UX Parity
* **Left Sidebar (Shape Palette):** Searchable, collapsible accordion of stencil libraries with drag-and-drop onto canvas and hover previews.
* **Right Sidebar (Format Inspector):** Context-sensitive tabs:
  * *Style:* Fill, gradient, stroke, opacity, drop shadow, corner radius, edge animation presets.
  * *Text:* Font family, weight, size, alignment, color, padding.
  * *Arrange:* Exact X/Y coordinates, Width/Height, rotation, layer order, and auto-layout triggers.
* **Top Menubar & Quick Toolbar:** File operations, Undo/Redo, Zoom presets, Alignment/Distribution tools, and default Edge Type/Animation switchers.
* **Keyboard Shortcuts:** Full parity with Draw.io (`Ctrl/Cmd + D` duplicate, `Ctrl/Cmd + G` group, `Delete/Backspace`, arrow-key nudging).

#### 3.5 Import, Export & Interoperability
* **Native Format:** Fast, versioned JSON/MessagePack format preserving animation configurations and custom metadata.
* **Draw.io Compatibility:** Rust-native bidirectional `.drawio` (XML) parser and serializer.
* **Export Pipeline:** Vector export (**SVG** with embedded CSS `@keyframes` to preserve edge animations), high-DPI Raster (**PNG/WebP** at 1x–4x), and **PDF**.

---

### 4. Non-Functional Requirements
* **Rendering Performance:** Maintain **60–120 FPS** with **5,000+ nodes and 10,000+ animated edges** using spatial R-Tree frustum culling and GPU instancing.
* **Web Bundle Size:** Core WASM payload under **2.5 MB** (gzipped) with cold-start initialization under **300 ms**.
* **Reliability & Memory Safety:** Zero-panic production guarantees in Rust core (`Result`-based error handling, fuzz-tested XML/JSON parsers).
* **Extensibility:** Plugin architecture allowing custom shape packs and custom layout algorithms (e.g., Sugiyama hierarchical, Force-directed).

---

## Part 2: Way of Work & SDLC Methodology

To build a complex graphics + desktop/web application efficiently without getting bogged down in cross-platform divergences, we adopt an **Iterative Dual-Track Agile SDLC** tailored for Rust systems and UI engineering.

### 1. SDLC Model: Dual-Track Iterative Agile
* **Track 1: Discovery & UX Prototyping (1-Week Sprints ahead):** Wireframing Draw.io UX flows, defining React Flow shader/animation aesthetics, and validating interaction edge cases before Rust implementation.
* **Track 2: Delivery & Engineering (2-Week Sprints):** Vertical-slice development where every sprint produces a runnable binary on **both** Desktop (Tauri) and Web (WASM) simultaneously.

---

### 2. The 6 Phases of the Lifecycle

#### Phase 1: Architecture & RFC (Request for Comments)
* **Goal:** Lock down technical boundaries before writing feature code.
* **Activities:**
  * Set up a **Cargo Workspace** separating platform-agnostic logic (`engine-core`, `renderer-wgpu`, `io-drawio`) from platform shells (`app-web`, `app-desktop`).
  * Write lightweight architectural RFCs for complex subsystems (e.g., *Spatial Indexing Strategy*, *Edge Animation Shader Pipeline*).
* **Exit Criteria:** Compiling "Hello Triangle / Pan-Zoom Grid" running natively on Desktop and in Browser via WASM.

#### Phase 2: Vertical-Slice Feature Development
* **Goal:** Build features end-to-end (Core State $\rightarrow$ Renderer $\rightarrow$ UI Controls) rather than building isolated layers.
* **Sprint Cadence (2 Weeks):**
  * **Day 1:** Sprint Planning & Acceptance Criteria review.
  * **Days 2–8:** Core Rust implementation + unit/property tests + UI integration.
  * **Days 9–10:** Cross-platform QA (testing WebGL/WebGPU vs. Metal/Vulkan parity), performance profiling, and Sprint Demo.

#### Phase 3: Quality Assurance & Automated Testing
* **Core Engine Tests:** Standard `cargo test` for graph mutations, undo/redo invariants, and `.drawio` XML round-trip parsing.
* **Visual Regression Testing (VRT):** Headless `wgpu` rendering tests that capture framebuffers of complex animated diagrams and compare pixel diffs against golden reference images.
* **Performance Benchmarks:** Automated `criterion` benchmarks tracking frame times at 1k, 5k, and 10k nodes on every pull request.

#### Phase 4: CI/CD & Release Engineering
* **Continuous Integration (on every PR):**
  * `cargo fmt --check` & `cargo clippy -- -D warnings` (strict linting).
  * Cross-compilation verification for `wasm32-unknown-unknown`, `x86_64-pc-windows-msvc`, `aarch64-apple-darwin`, and `x86_64-unknown-linux-gnu`.
* **Continuous Delivery:**
  * **Web:** Automatic staging deployment to CDN/Vercel/Cloudflare Pages on merge to `main`.
  * **Desktop:** Automated signed binary builds (`.dmg`, `.msi`, `.AppImage`) via GitHub Actions on tagged releases (`vX.Y.Z`).

#### Phase 5: User Acceptance & Beta Telemetry
* **Dogfooding:** Use the tool internally to draw its own architecture diagrams.
* **Opt-In Diagnostics:** Collect frame-time histograms and crash backtraces (via Sentry/Crashpad) to catch GPU driver quirks across different browsers and OSs.

#### Phase 6: Maintenance & Plugin Ecosystem
* **Semantic Versioning (`SemVer`):** Maintain strict backwards compatibility for the native file format and `.drawio` importer.
* **Community/Internal Extensions:** Expose a stable API for adding new stencil libraries and edge animation shaders without modifying the core engine.

---

### 3. Suggested Repository Structure (Cargo Workspace)
```text
rustflow-draw/
├── Cargo.toml                 # Virtual workspace manifest
├── crates/
│   ├── core/                  # Scene graph, R-Tree, Command stack, Undo/Redo
│   ├── renderer/              # wgpu shaders, Bezier tessellation, edge animations
│   ├── layout/                # Auto-layout engines (Hierarchical, Force, Grid)
│   ├── io-formats/            # Native JSON/MsgPack & .drawio XML parser/exporter
│   └── ui-bridge/             # Shared state bindings (WASM bindgen & Tauri commands)
├── apps/
│   ├── web/                   # Vite + TypeScript + WASM loader + DOM text overlays
│   └── desktop/               # Tauri v2 wrapper + native OS menu & file system hooks
├── tests/
│   ├── visual-regression/     # Headless GPU snapshot comparisons
│   └── benchmarks/            # Criterion stress tests (5k+ nodes)
└── docs/
    └── rfcs/                  # Architecture decision records
```

---

### 4. Phased Delivery Roadmap
1. **Milestone 1 (Foundation — Weeks 1–4):** Cargo workspace, `wgpu` infinite canvas, dot grid, basic React Flow node cards, and Bezier edge rendering on Web & Desktop.
2. **Milestone 2 (Draw.io UX Core — Weeks 5–8):** Left shape palette, right format inspector, multi-select, smart alignment guides, undo/redo stack, and inline text editing.
3. **Milestone 3 (Animations & Polish — Weeks 9–12):** GPU edge flow animations (marching ants & pulse particles), spring layout transitions, minimap, and multi-page tabs.
4. **Milestone 4 (Interoperability & Launch — Weeks 13–16):** `.drawio` XML import/export, animated SVG export, high-DPI PNG/PDF export, and v1.0 cross-platform release.
