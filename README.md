# Bohat Draw.io (RustFlow Draw)

A high-performance, Retro 90s Platinum diagramming studio with Draw.io-style UX, animated edge flows, Photoshop-style layers, Section/Parent/Child grouping, and realtime cooperative editing with a **Button-Mashing Ownership Minigame**.

- **Targets:** Desktop App (macOS WKWebView / Linux App Window), Web (PC, Tablet, Smartphone), & Rust Core Engine
- **Zero External Node Dependencies:** Pure Node.js HTTP + RFC 6455 WebSocket server with Brotli/Gzip pre-compressed production bundles.

---

## 1. Installation

### Option A: Install via Homebrew (macOS & Linux)
```bash
brew tap Bohatman/tap
brew install bohat-draw
```

Run as a **standalone desktop application** (no browser tab):
```bash
bohat-draw
```

Run in **headless server mode** (default port `6129`, for LAN/web & realtime co-op):
```bash
bohat-draw --server
# Or as a background service:
brew services start bohat-draw
```

### Option B: Run from Source (Node.js 20+)
```bash
git clone https://github.com/Bohatman/rustflow-draw.git
cd rustflow-draw/apps/web

# Development mode
npm run dev

# Or build & run production bundle (http://0.0.0.0:6129)
npm run build
npm start
```

### Option C: Linux `systemd --user` Background Service
```bash
# Enable user lingering so the service stays online after logout
loginctl enable-linger "$USER"

mkdir -p ~/.config/systemd/user
cp bohat-drawio.service ~/.config/systemd/user/
systemctl --user daemon-reload
systemctl --user enable --now bohat-drawio
```

---

## 2. Features & How to Use

### Canvas, Navigation & Multi-Device Support
- **PC:** Drag empty canvas to pan, scroll/pinch to zoom, `Ctrl/Cmd + 1` to fit diagram to screen, `Shift + Drag` for rubber-band box selection.
- **Tablet & Smartphone:** Single-finger drag/pan, double-tap to edit text, two-finger pinch-to-zoom & pan, and slide-over **Shapes** / **Inspector** drawers from the top toolbar.
- **Themes:** Switch between **Retro 90s Platinum**, **Dark Platinum**, **Light Modern**, and **Mono 1-Bit CRT** via **View** menu or the Inspector.

### Stencils, Shapes & Left Panel Categories
- **Add Items:** Drag any symbol from the left **Stencils & Symbols** panel onto the canvas (with live ghost preview & smart alignment guides), or click to add at center.
- **Category Filter & Collapse:** Click category chips (`All`, `Flowchart & Basic`, `Architecture`, `UML & BPMN`, `Network & Infra`, `Wireframes`) to show/hide categories (`Alt+Click` to isolate one), or click a section header (`▾` / `▸`) to collapse/expand. Searching reveals matches across all categories.
- **Custom Symbol Studio (`+ Create` / `+ Symbol`):** Draw custom vector shapes, paste SVG paths, or import `.svg`, `.png`, `.jpg`, or `.json` files into your persistent palette.

### Specialized Items
- **Image Node (`image_node`):** Renders an image directly on the canvas (drop an image file from your OS or use the **Image** toolbar button). Configure **Contain / Cover / Stretch** fit, optional border, and **Reset to Image Aspect Ratio** in the **Style** tab.
- **Folder Tree (`folder_tree`):** Edit indentation-based file/folder trees in the **Text** tab (`2 spaces = 1 level`, trailing `/` marks a folder). Click folder carets (`▸` / `▾`) directly on the canvas to expand or collapse branches.
- **Collapsible Panel (`collapsible`):** Click the header chevron (`▾` / `▸`) on the canvas to collapse/expand content, or edit the body text and collapsed state in the **Text** tab.

### Multi-Port Connections & Animated Flows
- **Hover-Reveal Ports:** Connection dots appear when hovering over an item (or when dragging an edge near a target). Supports `Auto` (1/3/5 per side by size) or `1`, `3`, `5`, `7` ports per side in the **Arrange** tab.
- **Routing & Arrow Styles:** Choose `Bezier`, `SmoothStep`, `Orthogonal`, or `Straight` routing, with `1-Way (→)`, `2-Way (↔)`, `No Arrow (—)`, `Reverse (←)`, `Open`, `Diamond (◇)`, `Socket (○)`, or custom start/end markers.
- **Flow Animations:** `Pulse` particles, `Dashed` marching flow, or `Static`. Drag an edge's midpoint handle to bend it; double-click the midpoint to reset.

### Section / Parent / Child Hierarchy
- **Section Container (`Section / Group`):** Drag items into a Section to automatically attach them; drag them out to detach.
- **Move & Nudge Together:** Moving or Arrow-nudging a Parent or Section moves all of its nested children together.
- **Group / Ungroup:** Select items and press `⇧⌘G` (or **Arrange → Group into Section**) to wrap them in a Section. Use the **Arrange** tab to set **Parent**, toggle **Section Container**, **Fit to Children**, or **Detach**.

### Photoshop-Style Layers Panel & Border Styles
- **Layers Tab:** Open the **Layers** tab in the right Inspector to view the hierarchical stack (children nested under parents).
  - Drag layer rows to reorder `z`-index (parents stay below their children so they never cover them).
  - Toggle **Visibility (`👁` / `○`)**, **Lock (`🔒` / `🔓`)**, and **Opacity (`10%–100%`)**.
- **Border Line Styles:** In the **Style** tab, customize item **Border Style** (`Solid`, `Dashed`, `Dotted`, `Dash-Dot`, `Double`, `No Border`), **Border Width**, and optional **Animate Border** (disabled by default).

### Overlap, Heap / Pile Up & Auto-Layouts
Available in the **Arrange** menu, **Arrange** tab, and `⌘K` Command Launcher:
- **Overlap Stack:** Tight alternating overlapping horizontal stack with ascending `z`-order.
- **Cascade Pile:** Classic diagonal window-style cascade (`+28px, +24px`).
- **Heap / Pile Up:** Compact organic pile around the selection centroid.
- **Separate Overlaps:** Iteratively pushes overlapping items apart cleanly.
- **Graph Auto-Layouts:** `Hierarchical` (`⇧⌘L`), `Uniform Grid`, and `Radial Ring`, plus **Align** (`Left/Center/Right/Top/Middle/Bottom`) and **Distribute** (`H/V`) for multi-selections.

### Realtime Co-Editing & Button-Mashing Ownership Minigame
1. Click **Share** in the top toolbar to start a live room and copy the shareable `?room=...` link.
2. Anyone opening the link joins the live canvas with real-time cursor tracking, item sync, and `"<Name> editing"` lock indicators.
3. **Ownership Clash Minigame:** If two co-editors try to move, resize, or edit the same item at the same time, a **5-second Button-Mashing Duel** pops up — mash **Space / Enter / Click / Tap** to win ownership of the item!

### Export & Import
- **`.drawio` XML (`⌘S`):** Bidirectional export and import preserving shapes, ports, layers, opacity, border styles, and Section/Parent hierarchy.
- **Animated `.gif`:** Self-contained GIF89a export with live edge flow animations.
- **High-DPI `.png`** and **Animated `.svg` (`⌘E`)**.

---

## 3. Keyboard Shortcuts

| Action | Shortcut |
| :--- | :--- |
| Command Launcher | `Ctrl/Cmd + K` |
| Keyboard Shortcuts | `?` |
| New Empty Workspace | `Shift + Ctrl/Cmd + N` |
| Open Diagram File | `Ctrl/Cmd + O` |
| Save `.drawio` XML | `Ctrl/Cmd + S` |
| Export Animated SVG | `Ctrl/Cmd + E` |
| Select All / Copy / Cut / Paste / Duplicate | `Ctrl/Cmd + A / C / X / V / D` |
| Undo / Redo | `Ctrl/Cmd + Z` / `Ctrl/Cmd + Y` (`Shift + Cmd + Z`) |
| Group into Section | `Shift + Ctrl/Cmd + G` |
| Bring Front / Forward / Send Backward / Back | `Cmd + ]` / `Shift+Cmd+]` / `Shift+Cmd+[` / `Cmd + [` |
| Auto-Layout Hierarchical | `Shift + Ctrl/Cmd + L` |
| Zoom In / Out / 100% / Fit | `Cmd + +` / `Cmd + -` / `Cmd + 0` / `Cmd + 1` |
| Cycle Grid Style | `Ctrl/Cmd + G` |
| Edit Label Inline | `F2` / `Enter` / Double-Click |

---

## 4. Repository Structure & Rust Core

```text
rustflow-draw/
├── Cargo.toml                 # Rust workspace manifest
├── Formula/bohat-draw.rb      # Homebrew formula (standalone app + --server)
├── crates/
│   ├── core/                  # Scene graph, R-Tree, Command Undo/Redo, CRDT, Telemetry, Plugins
│   ├── renderer/              # wgpu WGSL shaders, Bezier/Orthogonal tessellation, headless rasterizer
│   ├── layout/                # Sugiyama Hierarchical, Force-Directed, Grid & Spring transitions
│   ├── io-formats/            # SemVer Native JSON, bidirectional .drawio XML, Animated SVG & PDF
│   └── ui-bridge/             # Shared state bindings, Stencil catalog, Format Inspector & Shortcuts
├── apps/
│   ├── web/                   # Production web app, bundler (build.mjs), and WebSocket server (server.mjs)
│   └── desktop/               # Rust desktop CLI & macOS WKWebView app shell (BohatDrawApp.swift)
├── tests/                     # Visual regression & benchmark suites
└── scripts/verify-sdlc.sh     # SDLC quality gate (fmt, clippy, test, web check)
```

```bash
# Run Rust tests
cargo test

# Run SDLC verification script
./scripts/verify-sdlc.sh
```
