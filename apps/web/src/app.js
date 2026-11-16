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

  // Wireframes & Media
  { id: "wireframe.image", label: "Image / Media", category: "Wireframes", w: 170, h: 98, shape: "image_card" },
  { id: "wireframe.browser", label: "Browser UI", category: "Wireframes", w: 220, h: 132, shape: "browser" },
  { id: "wireframe.mobile", label: "Mobile UI", category: "Wireframes", w: 118, h: 180, shape: "mobile" },
];

// Preset Vector Templates for Custom Symbol Studio (normalized [0..1] coordinates)
const SYMBOL_STUDIO_TEMPLATES = {
  redis: {
    name: "Redis Cube",
    category: "Custom Symbols",
    w: 155,
    h: 90,
    fill: "#fdecee",
    stroke: "#c92027",
    paths: [
      {
        tool: "path",
        pts: [
          { x: 0.5, y: 0.08 },
          { x: 0.88, y: 0.28 },
          { x: 0.88, y: 0.72 },
          { x: 0.5, y: 0.92 },
          { x: 0.12, y: 0.72 },
          { x: 0.12, y: 0.28 },
          { x: 0.5, y: 0.08 },
        ],
      },
      {
        tool: "path",
        pts: [
          { x: 0.12, y: 0.28 },
          { x: 0.5, y: 0.48 },
          { x: 0.88, y: 0.28 },
        ],
      },
      {
        tool: "line",
        pts: [
          { x: 0.5, y: 0.48 },
          { x: 0.5, y: 0.92 },
        ],
      },
    ],
  },
  graphql: {
    name: "GraphQL API",
    category: "Custom Symbols",
    w: 155,
    h: 92,
    fill: "#f5edf7",
    stroke: "#7a267b",
    paths: [
      {
        tool: "path",
        pts: [
          { x: 0.5, y: 0.08 },
          { x: 0.86, y: 0.29 },
          { x: 0.86, y: 0.71 },
          { x: 0.5, y: 0.92 },
          { x: 0.14, y: 0.71 },
          { x: 0.14, y: 0.29 },
          { x: 0.5, y: 0.08 },
        ],
      },
      {
        tool: "path",
        pts: [
          { x: 0.5, y: 0.08 },
          { x: 0.86, y: 0.71 },
          { x: 0.14, y: 0.71 },
          { x: 0.5, y: 0.08 },
        ],
      },
    ],
  },
  webhook: {
    name: "Webhook Trigger",
    category: "Custom Symbols",
    w: 150,
    h: 84,
    fill: "#fff9e6",
    stroke: "#c47f00",
    paths: [
      { tool: "rect", pts: [{ x: 0.06, y: 0.08 }, { x: 0.94, y: 0.92 }] },
      {
        tool: "path",
        pts: [
          { x: 0.54, y: 0.18 },
          { x: 0.36, y: 0.52 },
          { x: 0.52, y: 0.52 },
          { x: 0.46, y: 0.82 },
          { x: 0.64, y: 0.48 },
          { x: 0.48, y: 0.48 },
          { x: 0.54, y: 0.18 },
        ],
      },
    ],
  },
  router: {
    name: "Mesh Router",
    category: "Custom Symbols",
    w: 150,
    h: 88,
    fill: "#e8f6fc",
    stroke: "#0077b5",
    paths: [
      { tool: "ellipse", pts: [{ x: 0.1, y: 0.1 }, { x: 0.9, y: 0.9 }] },
      { tool: "line", pts: [{ x: 0.25, y: 0.5 }, { x: 0.75, y: 0.5 }] },
      { tool: "line", pts: [{ x: 0.5, y: 0.22 }, { x: 0.5, y: 0.78 }] },
    ],
  },
};

const CUSTOM_STENCILS_STORAGE_KEY = "rustflow_custom_stencils_v1";
const customStencilsMap = new Map();

function loadStoredCustomStencils() {
  try {
    const raw = localStorage.getItem(CUSTOM_STENCILS_STORAGE_KEY);
    if (raw) {
      const parsed = JSON.parse(raw);
      if (Array.isArray(parsed)) return parsed;
    }
  } catch {
    // Ignore storage read errors
  }
  // Default starter custom symbols so the user immediately sees Custom Symbols in the palette
  return [
    {
      id: "custom.redis_cube",
      label: "Redis Cube",
      category: "Custom Symbols",
      w: 155,
      h: 90,
      shape: "custom.redis_cube",
      defaultFill: "#fdecee",
      defaultStroke: "#c92027",
      customPaths: SYMBOL_STUDIO_TEMPLATES.redis.paths,
      svgPath: "",
      imgUrl: "",
      isCustom: true,
    },
    {
      id: "custom.graphql_api",
      label: "GraphQL API",
      category: "Custom Symbols",
      w: 155,
      h: 92,
      shape: "custom.graphql_api",
      defaultFill: "#f5edf7",
      defaultStroke: "#7a267b",
      customPaths: SYMBOL_STUDIO_TEMPLATES.graphql.paths,
      svgPath: "",
      imgUrl: "",
      isCustom: true,
    },
  ];
}

function saveCustomStencilsToStorage() {
  try {
    const customList = STENCILS.filter((s) => s.isCustom);
    localStorage.setItem(CUSTOM_STENCILS_STORAGE_KEY, JSON.stringify(customList));
  } catch {
    // Ignore quota errors
  }
}

const ENTITY_ICONS = [
  ["none", "None"],
  ["globe", "Web / API"],
  ["cpu", "CPU / Core"],
  ["database", "Database"],
  ["server", "Server"],
  ["cloud", "Cloud"],
  ["aws_lambda", "Lambda"],
  ["aws_s3", "Storage"],
  ["k8s_pod", "K8s Pod"],
  ["firewall", "Shield"],
  ["lock", "Auth Lock"],
  ["user", "User"],
  ["code", "Code"],
  ["terminal", "Terminal"],
  ["gear", "Config"],
  ["rocket", "Deploy"],
];

// Built-in Macintosh System 8.5 Sample Graphic SVG Data URI for Image Entities
const DEFAULT_SAMPLE_IMAGE_DATA_URL =
  "data:image/svg+xml;utf8," +
  encodeURIComponent(
    `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 160 90">
      <rect width="160" height="90" fill="#e8f4fc"/>
      <rect y="62" width="160" height="28" fill="#61bb46"/>
      <circle cx="128" cy="24" r="11" fill="#fdb827" stroke="#000" stroke-width="1.5"/>
      <path d="M12 62 L52 24 L88 62 Z" fill="#8080a4" stroke="#000" stroke-width="1.5" stroke-linejoin="round"/>
      <path d="M58 62 L98 30 L138 62 Z" fill="#a3a3c2" stroke="#000" stroke-width="1.5" stroke-linejoin="round"/>
      <line x1="0" y1="62" x2="160" y2="62" stroke="#000" stroke-width="1.5"/>
    </svg>`
  );

// Fast in-memory HTMLImageElement cache for zero-overhead 60 FPS Canvas image rendering
const imageCache = new Map();
function getOrLoadImage(url) {
  if (!url) return null;
  let entry = imageCache.get(url);
  if (entry) return entry.loaded ? entry.img : null;
  const img = new Image();
  img.crossOrigin = "anonymous";
  entry = { img, loaded: false, error: false };
  imageCache.set(url, entry);
  img.onload = () => {
    entry.loaded = true;
    state.minimapDirty = true;
  };
  img.onerror = () => {
    entry.error = true;
  };
  img.src = url;
  return null;
}

const STYLE_SWATCHES = [
  // Apple 90s 6-Color Rainbow & Macintosh Platinum Swatches
  { fill: "#ffffff", stroke: "#000080" },
  { fill: "#f2faef", stroke: "#2d8a1e" },
  { fill: "#fff9e6", stroke: "#c47f00" },
  { fill: "#fff2eb", stroke: "#d95b00" },
  { fill: "#fdecee", stroke: "#c92027" },
  { fill: "#f5edf7", stroke: "#7a267b" },
  { fill: "#e8f6fc", stroke: "#0077b5" },
  { fill: "#e6e6e6", stroke: "#000000" },
  // Dark / Modern Swatches
  { fill: "#131b2e", stroke: "#38bdf8" },
  { fill: "#161632", stroke: "#818cf8" },
  { fill: "#0c2924", stroke: "#34d399" },
  { fill: "#2d1220", stroke: "#f43f5e" },
];

function createDogfoodPages() {
  return [
    {
      id: 1,
      name: "Page-1: RustFlow Architecture",
      pan: { x: 50, y: 40 },
      zoom: 1.0,
      nodes: [
        { id: 1, title: "crates/core", body: "", x: 70, y: 155, w: 185, h: 86, fill: "#ffffff", stroke: "#000080", shape: "uml_package", icon: "cpu", iconPos: "left", imgUrl: "", z: 1 },
        { id: 2, title: "crates/renderer", body: "", x: 355, y: 45, w: 185, h: 78, fill: "#e8f6fc", stroke: "#0077b5", shape: "uml_component", icon: "uml_component", iconPos: "left", imgUrl: "", z: 2 },
        { id: 3, title: "crates/layout", body: "", x: 355, y: 160, w: 185, h: 78, fill: "#f2faef", stroke: "#2d8a1e", shape: "hexagon", icon: "gear", iconPos: "left", imgUrl: "", z: 3 },
        { id: 4, title: "crates/io-formats", body: "", x: 355, y: 280, w: 185, h: 82, fill: "#fff9e6", stroke: "#c47f00", shape: "document", icon: "code", iconPos: "left", imgUrl: "", z: 4 },
        { id: 5, title: "crates/ui-bridge", body: "", x: 650, y: 155, w: 205, h: 108, fill: "#f5edf7", stroke: "#7a267b", shape: "browser", icon: "globe", iconPos: "left", imgUrl: "", z: 5 },
      ],
      edges: [
        { id: 101, source: 1, sourcePort: "right", target: 2, targetPort: "left", label: "", anim: "pulse", kind: "bezier", arrow: "one-way", color: "#0077b5", width: 1.8, bend: null },
        { id: 102, source: 1, sourcePort: "right", target: 3, targetPort: "left", label: "", anim: "ants", kind: "bezier", arrow: "two-way", color: "#2d8a1e", width: 1.8, bend: null },
        { id: 103, source: 1, sourcePort: "right", target: 4, targetPort: "left", label: "", anim: "pulse", kind: "bezier", arrow: "one-way", color: "#c47f00", width: 1.8, bend: null },
        { id: 104, source: 2, sourcePort: "right", target: 5, targetPort: "left", label: "", anim: "pulse", kind: "bezier", arrow: "one-way", color: "#0077b5", width: 1.8, bend: null },
        { id: 105, source: 3, sourcePort: "right", target: 5, targetPort: "left", label: "", anim: "ants", kind: "bezier", arrow: "one-way", color: "#2d8a1e", width: 1.8, bend: null },
        { id: 106, source: 4, sourcePort: "right", target: 5, targetPort: "left", label: "", anim: "ants", kind: "bezier", arrow: "two-way", color: "#7a267b", width: 1.8, bend: null },
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
        { id: 10, title: "Client User", body: "", x: 75, y: 132, w: 72, h: 76, fill: "#ffffff", stroke: "#000080", shape: "actor", icon: "none", iconPos: "left", imgUrl: "", z: 1 },
        { id: 11, title: "WAF Firewall", body: "", x: 235, y: 133, w: 180, h: 74, fill: "#fdecee", stroke: "#c92027", shape: "firewall", icon: "firewall", iconPos: "left", imgUrl: "", z: 2 },
        { id: 12, title: "Auth Check", body: "", x: 495, y: 130, w: 80, h: 80, fill: "#fff9e6", stroke: "#c47f00", shape: "gateway_xor", icon: "none", iconPos: "left", imgUrl: "", z: 3 },
        { id: 13, title: "Kafka Stream", body: "", x: 660, y: 80, w: 180, h: 72, fill: "#f2faef", stroke: "#2d8a1e", shape: "queue", icon: "none", iconPos: "left", imgUrl: "", z: 4 },
        { id: 14, title: "Postgres DB", body: "", x: 680, y: 210, w: 136, h: 92, fill: "#e8f6fc", stroke: "#0077b5", shape: "cylinder", icon: "database", iconPos: "left", imgUrl: "", z: 5 },
      ],
      edges: [
        { id: 201, source: 10, sourcePort: "right", target: 11, targetPort: "left", label: "", anim: "pulse", kind: "bezier", arrow: "two-way", color: "#000080", width: 1.8, bend: null },
        { id: 202, source: 11, sourcePort: "right", target: 12, targetPort: "left", label: "", anim: "ants", kind: "bezier", arrow: "one-way", color: "#c47f00", width: 1.8, bend: null },
        { id: 203, source: 12, sourcePort: "right", target: 13, targetPort: "left", label: "", anim: "pulse", kind: "bezier", arrow: "one-way", color: "#2d8a1e", width: 1.8, bend: null },
        { id: 204, source: 12, sourcePort: "bottom", target: 14, targetPort: "left", label: "", anim: "pulse", kind: "smoothstep", arrow: "two-way", color: "#0077b5", width: 1.8, bend: null },
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
  edgeArrowPreset: "one-way",
  theme: "apple90s",
  fps: 60,
  lastFrameTime: performance.now(),
  geometryVersion: 1,
  minimapDirty: true,
  arrangeDirty: false,
  layoutAnim: null,
  viewAnim: null,
};

function isLightCanvasTheme() {
  return state.theme === "apple90s" || state.theme === "light" || state.theme === "contrast";
}

function isDarkHexColor(hex) {
  if (!hex || typeof hex !== "string" || !hex.startsWith("#") || hex.length < 7) return false;
  const r = parseInt(hex.slice(1, 3), 16);
  const g = parseInt(hex.slice(3, 5), 16);
  const b = parseInt(hex.slice(5, 7), 16);
  return r * 0.299 + g * 0.587 + b * 0.114 < 128;
}

const macClockEl = document.getElementById("mac-clock");
function updateMacClock() {
  if (!macClockEl) return;
  const now = new Date();
  macClockEl.textContent = now.toLocaleTimeString([], { hour: "numeric", minute: "2-digit" });
}
updateMacClock();
setInterval(updateMacClock, 15000);

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
    case "new-empty-workspace":
      createEmptyWorkspace("Untitled Diagram");
      break;
    case "open-diagram-file":
      triggerOpenDiagramFile();
      break;
    case "open-welcome":
      openWelcomeModal();
      break;
    case "new-page":
      addNewPage();
      break;
    case "open-symbol-studio":
      openSymbolStudioModal();
      break;
    case "export-gif":
      exportAnimatedGif();
      break;
    case "export-png":
      exportPngImage();
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
        JSON.stringify(
          {
            version: "1.0.0",
            activePageId: state.activePageId,
            pages: state.pages.map((pg) => ({
              id: pg.id,
              name: pg.name,
              pan: pg.pan,
              zoom: pg.zoom,
              nodes: pg.nodes,
              edges: pg.edges.map(({ _geo, _ver, ...r }) => r),
            })),
            nodes: p.nodes,
            edges: p.edges.map(({ _geo, _ver, ...r }) => r),
          },
          null,
          2
        ),
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
    case "theme-apple90s":
    case "theme-dark":
    case "theme-light":
    case "theme-contrast":
      state.theme = action.replace("theme-", "");
      cachedGridKey = "";
      document.documentElement.setAttribute("data-theme", state.theme);
      renderPalette(document.getElementById("palette-search")?.value || "");
      renderFormatPanel();
      showToast(`Theme: ${state.theme === "apple90s" ? "Apple 90s Platinum" : state.theme}`);
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
  ["image_card", "Image / Media Card"],
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

function registerCustomStencil(item, persist = true) {
  const existingIdx = STENCILS.findIndex((s) => s.id === item.id);
  if (existingIdx >= 0) {
    STENCILS[existingIdx] = item;
  } else {
    STENCILS.unshift(item);
  }
  customStencilsMap.set(item.id, item);
  customStencilsMap.set(item.shape, item);
  if (!SHAPE_OPTIONS.some(([val]) => val === item.shape)) {
    SHAPE_OPTIONS.push([item.shape, `${item.label} (Custom)`]);
  }
  if (item.imgUrl) getOrLoadImage(item.imgUrl);
  if (persist) saveCustomStencilsToStorage();
}

function removeCustomStencil(id) {
  const idx = STENCILS.findIndex((s) => s.id === id && s.isCustom);
  if (idx >= 0) {
    const removed = STENCILS.splice(idx, 1)[0];
    customStencilsMap.delete(removed.id);
    customStencilsMap.delete(removed.shape);
    saveCustomStencilsToStorage();
    renderPalette(document.getElementById("palette-search")?.value || "");
    showToast(`Removed symbol "${removed.label}"`);
  }
}

for (const customItem of loadStoredCustomStencils()) {
  registerCustomStencil(customItem, false);
}

function getEffectiveNodeIcon(n) {
  if (n.icon && n.icon !== "none") return n.icon;
  if (n.icon === "none") return null;
  if (
    n.shape === "aws_lambda" ||
    n.shape === "aws_s3" ||
    n.shape === "k8s_pod" ||
    n.shape === "server" ||
    n.shape === "firewall" ||
    n.shape === "uml_component"
  ) {
    return n.shape;
  }
  return null;
}

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
        <label>Macintosh Appearance Theme</label>
        <select id="diag-theme">
          <option value="apple90s" ${state.theme === "apple90s" ? "selected" : ""}>Apple 90s Platinum (OS 8.5)</option>
          <option value="dark" ${state.theme === "dark" ? "selected" : ""}>Dark Platinum</option>
          <option value="light" ${state.theme === "light" ? "selected" : ""}>Light Modern</option>
          <option value="contrast" ${state.theme === "contrast" ? "selected" : ""}>System 1.0 B&amp;W CRT</option>
        </select>
      </div>
      <div class="format-section" style="border-bottom:none;">
        <label>Quick Guide</label>
        <p style="font-size:11px; color:var(--text-muted); line-height:1.6;">
          <b>Drag</b> any symbol from the left panel onto the canvas.<br/>
          <b>Shift+Drag</b> to box-select multiple items.<br/>
          <b>⌘A / ⌘C / ⌘V / ⌘D / ⌘Z</b> supported.
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
      renderPalette(document.getElementById("palette-search")?.value || "");
    };
    return;
  }

  formatTabsEl.style.display = "grid";

  if (node) {
    formatBadgeEl.textContent = `Node #${node.id}`;
    if (state.activeFormatTab === "style") {
      const currentIcon = getEffectiveNodeIcon(node) || "none";
      const currentPos = node.iconPos || "left";
      formatBodyEl.innerHTML = `
        <div class="format-section">
          <label>Minimalist Style Presets</label>
          <div class="swatch-grid" id="node-swatches"></div>
        </div>
        <div class="format-section">
          <label>Symbol Shape</label>
          <select id="insp-shape">
            ${SHAPE_OPTIONS.map(([val, lbl]) => `<option value="${val}" ${node.shape === val ? "selected" : ""}>${lbl}</option>`).join("")}
          </select>
        </div>
        <div class="format-section">
          <label>Entity Icon &amp; Image</label>
          <div class="icon-picker-grid" id="node-icon-grid"></div>
          <div class="format-row" style="margin-top:4px;">
            <button id="btn-node-upload-img" style="width:100%;">Upload Image...</button>
            <button id="btn-node-sample-img" style="width:100%;">Sample Graphic</button>
          </div>
          ${
            node.imgUrl
              ? `<div class="img-preview-box">
                   <img src="${node.imgUrl}" alt="entity-img" />
                   <span style="flex:1; overflow:hidden; text-overflow:ellipsis; white-space:nowrap;">Image Attached</span>
                   <button id="btn-node-clear-img" class="danger" style="padding:2px 6px;">Clear</button>
                 </div>`
              : ""
          }
          <input id="insp-img-url" type="text" value="${node.imgUrl && !node.imgUrl.startsWith("data:") ? node.imgUrl : ""}" placeholder="Or paste image URL (https://...)" />
          <div class="format-row">
            <span>Icon / Img Layout</span>
            <select id="insp-icon-pos">
              <option value="left" ${currentPos === "left" ? "selected" : ""}>Left Badge</option>
              <option value="top" ${currentPos === "top" ? "selected" : ""}>Top Centered</option>
              <option value="cover" ${currentPos === "cover" ? "selected" : ""}>Full Card Cover</option>
            </select>
          </div>
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

      const iconGridEl = document.getElementById("node-icon-grid");
      ENTITY_ICONS.forEach(([iconKey, iconLabel]) => {
        const btn = document.createElement("div");
        btn.className = "icon-picker-btn" + (currentIcon === iconKey ? " active" : "");
        btn.title = iconLabel;
        if (iconKey !== "none") {
          const icv = document.createElement("canvas");
          icv.width = 20;
          icv.height = 16;
          const ictx = icv.getContext("2d");
          const active = currentIcon === iconKey;
          drawMinimalistBadgeGlyph(
            ictx,
            iconKey,
            10,
            8,
            6.5,
            active ? "#ffffff" : isLightCanvasTheme() ? "#000080" : "#38bdf8"
          );
          btn.appendChild(icv);
        }
        const lbl = document.createElement("span");
        lbl.textContent = iconLabel.split(" ")[0];
        btn.appendChild(lbl);
        btn.onclick = () => {
          saveSnapshot();
          node.icon = iconKey;
          renderFormatPanel();
        };
        iconGridEl.appendChild(btn);
      });

      document.getElementById("btn-node-upload-img").onclick = () => {
        triggerEntityImageUpload(node);
      };
      document.getElementById("btn-node-sample-img").onclick = () => {
        saveSnapshot();
        node.imgUrl = DEFAULT_SAMPLE_IMAGE_DATA_URL;
        getOrLoadImage(node.imgUrl);
        renderFormatPanel();
        showToast("Attached Sample Graphic");
      };
      const clearBtn = document.getElementById("btn-node-clear-img");
      if (clearBtn) {
        clearBtn.onclick = () => {
          saveSnapshot();
          node.imgUrl = "";
          renderFormatPanel();
          showToast("Cleared Entity Image");
        };
      }
      document.getElementById("insp-img-url").onchange = (e) => {
        saveSnapshot();
        node.imgUrl = e.target.value.trim();
        if (node.imgUrl) getOrLoadImage(node.imgUrl);
        renderFormatPanel();
      };
      document.getElementById("insp-icon-pos").onchange = (e) => {
        saveSnapshot();
        node.iconPos = e.target.value;
      };
      document.getElementById("insp-shape").onchange = (e) => {
        saveSnapshot();
        node.shape = e.target.value;
        renderFormatPanel();
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
      const curArrow = edge.arrow || "one-way";
      const markers = getEdgeArrowMarkers(edge);
      formatBodyEl.innerHTML = `
        <div class="format-section">
          <label>Routing</label>
          <select id="edge-kind">
            <option value="bezier" ${edge.kind === "bezier" ? "selected" : ""}>Bezier Curve</option>
            <option value="smoothstep" ${edge.kind === "smoothstep" ? "selected" : ""}>SmoothStep</option>
            <option value="orthogonal" ${edge.kind === "orthogonal" ? "selected" : ""}>Orthogonal</option>
            <option value="straight" ${edge.kind === "straight" ? "selected" : ""}>Straight Line</option>
          </select>
          <label style="margin-top:6px;">Arrow Style</label>
          <select id="edge-arrow">
            <option value="one-way" ${curArrow === "one-way" ? "selected" : ""}>1-Way Directed (→)</option>
            <option value="two-way" ${curArrow === "two-way" ? "selected" : ""}>2-Way Bidirectional (↔)</option>
            <option value="none" ${curArrow === "none" ? "selected" : ""}>Non-Headed Line (—)</option>
            <option value="reverse" ${curArrow === "reverse" ? "selected" : ""}>1-Way Reverse (←)</option>
            <option value="open" ${curArrow === "open" ? "selected" : ""}>Open Arrow (—&gt;)</option>
            <option value="diamond" ${curArrow === "diamond" ? "selected" : ""}>UML Diamond (—◇)</option>
            <option value="circle" ${curArrow === "circle" ? "selected" : ""}>Socket Circle (—○)</option>
            <option value="custom" ${curArrow === "custom" ? "selected" : ""}>Custom Start / End</option>
          </select>
          <div class="format-row" style="margin-top:4px;">
            <span>Start</span>
            <select id="edge-start-marker">
              <option value="none" ${markers.start === "none" ? "selected" : ""}>None</option>
              <option value="classic" ${markers.start === "classic" ? "selected" : ""}>Arrow (◀)</option>
              <option value="open" ${markers.start === "open" ? "selected" : ""}>Open (&lt;)</option>
              <option value="diamond" ${markers.start === "diamond" ? "selected" : ""}>Diamond (◇)</option>
              <option value="circle" ${markers.start === "circle" ? "selected" : ""}>Circle (○)</option>
            </select>
            <span>End</span>
            <select id="edge-end-marker">
              <option value="none" ${markers.end === "none" ? "selected" : ""}>None</option>
              <option value="classic" ${markers.end === "classic" ? "selected" : ""}>Arrow (▶)</option>
              <option value="open" ${markers.end === "open" ? "selected" : ""}>Open (&gt;)</option>
              <option value="diamond" ${markers.end === "diamond" ? "selected" : ""}>Diamond (◇)</option>
              <option value="circle" ${markers.end === "circle" ? "selected" : ""}>Circle (○)</option>
            </select>
          </div>
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
      document.getElementById("edge-arrow").onchange = (e) => {
        saveSnapshot();
        edge.arrow = e.target.value;
        delete edge.sourceArrow;
        delete edge.targetArrow;
        const toolbarArrow = document.getElementById("select-edge-arrow");
        if (toolbarArrow && e.target.value !== "custom") toolbarArrow.value = e.target.value;
        renderFormatPanel();
      };
      document.getElementById("edge-start-marker").onchange = (e) => {
        saveSnapshot();
        const cur = getEdgeArrowMarkers(edge);
        edge.arrow = "custom";
        edge.sourceArrow = e.target.value;
        edge.targetArrow = cur.end;
        renderFormatPanel();
      };
      document.getElementById("edge-end-marker").onchange = (e) => {
        saveSnapshot();
        const cur = getEdgeArrowMarkers(edge);
        edge.arrow = "custom";
        edge.sourceArrow = cur.start;
        edge.targetArrow = e.target.value;
        renderFormatPanel();
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
  const lightCanvas = isLightCanvasTheme();
  const isImgCard = item.shape === "image_card";
  const hasCustomImg = Boolean(item.imgUrl);
  const newNode = {
    id: ++state.nextId,
    title: item.label,
    body: "",
    x: Math.round(worldX),
    y: Math.round(worldY),
    w: item.w,
    h: item.h,
    fill: item.defaultFill || (lightCanvas ? "#ffffff" : "#131b2e"),
    stroke:
      item.defaultStroke || (state.theme === "apple90s" ? "#000080" : state.theme === "contrast" ? "#000000" : "#38bdf8"),
    shape: item.shape,
    icon: "none",
    iconPos: isImgCard ? "cover" : hasCustomImg ? "top" : "left",
    imgUrl: item.imgUrl || (isImgCard ? DEFAULT_SAMPLE_IMAGE_DATA_URL : ""),
    customPaths: item.customPaths ? JSON.parse(JSON.stringify(item.customPaths)) : null,
    svgPath: item.svgPath || "",
    z: p.nodes.length + 1,
  };
  if (newNode.imgUrl) getOrLoadImage(newNode.imgUrl);
  p.nodes.push(newNode);
  invalidateGeometry();
  setSingleNodeSelection(newNode.id);
  renderFormatPanel();
  showToast(`Added ${item.label}`);
}

let pendingImageTargetNode = null;
const entityImageInput = document.getElementById("entity-image-input");

function triggerEntityImageUpload(targetNode = null) {
  pendingImageTargetNode = targetNode || nodeMap.get(state.selectedNodeId) || null;
  if (entityImageInput) {
    entityImageInput.value = "";
    entityImageInput.click();
  }
}

if (entityImageInput) {
  entityImageInput.addEventListener("change", (e) => {
    const file = e.target.files?.[0];
    if (!file) return;
    const reader = new FileReader();
    reader.onload = () => {
      const dataUrl = reader.result;
      if (typeof dataUrl !== "string") return;
      saveSnapshot();
      getOrLoadImage(dataUrl);
      if (pendingImageTargetNode) {
        pendingImageTargetNode.imgUrl = dataUrl;
        if (!pendingImageTargetNode.iconPos) pendingImageTargetNode.iconPos = "left";
        renderFormatPanel();
        showToast(`Attached ${file.name} to #${pendingImageTargetNode.id}`);
      } else {
        const p = page();
        const center = screenToWorld(canvas.width / 2 - 85, canvas.height / 2 - 50);
        const newNode = {
          id: ++state.nextId,
          title: file.name.replace(/\.[^.]+$/, ""),
          body: "",
          x: Math.round(center.x / 20) * 20,
          y: Math.round(center.y / 20) * 20,
          w: 170,
          h: 100,
          fill: isLightCanvasTheme() ? "#ffffff" : "#131b2e",
          stroke: state.theme === "apple90s" ? "#000080" : "#38bdf8",
          shape: "image_card",
          icon: "none",
          iconPos: "cover",
          imgUrl: dataUrl,
          z: p.nodes.length + 1,
        };
        p.nodes.push(newNode);
        invalidateGeometry();
        setSingleNodeSelection(newNode.id);
        renderFormatPanel();
        showToast(`Added Image Entity "${newNode.title}"`);
      }
      pendingImageTargetNode = null;
    };
    reader.readAsDataURL(file);
  });
}

const btnUploadImgToolbar = document.getElementById("btn-upload-img");
if (btnUploadImgToolbar) {
  btnUploadImgToolbar.onclick = () => triggerEntityImageUpload(nodeMap.get(state.selectedNodeId) || null);
}

function addStencilToCanvasCenter(item) {
  const center = screenToWorld(canvas.width / 2 - item.w / 2, canvas.height / 2 - item.h / 2);
  createNodeFromStencil(item, Math.round(center.x / 20) * 20, Math.round(center.y / 20) * 20);
}

// HTML5 Drag & Drop support on Canvas (Stencils + OS Image Files)
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
  const sx = e.clientX - canvasRect.left;
  const sy = e.clientY - canvasRect.top;
  const wpt = screenToWorld(sx, sy);

  // 1. OS Image File Dropped onto Canvas or Node
  const files = e.dataTransfer?.files;
  if (files && files.length > 0 && files[0].type.startsWith("image/")) {
    const file = files[0];
    const reader = new FileReader();
    reader.onload = () => {
      const dataUrl = reader.result;
      if (typeof dataUrl !== "string") return;
      saveSnapshot();
      getOrLoadImage(dataUrl);
      const p = page();
      const hitNode = [...p.nodes].reverse().find((n) => wpt.x >= n.x && wpt.x <= n.x + n.w && wpt.y >= n.y && wpt.y <= n.y + n.h);
      if (hitNode) {
        hitNode.imgUrl = dataUrl;
        setSingleNodeSelection(hitNode.id);
        renderFormatPanel();
        showToast(`Set Image on #${hitNode.id}`);
      } else {
        const newNode = {
          id: ++state.nextId,
          title: file.name.replace(/\.[^.]+$/, ""),
          body: "",
          x: Math.round(wpt.x - 85),
          y: Math.round(wpt.y - 50),
          w: 170,
          h: 100,
          fill: isLightCanvasTheme() ? "#ffffff" : "#131b2e",
          stroke: state.theme === "apple90s" ? "#000080" : "#38bdf8",
          shape: "image_card",
          icon: "none",
          iconPos: "cover",
          imgUrl: dataUrl,
          z: p.nodes.length + 1,
        };
        p.nodes.push(newNode);
        invalidateGeometry();
        setSingleNodeSelection(newNode.id);
        renderFormatPanel();
        showToast(`Dropped Image "${newNode.title}"`);
      }
    };
    reader.readAsDataURL(file);
    return;
  }

  // 2. Left Panel Stencil Dropped onto Canvas
  if (state.draggingStencil) {
    const item = state.draggingStencil.item;
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
          label: "",
          anim: state.edgeAnimPreset,
          kind: state.edgeKindPreset,
          arrow: state.edgeArrowPreset,
          color: state.theme === "apple90s" ? "#000080" : "#818cf8",
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
        label: "",
        anim: state.edgeAnimPreset,
        kind: state.edgeKindPreset,
        arrow: state.edgeArrowPreset,
        color: state.theme === "apple90s" ? "#000080" : "#818cf8",
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
    ctx.moveTo(cx - s * 0.45, cy - s * 0.65);
    ctx.lineTo(cx - s * 0.1, cy - s * 0.65);
    ctx.lineTo(cx + s * 0.5, cy + s * 0.65);
    ctx.moveTo(cx + s * 0.08, cy - s * 0.1);
    ctx.lineTo(cx - s * 0.48, cy + s * 0.65);
  } else if (shape === "aws_s3") {
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
    ctx.roundRect(cx - s * 0.7, cy - s * 0.65, s * 1.4, s * 0.52, 2);
    ctx.roundRect(cx - s * 0.7, cy + s * 0.13, s * 1.4, s * 0.52, 2);
    ctx.moveTo(cx - s * 0.4, cy - s * 0.39);
    ctx.lineTo(cx - s * 0.35, cy - s * 0.39);
    ctx.moveTo(cx - s * 0.4, cy + s * 0.39);
    ctx.lineTo(cx - s * 0.35, cy + s * 0.39);
  } else if (shape === "firewall") {
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
    ctx.roundRect(cx - s * 0.45, cy - s * 0.65, s * 1.05, s * 1.3, 2);
    ctx.moveTo(cx - s * 0.7, cy - s * 0.28);
    ctx.lineTo(cx - s * 0.2, cy - s * 0.28);
    ctx.moveTo(cx - s * 0.7, cy + s * 0.28);
    ctx.lineTo(cx - s * 0.2, cy + s * 0.28);
  } else if (shape === "globe") {
    ctx.arc(cx, cy, s * 0.68, 0, Math.PI * 2);
    ctx.moveTo(cx - s * 0.68, cy);
    ctx.lineTo(cx + s * 0.68, cy);
    ctx.moveTo(cx, cy - s * 0.68);
    ctx.ellipse(cx, cy, s * 0.32, s * 0.68, 0, 0, Math.PI * 2);
  } else if (shape === "cpu") {
    ctx.roundRect(cx - s * 0.48, cy - s * 0.48, s * 0.96, s * 0.96, 1.5);
    ctx.moveTo(cx - s * 0.22, cy - s * 0.72);
    ctx.lineTo(cx - s * 0.22, cy - s * 0.48);
    ctx.moveTo(cx + s * 0.22, cy - s * 0.72);
    ctx.lineTo(cx + s * 0.22, cy - s * 0.48);
    ctx.moveTo(cx - s * 0.22, cy + s * 0.48);
    ctx.lineTo(cx - s * 0.22, cy + s * 0.72);
    ctx.moveTo(cx + s * 0.22, cy + s * 0.48);
    ctx.lineTo(cx + s * 0.22, cy + s * 0.72);
    ctx.moveTo(cx - s * 0.72, cy - s * 0.22);
    ctx.lineTo(cx - s * 0.48, cy - s * 0.22);
    ctx.moveTo(cx - s * 0.72, cy + s * 0.22);
    ctx.lineTo(cx - s * 0.48, cy + s * 0.22);
    ctx.moveTo(cx + s * 0.48, cy - s * 0.22);
    ctx.lineTo(cx + s * 0.72, cy - s * 0.22);
    ctx.moveTo(cx + s * 0.48, cy + s * 0.22);
    ctx.lineTo(cx + s * 0.72, cy + s * 0.22);
  } else if (shape === "database") {
    ctx.ellipse(cx, cy - s * 0.42, s * 0.6, s * 0.24, 0, 0, Math.PI * 2);
    ctx.moveTo(cx - s * 0.6, cy - s * 0.42);
    ctx.lineTo(cx - s * 0.6, cy + s * 0.42);
    ctx.ellipse(cx, cy + s * 0.42, s * 0.6, s * 0.24, 0, 0, Math.PI);
    ctx.lineTo(cx + s * 0.6, cy - s * 0.42);
  } else if (shape === "cloud") {
    ctx.moveTo(cx - s * 0.45, cy + s * 0.42);
    ctx.lineTo(cx + s * 0.48, cy + s * 0.42);
    ctx.bezierCurveTo(cx + s * 0.75, cy + s * 0.42, cx + s * 0.75, cy - s * 0.05, cx + s * 0.45, cy - s * 0.05);
    ctx.bezierCurveTo(cx + s * 0.42, cy - s * 0.55, cx - s * 0.2, cy - s * 0.55, cx - s * 0.28, cy - s * 0.12);
    ctx.bezierCurveTo(cx - s * 0.75, cy - s * 0.12, cx - s * 0.75, cy + s * 0.42, cx - s * 0.45, cy + s * 0.42);
  } else if (shape === "lock") {
    ctx.roundRect(cx - s * 0.55, cy - s * 0.08, s * 1.1, s * 0.76, 2);
    ctx.moveTo(cx - s * 0.34, cy - s * 0.08);
    ctx.lineTo(cx - s * 0.34, cy - s * 0.36);
    ctx.arc(cx, cy - s * 0.36, s * 0.34, Math.PI, 0, false);
    ctx.lineTo(cx + s * 0.34, cy - s * 0.08);
  } else if (shape === "user") {
    ctx.arc(cx, cy - s * 0.26, s * 0.32, 0, Math.PI * 2);
    ctx.moveTo(cx - s * 0.6, cy + s * 0.64);
    ctx.bezierCurveTo(cx - s * 0.5, cy + s * 0.15, cx + s * 0.5, cy + s * 0.15, cx + s * 0.6, cy + s * 0.64);
  } else if (shape === "code") {
    ctx.moveTo(cx - s * 0.32, cy - s * 0.42);
    ctx.lineTo(cx - s * 0.68, cy);
    ctx.lineTo(cx - s * 0.32, cy + s * 0.42);
    ctx.moveTo(cx + s * 0.32, cy - s * 0.42);
    ctx.lineTo(cx + s * 0.68, cy);
    ctx.lineTo(cx + s * 0.32, cy + s * 0.42);
    ctx.moveTo(cx + s * 0.14, cy - s * 0.55);
    ctx.lineTo(cx - s * 0.14, cy + s * 0.55);
  } else if (shape === "terminal") {
    ctx.roundRect(cx - s * 0.68, cy - s * 0.54, s * 1.36, s * 1.08, 2);
    ctx.moveTo(cx - s * 0.38, cy - s * 0.2);
    ctx.lineTo(cx - s * 0.12, cy + s * 0.04);
    ctx.lineTo(cx - s * 0.38, cy + s * 0.28);
    ctx.moveTo(cx + s * 0.04, cy + s * 0.28);
    ctx.lineTo(cx + s * 0.36, cy + s * 0.28);
  } else if (shape === "gear") {
    ctx.arc(cx, cy, s * 0.32, 0, Math.PI * 2);
    for (let i = 0; i < 6; i++) {
      const a = (i / 6) * Math.PI * 2;
      ctx.moveTo(cx + Math.cos(a) * s * 0.42, cy + Math.sin(a) * s * 0.42);
      ctx.lineTo(cx + Math.cos(a) * s * 0.68, cy + Math.sin(a) * s * 0.68);
    }
  } else if (shape === "rocket") {
    ctx.moveTo(cx + s * 0.58, cy - s * 0.58);
    ctx.bezierCurveTo(cx + s * 0.1, cy - s * 0.58, cx - s * 0.35, cy - s * 0.15, cx - s * 0.48, cy + s * 0.22);
    ctx.lineTo(cx - s * 0.22, cy + s * 0.48);
    ctx.bezierCurveTo(cx + s * 0.15, cy + s * 0.35, cx + s * 0.58, cy - s * 0.1, cx + s * 0.58, cy - s * 0.58);
    ctx.moveTo(cx - s * 0.48, cy + s * 0.22);
    ctx.lineTo(cx - s * 0.66, cy + s * 0.66);
    ctx.lineTo(cx - s * 0.22, cy + s * 0.48);
  } else if (shape === "image_card") {
    ctx.roundRect(cx - s * 0.68, cy - s * 0.52, s * 1.36, s * 1.04, 2);
    ctx.moveTo(cx - s * 0.22, cy - s * 0.16);
    ctx.arc(cx - s * 0.32, cy - s * 0.16, s * 0.12, 0, Math.PI * 2);
    ctx.moveTo(cx - s * 0.68, cy + s * 0.32);
    ctx.lineTo(cx - s * 0.18, cy - s * 0.1);
    ctx.lineTo(cx + s * 0.18, cy + s * 0.22);
    ctx.lineTo(cx + s * 0.42, cy + s * 0.02);
    ctx.lineTo(cx + s * 0.68, cy + s * 0.32);
  }
  ctx.stroke();
  ctx.restore();
}

function drawNodeEntityMedia(ctx, n, strokeCol, fillCol) {
  const { x, y, w, h } = n;
  const iconKey = getEffectiveNodeIcon(n);
  const hasImg = Boolean(n.imgUrl);
  if (!hasImg && !iconKey) return;

  const pos = n.iconPos || (n.shape === "image_card" ? "cover" : "left");
  const imgObj = hasImg ? getOrLoadImage(n.imgUrl) : null;

  if (pos === "cover") {
    const bannerH = n.title ? 24 : 0;
    const mediaH = Math.max(20, h - bannerH);
    ctx.save();
    ctx.beginPath();
    ctx.roundRect(x + 2, y + 2, w - 4, mediaH - 2, bannerH ? [6, 6, 0, 0] : 6);
    ctx.clip();
    if (imgObj) {
      ctx.drawImage(imgObj, x + 2, y + 2, w - 4, mediaH - 2);
    } else {
      ctx.fillStyle = "rgba(128, 128, 164, 0.14)";
      ctx.fillRect(x + 2, y + 2, w - 4, mediaH - 2);
      drawMinimalistBadgeGlyph(
        ctx,
        iconKey || "image_card",
        x + w * 0.5,
        y + mediaH * 0.5,
        Math.min(w, mediaH) * 0.28,
        strokeCol
      );
    }
    ctx.restore();
    if (bannerH) {
      ctx.beginPath();
      ctx.moveTo(x, y + mediaH);
      ctx.lineTo(x + w, y + mediaH);
      ctx.stroke();
    }
    return;
  }

  if (pos === "top") {
    const bx = x + w * 0.5;
    const by = y + 20;
    ctx.save();
    ctx.beginPath();
    ctx.roundRect(bx - 13, by - 13, 26, 26, 6);
    ctx.fillStyle = "rgba(148, 163, 184, 0.1)";
    ctx.fill();
    ctx.stroke();
    if (imgObj) {
      ctx.clip();
      ctx.drawImage(imgObj, bx - 13, by - 13, 26, 26);
    } else if (iconKey) {
      drawMinimalistBadgeGlyph(ctx, iconKey, bx, by, 9, strokeCol);
    }
    ctx.restore();
    ctx.fillStyle = fillCol;
    return;
  }

  // Default: "left" badge
  const bx = x + 26;
  const by = y + h * 0.5;
  ctx.save();
  ctx.beginPath();
  ctx.roundRect(bx - 13, by - 13, 26, 26, 6);
  ctx.fillStyle = "rgba(148, 163, 184, 0.1)";
  ctx.fill();
  ctx.stroke();
  if (imgObj) {
    ctx.clip();
    ctx.drawImage(imgObj, bx - 13, by - 13, 26, 26);
  } else if (iconKey) {
    drawMinimalistBadgeGlyph(ctx, iconKey, bx, by, 9, strokeCol);
  }
  ctx.restore();
  ctx.fillStyle = fillCol;
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
  // 17. Minimalist Architecture Cards & Image Cards
  else if (
    shape === "image_card" ||
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
    }
  }
  // 18. Custom Vector / Imported Symbol or Standard Minimalist Process Card
  else {
    const customDef = customStencilsMap.get(shape);
    const paths = n.customPaths || customDef?.customPaths || null;
    const svgPath = n.svgPath || customDef?.svgPath || "";
    const previewImgUrl = n.imgUrl || customDef?.imgUrl || "";

    if (paths && Array.isArray(paths) && paths.length > 0) {
      drawNormalizedCustomPaths(ctx, x, y, w, h, paths, fillCol, strokeCol);
    } else if (svgPath) {
      ctx.roundRect(x, y, w, h, isMiniPreview ? 3 : 8);
      ctx.fill();
      ctx.stroke();
      try {
        ctx.save();
        ctx.translate(x + w * 0.14, y + h * 0.14);
        ctx.scale((w * 0.72) / 100, (h * 0.72) / 100);
        const p2d = new Path2D(svgPath);
        ctx.strokeStyle = strokeCol;
        ctx.lineWidth = 2.2;
        ctx.stroke(p2d);
        ctx.restore();
      } catch {
        // Ignore malformed SVG path
      }
    } else {
      ctx.roundRect(x, y, w, h, isMiniPreview ? 3 : 8);
      ctx.fill();
      ctx.stroke();
      if (isMiniPreview && previewImgUrl) {
        const img = getOrLoadImage(previewImgUrl);
        if (img) {
          ctx.drawImage(img, x + 2, y + 2, w - 4, h - 4);
        }
      }
    }
  }

  // Render Entity Icon or Image on any shape when on the main canvas
  if (!isMiniPreview) {
    drawNodeEntityMedia(ctx, n, strokeCol, fillCol);
  }
}

function drawNormalizedCustomPaths(ctx, x, y, w, h, paths, fillCol, strokeCol) {
  ctx.save();
  ctx.fillStyle = fillCol;
  ctx.strokeStyle = strokeCol;
  ctx.lineJoin = "round";
  ctx.lineCap = "round";

  for (let i = 0; i < paths.length; i++) {
    const strokeObj = paths[i];
    const pts = strokeObj.pts;
    if (!pts || pts.length === 0) continue;

    ctx.beginPath();
    if (strokeObj.tool === "rect" && pts.length >= 2) {
      const rx = x + Math.min(pts[0].x, pts[1].x) * w;
      const ry = y + Math.min(pts[0].y, pts[1].y) * h;
      const rw = Math.abs(pts[1].x - pts[0].x) * w;
      const rh = Math.abs(pts[1].y - pts[0].y) * h;
      ctx.roundRect(rx, ry, Math.max(2, rw), Math.max(2, rh), 4);
      if (i === 0) ctx.fill();
      ctx.stroke();
    } else if (strokeObj.tool === "ellipse" && pts.length >= 2) {
      const cx = x + ((pts[0].x + pts[1].x) * 0.5) * w;
      const cy = y + ((pts[0].y + pts[1].y) * 0.5) * h;
      const rx = Math.max(2, Math.abs(pts[1].x - pts[0].x) * 0.5 * w);
      const ry = Math.max(2, Math.abs(pts[1].y - pts[0].y) * 0.5 * h);
      ctx.ellipse(cx, cy, rx, ry, 0, 0, Math.PI * 2);
      if (i === 0) ctx.fill();
      ctx.stroke();
    } else if (strokeObj.tool === "diamond" && pts.length >= 2) {
      const x0 = x + Math.min(pts[0].x, pts[1].x) * w;
      const y0 = y + Math.min(pts[0].y, pts[1].y) * h;
      const dw = Math.abs(pts[1].x - pts[0].x) * w;
      const dh = Math.abs(pts[1].y - pts[0].y) * h;
      ctx.moveTo(x0 + dw * 0.5, y0);
      ctx.lineTo(x0 + dw, y0 + dh * 0.5);
      ctx.lineTo(x0 + dw * 0.5, y0 + dh);
      ctx.lineTo(x0, y0 + dh * 0.5);
      ctx.closePath();
      if (i === 0) ctx.fill();
      ctx.stroke();
    } else if (strokeObj.tool === "line" && pts.length >= 2) {
      ctx.moveTo(x + pts[0].x * w, y + pts[0].y * h);
      ctx.lineTo(x + pts[1].x * w, y + pts[1].y * h);
      ctx.stroke();
    } else {
      ctx.moveTo(x + pts[0].x * w, y + pts[0].y * h);
      for (let j = 1; j < pts.length; j++) {
        ctx.lineTo(x + pts[j].x * w, y + pts[j].y * h);
      }
      if (
        pts.length >= 3 &&
        Math.hypot(pts[0].x - pts[pts.length - 1].x, pts[0].y - pts[pts.length - 1].y) < 0.08
      ) {
        ctx.closePath();
        if (i === 0) ctx.fill();
      }
      ctx.stroke();
    }
  }
  ctx.restore();
}

function getNodeTextPlacement(n) {
  const { x, y, w, h, shape } = n;
  const cx = x + w * 0.5;
  const hasMedia = Boolean(n.imgUrl || getEffectiveNodeIcon(n));
  const mediaPos = n.iconPos || (shape === "image_card" ? "cover" : "left");

  if (shape === "actor" || shape === "gateway_xor" || shape === "gateway_parallel" || shape === "timer") {
    return {
      cx,
      titleY: y + h + 15,
      bodyY: y + h + 29,
      external: true,
    };
  }
  if (hasMedia && mediaPos === "cover") {
    return {
      cx,
      titleY: y + h - 8,
      bodyY: y + h + 14,
      external: false,
      hideBody: true,
    };
  }
  if (hasMedia && mediaPos === "top") {
    return {
      cx,
      titleY: y + 48,
      bodyY: y + 63,
      external: false,
    };
  }
  const textCx = hasMedia && mediaPos === "left" ? x + 44 + (w - 44) * 0.5 : cx;

  if (shape === "cylinder") {
    const bodyCenterY = y + 16 + (h - 16) * 0.5;
    return { cx: textCx, titleY: bodyCenterY - (n.body ? 5 : -4), bodyY: bodyCenterY + 12, external: false };
  }
  if (shape === "uml_package") {
    const bodyCenterY = y + 14 + (h - 14) * 0.5;
    return { cx: textCx, titleY: bodyCenterY - (n.body ? 5 : -4), bodyY: bodyCenterY + 12, external: false };
  }
  if (shape === "browser") {
    const bodyCenterY = y + 22 + (h - 22) * 0.5;
    return { cx: textCx, titleY: bodyCenterY - (n.body ? 5 : -4), bodyY: bodyCenterY + 12, external: false };
  }
  if (shape === "uml_class" || shape === "uml_interface") {
    return {
      cx: textCx,
      titleY: y + 19,
      bodyY: y + 30 + (h - 30) * 0.5 + 4,
      external: false,
    };
  }
  if (shape === "queue") {
    return {
      cx: hasMedia ? textCx : x + w * 0.5 - 4,
      titleY: y + h * 0.5 - (n.body ? 5 : -4),
      bodyY: y + h * 0.5 + 12,
      external: false,
    };
  }
  return {
    cx: textCx,
    titleY: y + h * 0.5 - (n.body ? 5 : -4),
    bodyY: y + h * 0.5 + 12,
    external: false,
  };
}

// Render Left Stencil Palette with Drag-and-Drop + Minimalist Monoline Thumbnails
function renderPalette(filter = "") {
  const container = document.getElementById("palette-list");
  if (!container) return;
  container.innerHTML = "";
  const countEl = document.getElementById("palette-count-label");
  if (countEl) countEl.textContent = `${STENCILS.length} items`;
  const searchEl = document.getElementById("palette-search");
  if (searchEl && !searchEl.value) searchEl.placeholder = `Search ${STENCILS.length} symbols...`;

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
      const lightUI = isLightCanvasTheme();
      tctx.fillStyle = item.defaultFill || (lightUI ? "#ffffff" : "rgba(56, 189, 248, 0.08)");
      tctx.strokeStyle =
        item.defaultStroke ||
        (state.theme === "apple90s" ? "#000080" : state.theme === "contrast" ? "#000000" : lightUI ? "#2563eb" : "#38bdf8");
      tctx.lineWidth = 1.4;
      drawNodeShapeGeometry(
        tctx,
        {
          x: 2.5,
          y: 2.5,
          w: 25,
          h: 17,
          shape: item.shape,
          customPaths: item.customPaths,
          svgPath: item.svgPath,
          imgUrl: item.imgUrl,
        },
        true
      );

      const labelSpan = document.createElement("span");
      labelSpan.textContent = item.label;
      labelSpan.style.overflow = "hidden";
      labelSpan.style.textOverflow = "ellipsis";

      card.appendChild(thumb);
      card.appendChild(labelSpan);

      if (item.isCustom) {
        const delBtn = document.createElement("button");
        delBtn.className = "stencil-del-btn";
        delBtn.textContent = "×";
        delBtn.title = `Remove "${item.label}" from palette`;
        delBtn.addEventListener("mousedown", (ev) => ev.stopPropagation());
        delBtn.addEventListener("click", (ev) => {
          ev.stopPropagation();
          removeCustomStencil(item.id);
        });
        card.appendChild(delBtn);
      }

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
  const titleEl = document.getElementById("window-canvas-title");
  if (titleEl) {
    titleEl.textContent = `${page().name} — Macintosh Platinum`;
  }
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
      body: "",
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
const selectEdgeArrowEl = document.getElementById("select-edge-arrow");
if (selectEdgeArrowEl) {
  selectEdgeArrowEl.onchange = (e) => {
    state.edgeArrowPreset = e.target.value;
    if (state.selectedEdgeIds.size > 0) saveSnapshot();
    for (const eid of state.selectedEdgeIds) {
      const ed = page().edges.find((x) => x.id === eid);
      if (ed) {
        ed.arrow = state.edgeArrowPreset;
        delete ed.sourceArrow;
        delete ed.targetArrow;
      }
    }
    renderFormatPanel();
  };
}

function getEdgeArrowMarkers(e) {
  if (e.arrow === "custom" || e.sourceArrow || e.targetArrow) {
    return {
      start: e.sourceArrow || "none",
      end: e.targetArrow || "classic",
    };
  }
  const preset = e.arrow || "one-way";
  switch (preset) {
    case "two-way":
      return { start: "classic", end: "classic" };
    case "none":
      return { start: "none", end: "none" };
    case "reverse":
      return { start: "classic", end: "none" };
    case "open":
      return { start: "none", end: "open" };
    case "diamond":
      return { start: "diamond", end: "classic" };
    case "circle":
      return { start: "circle", end: "classic" };
    case "one-way":
    default:
      return { start: "none", end: "classic" };
  }
}

function downloadFile(filename, content, mime) {
  const blob = content instanceof Blob ? content : new Blob([content], { type: mime });
  const a = document.createElement("a");
  a.href = URL.createObjectURL(blob);
  a.download = filename;
  a.click();
}

function getDiagramExportBounds(p, pad = 42) {
  if (!p.nodes.length) {
    return { minX: 0, minY: 0, width: 720, height: 420 };
  }
  let minX = Infinity,
    minY = Infinity,
    maxX = -Infinity,
    maxY = -Infinity;
  for (const n of p.nodes) {
    if (n.x < minX) minX = n.x;
    if (n.y < minY) minY = n.y;
    if (n.x + n.w > maxX) maxX = n.x + n.w;
    if (n.y + n.h + 26 > maxY) maxY = n.y + n.h + 26;
  }
  for (const e of p.edges) {
    const geo = getEdgeGeometry(e);
    if (!geo) continue;
    if (geo.bounds.minX < minX) minX = geo.bounds.minX;
    if (geo.bounds.minY < minY) minY = geo.bounds.minY;
    if (geo.bounds.maxX > maxX) maxX = geo.bounds.maxX;
    if (geo.bounds.maxY > maxY) maxY = geo.bounds.maxY;
  }
  return {
    minX: Math.floor(minX - pad),
    minY: Math.floor(minY - pad),
    width: Math.max(240, Math.ceil(maxX - minX + pad * 2)),
    height: Math.max(160, Math.ceil(maxY - minY + pad * 2)),
  };
}

function renderDiagramFrameToContext(tctx, p, bounds, scale, timeSec) {
  const lightCanvas = isLightCanvasTheme();
  const wPx = Math.round(bounds.width * scale);
  const hPx = Math.round(bounds.height * scale);

  tctx.save();
  tctx.fillStyle =
    state.theme === "apple90s"
      ? "#f5f3ec"
      : state.theme === "light"
      ? "#f8fafc"
      : state.theme === "contrast"
      ? "#ffffff"
      : "#0d121d";
  tctx.fillRect(0, 0, wPx, hPx);

  tctx.scale(scale, scale);
  tctx.translate(-bounds.minX, -bounds.minY);

  // Subtle drafting grid dots
  if (state.gridStyle !== "off") {
    tctx.fillStyle = lightCanvas ? "#cbc7bc" : "#1e293b";
    const step = 20;
    const gxStart = Math.floor(bounds.minX / step) * step;
    const gyStart = Math.floor(bounds.minY / step) * step;
    for (let gx = gxStart; gx < bounds.minX + bounds.width; gx += step) {
      for (let gy = gyStart; gy < bounds.minY + bounds.height; gy += step) {
        tctx.fillRect(gx, gy, 1.4, 1.4);
      }
    }
  }

  tctx.font = '700 11px "Charcoal", "Chicago", "Geneva", sans-serif';
  tctx.textAlign = "center";

  // Draw Edges + Animated Flows (ants & pulse)
  for (let i = 0; i < p.edges.length; i++) {
    const e = p.edges[i];
    const geo = getEdgeGeometry(e);
    if (!geo) continue;
    const strokeColor = e.color || "#000080";

    tctx.beginPath();
    if (e.kind === "straight" || e.kind === "orthogonal" || e.kind === "smoothstep") {
      tctx.moveTo(geo.samples[0].x, geo.samples[0].y);
      for (let j = 1; j < geo.samples.length; j++) {
        tctx.lineTo(geo.samples[j].x, geo.samples[j].y);
      }
    } else {
      tctx.moveTo(geo.p0.x, geo.p0.y);
      tctx.bezierCurveTo(geo.p1.x, geo.p1.y, geo.p2.x, geo.p2.y, geo.p3.x, geo.p3.y);
    }

    tctx.strokeStyle = strokeColor;
    tctx.lineWidth = e.width || 1.8;
    if (e.anim === "ants") {
      tctx.setLineDash([7, 5]);
      tctx.lineDashOffset = -timeSec * 36;
    }
    tctx.stroke();
    if (e.anim === "ants") tctx.setLineDash([]);

    drawEdgeArrowheads(tctx, e, geo, strokeColor);

    if (e.anim === "pulse") {
      const spacing = geo.totalLen * 0.5;
      const isTwoWay = (e.arrow || "one-way") === "two-way";
      for (let pIdx = 0; pIdx < 2; pIdx++) {
        const rawDist = ((timeSec * geo.totalLen + pIdx * spacing) % geo.totalLen + geo.totalLen) % geo.totalLen;
        const headDist = isTwoWay && pIdx === 1 ? geo.totalLen - rawDist : rawDist;
        const dirSign = isTwoWay && pIdx === 1 ? 1 : -1;
        const tail = getPointAtArcDistance(geo, headDist + dirSign * 6);
        const head = getPointAtArcDistance(geo, headDist);

        tctx.fillStyle = lightCanvas ? "rgba(0, 85, 204, 0.35)" : "rgba(56, 189, 248, 0.35)";
        tctx.beginPath();
        tctx.arc(tail.x, tail.y, 4.4, 0, Math.PI * 2);
        tctx.fill();

        tctx.fillStyle = lightCanvas ? strokeColor : "#e0f2fe";
        tctx.beginPath();
        tctx.arc(head.x, head.y, 3.4, 0, Math.PI * 2);
        tctx.fill();
      }
    }

    if (e.label && e.label.trim().length > 0) {
      const textW = geo.labelWidth;
      tctx.fillStyle = lightCanvas ? "#ffffff" : "#161b26";
      tctx.strokeStyle = strokeColor;
      tctx.lineWidth = 1.25;
      tctx.beginPath();
      tctx.roundRect(geo.midPoint.x - textW * 0.5, geo.midPoint.y - 9.5, textW, 19, 3);
      tctx.fill();
      tctx.stroke();
      tctx.fillStyle = lightCanvas ? "#000000" : "#f1f5f9";
      tctx.fillText(e.label, geo.midPoint.x, geo.midPoint.y + 3.5);
    }
  }

  // Draw Nodes + Custom Symbols + Icons + Images
  for (let i = 0; i < p.nodes.length; i++) {
    const n = p.nodes[i];
    tctx.save();
    if (state.theme === "apple90s" || state.theme === "contrast") {
      tctx.shadowColor = "rgba(0, 0, 0, 0.18)";
      tctx.shadowOffsetX = 3 * scale;
      tctx.shadowOffsetY = 3 * scale;
      tctx.shadowBlur = 0;
    }
    tctx.fillStyle = n.fill;
    tctx.strokeStyle = n.stroke;
    tctx.lineWidth = 1.75;
    drawNodeShapeGeometry(tctx, n, false);
    tctx.restore();
  }

  // Node Titles
  tctx.font = '700 12.5px "Charcoal", "Chicago", "Geneva", sans-serif';
  tctx.textAlign = "center";
  for (let i = 0; i < p.nodes.length; i++) {
    const n = p.nodes[i];
    const pos = getNodeTextPlacement(n);
    const darkFill = isDarkHexColor(n.fill);
    tctx.fillStyle = pos.external ? (lightCanvas ? "#000000" : "#f1f5f9") : darkFill ? "#f1f5f9" : "#000000";
    tctx.fillText(n.title, pos.cx, pos.titleY);
  }

  // Node Subtitles
  tctx.font = '600 10.5px "Monaco", "Menlo", monospace';
  for (let i = 0; i < p.nodes.length; i++) {
    const n = p.nodes[i];
    if (!n.body) continue;
    const pos = getNodeTextPlacement(n);
    if (pos.hideBody) continue;
    const darkFill = isDarkHexColor(n.fill);
    tctx.fillStyle = pos.external ? (lightCanvas ? "#4a4a4a" : "#94a3b8") : darkFill ? "#94a3b8" : "#4a4a4a";
    tctx.fillText(n.body, pos.cx, pos.bodyY);
  }

  tctx.restore();
}

// Self-Contained GIF89a Animated LZW Encoder (256-color table + Netscape 2.0 infinite loop)
function encodeGif89a(rgbaFrames, width, height, delayCs = 6) {
  // Build 256-color palette: 216 (6x6x6 RGB cube) + 40 grayscale/drafting steps
  const palette = new Uint8Array(256 * 3);
  let pIdx = 0;
  for (let r = 0; r < 6; r++) {
    for (let g = 0; g < 6; g++) {
      for (let b = 0; b < 6; b++) {
        palette[pIdx * 3] = Math.round((r * 255) / 5);
        palette[pIdx * 3 + 1] = Math.round((g * 255) / 5);
        palette[pIdx * 3 + 2] = Math.round((b * 255) / 5);
        pIdx++;
      }
    }
  }
  // Add warm Macintosh paper "#f5f3ec" at index 216 and grays for the rest
  palette[216 * 3] = 245;
  palette[216 * 3 + 1] = 243;
  palette[216 * 3 + 2] = 236;
  for (let i = 217; i < 256; i++) {
    const v = Math.round(((i - 217) * 255) / 38);
    palette[i * 3] = v;
    palette[i * 3 + 1] = v;
    palette[i * 3 + 2] = v;
  }

  const out = [];
  const writeByte = (b) => out.push(b & 0xff);
  const writeShort = (s) => {
    out.push(s & 0xff, (s >> 8) & 0xff);
  };
  const writeAscii = (str) => {
    for (let i = 0; i < str.length; i++) out.push(str.charCodeAt(i));
  };

  // GIF89a Header & Logical Screen Descriptor
  writeAscii("GIF89a");
  writeShort(width);
  writeShort(height);
  writeByte(0xf7); // Global Color Table Flag (1), Color Res (7), Sort (0), Size of GCT (7 => 256 colors)
  writeByte(0); // Background color
  writeByte(0); // Pixel aspect ratio
  for (let i = 0; i < palette.length; i++) writeByte(palette[i]);

  // Netscape 2.0 Application Extension for Infinite Looping
  writeByte(0x21);
  writeByte(0xff);
  writeByte(11);
  writeAscii("NETSCAPE2.0");
  writeByte(3);
  writeByte(1);
  writeShort(0); // Loop forever
  writeByte(0);

  const numPixels = width * height;
  const indexedPixels = new Uint8Array(numPixels);

  // LZW Compression for 8-bit minCodeSize
  function lzwEncodeFrame(pixels) {
    const minCodeSize = 8;
    const clearCode = 1 << minCodeSize; // 256
    const eoiCode = clearCode + 1; // 257

    let codeSize = minCodeSize + 1;
    let nextCode = eoiCode + 1;
    const dict = new Int32Array(4096 * 256).fill(-1);

    let bitBuf = 0;
    let bitCount = 0;
    const subBlock = [];

    const flushSubBlock = () => {
      if (subBlock.length > 0) {
        writeByte(subBlock.length);
        for (let i = 0; i < subBlock.length; i++) writeByte(subBlock[i]);
        subBlock.length = 0;
      }
    };

    const emitCode = (code) => {
      bitBuf |= code << bitCount;
      bitCount += codeSize;
      while (bitCount >= 8) {
        subBlock.push(bitBuf & 0xff);
        if (subBlock.length === 255) flushSubBlock();
        bitBuf >>= 8;
        bitCount -= 8;
      }
    };

    writeByte(minCodeSize);
    emitCode(clearCode);

    let prefix = pixels[0];
    for (let i = 1; i < pixels.length; i++) {
      const k = pixels[i];
      const key = (prefix << 8) | k;
      const hit = dict[key];
      if (hit !== -1) {
        prefix = hit;
      } else {
        emitCode(prefix);
        if (nextCode < 4096) {
          dict[key] = nextCode++;
          if (nextCode > 1 << codeSize && codeSize < 12) {
            codeSize++;
          }
        } else {
          emitCode(clearCode);
          dict.fill(-1);
          codeSize = minCodeSize + 1;
          nextCode = eoiCode + 1;
        }
        prefix = k;
      }
    }
    emitCode(prefix);
    emitCode(eoiCode);
    if (bitCount > 0) {
      subBlock.push(bitBuf & 0xff);
    }
    flushSubBlock();
    writeByte(0); // Block terminator
  }

  for (const rgba of rgbaFrames) {
    // Graphic Control Extension
    writeByte(0x21);
    writeByte(0xf9);
    writeByte(4);
    writeByte(0x00);
    writeShort(delayCs);
    writeByte(0);
    writeByte(0);

    // Image Descriptor
    writeByte(0x2c);
    writeShort(0);
    writeShort(0);
    writeShort(width);
    writeShort(height);
    writeByte(0x00);

    // Map RGBA to 256-color palette
    for (let p = 0, j = 0; p < numPixels; p++, j += 4) {
      const r = rgba[j];
      const g = rgba[j + 1];
      const b = rgba[j + 2];
      if (r >= 242 && r <= 248 && g >= 240 && g <= 246 && b >= 232 && b <= 240) {
        indexedPixels[p] = 216;
      } else {
        const ri = Math.round((r * 5) / 255);
        const gi = Math.round((g * 5) / 255);
        const bi = Math.round((b * 5) / 255);
        indexedPixels[p] = ri * 36 + gi * 6 + bi;
      }
    }

    lzwEncodeFrame(indexedPixels);
  }

  writeByte(0x3b); // GIF Trailer
  return new Uint8Array(out);
}

function exportAnimatedGif() {
  const p = page();
  const bounds = getDiagramExportBounds(p, 36);
  const maxDim = 820;
  const scale = Math.min(1.0, maxDim / Math.max(bounds.width, bounds.height));
  const width = Math.max(120, Math.round(bounds.width * scale));
  const height = Math.max(90, Math.round(bounds.height * scale));

  const offCanvas = document.createElement("canvas");
  offCanvas.width = width;
  offCanvas.height = height;
  const offCtx = offCanvas.getContext("2d", { willReadFrequently: true });

  const frameCount = 16;
  const rgbaFrames = [];
  for (let f = 0; f < frameCount; f++) {
    const tSec = f / frameCount;
    renderDiagramFrameToContext(offCtx, p, bounds, scale, tSec);
    rgbaFrames.push(offCtx.getImageData(0, 0, width, height).data);
  }

  const gifBytes = encodeGif89a(rgbaFrames, width, height, 6);
  const blob = new Blob([gifBytes], { type: "image/gif" });
  downloadFile("diagram-animated.gif", blob, "image/gif");
  showToast(`Exported diagram-animated.gif (${frameCount} frames, ${Math.round(gifBytes.byteLength / 1024)} KB)`);
}

function exportPngImage() {
  const p = page();
  const bounds = getDiagramExportBounds(p, 40);
  const scale = 2.0; // High-DPI 2x PNG
  const offCanvas = document.createElement("canvas");
  offCanvas.width = Math.round(bounds.width * scale);
  offCanvas.height = Math.round(bounds.height * scale);
  const offCtx = offCanvas.getContext("2d");

  renderDiagramFrameToContext(offCtx, p, bounds, scale, 0.25);
  offCanvas.toBlob((blob) => {
    if (blob) {
      downloadFile("diagram.png", blob, "image/png");
      showToast(`Exported High-DPI diagram.png (${offCanvas.width}×${offCanvas.height})`);
    }
  }, "image/png");
}

function escapeXmlAttr(str) {
  return String(str || "")
    .replace(/&/g, "&amp;")
    .replace(/"/g, "&quot;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;");
}

function exportDrawioXml() {
  let xml = `<mxfile host="RustFlowDraw" version="1.0.0">\n`;
  for (const p of state.pages) {
    xml += `  <diagram id="page-${p.id}" name="${escapeXmlAttr(p.name)}">\n    <mxGraphModel dx="1280" dy="720" grid="1" gridSize="20">\n      <root>\n        <mxCell id="0"/>\n        <mxCell id="1" parent="0"/>\n`;
    for (const n of p.nodes) {
      const style = [
        `shape=${n.shape}`,
        `rounded=1`,
        `fillColor=${n.fill}`,
        `strokeColor=${n.stroke}`,
        n.body ? `rfdBody=${encodeURIComponent(n.body)}` : "",
        n.icon ? `rfdIcon=${n.icon}` : "",
        n.iconPos ? `rfdIconPos=${n.iconPos}` : "",
        n.imgUrl ? `rfdImg=${encodeURIComponent(n.imgUrl)}` : "",
      ]
        .filter(Boolean)
        .join(";");
      xml += `        <mxCell id="node-${n.id}" value="${escapeXmlAttr(n.title)}" style="${style};" vertex="1" parent="1">\n          <mxGeometry x="${Math.round(n.x)}" y="${Math.round(n.y)}" width="${Math.round(n.w)}" height="${Math.round(n.h)}" as="geometry"/>\n        </mxCell>\n`;
    }
    for (const e of p.edges) {
      const style = [
        `curved=1`,
        `rfdAnim=${e.anim || "pulse"}`,
        `rfdKind=${e.kind || "bezier"}`,
        `rfdArrow=${e.arrow || "one-way"}`,
        `rfdSrcPort=${e.sourcePort || "right"}`,
        `rfdDstPort=${e.targetPort || "left"}`,
        `strokeColor=${e.color || "#000080"}`,
        `strokeWidth=${e.width || 1.8}`,
      ].join(";");
      xml += `        <mxCell id="edge-${e.id}" value="${escapeXmlAttr(e.label || "")}" style="${style};" edge="1" parent="1" source="node-${e.source}" target="node-${e.target}">\n          <mxGeometry relative="1" as="geometry"/>\n        </mxCell>\n`;
    }
    xml += `      </root>\n    </mxGraphModel>\n  </diagram>\n`;
  }
  xml += `</mxfile>`;
  downloadFile("diagram.drawio", xml, "application/xml");
  showToast("Saved diagram.drawio");
}

function exportAnimatedSvg() {
  const p = page();
  const bounds = getDiagramExportBounds(p, 40);
  const bg = state.theme === "apple90s" ? "#f5f3ec" : state.theme === "light" ? "#f8fafc" : "#0d121d";
  const textCol = isLightCanvasTheme() ? "#000000" : "#f8fafc";
  let svg = `<svg xmlns="http://www.w3.org/2000/svg" viewBox="${bounds.minX} ${bounds.minY} ${bounds.width} ${bounds.height}" width="${bounds.width}" height="${bounds.height}">\n  <style>\n    @keyframes rfd-ants { to { stroke-dashoffset: -24; } }\n    .ants { stroke-dasharray: 7 5; animation: rfd-ants 0.75s linear infinite; }\n    text { font-family: Charcoal, Chicago, Geneva, sans-serif; font-size: 12.5px; font-weight: 700; fill: ${textCol}; }\n  </style>\n  <rect x="${bounds.minX}" y="${bounds.minY}" width="${bounds.width}" height="${bounds.height}" fill="${bg}"/>\n`;
  for (const e of p.edges) {
    const geo = getEdgeGeometry(e);
    if (!geo) continue;
    const d = `M ${geo.p0.x} ${geo.p0.y} C ${geo.p1.x} ${geo.p1.y}, ${geo.p2.x} ${geo.p2.y}, ${geo.p3.x} ${geo.p3.y}`;
    const col = e.color || "#000080";
    svg += `  <path id="edge-path-${e.id}" d="${d}" fill="none" stroke="${col}" stroke-width="${e.width || 1.8}" class="${e.anim === "ants" ? "ants" : ""}"/>\n`;
    if (e.anim === "pulse") {
      svg += `  <circle r="3.6" fill="${col}"><animateMotion dur="1.4s" repeatCount="indefinite" path="${d}"/></circle>\n`;
    }
  }
  for (const n of p.nodes) {
    svg += `  <rect x="${n.x}" y="${n.y}" width="${n.w}" height="${n.h}" rx="7" fill="${n.fill}" stroke="${n.stroke}" stroke-width="1.8"/>\n`;
    svg += `  <text x="${n.x + n.w / 2}" y="${n.y + n.h / 2}" text-anchor="middle" dominant-baseline="middle">${n.title}</text>\n`;
  }
  svg += `</svg>`;
  downloadFile("diagram-animated.svg", svg, "image/svg+xml");
  showToast("Exported diagram-animated.svg");
}

const btnExportGif = document.getElementById("btn-export-gif");
if (btnExportGif) btnExportGif.onclick = exportAnimatedGif;
const btnExportPng = document.getElementById("btn-export-png");
if (btnExportPng) btnExportPng.onclick = exportPngImage;

// ============================================================================
// CUSTOM SYMBOL STUDIO & EXTERNAL SYMBOL IMPORTER
// ============================================================================
const symbolStudioOverlay = document.getElementById("symbol-studio-overlay");
const symDrawCanvas = document.getElementById("sym-draw-canvas");
const symDrawCtx = symDrawCanvas ? symDrawCanvas.getContext("2d") : null;
const symbolFileInput = document.getElementById("symbol-file-input");

const studioState = {
  tool: "path",
  paths: JSON.parse(JSON.stringify(SYMBOL_STUDIO_TEMPLATES.redis.paths)),
  activeStroke: null,
  importedImgUrl: "",
  svgPath: "",
};

function renderStudioPreviewCanvas() {
  if (!symDrawCtx || !symDrawCanvas) return;
  const w = symDrawCanvas.width;
  const h = symDrawCanvas.height;
  symDrawCtx.fillStyle = "#f5f3ec";
  symDrawCtx.fillRect(0, 0, w, h);

  // Subtle drafting grid
  symDrawCtx.fillStyle = "#cbc7bc";
  for (let gx = 16; gx < w; gx += 16) {
    for (let gy = 16; gy < h; gy += 16) {
      symDrawCtx.fillRect(gx, gy, 1.2, 1.2);
    }
  }

  const fillCol = document.getElementById("sym-fill")?.value || "#ffffff";
  const strokeCol = document.getElementById("sym-stroke")?.value || "#000080";
  const padX = 24;
  const padY = 16;
  const boxW = w - padX * 2;
  const boxH = h - padY * 2;

  // Guide bounding frame
  symDrawCtx.save();
  symDrawCtx.strokeStyle = "rgba(0, 0, 128, 0.2)";
  symDrawCtx.setLineDash([4, 4]);
  symDrawCtx.strokeRect(padX, padY, boxW, boxH);
  symDrawCtx.restore();

  const allPaths = studioState.activeStroke ? [...studioState.paths, studioState.activeStroke] : studioState.paths;

  if (studioState.importedImgUrl) {
    symDrawCtx.fillStyle = fillCol;
    symDrawCtx.strokeStyle = strokeCol;
    symDrawCtx.lineWidth = 2;
    symDrawCtx.beginPath();
    symDrawCtx.roundRect(padX, padY, boxW, boxH, 7);
    symDrawCtx.fill();
    symDrawCtx.stroke();
    const img = getOrLoadImage(studioState.importedImgUrl);
    if (img) {
      symDrawCtx.drawImage(img, padX + 12, padY + 10, boxW - 24, boxH - 20);
    }
  } else if (studioState.svgPath) {
    symDrawCtx.fillStyle = fillCol;
    symDrawCtx.strokeStyle = strokeCol;
    symDrawCtx.lineWidth = 2;
    symDrawCtx.beginPath();
    symDrawCtx.roundRect(padX, padY, boxW, boxH, 7);
    symDrawCtx.fill();
    symDrawCtx.stroke();
    try {
      symDrawCtx.save();
      symDrawCtx.translate(padX + boxW * 0.15, padY + boxH * 0.15);
      symDrawCtx.scale((boxW * 0.7) / 100, (boxH * 0.7) / 100);
      const p2d = new Path2D(studioState.svgPath);
      symDrawCtx.strokeStyle = strokeCol;
      symDrawCtx.lineWidth = 2.4;
      symDrawCtx.stroke(p2d);
      symDrawCtx.restore();
    } catch {
      // Ignore invalid SVG path while typing
    }
  } else if (allPaths.length > 0) {
    symDrawCtx.lineWidth = 2.2;
    drawNormalizedCustomPaths(symDrawCtx, padX, padY, boxW, boxH, allPaths, fillCol, strokeCol);
  }

  const countEl = document.getElementById("sym-stroke-count");
  if (countEl) {
    countEl.textContent = studioState.importedImgUrl
      ? "Image/SVG"
      : studioState.svgPath
      ? "SVG Path"
      : `${studioState.paths.length} paths`;
  }
}

function openSymbolStudioModal() {
  if (!symbolStudioOverlay) return;
  symbolStudioOverlay.classList.add("open");
  renderStudioPreviewCanvas();
}

function closeSymbolStudioModal() {
  if (!symbolStudioOverlay) return;
  symbolStudioOverlay.classList.remove("open");
}

document.getElementById("btn-open-symbol-studio")?.addEventListener("click", openSymbolStudioModal);
document.getElementById("btn-create-symbol")?.addEventListener("click", openSymbolStudioModal);
document.getElementById("btn-close-symbol-studio")?.addEventListener("click", closeSymbolStudioModal);
document.getElementById("btn-sym-cancel")?.addEventListener("click", closeSymbolStudioModal);
symbolStudioOverlay?.addEventListener("click", (e) => {
  if (e.target === symbolStudioOverlay) closeSymbolStudioModal();
});

// Interactive Vector Drawing on #sym-draw-canvas
if (symDrawCanvas) {
  const toNormPt = (e) => {
    const r = symDrawCanvas.getBoundingClientRect();
    const padX = 24;
    const padY = 16;
    const boxW = symDrawCanvas.width - padX * 2;
    const boxH = symDrawCanvas.height - padY * 2;
    const nx = Math.max(0, Math.min(1, (e.clientX - r.left - padX) / boxW));
    const ny = Math.max(0, Math.min(1, (e.clientY - r.top - padY) / boxH));
    return { x: Number(nx.toFixed(3)), y: Number(ny.toFixed(3)) };
  };

  symDrawCanvas.addEventListener("mousedown", (e) => {
    studioState.importedImgUrl = "";
    studioState.svgPath = "";
    const pt = toNormPt(e);
    studioState.activeStroke = {
      tool: studioState.tool,
      pts: studioState.tool === "path" ? [pt] : [pt, pt],
    };
    renderStudioPreviewCanvas();
  });

  window.addEventListener("mousemove", (e) => {
    if (!studioState.activeStroke) return;
    const pt = toNormPt(e);
    if (studioState.activeStroke.tool === "path") {
      const pts = studioState.activeStroke.pts;
      const last = pts[pts.length - 1];
      if (Math.hypot(pt.x - last.x, pt.y - last.y) > 0.02) {
        pts.push(pt);
      }
    } else {
      studioState.activeStroke.pts[1] = pt;
    }
    renderStudioPreviewCanvas();
  });

  window.addEventListener("mouseup", () => {
    if (!studioState.activeStroke) return;
    studioState.paths.push(studioState.activeStroke);
    studioState.activeStroke = null;
    renderStudioPreviewCanvas();
  });
}

document.querySelectorAll(".sym-tool-btn").forEach((btn) => {
  btn.addEventListener("click", () => {
    document.querySelectorAll(".sym-tool-btn").forEach((b) => b.classList.remove("active"));
    btn.classList.add("active");
    studioState.tool = btn.getAttribute("data-tool") || "path";
  });
});

document.getElementById("btn-sym-undo")?.addEventListener("click", () => {
  studioState.paths.pop();
  renderStudioPreviewCanvas();
});

document.getElementById("btn-sym-clear")?.addEventListener("click", () => {
  studioState.paths = [];
  studioState.importedImgUrl = "";
  studioState.svgPath = "";
  const svgIn = document.getElementById("sym-svg-input");
  const urlIn = document.getElementById("sym-url-input");
  if (svgIn) svgIn.value = "";
  if (urlIn) urlIn.value = "";
  renderStudioPreviewCanvas();
});

document.getElementById("sym-preset-template")?.addEventListener("change", (e) => {
  const key = e.target.value;
  const tpl = SYMBOL_STUDIO_TEMPLATES[key];
  if (!tpl) return;
  studioState.paths = JSON.parse(JSON.stringify(tpl.paths));
  studioState.importedImgUrl = "";
  studioState.svgPath = "";
  document.getElementById("sym-name").value = tpl.name;
  document.getElementById("sym-category").value = tpl.category;
  document.getElementById("sym-w").value = tpl.w;
  document.getElementById("sym-h").value = tpl.h;
  document.getElementById("sym-fill").value = tpl.fill;
  document.getElementById("sym-stroke").value = tpl.stroke;
  renderStudioPreviewCanvas();
  e.target.value = "";
});

document.getElementById("sym-fill")?.addEventListener("input", renderStudioPreviewCanvas);
document.getElementById("sym-stroke")?.addEventListener("input", renderStudioPreviewCanvas);

document.getElementById("sym-svg-input")?.addEventListener("input", (e) => {
  const val = e.target.value.trim();
  if (!val) {
    studioState.svgPath = "";
    renderStudioPreviewCanvas();
    return;
  }
  if (val.startsWith("<svg")) {
    const dataUrl = "data:image/svg+xml;utf8," + encodeURIComponent(val);
    studioState.importedImgUrl = dataUrl;
    studioState.svgPath = "";
    const img = getOrLoadImage(dataUrl);
    if (!img) setTimeout(renderStudioPreviewCanvas, 60);
  } else {
    const matchD = val.match(/d=["']([^"']+)["']/);
    studioState.svgPath = matchD ? matchD[1] : val;
    studioState.importedImgUrl = "";
  }
  renderStudioPreviewCanvas();
});

document.getElementById("sym-url-input")?.addEventListener("input", (e) => {
  const val = e.target.value.trim();
  studioState.importedImgUrl = val;
  if (val) {
    getOrLoadImage(val);
    setTimeout(renderStudioPreviewCanvas, 120);
  }
  renderStudioPreviewCanvas();
});

// Save Custom Symbol to Left Palette & Insert onto Canvas
document.getElementById("btn-sym-save")?.addEventListener("click", () => {
  const label = (document.getElementById("sym-name")?.value || "Custom Symbol").trim();
  const category = document.getElementById("sym-category")?.value || "Custom Symbols";
  const w = Math.max(40, Math.min(400, Number(document.getElementById("sym-w")?.value) || 165));
  const h = Math.max(40, Math.min(300, Number(document.getElementById("sym-h")?.value) || 84));
  const defaultFill = document.getElementById("sym-fill")?.value || "#ffffff";
  const defaultStroke = document.getElementById("sym-stroke")?.value || "#000080";

  const slug = label.toLowerCase().replace(/[^a-z0-9]+/g, "_") + "_" + (Date.now() % 10000);
  const id = `custom.${slug}`;
  const customItem = {
    id,
    label,
    category,
    w,
    h,
    shape: id,
    defaultFill,
    defaultStroke,
    customPaths:
      !studioState.importedImgUrl && !studioState.svgPath && studioState.paths.length
        ? JSON.parse(JSON.stringify(studioState.paths))
        : studioState.paths.length
        ? JSON.parse(JSON.stringify(studioState.paths))
        : [{ tool: "rect", pts: [{ x: 0.05, y: 0.08 }, { x: 0.95, y: 0.92 }] }],
    svgPath: studioState.svgPath || "",
    imgUrl: studioState.importedImgUrl || "",
    isCustom: true,
  };

  registerCustomStencil(customItem, true);
  renderPalette(document.getElementById("palette-search")?.value || "");
  addStencilToCanvasCenter(customItem);
  closeSymbolStudioModal();
  showToast(`Saved "${label}" to ${category}`);
});

// Export all custom symbols as a shareable .json pack
document.getElementById("btn-sym-export-pack")?.addEventListener("click", () => {
  const customList = STENCILS.filter((s) => s.isCustom);
  downloadFile("rustflow-custom-symbols.json", JSON.stringify(customList, null, 2), "application/json");
  showToast(`Exported ${customList.length} custom symbol(s)`);
});

// Import Symbols from File (.svg, .png, .jpg, .webp, .json)
let importFromStudioModal = false;
document.getElementById("btn-import-symbol-file")?.addEventListener("click", () => {
  importFromStudioModal = false;
  if (symbolFileInput) {
    symbolFileInput.value = "";
    symbolFileInput.click();
  }
});
document.getElementById("btn-sym-upload-file")?.addEventListener("click", () => {
  importFromStudioModal = true;
  if (symbolFileInput) {
    symbolFileInput.value = "";
    symbolFileInput.click();
  }
});

if (symbolFileInput) {
  symbolFileInput.addEventListener("change", (e) => {
    const files = Array.from(e.target.files || []);
    if (!files.length) return;

    for (const file of files) {
      const baseName = file.name.replace(/\.[^.]+$/, "");
      if (file.name.endsWith(".json")) {
        const reader = new FileReader();
        reader.onload = () => {
          try {
            const parsed = JSON.parse(String(reader.result));
            const list = Array.isArray(parsed) ? parsed : [parsed];
            let importedCount = 0;
            for (const item of list) {
              if (item && item.label) {
                const cid = item.id || `custom.${Date.now()}_${importedCount}`;
                registerCustomStencil({ ...item, id: cid, shape: item.shape || cid, isCustom: true }, true);
                importedCount++;
              }
            }
            renderPalette(document.getElementById("palette-search")?.value || "");
            showToast(`Imported ${importedCount} symbol(s) from ${file.name}`);
          } catch {
            showToast(`Invalid JSON symbol pack: ${file.name}`);
          }
        };
        reader.readAsText(file);
      } else {
        const reader = new FileReader();
        reader.onload = () => {
          const dataUrl = String(reader.result || "");
          if (!dataUrl) return;
          getOrLoadImage(dataUrl);
          if (importFromStudioModal && symbolStudioOverlay?.classList.contains("open")) {
            studioState.importedImgUrl = dataUrl;
            document.getElementById("sym-name").value = baseName;
            setTimeout(renderStudioPreviewCanvas, 50);
            showToast(`Loaded ${file.name} into Symbol Studio`);
          } else {
            const slug = baseName.toLowerCase().replace(/[^a-z0-9]+/g, "_") + "_" + (Date.now() % 10000);
            const id = `custom.${slug}`;
            const newStencil = {
              id,
              label: baseName,
              category: "Custom Symbols",
              w: 165,
              h: 92,
              shape: id,
              defaultFill: "#ffffff",
              defaultStroke: "#000080",
              customPaths: null,
              svgPath: "",
              imgUrl: dataUrl,
              isCustom: true,
            };
            registerCustomStencil(newStencil, true);
            renderPalette(document.getElementById("palette-search")?.value || "");
            addStencilToCanvasCenter(newStencil);
            showToast(`Imported symbol "${baseName}" to Palette`);
          }
        };
        reader.readAsDataURL(file);
      }
    }
  });
}

// ============================================================================
// STARTUP WELCOME DIALOG: CREATE NEW EMPTY SPACE OR LOAD FROM FILE (.DRAWIO / .JSON)
// ============================================================================
const welcomeOverlay = document.getElementById("welcome-overlay");
const welcomeDocNameInput = document.getElementById("welcome-doc-name");
const welcomeDropzone = document.getElementById("welcome-dropzone");
const diagramFileInput = document.getElementById("diagram-file-input");

function openWelcomeModal() {
  if (!welcomeOverlay) return;
  welcomeOverlay.classList.add("open");
  if (welcomeDocNameInput) {
    welcomeDocNameInput.focus();
    welcomeDocNameInput.select();
  }
}

function closeWelcomeModal() {
  if (!welcomeOverlay) return;
  welcomeOverlay.classList.remove("open");
}

function createEmptyWorkspace(workspaceName = "Untitled Diagram") {
  const cleanName = (workspaceName || "Untitled Diagram").trim() || "Untitled Diagram";
  state.pages = [
    {
      id: 1,
      name: `Page-1: ${cleanName}`,
      pan: { x: 80, y: 60 },
      zoom: 1.0,
      nodes: [],
      edges: [],
      undoStack: [],
      redoStack: [],
    },
  ];
  state.activePageId = 1;
  setSingleNodeSelection(null);
  invalidateGeometry();
  renderPageTabs();
  renderFormatPanel();
  updateZoomLabel();
  closeWelcomeModal();
  showToast(`Created Empty Workspace "${cleanName}"`);
}

function loadSampleArchitectureWorkspace() {
  state.pages = createDogfoodPages();
  state.activePageId = 1;
  setSingleNodeSelection(state.pages[0]?.nodes[0]?.id || null);
  invalidateGeometry();
  renderPageTabs();
  renderFormatPanel();
  updateZoomLabel();
  closeWelcomeModal();
  showToast("Loaded Sample Architecture Workspace");
}

function triggerOpenDiagramFile() {
  if (diagramFileInput) {
    diagramFileInput.value = "";
    diagramFileInput.click();
  }
}

function parseStyleMap(styleStr = "") {
  const map = {};
  for (const part of String(styleStr).split(";")) {
    const [k, v] = part.split("=");
    if (k && v !== undefined) map[k.trim()] = v.trim();
  }
  return map;
}

function parseDrawioXmlToPages(xmlText, fallbackName = "Imported Diagram") {
  const parser = new DOMParser();
  const doc = parser.parseFromString(xmlText, "text/xml");
  const diagramEls = Array.from(doc.querySelectorAll("diagram"));
  const roots = diagramEls.length > 0 ? diagramEls : [doc.documentElement];

  const loadedPages = [];
  let pageIdCounter = 1;

  for (const diagEl of roots) {
    const pageName = diagEl.getAttribute("name") || `Page-${pageIdCounter}: ${fallbackName}`;
    const cells = Array.from(diagEl.querySelectorAll("mxCell"));
    const nodes = [];
    const edges = [];
    const cellIdToNodeId = new Map();

    for (const cell of cells) {
      const rawId = cell.getAttribute("id") || "";
      if (rawId === "0" || rawId === "1") continue;
      const isVertex = cell.getAttribute("vertex") === "1";
      const isEdge = cell.getAttribute("edge") === "1";
      const styleMap = parseStyleMap(cell.getAttribute("style") || "");
      const value = cell.getAttribute("value") || "";

      if (isVertex) {
        const geo = cell.querySelector("mxGeometry");
        const numId = ++state.nextId;
        cellIdToNodeId.set(rawId, numId);
        nodes.push({
          id: numId,
          title: value || `Node-${nodes.length + 1}`,
          body: styleMap.rfdBody ? decodeURIComponent(styleMap.rfdBody) : "",
          x: Number(geo?.getAttribute("x")) || 80 + (nodes.length % 4) * 210,
          y: Number(geo?.getAttribute("y")) || 80 + Math.floor(nodes.length / 4) * 130,
          w: Math.max(50, Number(geo?.getAttribute("width")) || 170),
          h: Math.max(40, Number(geo?.getAttribute("height")) || 78),
          fill: styleMap.fillColor || "#ffffff",
          stroke: styleMap.strokeColor || "#000080",
          shape: styleMap.shape || "card",
          icon: styleMap.rfdIcon || "none",
          iconPos: styleMap.rfdIconPos || "left",
          imgUrl: styleMap.rfdImg ? decodeURIComponent(styleMap.rfdImg) : "",
          z: nodes.length + 1,
        });
      } else if (isEdge) {
        edges.push({
          rawId,
          rawSource: cell.getAttribute("source") || "",
          rawTarget: cell.getAttribute("target") || "",
          label: value || "",
          anim: styleMap.rfdAnim || "pulse",
          kind: styleMap.rfdKind || "bezier",
          arrow: styleMap.rfdArrow || "one-way",
          color: styleMap.strokeColor || "#000080",
          width: Number(styleMap.strokeWidth) || 1.8,
          sourcePort: styleMap.rfdSrcPort || "right",
          targetPort: styleMap.rfdDstPort || "left",
        });
      }
    }

    const resolvedEdges = [];
    for (const ed of edges) {
      const srcId = cellIdToNodeId.get(ed.rawSource) ?? Number(String(ed.rawSource).replace(/\D+/g, ""));
      const dstId = cellIdToNodeId.get(ed.rawTarget) ?? Number(String(ed.rawTarget).replace(/\D+/g, ""));
      if (nodes.some((n) => n.id === srcId) && nodes.some((n) => n.id === dstId)) {
        resolvedEdges.push({
          id: ++state.nextId,
          source: srcId,
          sourcePort: ed.sourcePort,
          target: dstId,
          targetPort: ed.targetPort,
          label: ed.label,
          anim: ed.anim,
          kind: ed.kind,
          arrow: ed.arrow,
          color: ed.color,
          width: ed.width,
          bend: null,
        });
      }
    }

    loadedPages.push({
      id: pageIdCounter++,
      name: pageName,
      pan: { x: 60, y: 50 },
      zoom: 1.0,
      nodes,
      edges: resolvedEdges,
      undoStack: [],
      redoStack: [],
    });
  }

  return loadedPages;
}

function loadDiagramFromFile(file) {
  if (!file) return;
  const reader = new FileReader();
  reader.onload = () => {
    const content = String(reader.result || "");
    const baseName = file.name.replace(/\.[^.]+$/, "");
    try {
      let loadedPages = [];
      if (file.name.endsWith(".json")) {
        const data = JSON.parse(content);
        if (Array.isArray(data.pages) && data.pages.length > 0) {
          loadedPages = data.pages.map((pg, idx) => ({
            id: pg.id || idx + 1,
            name: pg.name || `Page-${idx + 1}`,
            pan: pg.pan || { x: 60, y: 50 },
            zoom: pg.zoom || 1.0,
            nodes: Array.isArray(pg.nodes) ? pg.nodes : [],
            edges: Array.isArray(pg.edges) ? pg.edges : [],
            undoStack: [],
            redoStack: [],
          }));
        } else if (Array.isArray(data.nodes)) {
          loadedPages = [
            {
              id: 1,
              name: `Page-1: ${baseName}`,
              pan: { x: 60, y: 50 },
              zoom: 1.0,
              nodes: data.nodes,
              edges: Array.isArray(data.edges) ? data.edges : [],
              undoStack: [],
              redoStack: [],
            },
          ];
        }
      } else {
        loadedPages = parseDrawioXmlToPages(content, baseName);
      }

      if (!loadedPages.length) {
        showToast(`Could not parse diagram from ${file.name}`);
        return;
      }

      for (const pg of loadedPages) {
        for (const n of pg.nodes) {
          if (n.id >= state.nextId) state.nextId = n.id + 10;
          if (n.imgUrl) getOrLoadImage(n.imgUrl);
        }
        for (const ed of pg.edges) {
          if (ed.id >= state.nextId) state.nextId = ed.id + 10;
        }
      }

      state.pages = loadedPages;
      state.activePageId = loadedPages[0].id;
      setSingleNodeSelection(loadedPages[0].nodes[0]?.id || null);
      invalidateGeometry();
      renderPageTabs();
      renderFormatPanel();
      updateZoomLabel();
      closeWelcomeModal();
      fitToScreen();
      showToast(`Loaded "${file.name}" (${loadedPages[0].nodes.length} nodes, ${loadedPages[0].edges.length} edges)`);
    } catch {
      showToast(`Failed to load ${file.name}`);
    }
  };
  reader.readAsText(file);
}

document.getElementById("btn-new-empty")?.addEventListener("click", () => {
  openWelcomeModal();
});
document.getElementById("btn-open-diagram-file")?.addEventListener("click", triggerOpenDiagramFile);
document.getElementById("btn-close-welcome")?.addEventListener("click", closeWelcomeModal);
document.getElementById("btn-welcome-empty")?.addEventListener("click", () => {
  createEmptyWorkspace(welcomeDocNameInput?.value || "Untitled Diagram");
});
welcomeDocNameInput?.addEventListener("keydown", (e) => {
  if (e.key === "Enter") {
    e.preventDefault();
    createEmptyWorkspace(welcomeDocNameInput.value);
  }
});
document.getElementById("btn-welcome-load-file")?.addEventListener("click", triggerOpenDiagramFile);
document.getElementById("btn-welcome-sample")?.addEventListener("click", loadSampleArchitectureWorkspace);

if (diagramFileInput) {
  diagramFileInput.addEventListener("change", (e) => {
    const file = e.target.files?.[0];
    if (file) loadDiagramFromFile(file);
  });
}

if (welcomeDropzone) {
  welcomeDropzone.addEventListener("click", triggerOpenDiagramFile);
  welcomeDropzone.addEventListener("dragover", (e) => {
    e.preventDefault();
    welcomeDropzone.classList.add("dragover");
  });
  welcomeDropzone.addEventListener("dragleave", () => {
    welcomeDropzone.classList.remove("dragover");
  });
  welcomeDropzone.addEventListener("drop", (e) => {
    e.preventDefault();
    welcomeDropzone.classList.remove("dragover");
    const file = e.dataTransfer?.files?.[0];
    if (file) loadDiagramFromFile(file);
  });
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
  { label: "Workspace: Create New Empty Space", shortcut: "⇧⌘N", run: () => createEmptyWorkspace("Untitled Diagram") },
  { label: "Workspace: Load Diagram from File (.drawio / .json)", shortcut: "⌘O", run: triggerOpenDiagramFile },
  { label: "Workspace: Open Startup Welcome Dialog", shortcut: "File", run: openWelcomeModal },
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
  { label: "Export: Animated GIF (.gif)", shortcut: "File", run: exportAnimatedGif },
  { label: "Export: High-DPI PNG (.png)", shortcut: "File", run: exportPngImage },
  { label: "Export: Save .drawio XML", shortcut: "Ctrl+S", run: exportDrawioXml },
  { label: "Export: Animated SVG", shortcut: "Ctrl+E", run: exportAnimatedSvg },
  { label: "Symbol Studio: Create or Import Custom Symbol", shortcut: "⇧⌘S", run: openSymbolStudioModal },
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

  if (mod && key === "o") {
    e.preventDefault();
    triggerOpenDiagramFile();
    return;
  }

  if (mod && e.shiftKey && key === "n") {
    e.preventDefault();
    createEmptyWorkspace("Untitled Diagram");
    return;
  }

  if (e.key === "Escape") {
    closeCommandPalette();
    closeShortcutsModal();
    closeSymbolStudioModal();
    closeWelcomeModal();
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
      state.theme === "apple90s"
        ? "#b8b4a9"
        : state.theme === "light"
        ? "#cbd5e1"
        : state.theme === "contrast"
        ? "#000000"
        : "#1e293b";

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

function drawArrowhead(ctx, fromPt, toPt, color, markerType = "classic") {
  if (!markerType || markerType === "none") return;
  const angle = Math.atan2(toPt.y - fromPt.y, toPt.x - fromPt.x);
  const headLen = 9.5;
  ctx.save();
  ctx.fillStyle = color;
  ctx.strokeStyle = color;
  ctx.lineWidth = 1.8;
  ctx.lineJoin = "round";
  ctx.lineCap = "round";

  if (markerType === "open") {
    ctx.beginPath();
    ctx.moveTo(toPt.x - headLen * Math.cos(angle - Math.PI / 6), toPt.y - headLen * Math.sin(angle - Math.PI / 6));
    ctx.lineTo(toPt.x, toPt.y);
    ctx.lineTo(toPt.x - headLen * Math.cos(angle + Math.PI / 6), toPt.y - headLen * Math.sin(angle + Math.PI / 6));
    ctx.stroke();
  } else if (markerType === "diamond") {
    const dLen = 11;
    const midX = toPt.x - (dLen * 0.5) * Math.cos(angle);
    const midY = toPt.y - (dLen * 0.5) * Math.sin(angle);
    const backX = toPt.x - dLen * Math.cos(angle);
    const backY = toPt.y - dLen * Math.sin(angle);
    const perpX = -Math.sin(angle) * 4.4;
    const perpY = Math.cos(angle) * 4.4;
    ctx.beginPath();
    ctx.moveTo(toPt.x, toPt.y);
    ctx.lineTo(midX + perpX, midY + perpY);
    ctx.lineTo(backX, backY);
    ctx.lineTo(midX - perpX, midY - perpY);
    ctx.closePath();
    ctx.fillStyle = isLightCanvasTheme() ? "#ffffff" : "#0f172a";
    ctx.fill();
    ctx.stroke();
  } else if (markerType === "circle") {
    const r = 4.2;
    const cx = toPt.x - r * Math.cos(angle);
    const cy = toPt.y - r * Math.sin(angle);
    ctx.beginPath();
    ctx.arc(cx, cy, r, 0, Math.PI * 2);
    ctx.fillStyle = isLightCanvasTheme() ? "#ffffff" : "#0f172a";
    ctx.fill();
    ctx.stroke();
  } else {
    // "classic" filled triangle
    ctx.beginPath();
    ctx.moveTo(toPt.x, toPt.y);
    ctx.lineTo(toPt.x - headLen * Math.cos(angle - Math.PI / 6.5), toPt.y - headLen * Math.sin(angle - Math.PI / 6.5));
    ctx.lineTo(toPt.x - headLen * Math.cos(angle + Math.PI / 6.5), toPt.y - headLen * Math.sin(angle + Math.PI / 6.5));
    ctx.closePath();
    ctx.fill();
  }
  ctx.restore();
}

function drawEdgeArrowheads(ctx, e, geo, strokeColor) {
  const markers = getEdgeArrowMarkers(e);
  if (markers.end && markers.end !== "none") {
    const prevPt = geo.samples[Math.max(0, geo.samples.length - 2)];
    drawArrowhead(ctx, prevPt, geo.p3, strokeColor, markers.end);
  }
  if (markers.start && markers.start !== "none") {
    const nextPt = geo.samples[Math.min(geo.samples.length - 1, 1)];
    drawArrowhead(ctx, nextPt, geo.p0, strokeColor, markers.start);
  }
}

let frameCounter = 0;

function renderLoop(timestamp) {
  const dt = Math.max(1, timestamp - state.lastFrameTime);
  state.fps = Math.round(state.fps * 0.92 + (1000 / dt) * 0.08);
  state.lastFrameTime = timestamp;

  const timeSec = timestamp * 0.001;
  const p = page();
  const lightCanvas = isLightCanvasTheme();
  const selectColor = state.theme === "apple90s" ? "#0055cc" : state.theme === "contrast" ? "#000000" : "#38bdf8";

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

  ctx.fillStyle =
    state.theme === "apple90s"
      ? "#f5f3ec"
      : state.theme === "light"
      ? "#f8fafc"
      : state.theme === "contrast"
      ? "#ffffff"
      : "#0d121d";
  ctx.fillRect(0, 0, canvas.width, canvas.height);

  drawInfiniteGridFast(p);

  const viewTL = screenToWorld(0, 0);
  const viewBR = screenToWorld(canvas.width, canvas.height);

  ctx.save();
  ctx.translate(p.pan.x, p.pan.y);
  ctx.scale(p.zoom, p.zoom);

  if (state.smartGuides.length) {
    ctx.strokeStyle = "#e03a3e";
    ctx.lineWidth = 1.2 / p.zoom;
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

  ctx.font = '700 11px "Charcoal", "Chicago", "Geneva", sans-serif';
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
    const strokeColor = isSelected ? selectColor : e.color || "#000080";

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

    drawEdgeArrowheads(ctx, e, geo, strokeColor);

    if (e.anim === "pulse") {
      const speedPxPerSec = 140;
      const spacing = geo.totalLen * 0.5;
      const isTwoWay = (e.arrow || "one-way") === "two-way";
      for (let pIdx = 0; pIdx < 2; pIdx++) {
        const rawDist = (timeSec * speedPxPerSec + pIdx * spacing) % geo.totalLen;
        const headDist = isTwoWay && pIdx === 1 ? geo.totalLen - rawDist : rawDist;
        const dirSign = isTwoWay && pIdx === 1 ? 1 : -1;
        const tail1 = getPointAtArcDistance(geo, headDist + dirSign * 10);
        const tail2 = getPointAtArcDistance(geo, headDist + dirSign * 5);
        const head = getPointAtArcDistance(geo, headDist);

        ctx.fillStyle = lightCanvas ? "rgba(0, 85, 204, 0.16)" : "rgba(56, 189, 248, 0.16)";
        ctx.beginPath();
        ctx.arc(tail1.x, tail1.y, 3.6, 0, Math.PI * 2);
        ctx.fill();

        ctx.fillStyle = lightCanvas ? "rgba(0, 85, 204, 0.32)" : "rgba(56, 189, 248, 0.32)";
        ctx.beginPath();
        ctx.arc(tail2.x, tail2.y, 4.6, 0, Math.PI * 2);
        ctx.fill();

        ctx.fillStyle = lightCanvas ? strokeColor : "#e0f2fe";
        ctx.beginPath();
        ctx.arc(head.x, head.y, 3.2, 0, Math.PI * 2);
        ctx.fill();
      }
    }

    if (e.label && e.label.trim().length > 0) {
      const textW = geo.labelWidth;
      ctx.fillStyle = lightCanvas ? "#ffffff" : "#161b26";
      ctx.strokeStyle = strokeColor;
      ctx.lineWidth = 1.25;
      ctx.beginPath();
      ctx.roundRect(geo.midPoint.x - textW * 0.5, geo.midPoint.y - 9.5, textW, 19, 3);
      ctx.fill();
      ctx.stroke();
      ctx.fillStyle = lightCanvas ? "#000000" : "#f1f5f9";
      ctx.fillText(e.label, geo.midPoint.x, geo.midPoint.y + 3.5);
    }

    if (e.id === state.selectedEdgeId) {
      ctx.strokeStyle = "rgba(245, 130, 31, 0.55)";
      ctx.setLineDash([3, 3]);
      ctx.beginPath();
      ctx.moveTo(geo.p0.x, geo.p0.y);
      ctx.lineTo(geo.midPoint.x, geo.midPoint.y);
      ctx.lineTo(geo.p3.x, geo.p3.y);
      ctx.stroke();
      ctx.setLineDash([]);

      ctx.beginPath();
      ctx.arc(geo.midPoint.x, geo.midPoint.y, 6, 0, Math.PI * 2);
      ctx.fillStyle = "#fdb827";
      ctx.strokeStyle = "#000000";
      ctx.lineWidth = 1.6;
      ctx.fill();
      ctx.stroke();

      for (const ep of [geo.p0, geo.p3]) {
        ctx.beginPath();
        ctx.arc(ep.x, ep.y, 5.5, 0, Math.PI * 2);
        ctx.fillStyle = "#009ddc";
        ctx.strokeStyle = "#000000";
        ctx.lineWidth = 1.6;
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
    ctx.strokeStyle = state.snapTargetPort ? "#2d8a1e" : selectColor;
    ctx.lineWidth = 2;
    ctx.setLineDash([5, 4]);
    ctx.stroke();
    ctx.setLineDash([]);

    if (state.snapTargetPort) {
      ctx.beginPath();
      ctx.arc(state.snapTargetPort.port.x, state.snapTargetPort.port.y, 10, 0, Math.PI * 2);
      ctx.strokeStyle = "#2d8a1e";
      ctx.lineWidth = 2.5;
      ctx.stroke();
    }
  }

  // Frustum-Culled Minimalist Nodes with Apple 90s Platinum Hard Drop Shadow
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
      ctx.strokeStyle = lightCanvas ? "rgba(0, 85, 204, 0.25)" : "rgba(56, 189, 248, 0.22)";
      ctx.lineWidth = 4;
      ctx.strokeRect(n.x - 2.5, n.y - 2.5, n.w + 5, n.h + 5);
    }

    ctx.save();
    if (state.theme === "apple90s" || state.theme === "contrast") {
      ctx.shadowColor = state.theme === "contrast" ? "rgba(0, 0, 0, 0.85)" : "rgba(0, 0, 0, 0.18)";
      ctx.shadowOffsetX = 3 * p.zoom;
      ctx.shadowOffsetY = 3 * p.zoom;
      ctx.shadowBlur = 0;
    }
    ctx.fillStyle = n.fill;
    ctx.strokeStyle = selected ? selectColor : n.stroke;
    ctx.lineWidth = selected ? 2.1 : 1.65;
    drawNodeShapeGeometry(ctx, n, false);
    ctx.restore();

    for (const port of getNodePorts(n)) {
      const isHovered =
        state.hoveredPort && state.hoveredPort.nodeId === n.id && state.hoveredPort.side === port.side;
      ctx.beginPath();
      ctx.arc(port.x, port.y, isHovered ? 6.5 : 4, 0, Math.PI * 2);
      ctx.fillStyle = isHovered ? "#61bb46" : lightCanvas ? "#ffffff" : "#090d16";
      ctx.strokeStyle = isHovered ? "#000000" : lightCanvas ? "#000000" : "#64748b";
      ctx.lineWidth = 1.5;
      ctx.fill();
      ctx.stroke();
    }
  }

  // Batched Title Text Pass (Apple 90s Charcoal / Chicago Typography)
  ctx.font = '700 12.5px "Charcoal", "Chicago", "Geneva", sans-serif';
  ctx.textAlign = "center";
  for (let i = 0; i < visibleNodes.length; i++) {
    const n = visibleNodes[i];
    const pos = getNodeTextPlacement(n);
    const darkFill = isDarkHexColor(n.fill);
    ctx.fillStyle = pos.external
      ? lightCanvas
        ? "#000000"
        : "#f1f5f9"
      : darkFill
      ? "#f1f5f9"
      : "#000000";
    ctx.fillText(n.title, pos.cx, pos.titleY);
  }

  // Batched Subtitle Text Pass (Monaco Monospace)
  ctx.font = '600 10.5px "Monaco", "Menlo", monospace';
  for (let i = 0; i < visibleNodes.length; i++) {
    const n = visibleNodes[i];
    if (!n.body) continue;
    const pos = getNodeTextPlacement(n);
    if (pos.hideBody) continue;
    const darkFill = isDarkHexColor(n.fill);
    ctx.fillStyle = pos.external
      ? lightCanvas
        ? "#4a4a4a"
        : "#94a3b8"
      : darkFill
      ? "#94a3b8"
      : "#4a4a4a";
    ctx.fillText(n.body, pos.cx, pos.bodyY);
  }

  // Single-Selected Node Resize & Quick-Connect Overlay
  const selectedNode = nodeMap.get(state.selectedNodeId);
  if (selectedNode && state.selectedNodeIds.size === 1) {
    ctx.fillStyle = selectColor;
    ctx.fillRect(selectedNode.x + selectedNode.w - 4.5, selectedNode.y + selectedNode.h - 4.5, 9, 9);

    ctx.font = '700 10.5px "Charcoal", sans-serif';
    for (const arrow of getQuickArrows(selectedNode)) {
      ctx.beginPath();
      ctx.arc(arrow.x, arrow.y, 8.5, 0, Math.PI * 2);
      ctx.fillStyle = lightCanvas ? "#ffffff" : "#0f1420";
      ctx.strokeStyle = selectColor;
      ctx.lineWidth = 1.4;
      ctx.fill();
      ctx.stroke();
      ctx.fillStyle = selectColor;
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
      body: "",
    };
    ctx.save();
    ctx.globalAlpha = 0.82;
    ctx.fillStyle = lightCanvas ? "#ffffff" : "#131b2e";
    ctx.strokeStyle = selectColor;
    ctx.lineWidth = 2;
    drawNodeShapeGeometry(ctx, ghostNode, false);

    ctx.setLineDash([4, 4]);
    ctx.strokeStyle = selectColor;
    ctx.strokeRect(ghostX - 4, ghostY - 4, item.w + 8, item.h + 8);
    ctx.setLineDash([]);

    const pos = getNodeTextPlacement(ghostNode);
    ctx.font = '700 12.5px "Charcoal", "Chicago", sans-serif';
    ctx.fillStyle = lightCanvas ? "#000000" : "#f1f5f9";
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
    ctx.fillStyle = lightCanvas ? "rgba(0, 85, 204, 0.1)" : "rgba(56, 189, 248, 0.1)";
    ctx.strokeStyle = selectColor;
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
      minimapCtx.fillStyle = state.selectedNodeIds.has(n.id) ? selectColor : lightCanvas ? "#777777" : "#475569";
      minimapCtx.fillRect(n.x, n.y, n.w, n.h);
    }
    minimapCtx.strokeStyle = selectColor;
    minimapCtx.lineWidth = 10;
    minimapCtx.strokeRect(viewTL.x, viewTL.y, viewBR.x - viewTL.x, viewBR.y - viewTL.y);
    minimapCtx.restore();
  }

  if (frameCounter % 4 === 0) {
    syncArrangeInputsIfDirty();
  }

  if (frameCounter % 12 === 0) {
    statusBarEl.textContent = `${p.nodes.length} items, ${p.edges.length} flows | Selected: ${state.selectedNodeIds.size + state.selectedEdgeIds.size} | Zoom: ${Math.round(p.zoom * 100)}% | ${state.fps} FPS`;
  }

  requestAnimationFrame(renderLoop);
}
requestAnimationFrame(renderLoop);
