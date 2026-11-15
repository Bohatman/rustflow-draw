// High-Performance Draw.io + React Flow Web Engine
// Fixes:
// 1. Draw.io Dropdown Menu Bar (File, Edit, View, Arrange, Extras) + compact quick toolbar
// 2. Draw.io Context-Sensitive Right Format Panel (Diagram vs Node [Style/Text/Arrange] vs Edge [Style/Text/Arrange])
// 3. 60-120 FPS Performance Optimization: Offscreen cached grid pattern (O(1) draw), zero shadowBlur, O(1) node map, cached edge geometry, frustum culling, and dirty-checked minimap

const STENCILS = [
  // Flowchart
  { id: "flowchart.process", icon: "▭", label: "Process Card", category: "Flowchart", w: 175, h: 78, shape: "card" },
  { id: "flowchart.rounded", icon: "⬭", label: "Start / End Pill", category: "Flowchart", w: 155, h: 64, shape: "rounded" },
  { id: "flowchart.decision", icon: "◇", label: "Decision Diamond", category: "Flowchart", w: 140, h: 100, shape: "diamond" },
  { id: "flowchart.database", icon: "🛢", label: "Database Cylinder", category: "Flowchart", w: 135, h: 96, shape: "cylinder" },
  { id: "flowchart.data_io", icon: "▱", label: "Data I/O", category: "Flowchart", w: 165, h: 76, shape: "parallelogram" },
  { id: "flowchart.hexagon", icon: "⬡", label: "Prep Hexagon", category: "Flowchart", w: 160, h: 80, shape: "hexagon" },
  { id: "flowchart.document", icon: "📄", label: "Document", category: "Flowchart", w: 160, h: 86, shape: "document" },
  { id: "flowchart.note", icon: "📝", label: "Sticky Note", category: "Flowchart", w: 150, h: 92, shape: "note" },
  { id: "flowchart.circle", icon: "◯", label: "Circle / Junction", category: "Flowchart", w: 86, h: 86, shape: "circle" },

  // UML & BPMN
  { id: "uml.class", icon: "⊞", label: "UML Class", category: "UML & BPMN", w: 185, h: 100, shape: "uml_class" },
  { id: "uml.interface", icon: "«»", label: "UML Interface", category: "UML & BPMN", w: 180, h: 90, shape: "uml_interface" },
  { id: "uml.component", icon: "🧩", label: "UML Component", category: "UML & BPMN", w: 180, h: 86, shape: "uml_component" },
  { id: "uml.package", icon: "📁", label: "UML Package", category: "UML & BPMN", w: 180, h: 96, shape: "uml_package" },
  { id: "uml.actor", icon: "🧍", label: "UML Actor", category: "UML & BPMN", w: 86, h: 112, shape: "actor" },
  { id: "bpmn.gateway.xor", icon: "⊗", label: "XOR Gateway (×)", category: "UML & BPMN", w: 96, h: 96, shape: "gateway_xor" },
  { id: "bpmn.gateway.par", icon: "⊕", label: "Parallel Gateway (+)", category: "UML & BPMN", w: 96, h: 96, shape: "gateway_parallel" },
  { id: "bpmn.timer", icon: "⏱", label: "Timer Event", category: "UML & BPMN", w: 86, h: 86, shape: "timer" },

  // Cloud & Kubernetes
  { id: "cloud.aws.lambda", icon: "λ", label: "AWS Lambda", category: "Cloud & K8s", w: 165, h: 80, shape: "hexagon" },
  { id: "cloud.aws.s3", icon: "🪣", label: "AWS S3 Storage", category: "Cloud & K8s", w: 140, h: 96, shape: "cylinder" },
  { id: "cloud.gcp.cloudrun", icon: "☁", label: "Cloud Service", category: "Cloud & K8s", w: 175, h: 94, shape: "cloud" },
  { id: "cloud.k8s.pod", icon: "☸", label: "K8s Pod Replica", category: "Cloud & K8s", w: 170, h: 80, shape: "hexagon" },

  // Network & Infra
  { id: "network.queue", icon: "⇄", label: "Kafka / Queue", category: "Network & Infra", w: 175, h: 78, shape: "queue" },
  { id: "network.server", icon: "🖥", label: "Server Rack", category: "Network & Infra", w: 165, h: 96, shape: "server" },
  { id: "network.firewall", icon: "🧱", label: "WAF Firewall", category: "Network & Infra", w: 160, h: 84, shape: "firewall" },
  { id: "network.shield", icon: "🛡", label: "Auth Shield", category: "Network & Infra", w: 130, h: 104, shape: "shield" },

  // Wireframes
  { id: "wireframe.browser", icon: "🌐", label: "Browser Window", category: "Wireframes", w: 230, h: 145, shape: "browser" },
  { id: "wireframe.mobile", icon: "📱", label: "Mobile Screen", category: "Wireframes", w: 125, h: 195, shape: "mobile" },
];

const STYLE_SWATCHES = [
  { fill: "#1e293b", stroke: "#38bdf8" },
  { fill: "#1e1b4b", stroke: "#818cf8" },
  { fill: "#064e3b", stroke: "#34d399" },
  { fill: "#451a03", stroke: "#fbbf24" },
  { fill: "#4c0519", stroke: "#f43f5e" },
  { fill: "#3b0764", stroke: "#c084fc" },
  { fill: "#ffffff", stroke: "#4f46e5" },
  { fill: "#ecfeff", stroke: "#0891b2" },
  { fill: "#f0fdf4", stroke: "#16a34a" },
  { fill: "#fffbeb", stroke: "#d97706" },
  { fill: "#fff1f2", stroke: "#e11d48" },
  { fill: "#0f172a", stroke: "#f8fafc" },
];

function createDogfoodPages() {
  return [
    {
      id: 1,
      name: "Page-1: RustFlow Architecture",
      pan: { x: 50, y: 40 },
      zoom: 1.0,
      nodes: [
        { id: 1, title: "crates/core", body: "R-Tree + Scene + CRDT", x: 70, y: 155, w: 185, h: 82, fill: "#1e293b", stroke: "#818cf8", shape: "uml_package", z: 1 },
        { id: 2, title: "crates/renderer", body: "wgpu WGSL + Bezier", x: 355, y: 45, w: 185, h: 82, fill: "#1e293b", stroke: "#38bdf8", shape: "uml_component", z: 2 },
        { id: 3, title: "crates/layout", body: "Sugiyama + Force + Spring", x: 355, y: 160, w: 185, h: 82, fill: "#1e293b", stroke: "#34d399", shape: "hexagon", z: 3 },
        { id: 4, title: "crates/io-formats", body: ".drawio XML + SVG + PDF", x: 355, y: 280, w: 185, h: 86, fill: "#1e293b", stroke: "#fbbf24", shape: "document", z: 4 },
        { id: 5, title: "crates/ui-bridge", body: "WASM & Tauri v2 Shell", x: 650, y: 155, w: 200, h: 105, fill: "#1e293b", stroke: "#f472b6", shape: "browser", z: 5 },
      ],
      edges: [
        { id: 101, source: 1, sourcePort: "right", target: 2, targetPort: "left", label: "Frustum Query", anim: "pulse", kind: "bezier", color: "#818cf8", width: 2.2, bend: null },
        { id: 102, source: 1, sourcePort: "right", target: 3, targetPort: "left", label: "Graph Moves", anim: "ants", kind: "bezier", color: "#34d399", width: 2.2, bend: null },
        { id: 103, source: 1, sourcePort: "right", target: 4, targetPort: "left", label: "AST Round-Trip", anim: "pulse", kind: "bezier", color: "#fbbf24", width: 2.2, bend: null },
        { id: 104, source: 2, sourcePort: "right", target: 5, targetPort: "left", label: "Frame Buffers", anim: "pulse", kind: "bezier", color: "#38bdf8", width: 2.2, bend: null },
        { id: 105, source: 3, sourcePort: "right", target: 5, targetPort: "left", label: "Transitions", anim: "ants", kind: "bezier", color: "#818cf8", width: 2.2, bend: null },
        { id: 106, source: 4, sourcePort: "right", target: 5, targetPort: "left", label: "Exporters", anim: "ants", kind: "bezier", color: "#f472b6", width: 2.2, bend: null },
      ],
      undoStack: [],
      redoStack: [],
    },
    {
      id: 2,
      name: "Page-2: Cloud & BPMN Sandbox",
      pan: { x: 60, y: 50 },
      zoom: 1.0,
      nodes: [
        { id: 10, title: "Client Actor", body: "HTTPS", x: 70, y: 120, w: 86, h: 112, fill: "#1e293b", stroke: "#38bdf8", shape: "actor", z: 1 },
        { id: 11, title: "WAF Firewall", body: "Rate Limiter", x: 240, y: 135, w: 160, h: 84, fill: "#1e293b", stroke: "#f43f5e", shape: "firewall", z: 2 },
        { id: 12, title: "Auth Check", body: "XOR", x: 480, y: 130, w: 96, h: 96, fill: "#1e293b", stroke: "#fbbf24", shape: "gateway_xor", z: 3 },
        { id: 13, title: "Kafka Stream", body: "events-topic", x: 660, y: 80, w: 175, h: 78, fill: "#1e293b", stroke: "#34d399", shape: "queue", z: 4 },
        { id: 14, title: "Postgres Primary", body: "pgvector", x: 675, y: 210, w: 140, h: 96, fill: "#1e293b", stroke: "#818cf8", shape: "cylinder", z: 5 },
      ],
      edges: [
        { id: 201, source: 10, sourcePort: "right", target: 11, targetPort: "left", label: "Request", anim: "pulse", kind: "bezier", color: "#38bdf8", width: 2.2, bend: null },
        { id: 202, source: 11, sourcePort: "right", target: 12, targetPort: "left", label: "Verify", anim: "ants", kind: "bezier", color: "#fbbf24", width: 2.2, bend: null },
        { id: 203, source: 12, sourcePort: "right", target: 13, targetPort: "left", label: "Publish", anim: "pulse", kind: "bezier", color: "#34d399", width: 2.2, bend: null },
        { id: 204, source: 12, sourcePort: "bottom", target: 14, targetPort: "left", label: "Persist", anim: "pulse", kind: "orthogonal", color: "#818cf8", width: 2.2, bend: null },
      ],
      undoStack: [],
      redoStack: [],
    },
  ];
}

const state = {
  pages: createDogfoodPages(),
  activePageId: 1,
  selectedNodeId: 1,
  selectedEdgeId: null,
  activeFormatTab: "style", // "style" | "text" | "arrange"
  gridStyle: "dots",        // "dots" | "lines" | "off"
  gridSize: 20,
  hoveredPort: null,
  snapTargetPort: null,
  draggingNode: null,
  resizingNode: null,
  draggingEdgeBend: null,
  reconnectingEdge: null,
  panning: null,
  connectingFrom: null,
  mouseWorld: { x: 0, y: 0 },
  smartGuides: [],
  nextId: 400,
  edgeAnimPreset: "pulse",
  edgeKindPreset: "bezier",
  theme: "dark",
  fps: 60,
  lastFrameTime: performance.now(),
  geometryVersion: 1,
  minimapDirty: true,
};

// Performance Cache: O(1) Node Lookup Map & Offscreen Grid Pattern Cache
let nodeMap = new Map();
let cachedGridPattern = null;
let cachedGridKey = "";

function rebuildNodeMap() {
  const p = page();
  nodeMap.clear();
  for (const n of p.nodes) {
    nodeMap.set(n.id, n);
  }
}

function invalidateGeometry() {
  state.geometryVersion++;
  state.minimapDirty = true;
  rebuildNodeMap();
}

function page() {
  return state.pages.find((p) => p.id === state.activePageId) || state.pages[0];
}

function saveSnapshot() {
  const p = page();
  p.undoStack.push(
    JSON.stringify({
      nodes: p.nodes,
      edges: p.edges.map(({ _geo, _ver, ...rest }) => rest),
    })
  );
  if (p.undoStack.length > 64) p.undoStack.shift();
  p.redoStack = [];
}

const canvas = document.getElementById("rustflow-canvas");
const ctx = canvas.getContext("2d", { alpha: false });
const minimapCanvas = document.getElementById("minimap-canvas");
const minimapCtx = minimapCanvas.getContext("2d");
const inlineEditor = document.getElementById("inline-editor");

function resizeCanvas() {
  const rect = canvas.parentElement.getBoundingClientRect();
  canvas.width = rect.width;
  canvas.height = rect.height;
  state.minimapDirty = true;
}
window.addEventListener("resize", resizeCanvas);
resizeCanvas();
rebuildNodeMap();

function screenToWorld(sx, sy) {
  const p = page();
  return { x: (sx - p.pan.x) / p.zoom, y: (sy - p.pan.y) / p.zoom };
}

function worldToScreen(wx, wy) {
  const p = page();
  return { x: wx * p.zoom + p.pan.x, y: wy * p.zoom + p.pan.y };
}

function getNodePorts(n) {
  return [
    { side: "top", x: n.x + n.w * 0.5, y: n.y },
    { side: "right", x: n.x + n.w, y: n.y + n.h * 0.5 },
    { side: "bottom", x: n.x + n.w * 0.5, y: n.y + n.h },
    { side: "left", x: n.x, y: n.y + n.h * 0.5 },
  ];
}

function getPortPosition(node, side) {
  if (side === "top") return { side: "top", x: node.x + node.w * 0.5, y: node.y };
  if (side === "bottom") return { side: "bottom", x: node.x + node.w * 0.5, y: node.y + node.h };
  if (side === "left") return { side: "left", x: node.x, y: node.y + node.h * 0.5 };
  return { side: "right", x: node.x + node.w, y: node.y + node.h * 0.5 };
}

function findSnapTargetPort(worldPt, excludeNodeId) {
  const p = page();
  const portSnapRadius = 22 / Math.max(0.4, p.zoom);

  let best = null;
  for (const n of p.nodes) {
    if (n.id === excludeNodeId) continue;
    for (const port of getNodePorts(n)) {
      const d = Math.hypot(worldPt.x - port.x, worldPt.y - port.y);
      if (d <= portSnapRadius && (!best || d < best.dist)) {
        best = { dist: d, node: n, port };
      }
    }
  }
  if (best) return best;

  const margin = 14 / Math.max(0.4, p.zoom);
  for (let i = p.nodes.length - 1; i >= 0; i--) {
    const n = p.nodes[i];
    if (n.id === excludeNodeId) continue;
    if (
      worldPt.x >= n.x - margin &&
      worldPt.x <= n.x + n.w + margin &&
      worldPt.y >= n.y - margin &&
      worldPt.y <= n.y + n.h + margin
    ) {
      let closestPort = getNodePorts(n)[0];
      let minD = Infinity;
      for (const port of getNodePorts(n)) {
        const d = Math.hypot(worldPt.x - port.x, worldPt.y - port.y);
        if (d < minD) {
          minD = d;
          closestPort = port;
        }
      }
      return { dist: minD, node: n, port: closestPort };
    }
  }
  return null;
}

function getQuickArrows(n) {
  const pad = 24;
  return [
    { dir: "up", side: "top", targetSide: "bottom", x: n.x + n.w / 2, y: n.y - pad, dx: 0, dy: -(n.h + 75), label: "↑" },
    { dir: "right", side: "right", targetSide: "left", x: n.x + n.w + pad, y: n.y + n.h / 2, dx: n.w + 95, dy: 0, label: "→" },
    { dir: "down", side: "bottom", targetSide: "top", x: n.x + n.w / 2, y: n.y + n.h + pad, dx: 0, dy: n.h + 75, label: "↓" },
    { dir: "left", side: "left", targetSide: "right", x: n.x - pad, y: n.y + n.h / 2, dx: -(n.w + 95), dy: 0, label: "←" },
  ];
}

function evalBezier(p0, p1, p2, p3, t) {
  const u = 1 - t;
  return {
    x: u * u * u * p0.x + 3 * u * u * t * p1.x + 3 * u * t * t * p2.x + t * t * t * p3.x,
    y: u * u * u * p0.y + 3 * u * u * t * p1.y + 3 * u * t * t * p2.y + t * t * t * p3.y,
  };
}

// Cached O(1) Edge Geometry Lookup — only recomputes when geometryVersion changes!
function getEdgeGeometry(e) {
  if (e._geo && e._ver === state.geometryVersion) {
    return e._geo;
  }
  const src = nodeMap.get(e.source);
  const dst = nodeMap.get(e.target);
  if (!src || !dst) return null;

  const p0 = getPortPosition(src, e.sourcePort || "right");
  const p3 = getPortPosition(dst, e.targetPort || "left");

  const tangentOffset = (side, dist) => {
    if (side === "top") return { x: 0, y: -dist };
    if (side === "bottom") return { x: 0, y: dist };
    if (side === "left") return { x: -dist, y: 0 };
    return { x: dist, y: 0 };
  };

  const span = Math.max(55, Math.hypot(p3.x - p0.x, p3.y - p0.y) * 0.38);
  let p1, p2, midPoint;

  if (e.bend) {
    midPoint = { x: e.bend.x, y: e.bend.y };
    const cx = (8 * midPoint.x - p0.x - p3.x) / 6;
    const cy = (8 * midPoint.y - p0.y - p3.y) / 6;
    p1 = { x: cx, y: cy };
    p2 = { x: cx, y: cy };
  } else {
    const t1 = tangentOffset(e.sourcePort || "right", span);
    const t2 = tangentOffset(e.targetPort || "left", span);
    p1 = { x: p0.x + t1.x, y: p0.y + t1.y };
    p2 = { x: p3.x + t2.x, y: p3.y + t2.y };
    midPoint = evalBezier(p0, p1, p2, p3, 0.5);
  }

  const samples = [];
  if (e.kind === "straight") {
    if (e.bend) {
      samples.push(p0, midPoint, p3);
    } else {
      samples.push(p0, p3);
      midPoint = { x: (p0.x + p3.x) / 2, y: (p0.y + p3.y) / 2 };
    }
  } else if (e.kind === "orthogonal" || e.kind === "smoothstep") {
    const bendX = e.bend ? e.bend.x : (p0.x + p3.x) * 0.5;
    const bendY = e.bend ? e.bend.y : (p0.y + p3.y) * 0.5;
    midPoint = { x: bendX, y: bendY };
    samples.push(p0, { x: bendX, y: p0.y }, { x: bendX, y: p3.y }, p3);
  } else {
    for (let i = 0; i <= 20; i++) {
      samples.push(evalBezier(p0, p1, p2, p3, i / 20));
    }
  }

  const minX = Math.min(p0.x, p3.x, midPoint.x) - 20;
  const maxX = Math.max(p0.x, p3.x, midPoint.x) + 20;
  const minY = Math.min(p0.y, p3.y, midPoint.y) - 20;
  const maxY = Math.max(p0.y, p3.y, midPoint.y) + 20;

  e._geo = { p0, p1, p2, p3, midPoint, samples, bounds: { minX, maxX, minY, maxY } };
  e._ver = state.geometryVersion;
  return e._geo;
}

function distToSegment(pt, a, b) {
  const dx = b.x - a.x;
  const dy = b.y - a.y;
  const lenSq = dx * dx + dy * dy;
  if (lenSq < 1e-4) return Math.hypot(pt.x - a.x, pt.y - a.y);
  const t = Math.max(0, Math.min(1, ((pt.x - a.x) * dx + (pt.y - a.y) * dy) / lenSq));
  return Math.hypot(pt.x - (a.x + t * dx), pt.y - (a.y + t * dy));
}

function hitTestEdge(worldPt) {
  const p = page();
  const tolerance = 9 / Math.max(0.4, p.zoom);
  for (let i = p.edges.length - 1; i >= 0; i--) {
    const e = p.edges[i];
    const geo = getEdgeGeometry(e);
    if (!geo) continue;
    for (let j = 0; j + 1 < geo.samples.length; j++) {
      if (distToSegment(worldPt, geo.samples[j], geo.samples[j + 1]) <= tolerance) {
        return e;
      }
    }
  }
  return null;
}

function computeSmartGuides(movingNode) {
  const guides = [];
  const threshold = 6;
  for (const other of page().nodes) {
    if (other.id === movingNode.id) continue;
    const checksX = [
      [movingNode.x, other.x],
      [movingNode.x + movingNode.w / 2, other.x + other.w / 2],
      [movingNode.x + movingNode.w, other.x + other.w],
    ];
    for (const [mx, ox] of checksX) {
      if (Math.abs(mx - ox) <= threshold) {
        movingNode.x += ox - mx;
        guides.push({ type: "v", x: ox, y1: Math.min(movingNode.y, other.y) - 24, y2: Math.max(movingNode.y + movingNode.h, other.y + other.h) + 24 });
        break;
      }
    }
    const checksY = [
      [movingNode.y, other.y],
      [movingNode.y + movingNode.h / 2, other.y + other.h / 2],
      [movingNode.y + movingNode.h, other.y + other.h],
    ];
    for (const [my, oy] of checksY) {
      if (Math.abs(my - oy) <= threshold) {
        movingNode.y += oy - my;
        guides.push({ type: "h", y: oy, x1: Math.min(movingNode.x, other.x) - 24, x2: Math.max(movingNode.x + movingNode.w, other.x + other.w) + 24 });
        break;
      }
    }
  }
  return guides;
}

// ============================================================================
// 1. TOP DROPDOWN MENU BAR LOGIC
// ============================================================================
const menuItems = document.querySelectorAll(".menu-item");
menuItems.forEach((item) => {
  item.addEventListener("click", (e) => {
    e.stopPropagation();
    const wasOpen = item.classList.contains("open");
    menuItems.forEach((m) => m.classList.remove("open"));
    if (!wasOpen) item.classList.add("open");
  });
  item.addEventListener("mouseenter", () => {
    const anyOpen = [...menuItems].some((m) => m.classList.contains("open"));
    if (anyOpen) {
      menuItems.forEach((m) => m.classList.remove("open"));
      item.classList.add("open");
    }
  });
});

window.addEventListener("click", () => {
  menuItems.forEach((m) => m.classList.remove("open"));
});

document.querySelectorAll(".dropdown-entry").forEach((entry) => {
  entry.addEventListener("click", (e) => {
    e.stopPropagation();
    menuItems.forEach((m) => m.classList.remove("open"));
    const action = entry.getAttribute("data-action");
    handleMenuAction(action);
  });
});

function handleMenuAction(action) {
  const p = page();
  switch (action) {
    case "new-page":
      addNewPage();
      break;
    case "export-drawio":
      exportDrawioXml();
      break;
    case "export-svg":
      exportAnimatedSvg();
      break;
    case "export-json":
      downloadFile(
        "diagram.rfd.json",
        JSON.stringify({ version: "1.0.0", nodes: p.nodes, edges: p.edges.map(({ _geo, _ver, ...r }) => r) }, null, 2),
        "application/json"
      );
      break;
    case "undo":
      undo();
      break;
    case "redo":
      redo();
      break;
    case "duplicate":
      duplicateSelected();
      break;
    case "delete":
      deleteSelected();
      break;
    case "zoom-in":
      setZoom(p.zoom * 1.2);
      break;
    case "zoom-out":
      setZoom(p.zoom / 1.2);
      break;
    case "zoom-reset":
      p.zoom = 1.0;
      p.pan = { x: 60, y: 50 };
      state.minimapDirty = true;
      updateZoomLabel();
      break;
    case "toggle-grid":
      state.gridStyle = state.gridStyle === "dots" ? "lines" : state.gridStyle === "lines" ? "off" : "dots";
      renderFormatPanel();
      break;
    case "theme-dark":
    case "theme-light":
    case "theme-contrast":
      state.theme = action.replace("theme-", "");
      document.documentElement.setAttribute("data-theme", state.theme);
      renderFormatPanel();
      break;
    case "layout-hier":
      runHierarchicalLayout();
      break;
    case "layout-grid":
      runGridLayout();
      break;
    case "layout-radial":
      runRadialLayout();
      break;
    case "bring-front":
      bringToFront();
      break;
    case "send-back":
      sendToBack();
      break;
    case "cmd-palette":
      openCommandPalette();
      break;
    case "stress-100":
      generateStressGraph(100);
      break;
  }
}

function setZoom(targetZoom, cx = canvas.width / 2, cy = canvas.height / 2) {
  const p = page();
  const before = screenToWorld(cx, cy);
  p.zoom = Math.min(8.0, Math.max(0.1, targetZoom));
  p.pan.x = cx - before.x * p.zoom;
  p.pan.y = cy - before.y * p.zoom;
  state.minimapDirty = true;
  updateZoomLabel();
}

function updateZoomLabel() {
  document.getElementById("zoom-label").textContent = `${Math.round(page().zoom * 100)}%`;
}

// ============================================================================
// 2. DRAW.IO CONTEXT-SENSITIVE RIGHT FORMAT PANEL (Style / Text / Arrange)
// ============================================================================
const formatTabsEl = document.getElementById("format-tabs");
const formatBodyEl = document.getElementById("format-body");
const formatBadgeEl = document.getElementById("format-selection-badge");

formatTabsEl.querySelectorAll(".format-tab").forEach((tab) => {
  tab.addEventListener("click", () => {
    state.activeFormatTab = tab.getAttribute("data-tab");
    renderFormatPanel();
  });
});

function renderFormatPanel() {
  const p = page();
  const node = nodeMap.get(state.selectedNodeId);
  const edge = p.edges.find((e) => e.id === state.selectedEdgeId);

  formatTabsEl.querySelectorAll(".format-tab").forEach((t) => {
    t.classList.toggle("active", t.getAttribute("data-tab") === state.activeFormatTab);
  });

  // CASE A: No item selected -> Draw.io Diagram / Canvas Inspector
  if (!node && !edge) {
    formatBadgeEl.textContent = "Diagram";
    formatTabsEl.style.display = "none";
    formatBodyEl.innerHTML = `
      <div class="format-section">
        <label>Page Name</label>
        <input id="diag-page-name" type="text" value="${p.name}" />
      </div>
      <div class="format-section">
        <label>View & Grid</label>
        <div class="format-row">
          <span>Grid Style</span>
          <select id="diag-grid-style">
            <option value="dots" ${state.gridStyle === "dots" ? "selected" : ""}>Dots</option>
            <option value="lines" ${state.gridStyle === "lines" ? "selected" : ""}>Lines</option>
            <option value="off" ${state.gridStyle === "off" ? "selected" : ""}>Hidden</option>
          </select>
        </div>
        <div class="format-row">
          <span>Grid Size (px)</span>
          <input id="diag-grid-size" type="number" min="10" max="60" step="5" value="${state.gridSize}" />
        </div>
      </div>
      <div class="format-section">
        <label>Theme Preset</label>
        <select id="diag-theme">
          <option value="dark" ${state.theme === "dark" ? "selected" : ""}>Dark (Developer)</option>
          <option value="light" ${state.theme === "light" ? "selected" : ""}>Light</option>
          <option value="contrast" ${state.theme === "contrast" ? "selected" : ""}>High Contrast</option>
        </select>
      </div>
      <div class="format-section" style="border-bottom:none;">
        <label>Selection Hint</label>
        <p style="font-size:11px; color:var(--text-muted); line-height:1.5;">
          Click any <b>Shape</b> or <b>Edge</b> on the canvas to inspect and customize its <b>Style</b>, <b>Text</b>, and <b>Arrangement</b>.
        </p>
      </div>
    `;
    document.getElementById("diag-page-name").oninput = (e) => {
      p.name = e.target.value;
      renderPageTabs();
    };
    document.getElementById("diag-grid-style").onchange = (e) => {
      state.gridStyle = e.target.value;
    };
    document.getElementById("diag-grid-size").oninput = (e) => {
      state.gridSize = Math.max(10, Math.min(80, Number(e.target.value) || 20));
    };
    document.getElementById("diag-theme").onchange = (e) => {
      state.theme = e.target.value;
      document.documentElement.setAttribute("data-theme", state.theme);
    };
    return;
  }

  formatTabsEl.style.display = "grid";

  // CASE B: NODE Selected -> Draw.io [Style | Text | Arrange]
  if (node) {
    formatBadgeEl.textContent = `Node #${node.id}`;
    if (state.activeFormatTab === "style") {
      formatBodyEl.innerHTML = `
        <div class="format-section">
          <label>Draw.io Style Presets</label>
          <div class="swatch-grid" id="node-swatches"></div>
        </div>
        <div class="format-section">
          <label>Shape / Symbol</label>
          <select id="insp-shape">
            ${[
              ["card", "React Flow Card"],
              ["rounded", "Terminator / Pill"],
              ["diamond", "Decision Diamond"],
              ["cylinder", "Database Cylinder"],
              ["parallelogram", "Data I/O Parallelogram"],
              ["hexagon", "Hexagon"],
              ["document", "Document (Wavy)"],
              ["note", "Sticky Note"],
              ["circle", "Circle / Event"],
              ["actor", "UML Actor"],
              ["uml_class", "UML Class"],
              ["uml_interface", "UML Interface"],
              ["uml_component", "UML Component"],
              ["uml_package", "UML Package"],
              ["gateway_xor", "BPMN XOR Gateway (×)"],
              ["gateway_parallel", "BPMN Parallel Gateway (+)"],
              ["timer", "BPMN Timer Event (⏱)"],
              ["cloud", "Cloud"],
              ["queue", "Kafka Queue"],
              ["server", "Server Rack"],
              ["firewall", "WAF Firewall"],
              ["shield", "Auth Shield"],
              ["browser", "Browser Window"],
              ["mobile", "Mobile Screen"],
            ]
              .map(([val, lbl]) => `<option value="${val}" ${node.shape === val ? "selected" : ""}>${lbl}</option>`)
              .join("")}
          </select>
        </div>
        <div class="format-section">
          <label>Fill & Stroke Colors</label>
          <div class="format-row">
            <span>Fill</span>
            <input id="insp-fill" type="color" value="${node.fill}" style="width:100%" />
          </div>
          <div class="format-row">
            <span>Stroke</span>
            <input id="insp-stroke" type="color" value="${node.stroke}" style="width:100%" />
          </div>
        </div>
      `;
      const swatchContainer = document.getElementById("node-swatches");
      STYLE_SWATCHES.forEach((sw) => {
        const el = document.createElement("div");
        el.className = "swatch";
        el.style.background = sw.fill;
        el.style.borderColor = sw.stroke;
        el.onclick = () => {
          saveSnapshot();
          node.fill = sw.fill;
          node.stroke = sw.stroke;
          renderFormatPanel();
        };
        swatchContainer.appendChild(el);
      });
      document.getElementById("insp-shape").onchange = (e) => {
        saveSnapshot();
        node.shape = e.target.value;
      };
      document.getElementById("insp-fill").oninput = (e) => {
        node.fill = e.target.value;
      };
      document.getElementById("insp-stroke").oninput = (e) => {
        node.stroke = e.target.value;
      };
    } else if (state.activeFormatTab === "text") {
      formatBodyEl.innerHTML = `
        <div class="format-section">
          <label>Title Label</label>
          <input id="insp-title" type="text" value="${node.title}" />
          <label style="margin-top:6px;">Subtitle / Metadata</label>
          <input id="insp-body" type="text" value="${node.body || ""}" />
        </div>
      `;
      document.getElementById("insp-title").oninput = (e) => {
        node.title = e.target.value;
      };
      document.getElementById("insp-body").oninput = (e) => {
        node.body = e.target.value;
      };
    } else {
      // Arrange tab
      formatBodyEl.innerHTML = `
        <div class="format-section">
          <label>Position (X, Y)</label>
          <div class="format-row">
            <input id="insp-x" type="number" value="${Math.round(node.x)}" />
            <input id="insp-y" type="number" value="${Math.round(node.y)}" />
          </div>
          <label style="margin-top:6px;">Dimensions (W, H)</label>
          <div class="format-row">
            <input id="insp-w" type="number" value="${Math.round(node.w)}" />
            <input id="insp-h" type="number" value="${Math.round(node.h)}" />
          </div>
        </div>
        <div class="format-section">
          <label>Z-Order Layering</label>
          <div class="format-row">
            <button id="btn-front">Bring to Front</button>
            <button id="btn-back">Send to Back</button>
          </div>
        </div>
        <div class="format-section" style="border-bottom:none;">
          <button id="btn-node-dup" style="width:100%">⧉ Duplicate Node</button>
          <button id="btn-node-del" class="danger" style="width:100%; margin-top:4px;">✕ Delete Node</button>
        </div>
      `;
      for (const [id, key] of [
        ["insp-x", "x"],
        ["insp-y", "y"],
        ["insp-w", "w"],
        ["insp-h", "h"],
      ]) {
        document.getElementById(id).oninput = (e) => {
          node[key] = Number(e.target.value) || 40;
          invalidateGeometry();
        };
      }
      document.getElementById("btn-front").onclick = bringToFront;
      document.getElementById("btn-back").onclick = sendToBack;
      document.getElementById("btn-node-dup").onclick = duplicateSelected;
      document.getElementById("btn-node-del").onclick = deleteSelected;
    }
    return;
  }

  // CASE C: EDGE Selected -> Draw.io [Style | Text | Arrange]
  if (edge) {
    formatBadgeEl.textContent = `Edge #${edge.id}`;
    if (state.activeFormatTab === "style") {
      formatBodyEl.innerHTML = `
        <div class="format-section">
          <label>Edge Routing Style</label>
          <select id="edge-kind">
            <option value="bezier" ${edge.kind === "bezier" ? "selected" : ""}>Bezier Curve</option>
            <option value="orthogonal" ${edge.kind === "orthogonal" ? "selected" : ""}>Orthogonal</option>
            <option value="smoothstep" ${edge.kind === "smoothstep" ? "selected" : ""}>SmoothStep</option>
            <option value="straight" ${edge.kind === "straight" ? "selected" : ""}>Straight Line</option>
          </select>
          <label style="margin-top:6px;">Flow Animation</label>
          <select id="edge-anim">
            <option value="pulse" ${edge.anim === "pulse" ? "selected" : ""}>Pulse Particles</option>
            <option value="ants" ${edge.anim === "ants" ? "selected" : ""}>Marching Ants</option>
            <option value="none" ${edge.anim === "none" ? "selected" : ""}>Static Line</option>
          </select>
        </div>
        <div class="format-section">
          <label>Line Color & Width</label>
          <div class="format-row">
            <input id="edge-color" type="color" value="${edge.color || "#818cf8"}" style="width:100%" />
            <input id="edge-width" type="number" min="1" max="10" step="0.5" value="${edge.width || 2.2}" />
          </div>
        </div>
      `;
      document.getElementById("edge-kind").onchange = (e) => {
        edge.kind = e.target.value;
        invalidateGeometry();
      };
      document.getElementById("edge-anim").onchange = (e) => {
        edge.anim = e.target.value;
      };
      document.getElementById("edge-color").oninput = (e) => {
        edge.color = e.target.value;
      };
      document.getElementById("edge-width").oninput = (e) => {
        edge.width = Number(e.target.value) || 2;
      };
    } else if (state.activeFormatTab === "text") {
      formatBodyEl.innerHTML = `
        <div class="format-section">
          <label>Edge Label</label>
          <input id="edge-label" type="text" value="${edge.label || ""}" placeholder="Edge label..." />
        </div>
      `;
      document.getElementById("edge-label").oninput = (e) => {
        edge.label = e.target.value;
      };
    } else {
      formatBodyEl.innerHTML = `
        <div class="format-section">
          <label>Port Attachment Sides</label>
          <div class="format-row">
            <span>Source</span>
            <select id="edge-src-port">
              ${["top", "right", "bottom", "left"]
                .map((s) => `<option value="${s}" ${edge.sourcePort === s ? "selected" : ""}>${s}</option>`)
                .join("")}
            </select>
          </div>
          <div class="format-row">
            <span>Target</span>
            <select id="edge-dst-port">
              ${["top", "right", "bottom", "left"]
                .map((s) => `<option value="${s}" ${edge.targetPort === s ? "selected" : ""}>${s}</option>`)
                .join("")}
            </select>
          </div>
        </div>
        <div class="format-section" style="border-bottom:none;">
          <div class="format-row">
            <button id="btn-edge-reverse">⇄ Reverse</button>
            <button id="btn-edge-reset">↺ Reset Bend</button>
          </div>
          <button id="btn-edge-delete" class="danger" style="width:100%; margin-top:6px;">✕ Delete Edge</button>
        </div>
      `;
      document.getElementById("edge-src-port").onchange = (e) => {
        edge.sourcePort = e.target.value;
        invalidateGeometry();
      };
      document.getElementById("edge-dst-port").onchange = (e) => {
        edge.targetPort = e.target.value;
        invalidateGeometry();
      };
      document.getElementById("btn-edge-reverse").onclick = () => {
        saveSnapshot();
        const tmpN = edge.source, tmpP = edge.sourcePort;
        edge.source = edge.target;
        edge.sourcePort = edge.targetPort;
        edge.target = tmpN;
        edge.targetPort = tmpP;
        invalidateGeometry();
        renderFormatPanel();
      };
      document.getElementById("btn-edge-reset").onclick = () => {
        saveSnapshot();
        edge.bend = null;
        invalidateGeometry();
      };
      document.getElementById("btn-edge-delete").onclick = deleteSelected;
    }
  }
}
renderFormatPanel();

// Fast coordinate sync during drag without recreating DOM nodes
function updateArrangeInputsLive(node) {
  if (state.activeFormatTab !== "arrange") return;
  const xEl = document.getElementById("insp-x");
  const yEl = document.getElementById("insp-y");
  const wEl = document.getElementById("insp-w");
  const hEl = document.getElementById("insp-h");
  if (xEl) xEl.value = Math.round(node.x);
  if (yEl) yEl.value = Math.round(node.y);
  if (wEl) wEl.value = Math.round(node.w);
  if (hEl) hEl.value = Math.round(node.h);
}

// ============================================================================
// POINTER & INTERACTION HANDLERS
// ============================================================================
canvas.addEventListener("mousedown", (e) => {
  inlineEditor.style.display = "none";
  const rect = canvas.getBoundingClientRect();
  const sx = e.clientX - rect.left;
  const sy = e.clientY - rect.top;
  const wpt = screenToWorld(sx, sy);
  state.mouseWorld = wpt;
  const p = page();
  const handleHitRadius = 13 / Math.max(0.4, p.zoom);

  // 1. Selected Edge Bend & Endpoint Reconnect Handles
  const selectedEdge = p.edges.find((ed) => ed.id === state.selectedEdgeId);
  if (selectedEdge) {
    const geo = getEdgeGeometry(selectedEdge);
    if (geo) {
      if (Math.hypot(wpt.x - geo.midPoint.x, wpt.y - geo.midPoint.y) <= handleHitRadius) {
        saveSnapshot();
        state.draggingEdgeBend = selectedEdge;
        return;
      }
      if (Math.hypot(wpt.x - geo.p3.x, wpt.y - geo.p3.y) <= handleHitRadius) {
        saveSnapshot();
        state.reconnectingEdge = { edge: selectedEdge, end: "target" };
        return;
      }
      if (Math.hypot(wpt.x - geo.p0.x, wpt.y - geo.p0.y) <= handleHitRadius) {
        saveSnapshot();
        state.reconnectingEdge = { edge: selectedEdge, end: "source" };
        return;
      }
    }
  }

  // 2. Selected Node Resize Handle & Quick-Add Arrows
  const selectedNode = nodeMap.get(state.selectedNodeId);
  if (selectedNode) {
    if (Math.hypot(wpt.x - (selectedNode.x + selectedNode.w), wpt.y - (selectedNode.y + selectedNode.h)) <= handleHitRadius) {
      saveSnapshot();
      state.resizingNode = { node: selectedNode, aspect: selectedNode.w / Math.max(1, selectedNode.h) };
      return;
    }
    for (const arrow of getQuickArrows(selectedNode)) {
      if (Math.hypot(wpt.x - arrow.x, wpt.y - arrow.y) <= handleHitRadius) {
        saveSnapshot();
        const newNode = {
          ...selectedNode,
          id: ++state.nextId,
          x: selectedNode.x + arrow.dx,
          y: selectedNode.y + arrow.dy,
          z: p.nodes.length + 1,
        };
        p.nodes.push(newNode);
        p.edges.push({
          id: ++state.nextId,
          source: selectedNode.id,
          sourcePort: arrow.side,
          target: newNode.id,
          targetPort: arrow.targetSide,
          label: "Flow",
          anim: state.edgeAnimPreset,
          kind: state.edgeKindPreset,
          color: "#818cf8",
          width: 2.2,
          bend: null,
        });
        invalidateGeometry();
        state.selectedNodeId = newNode.id;
        state.selectedEdgeId = null;
        renderFormatPanel();
        return;
      }
    }
  }

  // 3. Port Handle Click (Drag-to-Connect)
  const portHitRadius = 15 / Math.max(0.4, p.zoom);
  for (const n of p.nodes) {
    for (const port of getNodePorts(n)) {
      if (Math.hypot(wpt.x - port.x, wpt.y - port.y) <= portHitRadius) {
        state.connectingFrom = { node: n, port };
        state.selectedNodeId = n.id;
        state.selectedEdgeId = null;
        renderFormatPanel();
        return;
      }
    }
  }

  // 4. Edge Hit-Test
  const hitEdge = hitTestEdge(wpt);
  if (hitEdge) {
    state.selectedEdgeId = hitEdge.id;
    state.selectedNodeId = null;
    renderFormatPanel();
    const geo = getEdgeGeometry(hitEdge);
    if (geo && Math.hypot(wpt.x - geo.midPoint.x, wpt.y - geo.midPoint.y) <= handleHitRadius * 1.3) {
      saveSnapshot();
      state.draggingEdgeBend = hitEdge;
    }
    return;
  }

  // 5. Node Body Hit-Test
  let hit = null;
  for (let i = p.nodes.length - 1; i >= 0; i--) {
    const n = p.nodes[i];
    if (wpt.x >= n.x && wpt.x <= n.x + n.w && wpt.y >= n.y && wpt.y <= n.y + n.h) {
      hit = n;
      break;
    }
  }
  if (hit) {
    saveSnapshot();
    const changed = state.selectedNodeId !== hit.id || state.selectedEdgeId !== null;
    state.selectedNodeId = hit.id;
    state.selectedEdgeId = null;
    state.draggingNode = { node: hit, offsetX: wpt.x - hit.x, offsetY: wpt.y - hit.y };
    if (changed) renderFormatPanel();
  } else {
    const changed = state.selectedNodeId !== null || state.selectedEdgeId !== null;
    state.selectedNodeId = null;
    state.selectedEdgeId = null;
    state.panning = { startX: sx - p.pan.x, startY: sy - p.pan.y };
    if (changed) renderFormatPanel();
  }
});

canvas.addEventListener("dblclick", (e) => {
  const rect = canvas.getBoundingClientRect();
  const wpt = screenToWorld(e.clientX - rect.left, e.clientY - rect.top);

  const hitEdge = hitTestEdge(wpt);
  if (hitEdge) {
    state.selectedEdgeId = hitEdge.id;
    state.selectedNodeId = null;
    const geo = getEdgeGeometry(hitEdge);
    if (geo && hitEdge.bend && Math.hypot(wpt.x - geo.midPoint.x, wpt.y - geo.midPoint.y) <= 16) {
      saveSnapshot();
      hitEdge.bend = null;
      invalidateGeometry();
      renderFormatPanel();
      return;
    }
    if (geo) {
      const spt = worldToScreen(geo.midPoint.x - 60, geo.midPoint.y - 14);
      inlineEditor.style.display = "block";
      inlineEditor.style.left = `${spt.x}px`;
      inlineEditor.style.top = `${spt.y}px`;
      inlineEditor.style.width = `120px`;
      inlineEditor.value = hitEdge.label || "";
      inlineEditor.focus();
      inlineEditor.onkeydown = (ev) => {
        if (ev.key === "Enter") {
          saveSnapshot();
          hitEdge.label = inlineEditor.value;
          inlineEditor.style.display = "none";
          renderFormatPanel();
        } else if (ev.key === "Escape") {
          inlineEditor.style.display = "none";
        }
      };
    }
    renderFormatPanel();
    return;
  }

  const hit = page().nodes.find((n) => wpt.x >= n.x && wpt.x <= n.x + n.w && wpt.y >= n.y && wpt.y <= n.y + n.h);
  if (!hit) return;
  state.selectedNodeId = hit.id;
  const spt = worldToScreen(hit.x, hit.y);
  inlineEditor.style.display = "block";
  inlineEditor.style.left = `${spt.x}px`;
  inlineEditor.style.top = `${spt.y}px`;
  inlineEditor.style.width = `${hit.w * page().zoom}px`;
  inlineEditor.value = hit.title;
  inlineEditor.focus();
  inlineEditor.onkeydown = (ev) => {
    if (ev.key === "Enter") {
      saveSnapshot();
      hit.title = inlineEditor.value;
      inlineEditor.style.display = "none";
      renderFormatPanel();
    } else if (ev.key === "Escape") {
      inlineEditor.style.display = "none";
    }
  };
});

window.addEventListener("mousemove", (e) => {
  const rect = canvas.getBoundingClientRect();
  const sx = e.clientX - rect.left;
  const sy = e.clientY - rect.top;
  state.mouseWorld = screenToWorld(sx, sy);
  const p = page();

  if (state.connectingFrom) {
    state.snapTargetPort = findSnapTargetPort(state.mouseWorld, state.connectingFrom.node.id);
    canvas.style.cursor = "crosshair";
    return;
  }

  if (state.reconnectingEdge) {
    const { edge, end } = state.reconnectingEdge;
    const excludeId = end === "target" ? edge.source : edge.target;
    state.snapTargetPort = findSnapTargetPort(state.mouseWorld, excludeId);
    canvas.style.cursor = "crosshair";
    return;
  }

  if (state.draggingEdgeBend) {
    state.draggingEdgeBend.bend = {
      x: Math.round(state.mouseWorld.x / 5) * 5,
      y: Math.round(state.mouseWorld.y / 5) * 5,
    };
    invalidateGeometry();
    canvas.style.cursor = "grabbing";
    return;
  }

  if (state.resizingNode) {
    const { node, aspect } = state.resizingNode;
    node.w = Math.max(70, Math.round((state.mouseWorld.x - node.x) / 10) * 10);
    if (e.shiftKey) {
      node.h = Math.max(48, Math.round(node.w / aspect));
    } else {
      node.h = Math.max(48, Math.round((state.mouseWorld.y - node.y) / 10) * 10);
    }
    invalidateGeometry();
    updateArrangeInputsLive(node);
    return;
  }

  if (state.draggingNode) {
    const { node, offsetX, offsetY } = state.draggingNode;
    node.x = Math.round((state.mouseWorld.x - offsetX) / 10) * 10;
    node.y = Math.round((state.mouseWorld.y - offsetY) / 10) * 10;
    state.smartGuides = computeSmartGuides(node);
    invalidateGeometry();
    updateArrangeInputsLive(node);
    return;
  }

  if (state.panning) {
    p.pan.x = sx - state.panning.startX;
    p.pan.y = sy - state.panning.startY;
    state.minimapDirty = true;
    return;
  }

  // Only check hovered ports when idle
  state.hoveredPort = null;
  const portHoverRadius = 15 / Math.max(0.4, p.zoom);
  for (const n of p.nodes) {
    if (
      state.mouseWorld.x < n.x - portHoverRadius ||
      state.mouseWorld.x > n.x + n.w + portHoverRadius ||
      state.mouseWorld.y < n.y - portHoverRadius ||
      state.mouseWorld.y > n.y + n.h + portHoverRadius
    ) {
      continue;
    }
    for (const port of getNodePorts(n)) {
      if (Math.hypot(state.mouseWorld.x - port.x, state.mouseWorld.y - port.y) <= portHoverRadius) {
        state.hoveredPort = { nodeId: n.id, side: port.side, x: port.x, y: port.y };
        break;
      }
    }
  }
  canvas.style.cursor = state.hoveredPort ? "crosshair" : "default";
});

window.addEventListener("mouseup", (e) => {
  const rect = canvas.getBoundingClientRect();
  state.mouseWorld = screenToWorld(e.clientX - rect.left, e.clientY - rect.top);
  const p = page();

  if (state.connectingFrom) {
    const srcNode = state.connectingFrom.node;
    const srcPort = state.connectingFrom.port;
    const snap = findSnapTargetPort(state.mouseWorld, srcNode.id);
    if (snap) {
      saveSnapshot();
      const newEdge = {
        id: ++state.nextId,
        source: srcNode.id,
        sourcePort: srcPort.side,
        target: snap.node.id,
        targetPort: snap.port.side,
        label: "Flow",
        anim: state.edgeAnimPreset,
        kind: state.edgeKindPreset,
        color: "#818cf8",
        width: 2.2,
        bend: null,
      };
      p.edges.push(newEdge);
      invalidateGeometry();
      state.selectedEdgeId = newEdge.id;
      state.selectedNodeId = null;
      renderFormatPanel();
    }
  }

  if (state.reconnectingEdge) {
    const { edge, end } = state.reconnectingEdge;
    const excludeId = end === "target" ? edge.source : edge.target;
    const snap = findSnapTargetPort(state.mouseWorld, excludeId);
    if (snap) {
      if (end === "target") {
        edge.target = snap.node.id;
        edge.targetPort = snap.port.side;
      } else {
        edge.source = snap.node.id;
        edge.sourcePort = snap.port.side;
      }
      invalidateGeometry();
      renderFormatPanel();
    }
  }

  state.draggingNode = null;
  state.resizingNode = null;
  state.draggingEdgeBend = null;
  state.reconnectingEdge = null;
  state.panning = null;
  state.connectingFrom = null;
  state.snapTargetPort = null;
  state.smartGuides = [];
});

canvas.addEventListener(
  "wheel",
  (e) => {
    e.preventDefault();
    const rect = canvas.getBoundingClientRect();
    const cx = e.clientX - rect.left;
    const cy = e.clientY - rect.top;
    const factor = e.deltaY < 0 ? 1.12 : 0.89;
    setZoom(page().zoom * factor, cx, cy);
  },
  { passive: false }
);

function addStencilToCanvas(item) {
  saveSnapshot();
  const p = page();
  const center = screenToWorld(canvas.width / 2 - item.w / 2, canvas.height / 2 - item.h / 2);
  const newNode = {
    id: ++state.nextId,
    title: item.label,
    body: item.id,
    x: Math.round(center.x / 20) * 20,
    y: Math.round(center.y / 20) * 20,
    w: item.w,
    h: item.h,
    fill: state.theme === "light" ? "#ffffff" : "#1e293b",
    stroke: "#818cf8",
    shape: item.shape,
    z: p.nodes.length + 1,
  };
  p.nodes.push(newNode);
  invalidateGeometry();
  state.selectedNodeId = newNode.id;
  state.selectedEdgeId = null;
  renderFormatPanel();
}

function renderPalette(filter = "") {
  const container = document.getElementById("palette-list");
  container.innerHTML = "";
  const q = filter.trim().toLowerCase();
  const categories = [...new Set(STENCILS.map((s) => s.category))];

  for (const cat of categories) {
    const items = STENCILS.filter(
      (s) => s.category === cat && (!q || s.label.toLowerCase().includes(q) || s.id.toLowerCase().includes(q))
    );
    if (!items.length) continue;

    const section = document.createElement("div");
    section.style.marginBottom = "10px";
    const title = document.createElement("div");
    title.className = "palette-section-title";
    title.textContent = cat;
    section.appendChild(title);

    const grid = document.createElement("div");
    grid.className = "stencil-grid";
    for (const item of items) {
      const card = document.createElement("div");
      card.className = "stencil-card";
      card.innerHTML = `<span class="stencil-icon">${item.icon}</span><span>${item.label}</span>`;
      card.onclick = () => addStencilToCanvas(item);
      grid.appendChild(card);
    }
    section.appendChild(grid);
    container.appendChild(section);
  }
}
document.getElementById("palette-search").addEventListener("input", (e) => renderPalette(e.target.value));
renderPalette();

function addNewPage() {
  const id = state.pages.length + 1;
  state.pages.push({
    id,
    name: `Page-${id}`,
    pan: { x: 80, y: 60 },
    zoom: 1.0,
    nodes: [],
    edges: [],
    undoStack: [],
    redoStack: [],
  });
  state.activePageId = id;
  state.selectedNodeId = null;
  state.selectedEdgeId = null;
  invalidateGeometry();
  renderPageTabs();
  renderFormatPanel();
  updateZoomLabel();
}

function renderPageTabs() {
  const container = document.getElementById("page-tabs");
  container.innerHTML = "";
  for (const p of state.pages) {
    const tab = document.createElement("div");
    tab.className = "page-tab" + (p.id === state.activePageId ? " active" : "");
    tab.textContent = p.name;
    tab.onclick = () => {
      state.activePageId = p.id;
      state.selectedNodeId = p.nodes[0]?.id || null;
      state.selectedEdgeId = null;
      invalidateGeometry();
      renderPageTabs();
      renderFormatPanel();
      updateZoomLabel();
    };
    container.appendChild(tab);
  }
  const addBtn = document.createElement("button");
  addBtn.textContent = "+ Page";
  addBtn.onclick = addNewPage;
  container.appendChild(addBtn);
}
renderPageTabs();

function bringToFront() {
  const p = page();
  const n = nodeMap.get(state.selectedNodeId);
  if (!n) return;
  n.z = Math.max(0, ...p.nodes.map((x) => x.z || 0)) + 1;
  p.nodes.sort((a, b) => (a.z || 0) - (b.z || 0));
}

function sendToBack() {
  const p = page();
  const n = nodeMap.get(state.selectedNodeId);
  if (!n) return;
  n.z = Math.min(0, ...p.nodes.map((x) => x.z || 0)) - 1;
  p.nodes.sort((a, b) => (a.z || 0) - (b.z || 0));
}

function undo() {
  const p = page();
  if (!p.undoStack.length) return;
  p.redoStack.push(JSON.stringify({ nodes: p.nodes, edges: p.edges.map(({ _geo, _ver, ...r }) => r) }));
  const prev = JSON.parse(p.undoStack.pop());
  p.nodes = prev.nodes;
  p.edges = prev.edges;
  invalidateGeometry();
  renderFormatPanel();
}

function redo() {
  const p = page();
  if (!p.redoStack.length) return;
  p.undoStack.push(JSON.stringify({ nodes: p.nodes, edges: p.edges.map(({ _geo, _ver, ...r }) => r) }));
  const next = JSON.parse(p.redoStack.pop());
  p.nodes = next.nodes;
  p.edges = next.edges;
  invalidateGeometry();
  renderFormatPanel();
}

function duplicateSelected() {
  const p = page();
  const n = nodeMap.get(state.selectedNodeId);
  if (!n) return;
  saveSnapshot();
  const copy = { ...n, id: ++state.nextId, x: n.x + 30, y: n.y + 30 };
  p.nodes.push(copy);
  invalidateGeometry();
  state.selectedNodeId = copy.id;
  renderFormatPanel();
}

function deleteSelected() {
  const p = page();
  if (state.selectedEdgeId) {
    saveSnapshot();
    p.edges = p.edges.filter((e) => e.id !== state.selectedEdgeId);
    state.selectedEdgeId = null;
    invalidateGeometry();
    renderFormatPanel();
    return;
  }
  if (!state.selectedNodeId) return;
  saveSnapshot();
  p.nodes = p.nodes.filter((n) => n.id !== state.selectedNodeId);
  p.edges = p.edges.filter((e) => e.source !== state.selectedNodeId && e.target !== state.selectedNodeId);
  state.selectedNodeId = null;
  invalidateGeometry();
  renderFormatPanel();
}

function runHierarchicalLayout() {
  const p = page();
  saveSnapshot();
  const layers = {};
  p.nodes.forEach((n) => (layers[n.id] = 0));
  for (let i = 0; i < Math.min(20, p.nodes.length); i++) {
    for (const e of p.edges) {
      if ((layers[e.target] || 0) <= (layers[e.source] || 0) && e.source !== e.target) {
        layers[e.target] = (layers[e.source] || 0) + 1;
      }
    }
  }
  const byLayer = {};
  for (const n of p.nodes) {
    const l = layers[n.id] || 0;
    byLayer[l] = byLayer[l] || [];
    byLayer[l].push(n);
  }
  Object.entries(byLayer).forEach(([layerIdx, group]) => {
    group.forEach((n, sibIdx) => {
      n.x = 90 + Number(layerIdx) * 250;
      n.y = 70 + sibIdx * 125;
    });
  });
  p.edges.forEach((e) => (e.bend = null));
  invalidateGeometry();
  renderFormatPanel();
}

function runGridLayout() {
  const p = page();
  saveSnapshot();
  const cols = Math.max(3, Math.ceil(Math.sqrt(p.nodes.length)));
  p.nodes.forEach((n, idx) => {
    const col = idx % cols;
    const row = Math.floor(idx / cols);
    n.x = 80 + col * 230;
    n.y = 70 + row * 130;
  });
  p.edges.forEach((e) => (e.bend = null));
  invalidateGeometry();
  renderFormatPanel();
}

function runRadialLayout() {
  const p = page();
  saveSnapshot();
  const count = Math.max(1, p.nodes.length);
  const radius = Math.max(180, count * 28);
  const cx = 420, cy = 280;
  p.nodes.forEach((n, idx) => {
    const angle = (idx / count) * Math.PI * 2;
    n.x = Math.round(cx + radius * Math.cos(angle) - n.w / 2);
    n.y = Math.round(cy + radius * Math.sin(angle) - n.h / 2);
  });
  p.edges.forEach((e) => (e.bend = null));
  invalidateGeometry();
  renderFormatPanel();
}

function generateStressGraph(count) {
  const p = page();
  saveSnapshot();
  p.nodes = [];
  p.edges = [];
  const cols = Math.ceil(Math.sqrt(count));
  for (let i = 1; i <= count; i++) {
    const c = (i - 1) % cols;
    const r = Math.floor((i - 1) / cols);
    p.nodes.push({
      id: i,
      title: `Service-${i}`,
      body: `shard-${c}-${r}`,
      x: 60 + c * 220,
      y: 60 + r * 120,
      w: 165,
      h: 74,
      fill: "#1e293b",
      stroke: "#38bdf8",
      shape: i % 4 === 0 ? "hexagon" : "card",
      z: i,
    });
    if (i > 1) {
      p.edges.push({
        id: 1000 + i,
        source: Math.max(1, i - 1),
        sourcePort: "right",
        target: i,
        targetPort: "left",
        label: "",
        anim: i % 2 === 0 ? "ants" : "pulse",
        kind: "bezier",
        color: "#818cf8",
        width: 2,
        bend: null,
      });
    }
  }
  invalidateGeometry();
  renderFormatPanel();
}

document.getElementById("btn-undo").onclick = undo;
document.getElementById("btn-redo").onclick = redo;
document.getElementById("btn-duplicate").onclick = duplicateSelected;
document.getElementById("btn-delete").onclick = deleteSelected;
document.getElementById("btn-zoom-in").onclick = () => setZoom(page().zoom * 1.2);
document.getElementById("btn-zoom-out").onclick = () => setZoom(page().zoom / 1.2);
document.getElementById("btn-layout-hier").onclick = runHierarchicalLayout;

document.getElementById("select-edge-anim").onchange = (e) => {
  state.edgeAnimPreset = e.target.value;
  const selEdge = page().edges.find((ed) => ed.id === state.selectedEdgeId);
  if (selEdge) {
    selEdge.anim = state.edgeAnimPreset;
    renderFormatPanel();
  }
};
document.getElementById("select-edge-kind").onchange = (e) => {
  state.edgeKindPreset = e.target.value;
  const selEdge = page().edges.find((ed) => ed.id === state.selectedEdgeId);
  if (selEdge) {
    selEdge.kind = state.edgeKindPreset;
    invalidateGeometry();
    renderFormatPanel();
  }
};

function downloadFile(filename, content, mime) {
  const blob = new Blob([content], { type: mime });
  const a = document.createElement("a");
  a.href = URL.createObjectURL(blob);
  a.download = filename;
  a.click();
}

function exportDrawioXml() {
  const p = page();
  let xml = `<mxfile host="RustFlowDraw" version="1.0.0">\n  <diagram id="page-${p.id}" name="${p.name}">\n    <mxGraphModel dx="1280" dy="720" grid="1" gridSize="20">\n      <root>\n        <mxCell id="0"/>\n        <mxCell id="1" parent="0"/>\n`;
  for (const n of p.nodes) {
    xml += `        <mxCell id="node-${n.id}" value="${n.title}" style="shape=${n.shape};rounded=1;fillColor=${n.fill};strokeColor=${n.stroke};" vertex="1" parent="1">\n          <mxGeometry x="${n.x}" y="${n.y}" width="${n.w}" height="${n.h}" as="geometry"/>\n        </mxCell>\n`;
  }
  for (const e of p.edges) {
    xml += `        <mxCell id="edge-${e.id}" value="${e.label}" style="curved=1;rfdAnim=${e.anim};" edge="1" parent="1" source="node-${e.source}" target="node-${e.target}">\n          <mxGeometry relative="1" as="geometry"/>\n        </mxCell>\n`;
  }
  xml += `      </root>\n    </mxGraphModel>\n  </diagram>\n</mxfile>`;
  downloadFile("diagram.drawio", xml, "application/xml");
}

function exportAnimatedSvg() {
  const p = page();
  let svg = `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 1000 600" width="1000" height="600">\n  <style>\n    @keyframes rfd-ants { to { stroke-dashoffset: -28; } }\n    .ants { stroke-dasharray: 9 5; animation: rfd-ants 0.8s linear infinite; }\n    text { font-family: Inter, sans-serif; font-size: 13px; font-weight: 600; fill: #f8fafc; }\n  </style>\n`;
  for (const e of p.edges) {
    const geo = getEdgeGeometry(e);
    if (!geo) continue;
    const d = `M ${geo.p0.x} ${geo.p0.y} C ${geo.p1.x} ${geo.p1.y}, ${geo.p2.x} ${geo.p2.y}, ${geo.p3.x} ${geo.p3.y}`;
    svg += `  <path d="${d}" fill="none" stroke="${e.color || "#818cf8"}" stroke-width="${e.width || 2.2}" class="${e.anim === "ants" ? "ants" : ""}"/>\n`;
  }
  for (const n of p.nodes) {
    svg += `  <rect x="${n.x}" y="${n.y}" width="${n.w}" height="${n.h}" rx="10" fill="${n.fill}" stroke="${n.stroke}" stroke-width="2"/>\n`;
    svg += `  <text x="${n.x + n.w / 2}" y="${n.y + n.h / 2}" text-anchor="middle" dominant-baseline="middle">${n.title}</text>\n`;
  }
  svg += `</svg>`;
  downloadFile("diagram-animated.svg", svg, "image/svg+xml");
}

// Command Palette (Ctrl+K)
const cmdOverlay = document.getElementById("cmd-overlay");
const cmdInput = document.getElementById("cmd-input");
const cmdList = document.getElementById("cmd-list");

const COMMANDS = [
  { label: "Auto-Layout: Sugiyama Hierarchical", shortcut: "Arrange", run: runHierarchicalLayout },
  { label: "Auto-Layout: Uniform Grid", shortcut: "Arrange", run: runGridLayout },
  { label: "Auto-Layout: Radial Ring", shortcut: "Arrange", run: runRadialLayout },
  { label: "Theme: Dark Mode", shortcut: "View", run: () => handleMenuAction("theme-dark") },
  { label: "Theme: Light Mode", shortcut: "View", run: () => handleMenuAction("theme-light") },
  { label: "Theme: High Contrast", shortcut: "View", run: () => handleMenuAction("theme-contrast") },
  { label: "Export: Download .drawio XML", shortcut: "File", run: exportDrawioXml },
  { label: "Export: Download Animated SVG", shortcut: "File", run: exportAnimatedSvg },
  { label: "Benchmark: Generate 100-Node Graph", shortcut: "Extras", run: () => generateStressGraph(100) },
  ...STENCILS.map((s) => ({ label: `Add ${s.icon} ${s.label}`, shortcut: s.category, run: () => addStencilToCanvas(s) })),
];

function openCommandPalette() {
  cmdOverlay.classList.add("open");
  cmdInput.value = "";
  renderCommandList("");
  cmdInput.focus();
}

function closeCommandPalette() {
  cmdOverlay.classList.remove("open");
}

function renderCommandList(query) {
  cmdList.innerHTML = "";
  const q = query.trim().toLowerCase();
  const filtered = COMMANDS.filter((c) => !q || c.label.toLowerCase().includes(q) || c.shortcut.toLowerCase().includes(q));
  for (const cmd of filtered) {
    const row = document.createElement("div");
    row.className = "cmd-item";
    row.innerHTML = `<span>${cmd.label}</span><span style="opacity:0.65">${cmd.shortcut}</span>`;
    row.onclick = () => {
      cmd.run();
      closeCommandPalette();
    };
    cmdList.appendChild(row);
  }
}

document.getElementById("btn-cmd-palette").onclick = openCommandPalette;
cmdInput.addEventListener("input", (e) => renderCommandList(e.target.value));
cmdOverlay.addEventListener("click", (e) => {
  if (e.target === cmdOverlay) closeCommandPalette();
});

window.addEventListener("keydown", (e) => {
  if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === "k") {
    e.preventDefault();
    if (cmdOverlay.classList.contains("open")) closeCommandPalette();
    else openCommandPalette();
    return;
  }
  if (e.key === "Escape") {
    closeCommandPalette();
    inlineEditor.style.display = "none";
    menuItems.forEach((m) => m.classList.remove("open"));
    return;
  }
  if (e.target.tagName === "INPUT" || e.target.tagName === "SELECT") return;
  if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === "z") {
    e.preventDefault();
    undo();
  } else if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === "y") {
    e.preventDefault();
    redo();
  } else if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === "d") {
    e.preventDefault();
    duplicateSelected();
  } else if (e.key === "Delete" || e.key === "Backspace") {
    deleteSelected();
  } else if (["ArrowUp", "ArrowDown", "ArrowLeft", "ArrowRight"].includes(e.key)) {
    const n = nodeMap.get(state.selectedNodeId);
    if (!n) return;
    e.preventDefault();
    const step = e.shiftKey ? 20 : 5;
    if (e.key === "ArrowUp") n.y -= step;
    if (e.key === "ArrowDown") n.y += step;
    if (e.key === "ArrowLeft") n.x -= step;
    if (e.key === "ArrowRight") n.x += step;
    invalidateGeometry();
    updateArrangeInputsLive(n);
  }
});

// ============================================================================
// 3. ULTRA-FAST O(1) OFFSCREEN GRID PATTERN & FRUSTUM-CULLED RENDERER
// ============================================================================
function drawInfiniteGridFast(p) {
  if (state.gridStyle === "off") return;
  const step = Math.round(state.gridSize * p.zoom);
  if (step < 6) return;

  const key = `${state.theme}:${state.gridStyle}:${step}`;
  if (key !== cachedGridKey) {
    cachedGridKey = key;
    const tile = document.createElement("canvas");
    tile.width = step;
    tile.height = step;
    const tctx = tile.getContext("2d");
    const color =
      state.theme === "light" ? "#cbd5e1" : state.theme === "contrast" ? "#404040" : "#263346";

    if (state.gridStyle === "dots") {
      tctx.fillStyle = color;
      tctx.fillRect(0, 0, 2, 2);
    } else {
      tctx.strokeStyle = color;
      tctx.lineWidth = 1;
      tctx.beginPath();
      tctx.moveTo(0, 0);
      tctx.lineTo(step, 0);
      tctx.moveTo(0, 0);
      tctx.lineTo(0, step);
      tctx.stroke();
    }
    cachedGridPattern = ctx.createPattern(tile, "repeat");
  }

  if (cachedGridPattern) {
    ctx.save();
    const ox = ((p.pan.x % step) + step) % step;
    const oy = ((p.pan.y % step) + step) % step;
    ctx.translate(ox, oy);
    ctx.fillStyle = cachedGridPattern;
    ctx.fillRect(-ox, -oy, canvas.width, canvas.height);
    ctx.restore();
  }
}

function drawNodeShape(ctx, n, textColor) {
  const { x, y, w, h, shape } = n;

  ctx.beginPath();
  if (shape === "diamond" || shape === "gateway_xor" || shape === "gateway_parallel") {
    ctx.moveTo(x + w / 2, y);
    ctx.lineTo(x + w, y + h / 2);
    ctx.lineTo(x + w / 2, y + h);
    ctx.lineTo(x, y + h / 2);
    ctx.closePath();
    ctx.fill();
    ctx.stroke();

    if (shape === "gateway_xor") {
      ctx.beginPath();
      const s = Math.min(w, h) * 0.16;
      const cx = x + w / 2, cy = y + h / 2 - 10;
      ctx.moveTo(cx - s, cy - s);
      ctx.lineTo(cx + s, cy + s);
      ctx.moveTo(cx + s, cy - s);
      ctx.lineTo(cx - s, cy + s);
      ctx.stroke();
    } else if (shape === "gateway_parallel") {
      ctx.beginPath();
      const s = Math.min(w, h) * 0.18;
      const cx = x + w / 2, cy = y + h / 2 - 10;
      ctx.moveTo(cx - s, cy);
      ctx.lineTo(cx + s, cy);
      ctx.moveTo(cx, cy - s);
      ctx.lineTo(cx, cy + s);
      ctx.stroke();
    }
  } else if (shape === "rounded") {
    ctx.roundRect(x, y, w, h, Math.min(w, h) * 0.5);
    ctx.fill();
    ctx.stroke();
  } else if (shape === "circle" || shape === "timer") {
    const r = Math.min(w, h) / 2;
    const cx = x + w / 2, cy = y + h / 2;
    ctx.arc(cx, cy, r, 0, Math.PI * 2);
    ctx.fill();
    ctx.stroke();
    if (shape === "timer") {
      ctx.beginPath();
      ctx.arc(cx, cy, r * 0.78, 0, Math.PI * 2);
      ctx.moveTo(cx, cy);
      ctx.lineTo(cx, cy - r * 0.5);
      ctx.moveTo(cx, cy);
      ctx.lineTo(cx + r * 0.38, cy);
      ctx.stroke();
    }
  } else if (shape === "cylinder") {
    const ry = 12;
    ctx.moveTo(x, y + ry);
    ctx.lineTo(x, y + h - ry);
    ctx.bezierCurveTo(x, y + h + ry * 0.6, x + w, y + h + ry * 0.6, x + w, y + h - ry);
    ctx.lineTo(x + w, y + ry);
    ctx.bezierCurveTo(x + w, y - ry * 0.6, x, y - ry * 0.6, x, y + ry);
    ctx.closePath();
    ctx.fill();
    ctx.stroke();
    ctx.beginPath();
    ctx.ellipse(x + w / 2, y + ry, w / 2, ry, 0, 0, Math.PI * 2);
    ctx.stroke();
  } else if (shape === "queue") {
    const rx = 14;
    ctx.roundRect(x, y, w, h, rx);
    ctx.fill();
    ctx.stroke();
    ctx.beginPath();
    ctx.ellipse(x + w - rx, y + h / 2, rx * 0.7, h / 2, 0, 0, Math.PI * 2);
    ctx.moveTo(x + w * 0.25, y);
    ctx.lineTo(x + w * 0.25, y + h);
    ctx.moveTo(x + w * 0.45, y);
    ctx.lineTo(x + w * 0.45, y + h);
    ctx.stroke();
  } else if (shape === "parallelogram") {
    const skew = 20;
    ctx.moveTo(x + skew, y);
    ctx.lineTo(x + w, y);
    ctx.lineTo(x + w - skew, y + h);
    ctx.lineTo(x, y + h);
    ctx.closePath();
    ctx.fill();
    ctx.stroke();
  } else if (shape === "hexagon") {
    const cut = 22;
    ctx.moveTo(x + cut, y);
    ctx.lineTo(x + w - cut, y);
    ctx.lineTo(x + w, y + h / 2);
    ctx.lineTo(x + w - cut, y + h);
    ctx.lineTo(x + cut, y + h);
    ctx.lineTo(x, y + h / 2);
    ctx.closePath();
    ctx.fill();
    ctx.stroke();
  } else if (shape === "document") {
    ctx.moveTo(x, y);
    ctx.lineTo(x + w, y);
    ctx.lineTo(x + w, y + h - 12);
    ctx.bezierCurveTo(x + w * 0.7, y + h + 8, x + w * 0.3, y + h - 24, x, y + h - 8);
    ctx.closePath();
    ctx.fill();
    ctx.stroke();
  } else if (shape === "note") {
    const fold = 18;
    ctx.moveTo(x, y);
    ctx.lineTo(x + w - fold, y);
    ctx.lineTo(x + w, y + fold);
    ctx.lineTo(x + w, y + h);
    ctx.lineTo(x, y + h);
    ctx.closePath();
    ctx.fill();
    ctx.stroke();
    ctx.beginPath();
    ctx.moveTo(x + w - fold, y);
    ctx.lineTo(x + w - fold, y + fold);
    ctx.lineTo(x + w, y + fold);
    ctx.stroke();
  } else if (shape === "cloud") {
    ctx.roundRect(x + 8, y + 10, w - 16, h - 16, 28);
    ctx.fill();
    ctx.stroke();
  } else if (shape === "actor") {
    const cx = x + w / 2;
    ctx.arc(cx, y + 18, 12, 0, Math.PI * 2);
    ctx.fill();
    ctx.stroke();
    ctx.beginPath();
    ctx.moveTo(cx, y + 30);
    ctx.lineTo(cx, y + 66);
    ctx.moveTo(cx - 22, y + 44);
    ctx.lineTo(cx + 22, y + 44);
    ctx.moveTo(cx, y + 66);
    ctx.lineTo(cx - 18, y + 92);
    ctx.moveTo(cx, y + 66);
    ctx.lineTo(cx + 18, y + 92);
    ctx.stroke();
  } else if (shape === "uml_package") {
    ctx.rect(x, y, 70, 18);
    ctx.fill();
    ctx.stroke();
    ctx.beginPath();
    ctx.roundRect(x, y + 18, w, h - 18, [0, 8, 8, 8]);
    ctx.fill();
    ctx.stroke();
  } else if (shape === "uml_component") {
    ctx.roundRect(x, y, w, h, 8);
    ctx.fill();
    ctx.stroke();
    ctx.fillRect(x - 8, y + 16, 16, 10);
    ctx.strokeRect(x - 8, y + 16, 16, 10);
    ctx.fillRect(x - 8, y + 34, 16, 10);
    ctx.strokeRect(x - 8, y + 34, 16, 10);
  } else if (shape === "uml_class" || shape === "uml_interface") {
    ctx.roundRect(x, y, w, h, 8);
    ctx.fill();
    ctx.stroke();
    ctx.beginPath();
    ctx.moveTo(x, y + 36);
    ctx.lineTo(x + w, y + 36);
    ctx.moveTo(x, y + 64);
    ctx.lineTo(x + w, y + 64);
    ctx.stroke();
  } else if (shape === "shield") {
    ctx.moveTo(x + w / 2, y);
    ctx.lineTo(x + w, y + 18);
    ctx.bezierCurveTo(x + w, y + h * 0.7, x + w / 2, y + h, x + w / 2, y + h);
    ctx.bezierCurveTo(x + w / 2, y + h, x, y + h * 0.7, x, y + 18);
    ctx.closePath();
    ctx.fill();
    ctx.stroke();
  } else if (shape === "firewall") {
    ctx.roundRect(x, y, w, h, 8);
    ctx.fill();
    ctx.stroke();
    ctx.beginPath();
    ctx.moveTo(x, y + 22);
    ctx.lineTo(x + w, y + 22);
    ctx.moveTo(x + w * 0.33, y);
    ctx.lineTo(x + w * 0.33, y + 22);
    ctx.moveTo(x + w * 0.66, y);
    ctx.lineTo(x + w * 0.66, y + 22);
    ctx.stroke();
  } else if (shape === "server") {
    ctx.roundRect(x, y, w, h, 8);
    ctx.fill();
    ctx.stroke();
    ctx.beginPath();
    ctx.moveTo(x, y + h / 3);
    ctx.lineTo(x + w, y + h / 3);
    ctx.moveTo(x, y + (2 * h) / 3);
    ctx.lineTo(x + w, y + (2 * h) / 3);
    ctx.stroke();
  } else if (shape === "browser") {
    ctx.roundRect(x, y, w, h, 10);
    ctx.fill();
    ctx.stroke();
    ctx.beginPath();
    ctx.moveTo(x, y + 26);
    ctx.lineTo(x + w, y + 26);
    ctx.stroke();
    const colors = ["#f43f5e", "#fbbf24", "#34d399"];
    colors.forEach((c, i) => {
      ctx.beginPath();
      ctx.arc(x + 14 + i * 12, y + 13, 4, 0, Math.PI * 2);
      ctx.fillStyle = c;
      ctx.fill();
    });
  } else if (shape === "mobile") {
    ctx.roundRect(x, y, w, h, 18);
    ctx.fill();
    ctx.stroke();
    ctx.beginPath();
    ctx.moveTo(x + w * 0.35, y + 10);
    ctx.lineTo(x + w * 0.65, y + 10);
    ctx.moveTo(x + w * 0.3, y + h - 10);
    ctx.lineTo(x + w * 0.7, y + h - 10);
    ctx.stroke();
  } else {
    ctx.roundRect(x, y, w, h, 10);
    ctx.fill();
    ctx.stroke();
  }

  ctx.fillStyle = textColor;
  ctx.font = "600 13px Inter, sans-serif";
  ctx.textAlign = "center";
  const titleY =
    shape === "actor"
      ? y + h - 4
      : shape === "uml_class" || shape === "uml_interface"
      ? y + 23
      : shape === "browser"
      ? y + 54
      : y + h / 2 - (n.body ? 6 : -4);
  ctx.fillText(n.title, x + w / 2, titleY);

  if (n.body && shape !== "actor") {
    ctx.fillStyle = "#94a3b8";
    ctx.font = "400 11px monospace";
    const bodyY = shape === "uml_class" || shape === "uml_interface" ? y + 54 : shape === "browser" ? y + 78 : y + h / 2 + 16;
    ctx.fillText(n.body, x + w / 2, bodyY);
  }
}

function drawArrowhead(ctx, fromPt, toPt, color) {
  const angle = Math.atan2(toPt.y - fromPt.y, toPt.x - fromPt.x);
  const headLen = 10;
  ctx.fillStyle = color;
  ctx.beginPath();
  ctx.moveTo(toPt.x, toPt.y);
  ctx.lineTo(toPt.x - headLen * Math.cos(angle - Math.PI / 6), toPt.y - headLen * Math.sin(angle - Math.PI / 6));
  ctx.lineTo(toPt.x - headLen * Math.cos(angle + Math.PI / 6), toPt.y - headLen * Math.sin(angle + Math.PI / 6));
  ctx.closePath();
  ctx.fill();
}

let statusFrameCounter = 0;

function renderLoop(timestamp) {
  const dt = Math.max(1, timestamp - state.lastFrameTime);
  state.fps = Math.round(state.fps * 0.9 + (1000 / dt) * 0.1);
  state.lastFrameTime = timestamp;

  const timeSec = timestamp * 0.001;
  const p = page();

  // Clear background without DOM getComputedStyle
  ctx.fillStyle = state.theme === "light" ? "#f8fafc" : state.theme === "contrast" ? "#000000" : "#0b0f19";
  ctx.fillRect(0, 0, canvas.width, canvas.height);

  // 1. O(1) Cached Pattern Grid
  drawInfiniteGridFast(p);

  // Compute visible world frustum for culling off-screen nodes/edges
  const viewTL = screenToWorld(0, 0);
  const viewBR = screenToWorld(canvas.width, canvas.height);

  ctx.save();
  ctx.translate(p.pan.x, p.pan.y);
  ctx.scale(p.zoom, p.zoom);

  // 2. Smart Alignment Guides
  if (state.smartGuides.length) {
    ctx.strokeStyle = "#f43f5e";
    ctx.lineWidth = 1.2 / p.zoom;
    ctx.setLineDash([4 / p.zoom, 4 / p.zoom]);
    ctx.beginPath();
    for (const g of state.smartGuides) {
      if (g.type === "v") {
        ctx.moveTo(g.x, g.y1);
        ctx.lineTo(g.x, g.y2);
      } else {
        ctx.moveTo(g.x1, g.y);
        ctx.lineTo(g.x2, g.y);
      }
    }
    ctx.stroke();
    ctx.setLineDash([]);
  }

  // 3. Frustum-Culled Edges & Zero-ShadowBlur Animations
  for (const e of p.edges) {
    const geo = getEdgeGeometry(e);
    if (!geo) continue;
    if (
      geo.bounds.maxX < viewTL.x ||
      geo.bounds.minX > viewBR.x ||
      geo.bounds.maxY < viewTL.y ||
      geo.bounds.minY > viewBR.y
    ) {
      continue;
    }

    const isSelected = e.id === state.selectedEdgeId;
    const strokeColor = isSelected ? "#38bdf8" : e.color || "#818cf8";

    ctx.beginPath();
    if (e.kind === "straight" || e.kind === "orthogonal" || e.kind === "smoothstep") {
      ctx.moveTo(geo.samples[0].x, geo.samples[0].y);
      for (let i = 1; i < geo.samples.length; i++) {
        ctx.lineTo(geo.samples[i].x, geo.samples[i].y);
      }
    } else {
      ctx.moveTo(geo.p0.x, geo.p0.y);
      ctx.bezierCurveTo(geo.p1.x, geo.p1.y, geo.p2.x, geo.p2.y, geo.p3.x, geo.p3.y);
    }

    ctx.strokeStyle = strokeColor;
    ctx.lineWidth = isSelected ? (e.width || 2.2) + 1.2 : e.width || 2.2;

    if (e.anim === "ants") {
      ctx.setLineDash([9, 5]);
      ctx.lineDashOffset = -timeSec * 36;
    }
    ctx.stroke();
    if (e.anim === "ants") ctx.setLineDash([]);

    const prevPt = geo.samples[Math.max(0, geo.samples.length - 2)];
    drawArrowhead(ctx, prevPt, geo.p3, strokeColor);

    // Fast dual-circle glow pulse (no expensive Gaussian shadowBlur!)
    if (e.anim === "pulse") {
      for (const offset of [0, 0.5]) {
        const t = (timeSec * 0.55 + offset) % 1;
        const idx = Math.min(geo.samples.length - 1, Math.floor(t * (geo.samples.length - 1)));
        const pt = geo.samples[idx];
        ctx.fillStyle = "rgba(56, 189, 248, 0.28)";
        ctx.beginPath();
        ctx.arc(pt.x, pt.y, 8, 0, Math.PI * 2);
        ctx.fill();
        ctx.fillStyle = "#38bdf8";
        ctx.beginPath();
        ctx.arc(pt.x, pt.y, 4, 0, Math.PI * 2);
        ctx.fill();
      }
    }

    if (e.label) {
      ctx.font = "600 11px Inter, sans-serif";
      const textW = e.label.length * 7 + 14;
      ctx.fillStyle = state.theme === "light" ? "#ffffff" : "#0f172a";
      ctx.strokeStyle = strokeColor;
      ctx.lineWidth = 1;
      ctx.beginPath();
      ctx.roundRect(geo.midPoint.x - textW / 2, geo.midPoint.y - 10, textW, 20, 6);
      ctx.fill();
      ctx.stroke();
      ctx.fillStyle = state.theme === "light" ? "#0f172a" : "#f8fafc";
      ctx.textAlign = "center";
      ctx.fillText(e.label, geo.midPoint.x, geo.midPoint.y + 4);
    }

    if (isSelected) {
      ctx.strokeStyle = "rgba(251, 191, 36, 0.45)";
      ctx.setLineDash([3, 3]);
      ctx.beginPath();
      ctx.moveTo(geo.p0.x, geo.p0.y);
      ctx.lineTo(geo.midPoint.x, geo.midPoint.y);
      ctx.lineTo(geo.p3.x, geo.p3.y);
      ctx.stroke();
      ctx.setLineDash([]);

      ctx.beginPath();
      ctx.arc(geo.midPoint.x, geo.midPoint.y, 7, 0, Math.PI * 2);
      ctx.fillStyle = "#fbbf24";
      ctx.strokeStyle = "#0b0f19";
      ctx.lineWidth = 2;
      ctx.fill();
      ctx.stroke();

      for (const ep of [geo.p0, geo.p3]) {
        ctx.beginPath();
        ctx.arc(ep.x, ep.y, 6.5, 0, Math.PI * 2);
        ctx.fillStyle = "#22d3ee";
        ctx.strokeStyle = "#0b0f19";
        ctx.lineWidth = 2;
        ctx.fill();
        ctx.stroke();
      }
    }
  }

  // Active connection drag preview line + Magnetic Target Port Snap Ring
  if (state.connectingFrom || state.reconnectingEdge) {
    let startPt = { x: 0, y: 0 };
    if (state.connectingFrom) {
      startPt = state.connectingFrom.port;
    } else if (state.reconnectingEdge) {
      const geo = getEdgeGeometry(state.reconnectingEdge.edge);
      startPt = state.reconnectingEdge.end === "target" ? geo.p0 : geo.p3;
    }
    const endPt = state.snapTargetPort ? state.snapTargetPort.port : state.mouseWorld;
    ctx.beginPath();
    ctx.moveTo(startPt.x, startPt.y);
    ctx.lineTo(endPt.x, endPt.y);
    ctx.strokeStyle = state.snapTargetPort ? "#34d399" : "#38bdf8";
    ctx.lineWidth = 2.5;
    ctx.setLineDash([6, 4]);
    ctx.stroke();
    ctx.setLineDash([]);

    if (state.snapTargetPort) {
      ctx.beginPath();
      ctx.arc(state.snapTargetPort.port.x, state.snapTargetPort.port.y, 11, 0, Math.PI * 2);
      ctx.strokeStyle = "#34d399";
      ctx.lineWidth = 3;
      ctx.stroke();
    }
  }

  // 4. Frustum-Culled Nodes (Zero shadowBlur!)
  for (const n of p.nodes) {
    if (
      n.x + n.w + 35 < viewTL.x ||
      n.x - 35 > viewBR.x ||
      n.y + n.h + 35 < viewTL.y ||
      n.y - 35 > viewBR.y
    ) {
      continue;
    }
    const selected = n.id === state.selectedNodeId;

    // Fast selection halo
    if (selected) {
      ctx.strokeStyle = "rgba(56, 189, 248, 0.28)";
      ctx.lineWidth = 6;
      ctx.strokeRect(n.x - 2, n.y - 2, n.w + 4, n.h + 4);
    }

    ctx.fillStyle = n.fill;
    ctx.strokeStyle = selected ? "#38bdf8" : n.stroke;
    ctx.lineWidth = selected ? 2.5 : 1.7;

    const textColor = state.theme === "light" && n.fill === "#ffffff" ? "#0f172a" : "#f8fafc";
    drawNodeShape(ctx, n, textColor);

    for (const port of getNodePorts(n)) {
      const isHovered =
        state.hoveredPort && state.hoveredPort.nodeId === n.id && state.hoveredPort.side === port.side;
      ctx.beginPath();
      ctx.arc(port.x, port.y, isHovered ? 7.5 : 5, 0, Math.PI * 2);
      ctx.fillStyle = isHovered ? "#34d399" : "#0b0f19";
      ctx.strokeStyle = isHovered ? "#ffffff" : "#818cf8";
      ctx.lineWidth = 2;
      ctx.fill();
      ctx.stroke();
    }

    if (selected) {
      ctx.fillStyle = "#38bdf8";
      ctx.fillRect(n.x + n.w - 5, n.y + n.h - 5, 10, 10);

      for (const arrow of getQuickArrows(n)) {
        ctx.beginPath();
        ctx.arc(arrow.x, arrow.y, 9.5, 0, Math.PI * 2);
        ctx.fillStyle = "#1e293b";
        ctx.strokeStyle = "#38bdf8";
        ctx.lineWidth = 1.5;
        ctx.fill();
        ctx.stroke();
        ctx.fillStyle = "#38bdf8";
        ctx.font = "700 11px sans-serif";
        ctx.textAlign = "center";
        ctx.fillText(arrow.label, arrow.x, arrow.y + 4);
      }
    }
  }

  ctx.restore();

  // 5. Dirty-Checked Minimap (only redraws when nodes, pan, or zoom change!)
  if (state.minimapDirty) {
    state.minimapDirty = false;
    minimapCtx.clearRect(0, 0, minimapCanvas.width, minimapCanvas.height);
    const scale = 0.13;
    minimapCtx.save();
    minimapCtx.translate(14, 14);
    minimapCtx.scale(scale, scale);
    for (const n of p.nodes) {
      minimapCtx.fillStyle = n.id === state.selectedNodeId ? "#38bdf8" : "#64748b";
      minimapCtx.fillRect(n.x, n.y, n.w, n.h);
    }
    minimapCtx.strokeStyle = "#818cf8";
    minimapCtx.lineWidth = 10;
    minimapCtx.strokeRect(viewTL.x, viewTL.y, viewBR.x - viewTL.x, viewBR.y - viewTL.y);
    minimapCtx.restore();
  }

  // Update status bar text every 10 frames to avoid DOM churn
  if (++statusFrameCounter % 10 === 0) {
    document.getElementById("status-bar").textContent = `FPS: ${state.fps} | Zoom: ${Math.round(p.zoom * 100)}% | Nodes: ${p.nodes.length} | Edges: ${p.edges.length}`;
  }
  requestAnimationFrame(renderLoop);
}
requestAnimationFrame(renderLoop);
