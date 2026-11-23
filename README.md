# Bohat Draw.io (RustFlow Draw) — 100% Rust

A 100% Rust, 120fps Retro 90s Platinum diagramming studio built with `egui`/`eframe` (Native Desktop + WebAssembly) and an `axum` HTTP/WebSocket collaboration server. Features 170+ vector cloud & architecture symbols (AWS, Azure, GCP, Kubernetes, Docker, ISO Flowchart), manual & auto edge routing, animated edge flows, Photoshop-style hierarchical layers, and realtime cooperative editing with an **Ownership Duel Minigame**.

- **Targets:** Native Desktop (`bohat-draw`) & WebAssembly (`apps/studio/dist`) across **PC, Mac, Smartphone, and Tablet**
- **100% Rust Architecture:** Zero Node.js, zero JavaScript/TypeScript framework dependencies.

---

## 1. Installation & Running

### Option A: Install via Homebrew (macOS & Linux)
```bash
brew tap Bohatman/tap
brew install bohat-draw
```

Run as a **standalone native desktop application**:
```bash
bohat-draw
```

Run in **Web + Realtime Co-Op server mode** (default port `6129`):
```bash
bohat-draw --server
# Or as a background service:
brew services start bohat-draw
```

### Option B: Build & Run from Source (Rust + Trunk)
```bash
git clone https://github.com/Bohatman/rustflow-draw.git
cd rustflow-draw

# Run native desktop app
cargo run --release -p rustflow-studio

# Or run the Web + WebSocket server (serves prebuilt WASM in apps/studio/dist on http://0.0.0.0:6129)
cargo run --release -p rustflow-studio -- --server

# Rebuild WebAssembly bundle (optional, requires trunk)
cd apps/studio && trunk build --release
```

### Option C: Linux `systemd --user` Background Service
```bash
loginctl enable-linger "$USER"

mkdir -p ~/.config/systemd/user
cp bohat-drawio.service ~/.config/systemd/user/
systemctl --user daemon-reload
systemctl --user enable --now bohat-drawio
```

---

## 2. Key Features

### Cross-Device Responsive UX (PC, Mac, Smartphone, Tablet)
- **Desktop (PC & Mac):** Full 3-column Platinum studio with virtualized **Symbols & Cloud** left palette, 120fps GPU-tiled canvas with interactive **Minimap**, and **Inspector** right panel.
- **Smartphone & Tablet:** Auto-collapses side panels on launch so the canvas gets 100% of the viewport; single-row horizontally scrollable top menubar and toolbar with quick **`Symbols`**, **`Inspector`**, and **`Pan` / `Select`** toggles; 2-finger pinch-to-zoom & pan; 1-finger canvas pan in `Pan` mode; auto-closes the Symbols drawer when tapping a symbol on smartphones.

### Copy Selection / Select All Directly as Picture into PowerPoint & Slides
- Press **`Ctrl/Cmd + A`** (`Select All`) or select any subset of nodes/edges and press **`Ctrl/Cmd + C`** (or click **Copy**):
  - Automatically renders the selection (including all AWS/Azure/GCP vector icons and connecting edges) as a crisp **2x HiDPI PNG picture** on a clean background and places it directly onto the system/browser clipboard.
  - Switch to **PowerPoint, Keynote, Google Slides, Word, or Slack** and press **`Ctrl/Cmd + V`** to paste directly as a picture, or paste inside Bohat Draw to duplicate the selected nodes and edges.

### 170+ Vector Cloud & Diagram Symbols + 3 Left-Panel View Modes
- **Providers & Categories:** 76 AWS services (EC2, Lambda, MSK, Managed Flink, S3, RDS, DynamoDB, Bedrock, VPC, EKS, etc.), 33 Microsoft Azure services, Google Cloud (GCP), Infra & DevOps (Kubernetes, Docker, Terraform, Kafka, Prometheus), Standard Icons, Flowchart & ISO, Network, UML, BPMN, and Wireframes.
- **Filterable Multi-Select Category Box** + quick provider pills (`All | AWS | Azure | GCP | Flow`) + cloud sub-group filter chips (`Compute | Storage | DB | Net | Msg | Data | Sec | Ops | AI`).
- **3 View Modes:**
  - **`Icon`**: 3-column visual icon tiles
  - **`Icon+Name`**: 1-per-row Finder-style list with vector icon, full service name, and subgroup tag
  - **`Name`**: Compact 1-per-row text list

### Smart & Manual Edge Routing + Flow Animations
- **4 Routing Styles:** `Bezier`, `SmoothStep`, `Orthogonal`, and `Straight`.
- **Manual Waypoint Override:** Drag blue midpoint `+` handles on any selected edge to insert custom waypoints; drag amber diamond handles to bend the path; double-click a waypoint or click **`Reset Auto`** in the Inspector / context menu to return to automatic routing.
- **Animated Flows:** `Marching Ants`, `Pulse Particles`, or `Static` lines with customizable arrowheads and curved `Along Path` labels.

---

## 3. Repository Structure

```text
rustflow-draw/
├── Cargo.toml                 # Rust workspace manifest
├── Formula/bohat-draw.rb      # Homebrew formula (native desktop + --server with WASM dist)
├── crates/
│   ├── core/                  # Scene graph, 133 SymbolIcon variants, R-Tree, Undo/Redo, CRDT
│   ├── renderer/              # Bezier/Ortho/Waypoint tessellation, 2x PNG & GIF89a vector rasterizer
│   ├── layout/                # Hierarchical, Force-Directed, Grid, Align/Distribute & Pile layouts
│   ├── io-formats/            # Native JSON, bidirectional .drawio XML, Animated SVG, PNG, GIF, PDF
│   ├── collab-protocol/       # Realtime WebSocket sync & Ownership Duel protocol
│   └── ui-bridge/             # Shared state bindings & 170+ stencil catalog
├── apps/
│   ├── studio/                # 100% Rust egui/eframe Native + WASM studio (with prebuilt apps/studio/dist)
│   ├── server/                # Rust axum HTTP + WebSocket server with Brotli/Gzip compression
│   └── desktop/               # CLI headless/benchmark entry point
└── tests/                     # Visual regression & 10k-node benchmark suites
```
cargo test

# Run SDLC verification script
./scripts/verify-sdlc.sh
```
