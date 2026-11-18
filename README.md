# RustFlow Draw — Cross-Platform Diagramming Engine

**Concept:** Draw.io Functional UX + React Flow Visual Aesthetic & Animations + Rust Core Performance  
**Targets:** Web (WASM / WebGPU / HTML5 Canvas) & Desktop (Linux, macOS, Windows)

---

## 1. Architecture & Repository Structure

```text
rustflow-draw/
├── Cargo.toml                 # Workspace manifest
├── crates/
│   ├── core/                  # Scene graph, R-Tree, Command Undo/Redo, CRDT, Telemetry, Plugins
│   ├── renderer/              # wgpu WGSL shaders, Bezier/Orthogonal tessellation, headless rasterizer
│   ├── layout/                # Sugiyama Hierarchical, Force-Directed, Grid & Spring transitions
│   ├── io-formats/            # SemVer Native JSON, bidirectional .drawio XML, Animated SVG & PDF
│   └── ui-bridge/             # Shared state bindings, Stencil catalog, Format Inspector & Shortcuts
├── apps/
│   ├── web/                   # Interactive Developer Web App (Command Palette Ctrl+K, Themes, Minimap)
│   └── desktop/               # Native Desktop CLI & Engine Runner
├── tests/
│   ├── visual-regression/     # Deterministic headless pixel-diff & golden checksum VRT suite
│   └── benchmarks/            # 1k, 5k, and 10k node + 15k animated edge stress tests
├── docs/
│   ├── SRS_and_SDLC_Plan.md   # Full Software Requirements Specification & 6-Phase SDLC Plan
│   └── rfcs/                  # Architectural Decision Records (RFC 0001 - RFC 0004)
└── scripts/
    └── verify-sdlc.sh         # One-command SDLC quality gate (fmt, clippy, test, web check)
```

---

## 2. Quick Start

### Run the Interactive Web App
```bash
cd apps/web
npm run dev
```
Open **`http://localhost:5173`** in your browser.

### Developer UX Highlights in the Web App
- **`Ctrl/Cmd + K` Command Palette:** Fuzzy-search any stencil, layout algorithm (`Sugiyama Hierarchical`, `Grid`), theme (`Dark`, `Light`, `High Contrast`), or export format.
- **Draw.io Quick-Connect Arrows (`↑ → ↓ ←`):** Select any node and click a directional arrow to clone and connect in 1 click.
- **GPU-Style Edge Animations:** Switch between **Pulse Particles**, **Marching Ants**, and **Static** edges across `Bezier`, `SmoothStep`, `Orthogonal`, and `Straight` routers.
- **Smart Alignment Guides & 8-Point Resize:** Drag nodes to see dynamic center/edge snap guides; drag the bottom-right handle (hold `Shift` to lock aspect ratio).
- **Inline Editing:** Double-click any node on the canvas to edit its title inline.

### Run Desktop Shell & SDLC Quality Gate
```bash
# Run the desktop engine binary
cargo run -p rustflow-desktop

# Run formatting, strict clippy, unit/VRT/benchmark tests, and web verification
./scripts/verify-sdlc.sh
```
