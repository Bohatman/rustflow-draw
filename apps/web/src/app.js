// High-Performance Minimalist Draw.io + React Flow Web Engine
// Features:
// 1. Drag-and-Drop Stencil Creation from Left Panel onto Canvas:
//    - Drag any shape from the left panel directly onto the canvas with a live translucent ghost preview & smart alignment guides
//    - Or click any shape in the left panel to insert at viewport center
// 2. Minimalist Monoline Design Language (Linear / Figma / Vercel / Lucide aesthetic):
//    - Clean monoline geometric silhouettes for Queue (horizontal cylinder), Database (vertical cylinder), Cloud (flat-bottom geometric cloud), Actor (minimalist user glyph), BPMN Gateways & Timer
//    - Sleek architecture cards with pure vector monoline icon badges for AWS Lambda, AWS S3, K8s Pod, Server Rack, and WAF Firewall
//    - Crisp 32x24 minimalist vector thumbnails in the left stencil palette
// 3. Complete Keyboard Shortcuts (Ctrl+A, Ctrl+C, Ctrl+X, Ctrl+V, Ctrl+D, Ctrl+Z/Y, Shift+Drag Marquee, F2, Ctrl+1, ?) & 60-144 FPS Sub-Pixel Edge Animations

const STENCILS = [
  // Flowchart
  { id: "flowchart.process", label: "Process", category: "Flowchart", w: 170, h: 74, shape: "card" },
  { id: "flowchart.rounded", label: "Start / End", category: "Flowchart", w: 150, h: 60, shape: "rounded" },
  { id: "flowchart.decision", label: "Decision", category: "Flowchart", w: 136, h: 90, shape: "diamond" },
  { id: "flowchart.database", label: "Database", category: "Flowchart", w: 136, h: 92, shape: "cylinder" },
  { id: "flowchart.data_io", label: "Data I/O", category: "Flowchart", w: 160, h: 72, shape: "parallelogram" },
  { id: "flowchart.hexagon", label: "Hexagon", category: "Flowchart", w: 160, h: 76, shape: "hexagon" },
  { id: "flowchart.document", label: "Document", category: "Flowchart", w: 155, h: 82, shape: "document" },
  { id: "flowchart.note", label: "Note", category: "Flowchart", w: 145, h: 86, shape: "note" },
  { id: "flowchart.circle", label: "Circle", category: "Flowchart", w: 80, h: 80, shape: "circle" },

  // UML & BPMN
  { id: "uml.class", label: "UML Class", category: "UML & BPMN", w: 180, h: 92, shape: "uml_class" },
  { id: "uml.interface", label: "Interface", category: "UML & BPMN", w: 180, h: 88, shape: "uml_interface" },
  { id: "uml.component", label: "Component", category: "UML & BPMN", w: 180, h: 78, shape: "uml_component" },
  { id: "uml.package", label: "Package", category: "UML & BPMN", w: 175, h: 90, shape: "uml_package" },
  { id: "uml.actor", label: "User / Actor", category: "UML & BPMN", w: 72, h: 76, shape: "actor" },
  { id: "bpmn.gateway.xor", label: "XOR Gateway", category: "UML & BPMN", w: 80, h: 80, shape: "gateway_xor" },
  { id: "bpmn.gateway.par", label: "Parallel (+)", category: "UML & BPMN", w: 80, h: 80, shape: "gateway_parallel" },
  { id: "bpmn.timer", label: "Timer Event", category: "UML & BPMN", w: 74, h: 74, shape: "timer" },

  // Cloud & Kubernetes
  { id: "cloud.aws.lambda", label: "AWS Lambda", category: "Cloud & K8s", w: 180, h: 74, shape: "aws_lambda" },
  { id: "cloud.aws.s3", label: "S3 Storage", category: "Cloud & K8s", w: 180, h: 74, shape: "aws_s3" },
  { id: "cloud.gcp.cloudrun", label: "Cloud Service", category: "Cloud & K8s", w: 175, h: 90, shape: "cloud" },
  { id: "cloud.k8s.pod", label: "K8s Pod", category: "Cloud & K8s", w: 180, h: 74, shape: "k8s_pod" },

  // Network & Infra
  { id: "network.queue", label: "Kafka / Queue", category: "Network & Infra", w: 180, h: 72, shape: "queue" },
  { id: "network.server", label: "Server Node", category: "Network & Infra", w: 180, h: 74, shape: "server" },
  { id: "network.firewall", label: "WAF Firewall", category: "Network & Infra", w: 180, h: 74, shape: "firewall" },
  { id: "network.shield", label: "Auth Shield", category: "Network & Infra", w: 126, h: 94, shape: "shield" },

  // Wireframes
  { id: "wireframe.browser", label: "Browser UI", category: "Wireframes", w: 220, h: 132, shape: "browser" },
  { id: "wireframe.mobile", label: "Mobile UI", category: "Wireframes", w: 118, h: 180, shape: "mobile" },
];

const STYLE_SWATCHES = [
  { fill: "#131b2e", stroke: "#38bdf8" },
  { fill: "#161632", stroke: "#818cf8" },
  { fill: "#0c2924", stroke: "#34d399" },
  { fill: "#2b1c10", stroke: "#fbbf24" },
  { fill: "#2d1220", stroke: "#f43f5e" },
  { fill: "#241236", stroke: "#c084fc" },
  { fill: "#ffffff", stroke: "#4f46e5" },
  { fill: "#f0f9ff", stroke: "#0284c7" },
  { fill: "#f0fdf4", stroke: "#16a34a" },
  { fill: "#fffbeb", stroke: "#d97706" },
  { fill: "#fff1f2", stroke: "#e11d48" },
  { fill: "#0f172a", stroke: "#94a3b8" },
];

function createDogfoodPages() {
  return [
    {
      id: 1,
      name: "Page-1: RustFlow Architecture",
      pan: { x: 50, y: 40 },
      zoom: 1.0,
      nodes: [
        { id: 1, title: "crates/core", body: "R-Tree + Scene + CRDT", x: 70, y: 155, w: 180, h: 86, fill: "#131b2e", stroke: "#818cf8", shape: "uml_package", z: 1 },
        { id: 2, title: "crates/renderer", body: "wgpu WGSL + Bezier", x: 355, y: 45, w: 185, h: 78, fill: "#131b2e", stroke: "#38bdf8", shape: "uml_component", z: 2 },
        { id: 3, title: "crates/layout", body: "Sugiyama + Force + Spring", x: 355, y: 160, w: 185, h: 78, fill: "#131b2e", stroke: "#34d399", shape: "hexagon", z: 3 },
        { id: 4, title: "crates/io-formats", body: ".drawio XML + SVG + PDF", x: 355, y: 280, w: 185, h: 82, fill: "#131b2e", stroke: "#fbbf24", shape: "document", z: 4 },
        { id: 5, title: "crates/ui-bridge", body: "WASM & Tauri v2 Shell", x: 650, y: 155, w: 205, h: 108, fill: "#131b2e", stroke: "#f472b6", shape: "browser", z: 5 },
      ],
      edges: [
        { id: 101, source: 1, sourcePort: "right", target: 2, targetPort: "left", label: "Frustum Query", anim: "pulse", kind: "bezier", color: "#818cf8", width: 1.8, bend: null },
        { id: 102, source: 1, sourcePort: "right", target: 3, targetPort: "left", label: "Graph Moves", anim: "ants", kind: "bezier", color: "#34d399", width: 1.8, bend: null },
        { id: 103, source: 1, sourcePort: "right", target: 4, targetPort: "left", label: "AST Round-Trip", anim: "pulse", kind: "bezier", color: "#fbbf24", width: 1.8, bend: null },
        { id: 104, source: 2, sourcePort: "right", target: 5, targetPort: "left", label: "Frame Buffers", anim: "pulse", kind: "bezier", color: "#38bdf8", width: 1.8, bend: null },
        { id: 105, source: 3, sourcePort: "right", target: 5, targetPort: "left", label: "Transitions", anim: "ants", kind: "bezier", color: "#818cf8", width: 1.8, bend: null },
        { id: 106, source: 4, sourcePort: "right", target: 5, targetPort: "left", label: "Exporters", anim: "ants", kind: "bezier", color: "#f472b6", width: 1.8, bend: null },
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
        { id: 10, title: "Client User", body: "HTTPS", x: 75, y: 132, w: 72, h: 76, fill: "#131b2e", stroke: "#38bdf8", shape: "actor", z: 1 },
        { id: 11, title: "WAF Firewall", body: "Rate Limiter", x: 235, y: 133, w: 180, h: 74, fill: "#131b2e", stroke: "#f43f5e", shape: "firewall", z: 2 },
        { id: 12, title: "Auth Check", body: "XOR", x: 495, y: 130, w: 80, h: 80, fill: "#131b2e", stroke: "#fbbf24", shape: "gateway_xor", z: 3 },
        { id: 13, title: "Kafka Stream", body: "events-topic", x: 660, y: 80, w: 180, h: 72, fill: "#131b2e", stroke: "#34d399", shape: "queue", z: 4 },
        { id: 14, title: "Postgres DB", body: "pgvector", x: 680, y: 210, w: 136, h: 92, fill: "#131b2e", stroke: "#818cf8", shape: "cylinder", z: 5 },
      ],
      edges: [
        { id: 201, source: 10, sourcePort: "right", target: 11, targetPort: "left", label: "Request", anim: "pulse", kind: "bezier", color: "#38bdf8", width: 1.8, bend: null },
        { id: 202, source: 11, sourcePort: "right", target: 12, targetPort: "left", label: "Verify", anim: "ants", kind: "bezier", color: "#fbbf24", width: 1.8, bend: null },
        { id: 203, source: 12, sourcePort: "right", target: 13, targetPort: "left", label: "Publish", anim: "pulse", kind: "bezier", color: "#34d399", width: 1.8, bend: null },
        { id: 204, source: 12, sourcePort: "bottom", target: 14, targetPort: "left", label: "Persist", anim: "pulse", kind: "smoothstep", color: "#818cf8", width: 1.8, bend: null },
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
  selectedNodeIds: new Set([1]),
  selectedEdgeIds: new Set(),
  clipboard: { nodes: [], edges: [], pasteStep: 0 },
  activeFormatTab: "style",
  gridStyle: "dots",
  gridSize: 20,
  snapToGrid: false,
  hoveredPort: null,
  snapTargetPort: null,
  draggingNode: null,
  resizingNode: null,
  draggingEdgeBend: null,
  reconnectingEdge: null,
  draggingStencil: null,    // { item, startX, startY, active, overCanvas, ghostX, ghostY }
  marqueeBox: null,
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
  arrangeDirty: false,
  layoutAnim: null,
  viewAnim: null,
};

const canvas = document.getElementById("rustflow-canvas");
const ctx = canvas.getContext("2d", { alpha: false, desynchronized: true }) || canvas.getContext("2d", { alpha: false });
const minimapCanvas = document.getElementById("minimap-canvas");
const minimapCtx = minimapCanvas.getContext("2d");
const inlineEditor = document.getElementById("inline-editor");
const statusBarEl = document.getElementById("status-bar");
const zoomLabelEl = document.getElementById("zoom-label");
const toastBadgeEl = document.getElementById("toast-badge");

let toastTimer = null;
function showToast(msg) {
  if (!toastBadgeEl) return;
  toastBadgeEl.textContent = msg;
  toastBadgeEl.classList.add("show");
  if (toastTimer) clearTimeout(toastTimer);
  toastTimer = setTimeout(() => {
    toastBadgeEl.classList.remove("show");
  }, 1600);
}

function setSingleNodeSelection(nodeId) {
  state.selectedNodeIds.clear();
  state.selectedEdgeIds.clear();
  if (nodeId != null) state.selectedNodeIds.add(nodeId);
  state.selectedNodeId = nodeId ?? null;
  state.selectedEdgeId = null;
}

function setSingleEdgeSelection(edgeId) {
  state.selectedNodeIds.clear();
  state.selectedEdgeIds.clear();
  if (edgeId != null) state.selectedEdgeIds.add(edgeId);
  state.selectedEdgeId = edgeId ?? null;
  state.selectedNodeId = null;
}

function syncPrimarySelectionFromSets() {
  state.selectedNodeId = state.selectedNodeIds.size === 1 ? [...state.selectedNodeIds][0] : null;
  state.selectedEdgeId =
    state.selectedNodeIds.size === 0 && state.selectedEdgeIds.size === 1 ? [...state.selectedEdgeIds][0] : null;
}

let canvasRect = { left: 0, top: 0, right: 800, bottom: 600, width: 800, height: 600 };
let currentCursor = "default";
let nodeMap = new Map();
let cachedGridPattern = null;
let cachedGridKey = "";

function setCanvasCursor(cursor) {
  if (cursor !== currentCursor) {
    currentCursor = cursor;
    canvas.style.cursor = cursor;
  }
}

function page() {
  return state.pages.find((p) => p.id === state.activePageId) || state.pages[0];
}

function rebuildNodeMap() {
  const p = page();
  nodeMap.clear();
  for (let i = 0; i < p.nodes.length; i++) {
    const n = p.nodes[i];
    nodeMap.set(n.id, n);
  }
}

function invalidateGeometry() {
  state.geometryVersion++;
  state.minimapDirty = true;
  rebuildNodeMap();
}

function invalidateNodeEdges(nodeId) {
  const edges = page().edges;
  for (let i = 0; i < edges.length; i++) {
    const e = edges[i];
    if (e.source === nodeId || e.target === nodeId) {
      e._ver = -1;
    }
  }
  state.minimapDirty = true;
  state.arrangeDirty = true;
}

function invalidateSelectedNodesEdges() {
  const edges = page().edges;
  for (let i = 0; i < edges.length; i++) {
    const e = edges[i];
    if (state.selectedNodeIds.has(e.source) || state.selectedNodeIds.has(e.target)) {
      e._ver = -1;
    }
  }
  state.minimapDirty = true;
  state.arrangeDirty = true;
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

function resizeCanvas() {
  const rect = canvas.parentElement.getBoundingClientRect();
  canvasRect = rect;
  canvas.width = Math.max(1, Math.floor(rect.width));
  canvas.height = Math.max(1, Math.floor(rect.height));
  cachedGridKey = "";
  state.minimapDirty = true;
}
window.addEventListener("resize", resizeCanvas);
window.addEventListener(
  "scroll",
  () => {
    canvasRect = canvas.parentElement.getBoundingClientRect();
  },
  true
);
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
  for (let i = 0; i < p.nodes.length; i++) {
    const n = p.nodes[i];
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
  const pad = 22;
  return [
    { dir: "up", side: "top", targetSide: "bottom", x: n.x + n.w / 2, y: n.y - pad, dx: 0, dy: -(n.h + 75), label: "↑" },
    { dir: "right", side: "right", targetSide: "left", x: n.x + n.w + pad, y: n.y + n.h / 2, dx: n.w + 95, dy: 0, label: "→" },
    { dir: "down", side: "bottom", targetSide: "top", x: n.x + n.w / 2, y: n.y + n.h + pad, dx: 0, dy: n.h + 75, label: "↓" },
    { dir: "left", side: "left", targetSide: "right", x: n.x - pad, y: n.y + n.h / 2, dx: -(n.w + 95), dy: 0, label: "←" },
  ];
}

function evalBezier(p0, p1, p2, p3, t) {
  const u = 1 - t;
  const uu = u * u;
  const tt = t * t;
  return {
    x: uu * u * p0.x + 3 * uu * t * p1.x + 3 * u * tt * p2.x + tt * t * p3.x,
    y: uu * u * p0.y + 3 * uu * t * p1.y + 3 * u * tt * p2.y + tt * t * p3.y,
  };
}

function buildSmoothStepSamples(p0, c1, c2, p3) {
  const pts = [p0];
  const corners = [
    [p0, c1, c2],
    [c1, c2, p3],
  ];
  for (const [prev, curr, next] of corners) {
    const dx1 = prev.x - curr.x;
    const dy1 = prev.y - curr.y;
    const len1 = Math.hypot(dx1, dy1);
    const dx2 = next.x - curr.x;
    const dy2 = next.y - curr.y;
    const len2 = Math.hypot(dx2, dy2);
    const r = Math.min(14, len1 * 0.45, len2 * 0.45);
    if (r < 1.5) {
      pts.push(curr);
      continue;
    }
    const start = { x: curr.x + (dx1 / len1) * r, y: curr.y + (dy1 / len1) * r };
    const end = { x: curr.x + (dx2 / len2) * r, y: curr.y + (dy2 / len2) * r };
    for (let s = 0; s <= 6; s++) {
      const t = s / 6;
      const u = 1 - t;
      pts.push({
        x: u * u * start.x + 2 * u * t * curr.x + t * t * end.x,
        y: u * u * start.y + 2 * u * t * curr.y + t * t * end.y,
      });
    }
  }
  pts.push(p3);
  return pts;
}

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

  let samples = [];
  if (e.kind === "straight") {
    if (e.bend) {
      samples = [p0, midPoint, p3];
    } else {
      samples = [p0, p3];
      midPoint = { x: (p0.x + p3.x) * 0.5, y: (p0.y + p3.y) * 0.5 };
    }
  } else if (e.kind === "orthogonal") {
    const bendX = e.bend ? e.bend.x : (p0.x + p3.x) * 0.5;
    const bendY = e.bend ? e.bend.y : (p0.y + p3.y) * 0.5;
    midPoint = { x: bendX, y: bendY };
    samples = [p0, { x: bendX, y: p0.y }, { x: bendX, y: p3.y }, p3];
  } else if (e.kind === "smoothstep") {
    const bendX = e.bend ? e.bend.x : (p0.x + p3.x) * 0.5;
    const bendY = e.bend ? e.bend.y : (p0.y + p3.y) * 0.5;
    midPoint = { x: bendX, y: bendY };
    samples = buildSmoothStepSamples(p0, { x: bendX, y: p0.y }, { x: bendX, y: p3.y }, p3);
  } else {
    const steps = 36;
    for (let i = 0; i <= steps; i++) {
      samples.push(evalBezier(p0, p1, p2, p3, i / steps));
    }
  }

  const cumLen = new Float32Array(samples.length);
  let totalLen = 0;
  let minX = p0.x, maxX = p0.x, minY = p0.y, maxY = p0.y;

  for (let i = 0; i < samples.length; i++) {
    const pt = samples[i];
    if (pt.x < minX) minX = pt.x;
    if (pt.x > maxX) maxX = pt.x;
    if (pt.y < minY) minY = pt.y;
    if (pt.y > maxY) maxY = pt.y;
    if (i > 0) {
      const prev = samples[i - 1];
      totalLen += Math.hypot(pt.x - prev.x, pt.y - prev.y);
    }
    cumLen[i] = totalLen;
  }

  const labelWidth = e.label ? e.label.length * 6.6 + 14 : 0;

  e._geo = {
    p0,
    p1,
    p2,
    p3,
    midPoint,
    samples,
    cumLen,
    totalLen: Math.max(1, totalLen),
    labelWidth,
    bounds: { minX: minX - 24, maxX: maxX + 24, minY: minY - 24, maxY: maxY + 24 },
  };
  e._ver = state.geometryVersion;
  return e._geo;
}

function getPointAtArcDistance(geo, dist) {
  const { samples, cumLen, totalLen } = geo;
  const n = samples.length;
  if (n < 2) return samples[0] || { x: 0, y: 0 };

  const d = ((dist % totalLen) + totalLen) % totalLen;
  let lo = 0;
  let hi = n - 1;
  while (lo + 1 < hi) {
    const mid = (lo + hi) >> 1;
    if (cumLen[mid] <= d) lo = mid;
    else hi = mid;
  }
  const segStart = cumLen[lo];
  const segLen = cumLen[hi] - segStart;
  const frac = segLen > 1e-4 ? (d - segStart) / segLen : 0;
  const a = samples[lo];
  const b = samples[hi];
  return {
    x: a.x + (b.x - a.x) * frac,
    y: a.y + (b.y - a.y) * frac,
  };
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

function computeSmartGuides(movingNode, ignoreSelection = false) {
  const guides = [];
  const threshold = 4.5;
  const nodes = page().nodes;
  for (let i = 0; i < nodes.length; i++) {
    const other = nodes[i];
    if (other.id === movingNode.id || (!ignoreSelection && state.selectedNodeIds.has(other.id))) continue;
    const checksX = [
      [movingNode.x, other.x],
      [movingNode.x + movingNode.w * 0.5, other.x + other.w * 0.5],
      [movingNode.x + movingNode.w, other.x + other.w],
    ];
    for (let k = 0; k < 3; k++) {
      const [mx, ox] = checksX[k];
      if (Math.abs(mx - ox) <= threshold) {
        movingNode.x += ox - mx;
        guides.push({ type: "v", x: ox, y1: Math.min(movingNode.y, other.y) - 24, y2: Math.max(movingNode.y + movingNode.h, other.y + other.h) + 24 });
        break;
      }
    }
    const checksY = [
      [movingNode.y, other.y],
      [movingNode.y + movingNode.h * 0.5, other.y + other.h * 0.5],
      [movingNode.y + movingNode.h, other.y + other.h],
    ];
    for (let k = 0; k < 3; k++) {
      const [my, oy] = checksY[k];
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
// 1. TOP DROPDOWN MENU BAR & ACTIONS
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
      showToast("Exported diagram.rfd.json");
      break;
    case "undo":
      undo();
      break;
    case "redo":
      redo();
      break;
    case "cut":
      cutSelection();
      break;
    case "copy":
      copySelection();
      break;
    case "paste":
      pasteSelection();
      break;
    case "duplicate":
      duplicateSelected();
      break;
    case "select-all":
      selectAll();
      break;
    case "clear-selection":
      clearSelection();
      break;
    case "delete":
      deleteSelected();
      break;
    case "fit-view":
      fitToScreen();
      break;
    case "zoom-in":
      animateZoom(p.zoom * 1.25);
      break;
    case "zoom-out":
      animateZoom(p.zoom / 1.25);
      break;
    case "zoom-reset":
      animateViewTo(60, 50, 1.0);
      showToast("Reset Zoom (100%)");
      break;
    case "toggle-grid":
      state.gridStyle = state.gridStyle === "dots" ? "lines" : state.gridStyle === "lines" ? "off" : "dots";
      cachedGridKey = "";
      renderFormatPanel();
      showToast(`Grid: ${state.gridStyle.toUpperCase()}`);
      break;
    case "theme-dark":
    case "theme-light":
    case "theme-contrast":
      state.theme = action.replace("theme-", "");
      cachedGridKey = "";
      document.documentElement.setAttribute("data-theme", state.theme);
      renderFormatPanel();
      showToast(`Theme: ${state.theme}`);
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
    case "shortcuts":
      openShortcutsModal();
      break;
    case "cmd-palette":
      openCommandPalette();
      break;
    case "stress-100":
      generateStressGraph(100);
      break;
  }
}

function setZoomImmediate(targetZoom, cx = canvas.width / 2, cy = canvas.height / 2) {
  const p = page();
  state.viewAnim = null;
  const before = screenToWorld(cx, cy);
  p.zoom = Math.min(8.0, Math.max(0.1, targetZoom));
  p.pan.x = cx - before.x * p.zoom;
  p.pan.y = cy - before.y * p.zoom;
  state.minimapDirty = true;
  updateZoomLabel();
}

function animateZoom(targetZoom, cx = canvas.width / 2, cy = canvas.height / 2) {
  const clampedZoom = Math.min(8.0, Math.max(0.1, targetZoom));
  const before = screenToWorld(cx, cy);
  const targetPanX = cx - before.x * clampedZoom;
  const targetPanY = cy - before.y * clampedZoom;
  animateViewTo(targetPanX, targetPanY, clampedZoom);
}

function animateViewTo(targetPanX, targetPanY, targetZoom) {
  const p = page();
  state.viewAnim = {
    startTime: performance.now(),
    duration: 200,
    startPanX: p.pan.x,
    startPanY: p.pan.y,
    startZoom: p.zoom,
    targetPanX,
    targetPanY,
    targetZoom,
  };
}

function fitToScreen() {
  const p = page();
  if (!p.nodes.length) {
    animateViewTo(60, 50, 1.0);
    return;
  }
  let minX = Infinity, minY = Infinity, maxX = -Infinity, maxY = -Infinity;
  for (const n of p.nodes) {
    if (n.x < minX) minX = n.x;
    if (n.y < minY) minY = n.y;
    if (n.x + n.w > maxX) maxX = n.x + n.w;
    if (n.y + n.h > maxY) maxY = n.y + n.h + 24;
  }
  const contentW = Math.max(100, maxX - minX);
  const contentH = Math.max(100, maxY - minY);
  const pad = 70;
  const zoomX = (canvas.width - pad * 2) / contentW;
  const zoomY = (canvas.height - pad * 2) / contentH;
  const targetZoom = Math.min(2.0, Math.max(0.2, Math.min(zoomX, zoomY)));
  const targetPanX = (canvas.width - contentW * targetZoom) * 0.5 - minX * targetZoom;
  const targetPanY = (canvas.height - contentH * targetZoom) * 0.5 - minY * targetZoom;
  animateViewTo(targetPanX, targetPanY, targetZoom);
  showToast("Fit Diagram to Screen");
}

function animateNodesToTargets(targetsMap) {
  const p = page();
  const entries = [];
  for (const n of p.nodes) {
    const tgt = targetsMap.get(n.id);
    if (tgt) {
      entries.push({ node: n, sx: n.x, sy: n.y, tx: tgt.x, ty: tgt.y });
    }
  }
  state.layoutAnim = {
    startTime: performance.now(),
    duration: 320,
    entries,
  };
}

function updateZoomLabel() {
  zoomLabelEl.textContent = `${Math.round(page().zoom * 100)}%`;
}

// ============================================================================
// 2. DRAW.IO MINIMALIST CONTEXT-SENSITIVE RIGHT FORMAT PANEL
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

const SHAPE_OPTIONS = [
  ["card", "Process Card"],
  ["rounded", "Start / End Pill"],
  ["diamond", "Decision Diamond"],
  ["cylinder", "Database Cylinder"],
  ["queue", "Kafka / Queue Pipe"],
  ["parallelogram", "Data I/O Parallelogram"],
  ["hexagon", "Hexagon"],
  ["document", "Document"],
  ["note", "Note"],
  ["circle", "Circle"],
  ["actor", "User / Actor"],
  ["uml_class", "UML Class"],
  ["uml_interface", "UML Interface"],
  ["uml_component", "UML Component"],
  ["uml_package", "UML Package"],
  ["gateway_xor", "XOR Gateway (×)"],
  ["gateway_parallel", "Parallel Gateway (+)"],
  ["timer", "Timer Event"],
  ["aws_lambda", "AWS Lambda"],
  ["aws_s3", "S3 Storage"],
  ["cloud", "Cloud Service"],
  ["k8s_pod", "K8s Pod"],
  ["server", "Server Node"],
  ["firewall", "WAF Firewall"],
  ["shield", "Auth Shield"],
  ["browser", "Browser UI"],
  ["mobile", "Mobile UI"],
];

function renderFormatPanel() {
  const p = page();
  const totalSelected = state.selectedNodeIds.size + state.selectedEdgeIds.size;
  const node = nodeMap.get(state.selectedNodeId);
  const edge = p.edges.find((e) => e.id === state.selectedEdgeId);

  formatTabsEl.querySelectorAll(".format-tab").forEach((t) => {
    t.classList.toggle("active", t.getAttribute("data-tab") === state.activeFormatTab);
  });

  if (totalSelected > 1) {
    formatBadgeEl.textContent = `${state.selectedNodeIds.size}N · ${state.selectedEdgeIds.size}E`;
    formatTabsEl.style.display = "none";
    formatBodyEl.innerHTML = `
      <div class="format-section">
        <label>Batch Style Presets (${state.selectedNodeIds.size} Nodes)</label>
        <div class="swatch-grid" id="multi-swatches"></div>
      </div>
      <div class="format-section">
        <label>Selection Actions</label>
        <div class="format-row">
          <button id="btn-multi-copy">Copy (Ctrl+C)</button>
          <button id="btn-multi-dup">Duplicate (Ctrl+D)</button>
        </div>
        <button id="btn-multi-del" class="danger" style="width:100%; margin-top:4px;">Delete Selected (Del)</button>
      </div>
    `;
    const swatchContainer = document.getElementById("multi-swatches");
    STYLE_SWATCHES.forEach((sw) => {
      const el = document.createElement("div");
      el.className = "swatch";
      el.style.background = sw.fill;
      el.style.borderColor = sw.stroke;
      el.onclick = () => {
        saveSnapshot();
        for (const nid of state.selectedNodeIds) {
          const n = nodeMap.get(nid);
          if (n) {
            n.fill = sw.fill;
            n.stroke = sw.stroke;
          }
        }
      };
      swatchContainer.appendChild(el);
    });
    document.getElementById("btn-multi-copy").onclick = copySelection;
    document.getElementById("btn-multi-dup").onclick = duplicateSelected;
    document.getElementById("btn-multi-del").onclick = deleteSelected;
    return;
  }

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
        <div class="format-row">
          <span>Snap to Grid</span>
          <input id="diag-snap-grid" type="checkbox" ${state.snapToGrid ? "checked" : ""} />
        </div>
      </div>
      <div class="format-section">
        <label>Theme</label>
        <select id="diag-theme">
          <option value="dark" ${state.theme === "dark" ? "selected" : ""}>Dark Minimal</option>
          <option value="light" ${state.theme === "light" ? "selected" : ""}>Light Minimal</option>
          <option value="contrast" ${state.theme === "contrast" ? "selected" : ""}>High Contrast</option>
        </select>
      </div>
      <div class="format-section" style="border-bottom:none;">
        <label>Quick Guide</label>
        <p style="font-size:11px; color:var(--text-muted); line-height:1.6;">
          <b>Drag</b> any symbol from the left panel onto the canvas.<br/>
          <b>Shift+Drag</b> to box-select multiple items.<br/>
          <b>Ctrl+A / C / V / D / Z</b> supported.
        </p>
      </div>
    `;
    document.getElementById("diag-page-name").oninput = (e) => {
      p.name = e.target.value;
      renderPageTabs();
    };
    document.getElementById("diag-grid-style").onchange = (e) => {
      state.gridStyle = e.target.value;
      cachedGridKey = "";
    };
    document.getElementById("diag-grid-size").oninput = (e) => {
      state.gridSize = Math.max(10, Math.min(80, Number(e.target.value) || 20));
      cachedGridKey = "";
    };
    document.getElementById("diag-snap-grid").onchange = (e) => {
      state.snapToGrid = e.target.checked;
    };
    document.getElementById("diag-theme").onchange = (e) => {
      state.theme = e.target.value;
      cachedGridKey = "";
      document.documentElement.setAttribute("data-theme", state.theme);
    };
    return;
  }

  formatTabsEl.style.display = "grid";

  if (node) {
    formatBadgeEl.textContent = `Node #${node.id}`;
    if (state.activeFormatTab === "style") {
      formatBodyEl.innerHTML = `
        <div class="format-section">
          <label>Minimalist Style Presets</label>
          <div class="swatch-grid" id="node-swatches"></div>
        </div>
        <div class="format-section">
          <label>Symbol</label>
          <select id="insp-shape">
            ${SHAPE_OPTIONS.map(([val, lbl]) => `<option value="${val}" ${node.shape === val ? "selected" : ""}>${lbl}</option>`).join("")}
          </select>
        </div>
        <div class="format-section">
          <label>Colors</label>
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
          <label>Title</label>
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
          <label>Layering</label>
          <div class="format-row">
            <button id="btn-front">Bring Front</button>
            <button id="btn-back">Send Back</button>
          </div>
        </div>
        <div class="format-section" style="border-bottom:none;">
          <button id="btn-node-dup" style="width:100%">Duplicate (Ctrl+D)</button>
          <button id="btn-node-del" class="danger" style="width:100%; margin-top:4px;">Delete (Del)</button>
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
          invalidateNodeEdges(node.id);
        };
      }
      document.getElementById("btn-front").onclick = bringToFront;
      document.getElementById("btn-back").onclick = sendToBack;
      document.getElementById("btn-node-dup").onclick = duplicateSelected;
      document.getElementById("btn-node-del").onclick = deleteSelected;
    }
    return;
  }

  if (edge) {
    formatBadgeEl.textContent = `Edge #${edge.id}`;
    if (state.activeFormatTab === "style") {
      formatBodyEl.innerHTML = `
        <div class="format-section">
          <label>Routing</label>
          <select id="edge-kind">
            <option value="bezier" ${edge.kind === "bezier" ? "selected" : ""}>Bezier Curve</option>
            <option value="smoothstep" ${edge.kind === "smoothstep" ? "selected" : ""}>SmoothStep</option>
            <option value="orthogonal" ${edge.kind === "orthogonal" ? "selected" : ""}>Orthogonal</option>
            <option value="straight" ${edge.kind === "straight" ? "selected" : ""}>Straight Line</option>
          </select>
          <label style="margin-top:6px;">Flow Animation</label>
          <select id="edge-anim">
            <option value="pulse" ${edge.anim === "pulse" ? "selected" : ""}>Pulse Particles</option>
            <option value="ants" ${edge.anim === "ants" ? "selected" : ""}>Dashed Flow</option>
            <option value="none" ${edge.anim === "none" ? "selected" : ""}>Static</option>
          </select>
        </div>
        <div class="format-section">
          <label>Color & Width</label>
          <div class="format-row">
            <input id="edge-color" type="color" value="${edge.color || "#818cf8"}" style="width:100%" />
            <input id="edge-width" type="number" min="1" max="10" step="0.5" value="${edge.width || 1.8}" />
          </div>
        </div>
      `;
      document.getElementById("edge-kind").onchange = (e) => {
        edge.kind = e.target.value;
        edge._ver = -1;
      };
      document.getElementById("edge-anim").onchange = (e) => {
        edge.anim = e.target.value;
      };
      document.getElementById("edge-color").oninput = (e) => {
        edge.color = e.target.value;
      };
      document.getElementById("edge-width").oninput = (e) => {
        edge.width = Number(e.target.value) || 1.8;
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
        edge._ver = -1;
      };
    } else {
      formatBodyEl.innerHTML = `
        <div class="format-section">
          <label>Port Sides</label>
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
            <button id="btn-edge-reverse">Reverse</button>
            <button id="btn-edge-reset">Reset Bend</button>
          </div>
          <button id="btn-edge-delete" class="danger" style="width:100%; margin-top:6px;">Delete Edge</button>
        </div>
      `;
      document.getElementById("edge-src-port").onchange = (e) => {
        edge.sourcePort = e.target.value;
        edge._ver = -1;
      };
      document.getElementById("edge-dst-port").onchange = (e) => {
        edge.targetPort = e.target.value;
        edge._ver = -1;
      };
      document.getElementById("btn-edge-reverse").onclick = () => {
        saveSnapshot();
        const tmpN = edge.source, tmpP = edge.sourcePort;
        edge.source = edge.target;
        edge.sourcePort = edge.targetPort;
        edge.target = tmpN;
        edge.targetPort = tmpP;
        edge._ver = -1;
        renderFormatPanel();
      };
      document.getElementById("btn-edge-reset").onclick = () => {
        saveSnapshot();
        edge.bend = null;
        edge._ver = -1;
      };
      document.getElementById("btn-edge-delete").onclick = deleteSelected;
    }
  }
}
renderFormatPanel();

function syncArrangeInputsIfDirty() {
  if (!state.arrangeDirty) return;
  state.arrangeDirty = false;
  if (state.activeFormatTab !== "arrange") return;
  const node = nodeMap.get(state.selectedNodeId);
  if (!node) return;
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
// POINTER, DRAG-AND-DROP STENCIL & INTERACTION HANDLERS
// ============================================================================
function createNodeFromStencil(item, worldX, worldY) {
  saveSnapshot();
  const p = page();
  const newNode = {
    id: ++state.nextId,
    title: item.label,
    body: item.id.split(".").pop(),
    x: Math.round(worldX),
    y: Math.round(worldY),
    w: item.w,
    h: item.h,
    fill: state.theme === "light" ? "#ffffff" : "#131b2e",
    stroke: "#38bdf8",
    shape: item.shape,
    z: p.nodes.length + 1,
  };
  p.nodes.push(newNode);
  invalidateGeometry();
  setSingleNodeSelection(newNode.id);
  renderFormatPanel();
  showToast(`Added ${item.label}`);
}

function addStencilToCanvasCenter(item) {
  const center = screenToWorld(canvas.width / 2 - item.w / 2, canvas.height / 2 - item.h / 2);
  createNodeFromStencil(item, Math.round(center.x / 20) * 20, Math.round(center.y / 20) * 20);
}

// HTML5 Drag & Drop support on Canvas
canvas.addEventListener("dragover", (e) => {
  e.preventDefault();
  e.dataTransfer.dropEffect = "copy";
  if (state.draggingStencil) {
    const sx = e.clientX - canvasRect.left;
    const sy = e.clientY - canvasRect.top;
    const wpt = screenToWorld(sx, sy);
    const item = state.draggingStencil.item;
    const ghost = {
      id: -999,
      x: state.snapToGrid ? Math.round((wpt.x - item.w * 0.5) / state.gridSize) * state.gridSize : wpt.x - item.w * 0.5,
      y: state.snapToGrid ? Math.round((wpt.y - item.h * 0.5) / state.gridSize) * state.gridSize : wpt.y - item.h * 0.5,
      w: item.w,
      h: item.h,
    };
    state.smartGuides = computeSmartGuides(ghost, true);
    state.draggingStencil.active = true;
    state.draggingStencil.overCanvas = true;
    state.draggingStencil.ghostX = ghost.x;
    state.draggingStencil.ghostY = ghost.y;
  }
});

canvas.addEventListener("dragleave", () => {
  if (state.draggingStencil) {
    state.draggingStencil.overCanvas = false;
    state.smartGuides = [];
  }
});

canvas.addEventListener("drop", (e) => {
  e.preventDefault();
  if (state.draggingStencil) {
    const item = state.draggingStencil.item;
    const sx = e.clientX - canvasRect.left;
    const sy = e.clientY - canvasRect.top;
    const wpt = screenToWorld(sx, sy);
    const dropX = state.draggingStencil.overCanvas ? state.draggingStencil.ghostX : wpt.x - item.w * 0.5;
    const dropY = state.draggingStencil.overCanvas ? state.draggingStencil.ghostY : wpt.y - item.h * 0.5;
    state.draggingStencil = null;
    state.smartGuides = [];
    createNodeFromStencil(item, dropX, dropY);
  }
});

canvas.addEventListener("mousedown", (e) => {
  inlineEditor.style.display = "none";
  state.layoutAnim = null;
  state.viewAnim = null;

  const sx = e.clientX - canvasRect.left;
  const sy = e.clientY - canvasRect.top;
  const wpt = screenToWorld(sx, sy);
  state.mouseWorld = wpt;
  const p = page();
  const handleHitRadius = 13 / Math.max(0.4, p.zoom);

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

  const selectedNode = nodeMap.get(state.selectedNodeId);
  if (selectedNode && state.selectedNodeIds.size === 1) {
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
          width: 1.8,
          bend: null,
        });
        invalidateGeometry();
        setSingleNodeSelection(newNode.id);
        renderFormatPanel();
        return;
      }
    }
  }

  const portHitRadius = 15 / Math.max(0.4, p.zoom);
  for (let i = 0; i < p.nodes.length; i++) {
    const n = p.nodes[i];
    for (const port of getNodePorts(n)) {
      if (Math.hypot(wpt.x - port.x, wpt.y - port.y) <= portHitRadius) {
        state.connectingFrom = { node: n, port };
        setSingleNodeSelection(n.id);
        renderFormatPanel();
        return;
      }
    }
  }

  const hitEdge = hitTestEdge(wpt);
  if (hitEdge) {
    if (e.shiftKey) {
      if (state.selectedEdgeIds.has(hitEdge.id)) state.selectedEdgeIds.delete(hitEdge.id);
      else state.selectedEdgeIds.add(hitEdge.id);
      syncPrimarySelectionFromSets();
    } else {
      setSingleEdgeSelection(hitEdge.id);
    }
    renderFormatPanel();
    const geo = getEdgeGeometry(hitEdge);
    if (geo && Math.hypot(wpt.x - geo.midPoint.x, wpt.y - geo.midPoint.y) <= handleHitRadius * 1.3) {
      saveSnapshot();
      state.draggingEdgeBend = hitEdge;
    }
    return;
  }

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
    if (e.shiftKey) {
      if (state.selectedNodeIds.has(hit.id)) state.selectedNodeIds.delete(hit.id);
      else state.selectedNodeIds.add(hit.id);
      syncPrimarySelectionFromSets();
      renderFormatPanel();
    } else if (!state.selectedNodeIds.has(hit.id)) {
      setSingleNodeSelection(hit.id);
      renderFormatPanel();
    }

    const initialPositions = new Map();
    for (const nid of state.selectedNodeIds) {
      const n = nodeMap.get(nid);
      if (n) initialPositions.set(nid, { x: n.x, y: n.y });
    }
    state.draggingNode = {
      leadNode: hit,
      offsetX: wpt.x - hit.x,
      offsetY: wpt.y - hit.y,
      initialPositions,
    };
  } else {
    if (e.shiftKey) {
      state.marqueeBox = { startX: wpt.x, startY: wpt.y, endX: wpt.x, endY: wpt.y };
      return;
    }
    const hadSelection = state.selectedNodeIds.size > 0 || state.selectedEdgeIds.size > 0;
    setSingleNodeSelection(null);
    state.panning = { startX: sx - p.pan.x, startY: sy - p.pan.y };
    if (hadSelection) renderFormatPanel();
  }
});

function openInlineEditorForSelection() {
  const p = page();
  const edge = p.edges.find((ed) => ed.id === state.selectedEdgeId);
  if (edge) {
    const geo = getEdgeGeometry(edge);
    if (!geo) return;
    const spt = worldToScreen(geo.midPoint.x - 60, geo.midPoint.y - 14);
    inlineEditor.style.display = "block";
    inlineEditor.style.left = `${spt.x}px`;
    inlineEditor.style.top = `${spt.y}px`;
    inlineEditor.style.width = `120px`;
    inlineEditor.value = edge.label || "";
    inlineEditor.focus();
    inlineEditor.select();
    inlineEditor.onkeydown = (ev) => {
      if (ev.key === "Enter") {
        saveSnapshot();
        edge.label = inlineEditor.value;
        edge._ver = -1;
        inlineEditor.style.display = "none";
        renderFormatPanel();
      } else if (ev.key === "Escape") {
        inlineEditor.style.display = "none";
      }
    };
    return;
  }

  const node = nodeMap.get(state.selectedNodeId);
  if (!node) return;
  const spt = worldToScreen(node.x, node.y);
  inlineEditor.style.display = "block";
  inlineEditor.style.left = `${spt.x}px`;
  inlineEditor.style.top = `${spt.y}px`;
  inlineEditor.style.width = `${node.w * p.zoom}px`;
  inlineEditor.value = node.title;
  inlineEditor.focus();
  inlineEditor.select();
  inlineEditor.onkeydown = (ev) => {
    if (ev.key === "Enter") {
      saveSnapshot();
      node.title = inlineEditor.value;
      inlineEditor.style.display = "none";
      renderFormatPanel();
    } else if (ev.key === "Escape") {
      inlineEditor.style.display = "none";
    }
  };
}

canvas.addEventListener("dblclick", (e) => {
  const wpt = screenToWorld(e.clientX - canvasRect.left, e.clientY - canvasRect.top);

  const hitEdge = hitTestEdge(wpt);
  if (hitEdge) {
    setSingleEdgeSelection(hitEdge.id);
    const geo = getEdgeGeometry(hitEdge);
    if (geo && hitEdge.bend && Math.hypot(wpt.x - geo.midPoint.x, wpt.y - geo.midPoint.y) <= 16) {
      saveSnapshot();
      hitEdge.bend = null;
      hitEdge._ver = -1;
      renderFormatPanel();
      return;
    }
    renderFormatPanel();
    openInlineEditorForSelection();
    return;
  }

  const hit = page().nodes.find((n) => wpt.x >= n.x && wpt.x <= n.x + n.w && wpt.y >= n.y && wpt.y <= n.y + n.h);
  if (!hit) return;
  setSingleNodeSelection(hit.id);
  renderFormatPanel();
  openInlineEditorForSelection();
});

window.addEventListener(
  "mousemove",
  (e) => {
    const sx = e.clientX - canvasRect.left;
    const sy = e.clientY - canvasRect.top;
    const p = page();
    const wx = (sx - p.pan.x) / p.zoom;
    const wy = (sy - p.pan.y) / p.zoom;
    state.mouseWorld.x = wx;
    state.mouseWorld.y = wy;

    // 0. Pointer Drag-and-Drop of Stencil from Left Panel
    if (state.draggingStencil) {
      const dist = Math.hypot(e.clientX - state.draggingStencil.startX, e.clientY - state.draggingStencil.startY);
      if (dist > 4) state.draggingStencil.active = true;
      if (state.draggingStencil.active) {
        const over = sx >= 0 && sx <= canvasRect.width && sy >= 0 && sy <= canvasRect.height;
        state.draggingStencil.overCanvas = over;
        if (over) {
          const item = state.draggingStencil.item;
          const ghost = {
            id: -999,
            x: state.snapToGrid ? Math.round((wx - item.w * 0.5) / state.gridSize) * state.gridSize : wx - item.w * 0.5,
            y: state.snapToGrid ? Math.round((wy - item.h * 0.5) / state.gridSize) * state.gridSize : wy - item.h * 0.5,
            w: item.w,
            h: item.h,
          };
          state.smartGuides = computeSmartGuides(ghost, true);
          state.draggingStencil.ghostX = ghost.x;
          state.draggingStencil.ghostY = ghost.y;
          setCanvasCursor("copy");
        } else {
          state.smartGuides = [];
        }
      }
      return;
    }

    if (state.marqueeBox) {
      state.marqueeBox.endX = wx;
      state.marqueeBox.endY = wy;
      const minX = Math.min(state.marqueeBox.startX, wx);
      const maxX = Math.max(state.marqueeBox.startX, wx);
      const minY = Math.min(state.marqueeBox.startY, wy);
      const maxY = Math.max(state.marqueeBox.startY, wy);

      state.selectedNodeIds.clear();
      state.selectedEdgeIds.clear();
      for (const n of p.nodes) {
        if (n.x + n.w >= minX && n.x <= maxX && n.y + n.h >= minY && n.y <= maxY) {
          state.selectedNodeIds.add(n.id);
        }
      }
      for (const ed of p.edges) {
        if (state.selectedNodeIds.has(ed.source) && state.selectedNodeIds.has(ed.target)) {
          state.selectedEdgeIds.add(ed.id);
        }
      }
      syncPrimarySelectionFromSets();
      setCanvasCursor("crosshair");
      return;
    }

    if (state.connectingFrom) {
      state.snapTargetPort = findSnapTargetPort(state.mouseWorld, state.connectingFrom.node.id);
      setCanvasCursor("crosshair");
      return;
    }

    if (state.reconnectingEdge) {
      const { edge, end } = state.reconnectingEdge;
      const excludeId = end === "target" ? edge.source : edge.target;
      state.snapTargetPort = findSnapTargetPort(state.mouseWorld, excludeId);
      setCanvasCursor("crosshair");
      return;
    }

    if (state.draggingEdgeBend) {
      state.draggingEdgeBend.bend = {
        x: state.snapToGrid ? Math.round(wx / state.gridSize) * state.gridSize : wx,
        y: state.snapToGrid ? Math.round(wy / state.gridSize) * state.gridSize : wy,
      };
      state.draggingEdgeBend._ver = -1;
      setCanvasCursor("grabbing");
      return;
    }

    if (state.resizingNode) {
      const { node, aspect } = state.resizingNode;
      const rawW = Math.max(68, wx - node.x);
      node.w = state.snapToGrid ? Math.round(rawW / state.gridSize) * state.gridSize : rawW;
      if (e.shiftKey) {
        node.h = Math.max(48, node.w / aspect);
      } else {
        const rawH = Math.max(48, wy - node.y);
        node.h = state.snapToGrid ? Math.round(rawH / state.gridSize) * state.gridSize : rawH;
      }
      invalidateNodeEdges(node.id);
      setCanvasCursor("nwse-resize");
      return;
    }

    if (state.draggingNode) {
      const { leadNode, offsetX, offsetY, initialPositions } = state.draggingNode;
      const leadStart = initialPositions.get(leadNode.id) || { x: leadNode.x, y: leadNode.y };
      const rawX = wx - offsetX;
      const rawY = wy - offsetY;
      leadNode.x = state.snapToGrid ? Math.round(rawX / state.gridSize) * state.gridSize : rawX;
      leadNode.y = state.snapToGrid ? Math.round(rawY / state.gridSize) * state.gridSize : rawY;
      state.smartGuides = computeSmartGuides(leadNode);

      const dx = leadNode.x - leadStart.x;
      const dy = leadNode.y - leadStart.y;
      for (const [nid, initPos] of initialPositions.entries()) {
        if (nid === leadNode.id) continue;
        const other = nodeMap.get(nid);
        if (other) {
          other.x = initPos.x + dx;
          other.y = initPos.y + dy;
        }
      }
      invalidateSelectedNodesEdges();
      setCanvasCursor("grabbing");
      return;
    }

    if (state.panning) {
      p.pan.x = sx - state.panning.startX;
      p.pan.y = sy - state.panning.startY;
      state.minimapDirty = true;
      setCanvasCursor("grabbing");
      return;
    }

    state.hoveredPort = null;
    const portHoverRadius = 15 / Math.max(0.4, p.zoom);
    for (let i = 0; i < p.nodes.length; i++) {
      const n = p.nodes[i];
      if (
        wx < n.x - portHoverRadius ||
        wx > n.x + n.w + portHoverRadius ||
        wy < n.y - portHoverRadius ||
        wy > n.y + n.h + portHoverRadius
      ) {
        continue;
      }
      for (const port of getNodePorts(n)) {
        if (Math.hypot(wx - port.x, wy - port.y) <= portHoverRadius) {
          state.hoveredPort = { nodeId: n.id, side: port.side, x: port.x, y: port.y };
          break;
        }
      }
      if (state.hoveredPort) break;
    }
    setCanvasCursor(state.hoveredPort ? "crosshair" : "default");
  },
  { passive: true }
);

window.addEventListener("mouseup", (e) => {
  state.mouseWorld = screenToWorld(e.clientX - canvasRect.left, e.clientY - canvasRect.top);
  const p = page();

  // Complete Drag-and-Drop or Click from Left Stencil Palette
  if (state.draggingStencil) {
    const st = state.draggingStencil;
    state.draggingStencil = null;
    state.smartGuides = [];
    if (st.active && st.overCanvas) {
      createNodeFromStencil(st.item, st.ghostX, st.ghostY);
    } else if (!st.active) {
      addStencilToCanvasCenter(st.item);
    }
    setCanvasCursor("default");
    return;
  }

  if (state.marqueeBox) {
    state.marqueeBox = null;
    renderFormatPanel();
  }

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
        width: 1.8,
        bend: null,
      };
      p.edges.push(newEdge);
      invalidateGeometry();
      setSingleEdgeSelection(newEdge.id);
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
      edge._ver = -1;
      renderFormatPanel();
    }
  }

  if (state.draggingNode) {
    for (const nid of state.selectedNodeIds) {
      const n = nodeMap.get(nid);
      if (n) {
        n.x = Math.round(n.x);
        n.y = Math.round(n.y);
      }
    }
    invalidateSelectedNodesEdges();
  }
  if (state.resizingNode) {
    const n = state.resizingNode.node;
    n.w = Math.round(n.w);
    n.h = Math.round(n.h);
    invalidateNodeEdges(n.id);
  }

  state.draggingNode = null;
  state.resizingNode = null;
  state.draggingEdgeBend = null;
  state.reconnectingEdge = null;
  state.panning = null;
  state.connectingFrom = null;
  state.snapTargetPort = null;
  state.smartGuides = [];
  setCanvasCursor("default");
});

canvas.addEventListener(
  "wheel",
  (e) => {
    e.preventDefault();
    const cx = e.clientX - canvasRect.left;
    const cy = e.clientY - canvasRect.top;
    const factor = Math.exp(-e.deltaY * 0.0014);
    setZoomImmediate(page().zoom * factor, cx, cy);
  },
  { passive: false }
);

// ============================================================================
// MINIMALIST MONOLINE VECTOR SYMBOL SYSTEM (Main Canvas + Palette Previews)
// ============================================================================
function drawMinimalistBadgeGlyph(ctx, shape, cx, cy, s, strokeCol) {
  ctx.save();
  ctx.strokeStyle = strokeCol;
  ctx.lineWidth = 1.65;
  ctx.lineCap = "round";
  ctx.lineJoin = "round";
  ctx.beginPath();

  if (shape === "aws_lambda") {
    // Minimalist vector Lambda (λ)
    ctx.moveTo(cx - s * 0.45, cy - s * 0.65);
    ctx.lineTo(cx - s * 0.1, cy - s * 0.65);
    ctx.lineTo(cx + s * 0.5, cy + s * 0.65);
    ctx.moveTo(cx + s * 0.08, cy - s * 0.1);
    ctx.lineTo(cx - s * 0.48, cy + s * 0.65);
  } else if (shape === "aws_s3") {
    // Minimalist isometric storage cube / bucket
    ctx.moveTo(cx, cy - s * 0.7);
    ctx.lineTo(cx + s * 0.65, cy - s * 0.35);
    ctx.lineTo(cx + s * 0.65, cy + s * 0.35);
    ctx.lineTo(cx, cy + s * 0.7);
    ctx.lineTo(cx - s * 0.65, cy + s * 0.35);
    ctx.lineTo(cx - s * 0.65, cy - s * 0.35);
    ctx.closePath();
    ctx.moveTo(cx - s * 0.65, cy - s * 0.35);
    ctx.lineTo(cx, cy);
    ctx.lineTo(cx + s * 0.65, cy - s * 0.35);
    ctx.moveTo(cx, cy);
    ctx.lineTo(cx, cy + s * 0.7);
  } else if (shape === "k8s_pod") {
    // Minimalist Kubernetes helm wheel / pod hexagon
    for (let i = 0; i < 6; i++) {
      const a = (i / 6) * Math.PI * 2 - Math.PI * 0.5;
      const px = cx + Math.cos(a) * s * 0.7;
      const py = cy + Math.sin(a) * s * 0.7;
      if (i === 0) ctx.moveTo(px, py);
      else ctx.lineTo(px, py);
    }
    ctx.closePath();
    ctx.moveTo(cx + s * 0.28, cy);
    ctx.arc(cx, cy, s * 0.28, 0, Math.PI * 2);
  } else if (shape === "server") {
    // Minimalist stacked dual server blades (Lucide server style)
    ctx.roundRect(cx - s * 0.7, cy - s * 0.65, s * 1.4, s * 0.52, 2);
    ctx.roundRect(cx - s * 0.7, cy + s * 0.13, s * 1.4, s * 0.52, 2);
    ctx.moveTo(cx - s * 0.4, cy - s * 0.39);
    ctx.lineTo(cx - s * 0.35, cy - s * 0.39);
    ctx.moveTo(cx - s * 0.4, cy + s * 0.39);
    ctx.lineTo(cx - s * 0.35, cy + s * 0.39);
  } else if (shape === "firewall") {
    // Minimalist security shield-grid glyph
    ctx.moveTo(cx, cy - s * 0.72);
    ctx.lineTo(cx + s * 0.62, cy - s * 0.42);
    ctx.bezierCurveTo(cx + s * 0.62, cy + s * 0.2, cx + s * 0.15, cy + s * 0.6, cx, cy + s * 0.72);
    ctx.bezierCurveTo(cx - s * 0.15, cy + s * 0.6, cx - s * 0.62, cy + s * 0.2, cx - s * 0.62, cy - s * 0.42);
    ctx.closePath();
    ctx.moveTo(cx, cy - s * 0.3);
    ctx.lineTo(cx, cy + s * 0.32);
    ctx.moveTo(cx - s * 0.28, cy);
    ctx.lineTo(cx + s * 0.28, cy);
  } else if (shape === "uml_component") {
    // Minimalist UML component glyph (box + 2 left pins)
    ctx.roundRect(cx - s * 0.45, cy - s * 0.65, s * 1.05, s * 1.3, 2);
    ctx.moveTo(cx - s * 0.7, cy - s * 0.28);
    ctx.lineTo(cx - s * 0.2, cy - s * 0.28);
    ctx.moveTo(cx - s * 0.7, cy + s * 0.28);
    ctx.lineTo(cx - s * 0.2, cy + s * 0.28);
  }
  ctx.stroke();
  ctx.restore();
}

function drawNodeShapeGeometry(ctx, n, isMiniPreview = false) {
  const { x, y, w, h, shape } = n;
  const strokeCol = ctx.strokeStyle;
  const fillCol = ctx.fillStyle;

  ctx.beginPath();

  // 1. Minimalist Diamond & BPMN Gateways
  if (shape === "diamond" || shape === "gateway_xor" || shape === "gateway_parallel") {
    const cx = x + w * 0.5;
    const cy = y + h * 0.5;
    ctx.moveTo(cx, y);
    ctx.lineTo(x + w, cy);
    ctx.lineTo(cx, y + h);
    ctx.lineTo(x, cy);
    ctx.closePath();
    ctx.fill();
    ctx.stroke();

    if (shape === "gateway_xor" || shape === "gateway_parallel") {
      ctx.save();
      ctx.lineWidth = isMiniPreview ? 1.5 : 2.0;
      ctx.lineCap = "round";
      const s = Math.min(w, h) * 0.17;
      ctx.beginPath();
      if (shape === "gateway_xor") {
        ctx.moveTo(cx - s, cy - s);
        ctx.lineTo(cx + s, cy + s);
        ctx.moveTo(cx + s, cy - s);
        ctx.lineTo(cx - s, cy + s);
      } else {
        ctx.moveTo(cx - s * 1.2, cy);
        ctx.lineTo(cx + s * 1.2, cy);
        ctx.moveTo(cx, cy - s * 1.2);
        ctx.lineTo(cx, cy + s * 1.2);
      }
      ctx.stroke();
      ctx.restore();
    }
  }
  // 2. Minimalist Capsule Pill
  else if (shape === "rounded") {
    ctx.roundRect(x, y, w, h, Math.min(w, h) * 0.5);
    ctx.fill();
    ctx.stroke();
  }
  // 3. Minimalist Circle & Timer Event
  else if (shape === "circle" || shape === "timer") {
    const r = Math.min(w, h) * 0.5;
    const cx = x + w * 0.5;
    const cy = y + h * 0.5;
    ctx.arc(cx, cy, r, 0, Math.PI * 2);
    ctx.fill();
    ctx.stroke();

    if (shape === "timer") {
      ctx.beginPath();
      ctx.arc(cx, cy, r * 0.76, 0, Math.PI * 2);
      ctx.moveTo(cx, cy - r * 0.45);
      ctx.lineTo(cx, cy);
      ctx.lineTo(cx + r * 0.34, cy + r * 0.15);
      ctx.stroke();
    }
  }
  // 4. Minimalist Monoline Vertical Database Cylinder
  else if (shape === "cylinder") {
    const ry = Math.min(11, h * 0.14);
    const rx = w * 0.5;
    const cx = x + rx;

    ctx.ellipse(cx, y + ry, rx, ry, 0, Math.PI, Math.PI * 2, false);
    ctx.lineTo(x + w, y + h - ry);
    ctx.ellipse(cx, y + h - ry, rx, ry, 0, 0, Math.PI, false);
    ctx.closePath();
    ctx.fill();
    ctx.stroke();

    // Crisp single elliptical top rim
    ctx.beginPath();
    ctx.ellipse(cx, y + ry, rx, ry, 0, 0, Math.PI * 2);
    ctx.stroke();
  }
  // 5. Minimalist Monoline Horizontal Cylinder (Kafka / Queue)
  else if (shape === "queue") {
    const rx = Math.min(11, w * 0.075);
    const ry = h * 0.5;
    const cy = y + ry;

    // Clean horizontal cylinder silhouette
    ctx.ellipse(x + rx, cy, rx, ry, 0, Math.PI * 0.5, Math.PI * 1.5, false);
    ctx.lineTo(x + w - rx, y);
    ctx.ellipse(x + w - rx, cy, rx, ry, 0, -Math.PI * 0.5, Math.PI * 0.5, false);
    ctx.closePath();
    ctx.fill();
    ctx.stroke();

    // Single clean monoline elliptical right cap
    ctx.beginPath();
    ctx.ellipse(x + w - rx, cy, rx, ry, 0, 0, Math.PI * 2);
    ctx.stroke();
  }
  // 6. Minimalist Parallelogram
  else if (shape === "parallelogram") {
    const skew = Math.min(16, w * 0.14);
    ctx.moveTo(x + skew, y);
    ctx.lineTo(x + w, y);
    ctx.lineTo(x + w - skew, y + h);
    ctx.lineTo(x, y + h);
    ctx.closePath();
    ctx.fill();
    ctx.stroke();
  }
  // 7. Minimalist Hexagon
  else if (shape === "hexagon") {
    const cut = Math.min(18, w * 0.13);
    ctx.moveTo(x + cut, y);
    ctx.lineTo(x + w - cut, y);
    ctx.lineTo(x + w, y + h * 0.5);
    ctx.lineTo(x + w - cut, y + h);
    ctx.lineTo(x + cut, y + h);
    ctx.lineTo(x, y + h * 0.5);
    ctx.closePath();
    ctx.fill();
    ctx.stroke();
  }
  // 8. Minimalist Document
  else if (shape === "document") {
    const wave = Math.min(10, h * 0.13);
    ctx.moveTo(x, y);
    ctx.lineTo(x + w, y);
    ctx.lineTo(x + w, y + h - wave);
    ctx.bezierCurveTo(
      x + w * 0.7,
      y + h + wave * 0.75,
      x + w * 0.3,
      y + h - wave * 1.9,
      x,
      y + h - wave * 0.5
    );
    ctx.closePath();
    ctx.fill();
    ctx.stroke();
  }
  // 9. Minimalist Note
  else if (shape === "note") {
    const fold = Math.min(15, w * 0.16, h * 0.2);
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
  }
  // 10. Minimalist Geometric Cloud (Flat base + clean smooth arcs)
  else if (shape === "cloud") {
    const l = x + w * 0.08;
    const r = x + w * 0.92;
    const t = y + h * 0.16;
    const b = y + h * 0.86;
    const cw = r - l;
    const ch = b - t;

    ctx.moveTo(l + cw * 0.18, b);
    ctx.lineTo(l + cw * 0.82, b);
    ctx.bezierCurveTo(r + cw * 0.04, b, r + cw * 0.04, t + ch * 0.42, l + cw * 0.82, t + ch * 0.42);
    ctx.bezierCurveTo(l + cw * 0.82, t - ch * 0.05, l + cw * 0.36, t - ch * 0.08, l + cw * 0.32, t + ch * 0.32);
    ctx.bezierCurveTo(l - cw * 0.04, t + ch * 0.32, l - cw * 0.04, b, l + cw * 0.18, b);
    ctx.closePath();
    ctx.fill();
    ctx.stroke();
  }
  // 11. Minimalist User / Actor Glyph (Lucide User style: head circle + smooth shoulder arc)
  else if (shape === "actor") {
    const cx = x + w * 0.5;
    const cy = y + h * 0.5;
    const r = Math.min(w, h) * 0.46;

    // Outer minimalist avatar circle
    ctx.arc(cx, cy, r, 0, Math.PI * 2);
    ctx.fill();
    ctx.stroke();

    // Inner head circle + shoulder arc
    const headR = r * 0.28;
    ctx.beginPath();
    ctx.arc(cx, cy - r * 0.2, headR, 0, Math.PI * 2);
    ctx.moveTo(cx - r * 0.54, cy + r * 0.62);
    ctx.bezierCurveTo(cx - r * 0.45, cy + r * 0.16, cx + r * 0.45, cy + r * 0.16, cx + r * 0.54, cy + r * 0.62);
    ctx.stroke();
  }
  // 12. Minimalist UML Package
  else if (shape === "uml_package") {
    const tabW = Math.min(64, w * 0.4);
    const tabH = Math.min(14, h * 0.2);
    ctx.roundRect(x, y, tabW, tabH, [4, 4, 0, 0]);
    ctx.fill();
    ctx.stroke();

    ctx.beginPath();
    ctx.roundRect(x, y + tabH, w, h - tabH, [0, 7, 7, 7]);
    ctx.fill();
    ctx.stroke();
  }
  // 13. Minimalist UML Class & Interface
  else if (shape === "uml_class" || shape === "uml_interface") {
    const headerH = isMiniPreview ? h * 0.4 : 30;
    ctx.roundRect(x, y, w, h, isMiniPreview ? 3 : 7);
    ctx.fill();
    ctx.stroke();

    ctx.beginPath();
    ctx.moveTo(x, y + headerH);
    ctx.lineTo(x + w, y + headerH);
    ctx.stroke();
  }
  // 14. Minimalist Auth Shield
  else if (shape === "shield") {
    const cx = x + w * 0.5;
    ctx.moveTo(cx, y);
    ctx.lineTo(x + w, y + h * 0.2);
    ctx.bezierCurveTo(x + w, y + h * 0.66, cx + w * 0.1, y + h * 0.9, cx, y + h);
    ctx.bezierCurveTo(cx - w * 0.1, y + h * 0.9, x, y + h * 0.66, x, y + h * 0.2);
    ctx.closePath();
    ctx.fill();
    ctx.stroke();
  }
  // 15. Minimalist Browser UI Wireframe
  else if (shape === "browser") {
    const headerH = isMiniPreview ? h * 0.3 : 22;
    ctx.roundRect(x, y, w, h, isMiniPreview ? 3 : 8);
    ctx.fill();
    ctx.stroke();
    ctx.beginPath();
    ctx.moveTo(x, y + headerH);
    ctx.lineTo(x + w, y + headerH);
    ctx.stroke();
    if (!isMiniPreview) {
      for (let i = 0; i < 3; i++) {
        ctx.beginPath();
        ctx.arc(x + 12 + i * 9, y + 11, 2.4, 0, Math.PI * 2);
        ctx.stroke();
      }
    }
  }
  // 16. Minimalist Mobile UI Wireframe
  else if (shape === "mobile") {
    ctx.roundRect(x, y, w, h, isMiniPreview ? 5 : 14);
    ctx.fill();
    ctx.stroke();
    ctx.beginPath();
    ctx.moveTo(x + w * 0.38, y + (isMiniPreview ? 3.5 : 8));
    ctx.lineTo(x + w * 0.62, y + (isMiniPreview ? 3.5 : 8));
    ctx.moveTo(x + w * 0.35, y + h - (isMiniPreview ? 3.5 : 8));
    ctx.lineTo(x + w * 0.65, y + h - (isMiniPreview ? 3.5 : 8));
    ctx.stroke();
  }
  // 17. Minimalist Architecture Cards with Left Monoline Badge (AWS Lambda, S3, K8s, Server, Firewall, UML Component)
  else if (
    shape === "aws_lambda" ||
    shape === "aws_s3" ||
    shape === "k8s_pod" ||
    shape === "server" ||
    shape === "firewall" ||
    shape === "uml_component"
  ) {
    ctx.roundRect(x, y, w, h, isMiniPreview ? 3 : 8);
    ctx.fill();
    ctx.stroke();

    if (isMiniPreview) {
      drawMinimalistBadgeGlyph(ctx, shape, x + w * 0.5, y + h * 0.5, Math.min(w, h) * 0.42, strokeCol);
    } else {
      const bx = x + 26;
      const by = y + h * 0.5;
      ctx.beginPath();
      ctx.roundRect(bx - 13, by - 13, 26, 26, 6);
      ctx.fillStyle = "rgba(148, 163, 184, 0.08)";
      ctx.fill();
      ctx.stroke();
      ctx.fillStyle = fillCol;
      drawMinimalistBadgeGlyph(ctx, shape, bx, by, 9, strokeCol);
    }
  }
  // 18. Standard Minimalist Process Card
  else {
    ctx.roundRect(x, y, w, h, isMiniPreview ? 3 : 8);
    ctx.fill();
    ctx.stroke();
  }
}

function getNodeTextPlacement(n) {
  const { x, y, w, h, shape } = n;
  const cx = x + w * 0.5;

  if (shape === "actor" || shape === "gateway_xor" || shape === "gateway_parallel" || shape === "timer") {
    return {
      cx,
      titleY: y + h + 15,
      bodyY: y + h + 29,
      external: true,
    };
  }
  if (shape === "cylinder") {
    const bodyCenterY = y + 16 + (h - 16) * 0.5;
    return { cx, titleY: bodyCenterY - (n.body ? 5 : -4), bodyY: bodyCenterY + 12, external: false };
  }
  if (shape === "uml_package") {
    const bodyCenterY = y + 14 + (h - 14) * 0.5;
    return { cx, titleY: bodyCenterY - (n.body ? 5 : -4), bodyY: bodyCenterY + 12, external: false };
  }
  if (shape === "browser") {
    const bodyCenterY = y + 22 + (h - 22) * 0.5;
    return { cx, titleY: bodyCenterY - (n.body ? 5 : -4), bodyY: bodyCenterY + 12, external: false };
  }
  if (shape === "uml_class" || shape === "uml_interface") {
    return {
      cx,
      titleY: y + 19,
      bodyY: y + 30 + (h - 30) * 0.5 + 4,
      external: false,
    };
  }
  if (shape === "queue") {
    return {
      cx: x + w * 0.5 - 4,
      titleY: y + h * 0.5 - (n.body ? 5 : -4),
      bodyY: y + h * 0.5 + 12,
      external: false,
    };
  }
  if (
    shape === "aws_lambda" ||
    shape === "aws_s3" ||
    shape === "k8s_pod" ||
    shape === "server" ||
    shape === "firewall" ||
    shape === "uml_component"
  ) {
    return {
      cx: x + 44 + (w - 44) * 0.5,
      titleY: y + h * 0.5 - (n.body ? 5 : -4),
      bodyY: y + h * 0.5 + 12,
      external: false,
    };
  }
  return {
    cx,
    titleY: y + h * 0.5 - (n.body ? 5 : -4),
    bodyY: y + h * 0.5 + 12,
    external: false,
  };
}

// Render Left Stencil Palette with Drag-and-Drop + Minimalist Monoline Thumbnails
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
      card.draggable = true;
      card.title = `Drag "${item.label}" onto canvas or click to add`;

      const thumb = document.createElement("canvas");
      thumb.width = 30;
      thumb.height = 22;
      thumb.style.flexShrink = "0";
      const tctx = thumb.getContext("2d");
      tctx.fillStyle = "rgba(56, 189, 248, 0.08)";
      tctx.strokeStyle = "#38bdf8";
      tctx.lineWidth = 1.35;
      drawNodeShapeGeometry(tctx, { x: 2.5, y: 2.5, w: 25, h: 17, shape: item.shape }, true);

      const labelSpan = document.createElement("span");
      labelSpan.textContent = item.label;
      labelSpan.style.overflow = "hidden";
      labelSpan.style.textOverflow = "ellipsis";

      card.appendChild(thumb);
      card.appendChild(labelSpan);

      // Pointer drag-and-drop initiation
      card.addEventListener("mousedown", (e) => {
        if (e.button !== 0) return;
        state.draggingStencil = {
          item,
          startX: e.clientX,
          startY: e.clientY,
          active: false,
          overCanvas: false,
          ghostX: 0,
          ghostY: 0,
        };
      });

      // HTML5 dragstart fallback
      card.addEventListener("dragstart", (e) => {
        state.draggingStencil = {
          item,
          startX: e.clientX,
          startY: e.clientY,
          active: true,
          overCanvas: false,
          ghostX: 0,
          ghostY: 0,
        };
        if (e.dataTransfer) {
          e.dataTransfer.setData("text/plain", item.id);
          e.dataTransfer.effectAllowed = "copy";
        }
      });

      card.addEventListener("dragend", () => {
        state.draggingStencil = null;
        state.smartGuides = [];
      });

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
  setSingleNodeSelection(null);
  invalidateGeometry();
  renderPageTabs();
  renderFormatPanel();
  updateZoomLabel();
  showToast(`Created Page-${id}`);
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
      setSingleNodeSelection(p.nodes[0]?.id || null);
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

// ============================================================================
// CLIPBOARD, MULTI-SELECTION & EDIT OPERATIONS
// ============================================================================
function selectAll() {
  const p = page();
  state.selectedNodeIds.clear();
  state.selectedEdgeIds.clear();
  for (const n of p.nodes) state.selectedNodeIds.add(n.id);
  for (const e of p.edges) state.selectedEdgeIds.add(e.id);
  syncPrimarySelectionFromSets();
  state.minimapDirty = true;
  renderFormatPanel();
  showToast(`Selected All (${p.nodes.length} nodes, ${p.edges.length} edges)`);
}

function clearSelection() {
  setSingleNodeSelection(null);
  state.minimapDirty = true;
  renderFormatPanel();
}

function copySelection() {
  const p = page();
  const nodesToCopy = p.nodes.filter((n) => state.selectedNodeIds.has(n.id));
  const edgesToCopy = p.edges.filter(
    (e) =>
      state.selectedEdgeIds.has(e.id) ||
      (state.selectedNodeIds.has(e.source) && state.selectedNodeIds.has(e.target))
  );
  if (!nodesToCopy.length && !edgesToCopy.length) {
    showToast("Nothing selected to copy");
    return;
  }
  state.clipboard = {
    nodes: nodesToCopy.map((n) => ({ ...n })),
    edges: edgesToCopy.map(({ _geo, _ver, ...rest }) => ({ ...rest, bend: rest.bend ? { ...rest.bend } : null })),
    pasteStep: 0,
  };
  showToast(`Copied ${nodesToCopy.length} node(s)${edgesToCopy.length ? ` & ${edgesToCopy.length} edge(s)` : ""}`);
}

function cutSelection() {
  if (!state.selectedNodeIds.size && !state.selectedEdgeIds.size) {
    showToast("Nothing selected to cut");
    return;
  }
  copySelection();
  deleteSelected(false);
  showToast(`Cut selection to clipboard`);
}

function pasteSelection() {
  const { nodes: clipNodes, edges: clipEdges } = state.clipboard;
  if (!clipNodes.length && !clipEdges.length) {
    showToast("Clipboard is empty");
    return;
  }
  const p = page();
  saveSnapshot();
  state.clipboard.pasteStep++;
  const offset = 28 * state.clipboard.pasteStep;

  const idMap = new Map();
  state.selectedNodeIds.clear();
  state.selectedEdgeIds.clear();

  for (const cn of clipNodes) {
    const newId = ++state.nextId;
    idMap.set(cn.id, newId);
    const cloned = {
      ...cn,
      id: newId,
      x: cn.x + offset,
      y: cn.y + offset,
      z: p.nodes.length + 1,
    };
    p.nodes.push(cloned);
    state.selectedNodeIds.add(newId);
  }

  for (const ce of clipEdges) {
    const newSrc = idMap.get(ce.source) ?? ce.source;
    const newDst = idMap.get(ce.target) ?? ce.target;
    if (p.nodes.some((n) => n.id === newSrc) && p.nodes.some((n) => n.id === newDst)) {
      const newEdgeId = ++state.nextId;
      p.edges.push({
        ...ce,
        id: newEdgeId,
        source: newSrc,
        target: newDst,
        bend: ce.bend ? { x: ce.bend.x + offset, y: ce.bend.y + offset } : null,
      });
      state.selectedEdgeIds.add(newEdgeId);
    }
  }

  syncPrimarySelectionFromSets();
  invalidateGeometry();
  renderFormatPanel();
  showToast(`Pasted ${clipNodes.length} node(s)`);
}

function duplicateSelected() {
  if (!state.selectedNodeIds.size && !state.selectedEdgeIds.size) {
    showToast("Nothing selected to duplicate");
    return;
  }
  copySelection();
  pasteSelection();
}

function deleteSelected(withToast = true) {
  const p = page();
  const delNodes = state.selectedNodeIds.size;
  const delEdges = state.selectedEdgeIds.size;
  if (!delNodes && !delEdges) return;

  saveSnapshot();
  if (delNodes > 0) {
    p.nodes = p.nodes.filter((n) => !state.selectedNodeIds.has(n.id));
    p.edges = p.edges.filter(
      (e) =>
        !state.selectedEdgeIds.has(e.id) &&
        !state.selectedNodeIds.has(e.source) &&
        !state.selectedNodeIds.has(e.target)
    );
  } else {
    p.edges = p.edges.filter((e) => !state.selectedEdgeIds.has(e.id));
  }
  setSingleNodeSelection(null);
  invalidateGeometry();
  renderFormatPanel();
  if (withToast) showToast(`Deleted ${delNodes ? `${delNodes} node(s)` : `${delEdges} edge(s)`}`);
}

function bringToFront() {
  const p = page();
  if (!state.selectedNodeIds.size) return;
  let maxZ = Math.max(0, ...p.nodes.map((x) => x.z || 0));
  for (const nid of state.selectedNodeIds) {
    const n = nodeMap.get(nid);
    if (n) n.z = ++maxZ;
  }
  p.nodes.sort((a, b) => (a.z || 0) - (b.z || 0));
  showToast("Brought to Front");
}

function sendToBack() {
  const p = page();
  if (!state.selectedNodeIds.size) return;
  let minZ = Math.min(0, ...p.nodes.map((x) => x.z || 0));
  for (const nid of state.selectedNodeIds) {
    const n = nodeMap.get(nid);
    if (n) n.z = --minZ;
  }
  p.nodes.sort((a, b) => (a.z || 0) - (b.z || 0));
  showToast("Sent to Back");
}

function undo() {
  const p = page();
  if (!p.undoStack.length) {
    showToast("Nothing to Undo");
    return;
  }
  p.redoStack.push(JSON.stringify({ nodes: p.nodes, edges: p.edges.map(({ _geo, _ver, ...r }) => r) }));
  const prev = JSON.parse(p.undoStack.pop());
  p.nodes = prev.nodes;
  p.edges = prev.edges;
  setSingleNodeSelection(p.nodes[0]?.id || null);
  invalidateGeometry();
  renderFormatPanel();
  showToast("Undo");
}

function redo() {
  const p = page();
  if (!p.redoStack.length) {
    showToast("Nothing to Redo");
    return;
  }
  p.undoStack.push(JSON.stringify({ nodes: p.nodes, edges: p.edges.map(({ _geo, _ver, ...r }) => r) }));
  const next = JSON.parse(p.redoStack.pop());
  p.nodes = next.nodes;
  p.edges = next.edges;
  setSingleNodeSelection(p.nodes[0]?.id || null);
  invalidateGeometry();
  renderFormatPanel();
  showToast("Redo");
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
  const targets = new Map();
  Object.entries(byLayer).forEach(([layerIdx, group]) => {
    group.forEach((n, sibIdx) => {
      targets.set(n.id, {
        x: 90 + Number(layerIdx) * 250,
        y: 70 + sibIdx * 135,
      });
    });
  });
  p.edges.forEach((e) => (e.bend = null));
  animateNodesToTargets(targets);
  showToast("Auto-Layout: Hierarchical");
}

function runGridLayout() {
  const p = page();
  saveSnapshot();
  const cols = Math.max(3, Math.ceil(Math.sqrt(p.nodes.length)));
  const targets = new Map();
  p.nodes.forEach((n, idx) => {
    const col = idx % cols;
    const row = Math.floor(idx / cols);
    targets.set(n.id, {
      x: 80 + col * 235,
      y: 70 + row * 140,
    });
  });
  p.edges.forEach((e) => (e.bend = null));
  animateNodesToTargets(targets);
  showToast("Auto-Layout: Grid");
}

function runRadialLayout() {
  const p = page();
  saveSnapshot();
  const count = Math.max(1, p.nodes.length);
  const radius = Math.max(180, count * 28);
  const cx = 420, cy = 280;
  const targets = new Map();
  p.nodes.forEach((n, idx) => {
    const angle = (idx / count) * Math.PI * 2;
    targets.set(n.id, {
      x: Math.round(cx + radius * Math.cos(angle) - n.w / 2),
      y: Math.round(cy + radius * Math.sin(angle) - n.h / 2),
    });
  });
  p.edges.forEach((e) => (e.bend = null));
  animateNodesToTargets(targets);
  showToast("Auto-Layout: Radial");
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
      h: 72,
      fill: "#131b2e",
      stroke: "#38bdf8",
      shape: i % 4 === 0 ? "queue" : i % 3 === 0 ? "cylinder" : "card",
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
        width: 1.8,
        bend: null,
      });
    }
  }
  setSingleNodeSelection(1);
  invalidateGeometry();
  renderFormatPanel();
  showToast(`Generated ${count}-Node Graph`);
}

document.getElementById("btn-undo").onclick = undo;
document.getElementById("btn-redo").onclick = redo;
document.getElementById("btn-copy").onclick = copySelection;
document.getElementById("btn-paste").onclick = pasteSelection;
document.getElementById("btn-duplicate").onclick = duplicateSelected;
document.getElementById("btn-select-all").onclick = selectAll;
document.getElementById("btn-delete").onclick = () => deleteSelected(true);
document.getElementById("btn-zoom-in").onclick = () => animateZoom(page().zoom * 1.25);
document.getElementById("btn-zoom-out").onclick = () => animateZoom(page().zoom / 1.25);
document.getElementById("btn-fit-view").onclick = fitToScreen;
document.getElementById("btn-layout-hier").onclick = runHierarchicalLayout;

document.getElementById("select-edge-anim").onchange = (e) => {
  state.edgeAnimPreset = e.target.value;
  for (const eid of state.selectedEdgeIds) {
    const ed = page().edges.find((x) => x.id === eid);
    if (ed) ed.anim = state.edgeAnimPreset;
  }
  renderFormatPanel();
};
document.getElementById("select-edge-kind").onchange = (e) => {
  state.edgeKindPreset = e.target.value;
  for (const eid of state.selectedEdgeIds) {
    const ed = page().edges.find((x) => x.id === eid);
    if (ed) {
      ed.kind = state.edgeKindPreset;
      ed._ver = -1;
    }
  }
  renderFormatPanel();
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
    xml += `        <mxCell id="node-${n.id}" value="${n.title}" style="shape=${n.shape};rounded=1;fillColor=${n.fill};strokeColor=${n.stroke};" vertex="1" parent="1">\n          <mxGeometry x="${Math.round(n.x)}" y="${Math.round(n.y)}" width="${Math.round(n.w)}" height="${Math.round(n.h)}" as="geometry"/>\n        </mxCell>\n`;
  }
  for (const e of p.edges) {
    xml += `        <mxCell id="edge-${e.id}" value="${e.label}" style="curved=1;rfdAnim=${e.anim};" edge="1" parent="1" source="node-${e.source}" target="node-${e.target}">\n          <mxGeometry relative="1" as="geometry"/>\n        </mxCell>\n`;
  }
  xml += `      </root>\n    </mxGraphModel>\n  </diagram>\n</mxfile>`;
  downloadFile("diagram.drawio", xml, "application/xml");
  showToast("Saved diagram.drawio");
}

function exportAnimatedSvg() {
  const p = page();
  let svg = `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 1000 600" width="1000" height="600">\n  <style>\n    @keyframes rfd-ants { to { stroke-dashoffset: -28; } }\n    .ants { stroke-dasharray: 9 5; animation: rfd-ants 0.8s linear infinite; }\n    text { font-family: Inter, sans-serif; font-size: 13px; font-weight: 600; fill: #f8fafc; }\n  </style>\n`;
  for (const e of p.edges) {
    const geo = getEdgeGeometry(e);
    if (!geo) continue;
    const d = `M ${geo.p0.x} ${geo.p0.y} C ${geo.p1.x} ${geo.p1.y}, ${geo.p2.x} ${geo.p2.y}, ${geo.p3.x} ${geo.p3.y}`;
    svg += `  <path d="${d}" fill="none" stroke="${e.color || "#818cf8"}" stroke-width="${e.width || 1.8}" class="${e.anim === "ants" ? "ants" : ""}"/>\n`;
  }
  for (const n of p.nodes) {
    svg += `  <rect x="${n.x}" y="${n.y}" width="${n.w}" height="${n.h}" rx="8" fill="${n.fill}" stroke="${n.stroke}" stroke-width="1.8"/>\n`;
    svg += `  <text x="${n.x + n.w / 2}" y="${n.y + n.h / 2}" text-anchor="middle" dominant-baseline="middle">${n.title}</text>\n`;
  }
  svg += `</svg>`;
  downloadFile("diagram-animated.svg", svg, "image/svg+xml");
  showToast("Exported diagram-animated.svg");
}

// ============================================================================
// COMMAND PALETTE (Ctrl+K) & SHORTCUTS MODAL (?)
// ============================================================================
const cmdOverlay = document.getElementById("cmd-overlay");
const cmdInput = document.getElementById("cmd-input");
const cmdList = document.getElementById("cmd-list");
const shortcutsOverlay = document.getElementById("shortcuts-overlay");

function openShortcutsModal() {
  shortcutsOverlay.classList.add("open");
}

function closeShortcutsModal() {
  shortcutsOverlay.classList.remove("open");
}

document.getElementById("btn-shortcuts").onclick = openShortcutsModal;
document.getElementById("btn-close-shortcuts").onclick = closeShortcutsModal;
shortcutsOverlay.addEventListener("click", (e) => {
  if (e.target === shortcutsOverlay) closeShortcutsModal();
});

const COMMANDS = [
  { label: "Select All Nodes & Edges", shortcut: "Ctrl+A", run: selectAll },
  { label: "Copy Selection", shortcut: "Ctrl+C", run: copySelection },
  { label: "Cut Selection", shortcut: "Ctrl+X", run: cutSelection },
  { label: "Paste Selection", shortcut: "Ctrl+V", run: pasteSelection },
  { label: "Duplicate Selection", shortcut: "Ctrl+D", run: duplicateSelected },
  { label: "Fit Diagram to Screen", shortcut: "Ctrl+1", run: fitToScreen },
  { label: "Auto-Layout: Sugiyama Hierarchical", shortcut: "Ctrl+Shift+L", run: runHierarchicalLayout },
  { label: "Auto-Layout: Uniform Grid", shortcut: "Arrange", run: runGridLayout },
  { label: "Auto-Layout: Radial Ring", shortcut: "Arrange", run: runRadialLayout },
  { label: "Theme: Dark Minimal", shortcut: "View", run: () => handleMenuAction("theme-dark") },
  { label: "Theme: Light Minimal", shortcut: "View", run: () => handleMenuAction("theme-light") },
  { label: "Theme: High Contrast", shortcut: "View", run: () => handleMenuAction("theme-contrast") },
  { label: "Export: Save .drawio XML", shortcut: "Ctrl+S", run: exportDrawioXml },
  { label: "Export: Animated SVG", shortcut: "Ctrl+E", run: exportAnimatedSvg },
  { label: "Keyboard Shortcuts Cheat-Sheet", shortcut: "?", run: openShortcutsModal },
  { label: "Benchmark: Generate 100-Node Graph", shortcut: "Extras", run: () => generateStressGraph(100) },
  ...STENCILS.map((s) => ({ label: `Add ${s.label}`, shortcut: s.category, run: () => addStencilToCanvasCenter(s) })),
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

// ============================================================================
// GLOBAL KEYBOARD SHORTCUT DISPATCHER
// ============================================================================
window.addEventListener("keydown", (e) => {
  const mod = e.ctrlKey || e.metaKey;
  const key = e.key.toLowerCase();

  if (mod && key === "k") {
    e.preventDefault();
    if (cmdOverlay.classList.contains("open")) closeCommandPalette();
    else openCommandPalette();
    return;
  }

  if (e.key === "Escape") {
    closeCommandPalette();
    closeShortcutsModal();
    inlineEditor.style.display = "none";
    state.draggingStencil = null;
    state.smartGuides = [];
    menuItems.forEach((m) => m.classList.remove("open"));
    clearSelection();
    return;
  }

  if (e.target.tagName === "INPUT" || e.target.tagName === "SELECT" || e.target.tagName === "TEXTAREA") {
    return;
  }

  if (e.key === "?" || e.key === "F1") {
    e.preventDefault();
    if (shortcutsOverlay.classList.contains("open")) closeShortcutsModal();
    else openShortcutsModal();
    return;
  }

  if (e.key === "F2" || (e.key === "Enter" && !mod)) {
    if (state.selectedNodeId || state.selectedEdgeId) {
      e.preventDefault();
      openInlineEditorForSelection();
      return;
    }
  }

  if (mod) {
    if (key === "a") {
      e.preventDefault();
      selectAll();
    } else if (key === "c") {
      e.preventDefault();
      copySelection();
    } else if (key === "x") {
      e.preventDefault();
      cutSelection();
    } else if (key === "v") {
      e.preventDefault();
      pasteSelection();
    } else if (key === "d") {
      e.preventDefault();
      duplicateSelected();
    } else if (key === "z") {
      e.preventDefault();
      if (e.shiftKey) redo();
      else undo();
    } else if (key === "y") {
      e.preventDefault();
      redo();
    } else if (key === "s") {
      e.preventDefault();
      exportDrawioXml();
    } else if (key === "e") {
      e.preventDefault();
      exportAnimatedSvg();
    } else if (key === "g") {
      e.preventDefault();
      handleMenuAction("toggle-grid");
    } else if (key === "l" && e.shiftKey) {
      e.preventDefault();
      runHierarchicalLayout();
    } else if (e.key === "=" || e.key === "+") {
      e.preventDefault();
      animateZoom(page().zoom * 1.25);
    } else if (e.key === "-") {
      e.preventDefault();
      animateZoom(page().zoom / 1.25);
    } else if (e.key === "0") {
      e.preventDefault();
      handleMenuAction("zoom-reset");
    } else if (e.key === "1") {
      e.preventDefault();
      fitToScreen();
    } else if (e.key === "]") {
      e.preventDefault();
      bringToFront();
    } else if (e.key === "[") {
      e.preventDefault();
      sendToBack();
    }
    return;
  }

  if (e.key === "Delete" || e.key === "Backspace") {
    e.preventDefault();
    deleteSelected(true);
    return;
  }

  if (["ArrowUp", "ArrowDown", "ArrowLeft", "ArrowRight"].includes(e.key)) {
    if (!state.selectedNodeIds.size) return;
    e.preventDefault();
    const step = e.shiftKey ? 20 : e.altKey ? 1 : 5;
    for (const nid of state.selectedNodeIds) {
      const n = nodeMap.get(nid);
      if (!n) continue;
      if (e.key === "ArrowUp") n.y -= step;
      if (e.key === "ArrowDown") n.y += step;
      if (e.key === "ArrowLeft") n.x -= step;
      if (e.key === "ArrowRight") n.x += step;
    }
    invalidateSelectedNodesEdges();
  }
});

// ============================================================================
// 3. MACRO-TILE OFFSCREEN GRID PATTERN & 60-144 FPS RENDERER
// ============================================================================
function drawInfiniteGridFast(p) {
  if (state.gridStyle === "off") return;
  const step = Math.round(state.gridSize * p.zoom);
  if (step < 6) return;

  const key = `${state.theme}:${state.gridStyle}:${step}`;
  if (key !== cachedGridKey) {
    cachedGridKey = key;
    const reps = Math.max(1, Math.round(240 / step));
    const macro = step * reps;

    const tile = document.createElement("canvas");
    tile.width = macro;
    tile.height = macro;
    const tctx = tile.getContext("2d");
    const color =
      state.theme === "light" ? "#cbd5e1" : state.theme === "contrast" ? "#333333" : "#1e293b";

    if (state.gridStyle === "dots") {
      tctx.fillStyle = color;
      for (let gx = 0; gx < macro; gx += step) {
        for (let gy = 0; gy < macro; gy += step) {
          tctx.fillRect(gx, gy, 1.5, 1.5);
        }
      }
    } else {
      tctx.strokeStyle = color;
      tctx.lineWidth = 1;
      tctx.beginPath();
      for (let g = 0; g < macro; g += step) {
        tctx.moveTo(0, g);
        tctx.lineTo(macro, g);
        tctx.moveTo(g, 0);
        tctx.lineTo(g, macro);
      }
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

function drawArrowhead(ctx, fromPt, toPt, color) {
  const angle = Math.atan2(toPt.y - fromPt.y, toPt.x - fromPt.x);
  const headLen = 9;
  ctx.fillStyle = color;
  ctx.beginPath();
  ctx.moveTo(toPt.x, toPt.y);
  ctx.lineTo(toPt.x - headLen * Math.cos(angle - Math.PI / 6.5), toPt.y - headLen * Math.sin(angle - Math.PI / 6.5));
  ctx.lineTo(toPt.x - headLen * Math.cos(angle + Math.PI / 6.5), toPt.y - headLen * Math.sin(angle + Math.PI / 6.5));
  ctx.closePath();
  ctx.fill();
}

let frameCounter = 0;

function renderLoop(timestamp) {
  const dt = Math.max(1, timestamp - state.lastFrameTime);
  state.fps = Math.round(state.fps * 0.92 + (1000 / dt) * 0.08);
  state.lastFrameTime = timestamp;

  const timeSec = timestamp * 0.001;
  const p = page();

  if (state.layoutAnim) {
    const elapsed = timestamp - state.layoutAnim.startTime;
    const rawT = Math.min(1, elapsed / state.layoutAnim.duration);
    const ease = 1 - Math.pow(1 - rawT, 3);
    for (const item of state.layoutAnim.entries) {
      item.node.x = item.sx + (item.tx - item.sx) * ease;
      item.node.y = item.sy + (item.ty - item.sy) * ease;
    }
    state.geometryVersion++;
    state.minimapDirty = true;
    if (rawT >= 1) {
      state.layoutAnim = null;
      renderFormatPanel();
    }
  }

  if (state.viewAnim) {
    const elapsed = timestamp - state.viewAnim.startTime;
    const rawT = Math.min(1, elapsed / state.viewAnim.duration);
    const ease = 1 - Math.pow(1 - rawT, 3);
    p.pan.x = state.viewAnim.startPanX + (state.viewAnim.targetPanX - state.viewAnim.startPanX) * ease;
    p.pan.y = state.viewAnim.startPanY + (state.viewAnim.targetPanY - state.viewAnim.startPanY) * ease;
    p.zoom = state.viewAnim.startZoom + (state.viewAnim.targetZoom - state.viewAnim.startZoom) * ease;
    state.minimapDirty = true;
    updateZoomLabel();
    if (rawT >= 1) {
      state.viewAnim = null;
    }
  }

  ctx.fillStyle = state.theme === "light" ? "#f8fafc" : state.theme === "contrast" ? "#000000" : "#090d16";
  ctx.fillRect(0, 0, canvas.width, canvas.height);

  drawInfiniteGridFast(p);

  const viewTL = screenToWorld(0, 0);
  const viewBR = screenToWorld(canvas.width, canvas.height);

  ctx.save();
  ctx.translate(p.pan.x, p.pan.y);
  ctx.scale(p.zoom, p.zoom);

  if (state.smartGuides.length) {
    ctx.strokeStyle = "#f43f5e";
    ctx.lineWidth = 1.1 / p.zoom;
    ctx.setLineDash([4 / p.zoom, 4 / p.zoom]);
    ctx.beginPath();
    for (let i = 0; i < state.smartGuides.length; i++) {
      const g = state.smartGuides[i];
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

  ctx.font = "500 11px Inter, sans-serif";
  ctx.textAlign = "center";

  for (let i = 0; i < p.edges.length; i++) {
    const e = p.edges[i];
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

    const isSelected = state.selectedEdgeIds.has(e.id) || e.id === state.selectedEdgeId;
    const strokeColor = isSelected ? "#38bdf8" : e.color || "#818cf8";

    ctx.beginPath();
    if (e.kind === "straight" || e.kind === "orthogonal" || e.kind === "smoothstep") {
      ctx.moveTo(geo.samples[0].x, geo.samples[0].y);
      for (let j = 1; j < geo.samples.length; j++) {
        ctx.lineTo(geo.samples[j].x, geo.samples[j].y);
      }
    } else {
      ctx.moveTo(geo.p0.x, geo.p0.y);
      ctx.bezierCurveTo(geo.p1.x, geo.p1.y, geo.p2.x, geo.p2.y, geo.p3.x, geo.p3.y);
    }

    ctx.strokeStyle = strokeColor;
    ctx.lineWidth = isSelected ? (e.width || 1.8) + 1.0 : e.width || 1.8;

    if (e.anim === "ants") {
      ctx.setLineDash([7, 5]);
      ctx.lineDashOffset = -timeSec * 38;
    }
    ctx.stroke();
    if (e.anim === "ants") ctx.setLineDash([]);

    const prevPt = geo.samples[Math.max(0, geo.samples.length - 2)];
    drawArrowhead(ctx, prevPt, geo.p3, strokeColor);

    if (e.anim === "pulse") {
      const speedPxPerSec = 140;
      const spacing = geo.totalLen * 0.5;
      for (let pIdx = 0; pIdx < 2; pIdx++) {
        const headDist = (timeSec * speedPxPerSec + pIdx * spacing) % geo.totalLen;
        const tail1 = getPointAtArcDistance(geo, headDist - 10);
        const tail2 = getPointAtArcDistance(geo, headDist - 5);
        const head = getPointAtArcDistance(geo, headDist);

        ctx.fillStyle = "rgba(56, 189, 248, 0.16)";
        ctx.beginPath();
        ctx.arc(tail1.x, tail1.y, 3.6, 0, Math.PI * 2);
        ctx.fill();

        ctx.fillStyle = "rgba(56, 189, 248, 0.32)";
        ctx.beginPath();
        ctx.arc(tail2.x, tail2.y, 4.6, 0, Math.PI * 2);
        ctx.fill();

        ctx.fillStyle = "#e0f2fe";
        ctx.beginPath();
        ctx.arc(head.x, head.y, 3.2, 0, Math.PI * 2);
        ctx.fill();
      }
    }

    if (e.label) {
      const textW = geo.labelWidth;
      ctx.fillStyle = state.theme === "light" ? "#ffffff" : "#0f1420";
      ctx.strokeStyle = strokeColor;
      ctx.lineWidth = 1;
      ctx.beginPath();
      ctx.roundRect(geo.midPoint.x - textW * 0.5, geo.midPoint.y - 9.5, textW, 19, 5);
      ctx.fill();
      ctx.stroke();
      ctx.fillStyle = state.theme === "light" ? "#0f172a" : "#e2e8f0";
      ctx.fillText(e.label, geo.midPoint.x, geo.midPoint.y + 3.5);
    }

    if (e.id === state.selectedEdgeId) {
      ctx.strokeStyle = "rgba(251, 191, 36, 0.4)";
      ctx.setLineDash([3, 3]);
      ctx.beginPath();
      ctx.moveTo(geo.p0.x, geo.p0.y);
      ctx.lineTo(geo.midPoint.x, geo.midPoint.y);
      ctx.lineTo(geo.p3.x, geo.p3.y);
      ctx.stroke();
      ctx.setLineDash([]);

      ctx.beginPath();
      ctx.arc(geo.midPoint.x, geo.midPoint.y, 6, 0, Math.PI * 2);
      ctx.fillStyle = "#fbbf24";
      ctx.strokeStyle = "#090d16";
      ctx.lineWidth = 1.8;
      ctx.fill();
      ctx.stroke();

      for (const ep of [geo.p0, geo.p3]) {
        ctx.beginPath();
        ctx.arc(ep.x, ep.y, 5.5, 0, Math.PI * 2);
        ctx.fillStyle = "#22d3ee";
        ctx.strokeStyle = "#090d16";
        ctx.lineWidth = 1.8;
        ctx.fill();
        ctx.stroke();
      }
    }
  }

  if (state.connectingFrom || state.reconnectingEdge) {
    let startPt = { x: 0, y: 0 };
    if (state.connectingFrom) {
      startPt = state.connectingFrom.port;
    } else if (state.reconnectingEdge) {
      const geo = getEdgeGeometry(state.reconnectingEdge.edge);
      if (geo) startPt = state.reconnectingEdge.end === "target" ? geo.p0 : geo.p3;
    }
    const endPt = state.snapTargetPort ? state.snapTargetPort.port : state.mouseWorld;
    ctx.beginPath();
    ctx.moveTo(startPt.x, startPt.y);
    ctx.lineTo(endPt.x, endPt.y);
    ctx.strokeStyle = state.snapTargetPort ? "#34d399" : "#38bdf8";
    ctx.lineWidth = 2;
    ctx.setLineDash([5, 4]);
    ctx.stroke();
    ctx.setLineDash([]);

    if (state.snapTargetPort) {
      ctx.beginPath();
      ctx.arc(state.snapTargetPort.port.x, state.snapTargetPort.port.y, 10, 0, Math.PI * 2);
      ctx.strokeStyle = "#34d399";
      ctx.lineWidth = 2.5;
      ctx.stroke();
    }
  }

  // Frustum-Culled Minimalist Nodes
  const visibleNodes = [];
  for (let i = 0; i < p.nodes.length; i++) {
    const n = p.nodes[i];
    if (
      n.x + n.w + 45 < viewTL.x ||
      n.x - 45 > viewBR.x ||
      n.y + n.h + 45 < viewTL.y ||
      n.y - 45 > viewBR.y
    ) {
      continue;
    }
    visibleNodes.push(n);

    const selected = state.selectedNodeIds.has(n.id);
    if (selected) {
      ctx.strokeStyle = "rgba(56, 189, 248, 0.22)";
      ctx.lineWidth = 4;
      ctx.strokeRect(n.x - 2.5, n.y - 2.5, n.w + 5, n.h + 5);
    }

    ctx.fillStyle = n.fill;
    ctx.strokeStyle = selected ? "#38bdf8" : n.stroke;
    ctx.lineWidth = selected ? 2.0 : 1.5;
    drawNodeShapeGeometry(ctx, n, false);

    for (const port of getNodePorts(n)) {
      const isHovered =
        state.hoveredPort && state.hoveredPort.nodeId === n.id && state.hoveredPort.side === port.side;
      ctx.beginPath();
      ctx.arc(port.x, port.y, isHovered ? 6.5 : 4, 0, Math.PI * 2);
      ctx.fillStyle = isHovered ? "#34d399" : "#090d16";
      ctx.strokeStyle = isHovered ? "#ffffff" : "#64748b";
      ctx.lineWidth = 1.6;
      ctx.fill();
      ctx.stroke();
    }
  }

  // Batched Title Text Pass
  ctx.font = "600 12.5px Inter, sans-serif";
  ctx.textAlign = "center";
  for (let i = 0; i < visibleNodes.length; i++) {
    const n = visibleNodes[i];
    const pos = getNodeTextPlacement(n);
    ctx.fillStyle =
      pos.external
        ? state.theme === "light"
          ? "#0f172a"
          : "#f1f5f9"
        : state.theme === "light" && n.fill === "#ffffff"
        ? "#0f172a"
        : "#f1f5f9";
    ctx.fillText(n.title, pos.cx, pos.titleY);
  }

  // Batched Subtitle Text Pass
  ctx.font = "400 10.5px monospace";
  ctx.fillStyle = "#8b9bb4";
  for (let i = 0; i < visibleNodes.length; i++) {
    const n = visibleNodes[i];
    if (!n.body) continue;
    const pos = getNodeTextPlacement(n);
    ctx.fillText(n.body, pos.cx, pos.bodyY);
  }

  // Single-Selected Node Resize & Quick-Connect Overlay
  const selectedNode = nodeMap.get(state.selectedNodeId);
  if (selectedNode && state.selectedNodeIds.size === 1) {
    ctx.fillStyle = "#38bdf8";
    ctx.fillRect(selectedNode.x + selectedNode.w - 4.5, selectedNode.y + selectedNode.h - 4.5, 9, 9);

    ctx.font = "600 10.5px sans-serif";
    for (const arrow of getQuickArrows(selectedNode)) {
      ctx.beginPath();
      ctx.arc(arrow.x, arrow.y, 8.5, 0, Math.PI * 2);
      ctx.fillStyle = "#0f1420";
      ctx.strokeStyle = "#38bdf8";
      ctx.lineWidth = 1.3;
      ctx.fill();
      ctx.stroke();
      ctx.fillStyle = "#38bdf8";
      ctx.fillText(arrow.label, arrow.x, arrow.y + 3.5);
    }
  }

  // Live Ghost Preview When Dragging a Stencil from the Left Panel onto the Canvas
  if (state.draggingStencil && state.draggingStencil.active && state.draggingStencil.overCanvas) {
    const { item, ghostX, ghostY } = state.draggingStencil;
    const ghostNode = {
      x: ghostX,
      y: ghostY,
      w: item.w,
      h: item.h,
      shape: item.shape,
      title: item.label,
      body: item.id.split(".").pop(),
    };
    ctx.save();
    ctx.globalAlpha = 0.8;
    ctx.fillStyle = state.theme === "light" ? "#ffffff" : "#131b2e";
    ctx.strokeStyle = "#38bdf8";
    ctx.lineWidth = 2;
    drawNodeShapeGeometry(ctx, ghostNode, false);

    ctx.setLineDash([4, 4]);
    ctx.strokeStyle = "rgba(56, 189, 248, 0.55)";
    ctx.strokeRect(ghostX - 4, ghostY - 4, item.w + 8, item.h + 8);
    ctx.setLineDash([]);

    const pos = getNodeTextPlacement(ghostNode);
    ctx.font = "600 12.5px Inter, sans-serif";
    ctx.fillStyle = state.theme === "light" ? "#0f172a" : "#f1f5f9";
    ctx.textAlign = "center";
    ctx.fillText(ghostNode.title, pos.cx, pos.titleY);
    ctx.restore();
  }

  // Rubber-Band Marquee Selection Box
  if (state.marqueeBox) {
    const mx = Math.min(state.marqueeBox.startX, state.marqueeBox.endX);
    const my = Math.min(state.marqueeBox.startY, state.marqueeBox.endY);
    const mw = Math.abs(state.marqueeBox.endX - state.marqueeBox.startX);
    const mh = Math.abs(state.marqueeBox.endY - state.marqueeBox.startY);
    ctx.fillStyle = "rgba(56, 189, 248, 0.1)";
    ctx.strokeStyle = "#38bdf8";
    ctx.lineWidth = 1.4 / p.zoom;
    ctx.setLineDash([5 / p.zoom, 3 / p.zoom]);
    ctx.fillRect(mx, my, mw, mh);
    ctx.strokeRect(mx, my, mw, mh);
    ctx.setLineDash([]);
  }

  ctx.restore();

  frameCounter++;
  if (state.minimapDirty && (!state.draggingNode || frameCounter % 3 === 0)) {
    state.minimapDirty = false;
    minimapCtx.clearRect(0, 0, minimapCanvas.width, minimapCanvas.height);
    const scale = 0.13;
    minimapCtx.save();
    minimapCtx.translate(14, 14);
    minimapCtx.scale(scale, scale);
    for (let i = 0; i < p.nodes.length; i++) {
      const n = p.nodes[i];
      minimapCtx.fillStyle = state.selectedNodeIds.has(n.id) ? "#38bdf8" : "#475569";
      minimapCtx.fillRect(n.x, n.y, n.w, n.h);
    }
    minimapCtx.strokeStyle = "#818cf8";
    minimapCtx.lineWidth = 10;
    minimapCtx.strokeRect(viewTL.x, viewTL.y, viewBR.x - viewTL.x, viewBR.y - viewTL.y);
    minimapCtx.restore();
  }

  if (frameCounter % 4 === 0) {
    syncArrangeInputsIfDirty();
  }

  if (frameCounter % 12 === 0) {
    statusBarEl.textContent = `FPS: ${state.fps} | Zoom: ${Math.round(p.zoom * 100)}% | Selected: ${state.selectedNodeIds.size + state.selectedEdgeIds.size} | Nodes: ${p.nodes.length} | Edges: ${p.edges.length}`;
  }

  requestAnimationFrame(renderLoop);
}
requestAnimationFrame(renderLoop);
