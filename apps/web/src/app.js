// Interactive Web Frontend for RustFlow Draw
const STENCILS = [
  { id: "flowchart.process", label: "Process Card", category: "Flowchart", w: 170, h: 76, shape: "card" },
  { id: "flowchart.decision", label: "Decision Diamond", category: "Flowchart", w: 130, h: 96, shape: "diamond" },
  { id: "flowchart.database", label: "Database Cylinder", category: "Flowchart", w: 130, h: 86, shape: "card" },
  { id: "uml.class", label: "UML Class", category: "UML & BPMN", w: 180, h: 96, shape: "card" },
  { id: "bpmn.task", label: "BPMN Task", category: "UML & BPMN", w: 165, h: 76, shape: "card" },
  { id: "cloud.aws.lambda", label: "AWS Lambda", category: "Cloud (AWS/GCP/Azure)", w: 170, h: 78, shape: "card" },
  { id: "cloud.gcp.cloudrun", label: "GCP Cloud Run", category: "Cloud (AWS/GCP/Azure)", w: 170, h: 78, shape: "card" },
  { id: "cloud.azure.aks", label: "Azure AKS Cluster", category: "Cloud (AWS/GCP/Azure)", w: 175, h: 78, shape: "card" },
];

function createInitialPages() {
  return [
    {
      id: 1,
      name: "Page-1: Architecture",
      pan: { x: 80, y: 60 },
      zoom: 1.0,
      nodes: [
        { id: 1, title: "API Gateway", body: "Rust + WebGPU", x: 100, y: 120, w: 180, h: 80, fill: "#ffffff", stroke: "#6366f1", shape: "card" },
        { id: 2, title: "Auth Worker", body: "JWT Verification", x: 380, y: 60, w: 180, h: 80, fill: "#ffffff", stroke: "#6366f1", shape: "card" },
        { id: 3, title: "Core Scene Engine", body: "R-Tree + Undo Stack", x: 380, y: 200, w: 180, h: 80, fill: "#ffffff", stroke: "#10b981", shape: "card" },
        { id: 4, title: "Event Store DB", body: "Local-First CRDT", x: 660, y: 130, w: 180, h: 80, fill: "#ffffff", stroke: "#f59e0b", shape: "card" },
      ],
      edges: [
        { id: 101, source: 1, target: 2, label: "HTTPS", anim: "pulse" },
        { id: 102, source: 1, target: 3, label: "gRPC", anim: "ants" },
        { id: 103, source: 2, target: 4, label: "Audit", anim: "pulse" },
        { id: 104, source: 3, target: 4, label: "Snapshots", anim: "ants" },
      ],
      undoStack: [],
      redoStack: [],
    },
    {
      id: 2,
      name: "Page-2: Sandbox",
      pan: { x: 80, y: 60 },
      zoom: 1.0,
      nodes: [
        { id: 10, title: "Ingress Node", body: "Double-click or edit right", x: 160, y: 160, w: 180, h: 80, fill: "#ffffff", stroke: "#6366f1", shape: "card" },
      ],
      edges: [],
      undoStack: [],
      redoStack: [],
    },
  ];
}

const state = {
  pages: createInitialPages(),
  activePageId: 1,
  selectedNodeId: 1,
  draggingNode: null,
  panning: null,
  connectingFromNode: null,
  mouseWorld: { x: 0, y: 0 },
  smartGuides: [],
  nextId: 200,
  edgeAnimPreset: "pulse",
};

function page() {
  return state.pages.find((p) => p.id === state.activePageId) || state.pages[0];
}

function saveSnapshot() {
  const p = page();
  p.undoStack.push(JSON.stringify({ nodes: p.nodes, edges: p.edges }));
  if (p.undoStack.length > 64) p.undoStack.shift();
  p.redoStack = [];
}

const canvas = document.getElementById("rustflow-canvas");
const ctx = canvas.getContext("2d");
const minimapCanvas = document.getElementById("minimap-canvas");
const minimapCtx = minimapCanvas.getContext("2d");

function resizeCanvas() {
  const rect = canvas.parentElement.getBoundingClientRect();
  canvas.width = rect.width;
  canvas.height = rect.height;
}
window.addEventListener("resize", resizeCanvas);
resizeCanvas();

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
        guides.push({ type: "v", x: ox, y1: Math.min(movingNode.y, other.y) - 20, y2: Math.max(movingNode.y + movingNode.h, other.y + other.h) + 20 });
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
        guides.push({ type: "h", y: oy, x1: Math.min(movingNode.x, other.x) - 20, x2: Math.max(movingNode.x + movingNode.w, other.x + other.w) + 20 });
        break;
      }
    }
  }
  return guides;
}

// Pointer interaction
canvas.addEventListener("mousedown", (e) => {
  const rect = canvas.getBoundingClientRect();
  const sx = e.clientX - rect.left;
  const sy = e.clientY - rect.top;
  const wpt = screenToWorld(sx, sy);
  const p = page();

  // Check port handle click first (connection drag)
  for (const n of p.nodes) {
    for (const port of getNodePorts(n)) {
      if (Math.hypot(wpt.x - port.x, wpt.y - port.y) <= 9) {
        state.connectingFromNode = { node: n, port };
        return;
      }
    }
  }

  // Check node hit-test
  const hit = [...p.nodes].reverse().find((n) => wpt.x >= n.x && wpt.x <= n.x + n.w && wpt.y >= n.y && wpt.y <= n.y + n.h);
  if (hit) {
    saveSnapshot();
    state.selectedNodeId = hit.id;
    state.draggingNode = { node: hit, offsetX: wpt.x - hit.x, offsetY: wpt.y - hit.y };
    syncInspector();
  } else {
    state.selectedNodeId = null;
    state.panning = { startX: sx - p.pan.x, startY: sy - p.pan.y };
    syncInspector();
  }
});

window.addEventListener("mousemove", (e) => {
  const rect = canvas.getBoundingClientRect();
  const sx = e.clientX - rect.left;
  const sy = e.clientY - rect.top;
  state.mouseWorld = screenToWorld(sx, sy);
  const p = page();

  if (state.draggingNode) {
    const { node, offsetX, offsetY } = state.draggingNode;
    node.x = Math.round((state.mouseWorld.x - offsetX) / 10) * 10;
    node.y = Math.round((state.mouseWorld.y - offsetY) / 10) * 10;
    state.smartGuides = computeSmartGuides(node);
    syncInspector();
  } else if (state.panning) {
    p.pan.x = sx - state.panning.startX;
    p.pan.y = sy - state.panning.startY;
  }
});

window.addEventListener("mouseup", () => {
  const p = page();
  if (state.connectingFromNode) {
    const src = state.connectingFromNode.node;
    const dst = p.nodes.find(
      (n) =>
        n.id !== src.id &&
        state.mouseWorld.x >= n.x - 12 &&
        state.mouseWorld.x <= n.x + n.w + 12 &&
        state.mouseWorld.y >= n.y - 12 &&
        state.mouseWorld.y <= n.y + n.h + 12
    );
    if (dst) {
      saveSnapshot();
      p.edges.push({
        id: ++state.nextId,
        source: src.id,
        target: dst.id,
        label: "Flow",
        anim: state.edgeAnimPreset,
      });
    }
  }
  state.draggingNode = null;
  state.panning = null;
  state.connectingFromNode = null;
  state.smartGuides = [];
});

canvas.addEventListener(
  "wheel",
  (e) => {
    e.preventDefault();
    const rect = canvas.getBoundingClientRect();
    const cx = e.clientX - rect.left;
    const cy = e.clientY - rect.top;
    const p = page();
    const before = screenToWorld(cx, cy);
    const factor = e.deltaY < 0 ? 1.12 : 0.89;
    p.zoom = Math.min(8.0, Math.max(0.1, p.zoom * factor));
    p.pan.x = cx - before.x * p.zoom;
    p.pan.y = cy - before.y * p.zoom;
  },
  { passive: false }
);

// Palette rendering & search
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
    section.style.marginBottom = "12px";
    const title = document.createElement("div");
    title.className = "palette-section-title";
    title.textContent = cat;
    section.appendChild(title);

    const grid = document.createElement("div");
    grid.className = "stencil-grid";
    for (const item of items) {
      const card = document.createElement("div");
      card.className = "stencil-card";
      card.textContent = "+ " + item.label;
      card.onclick = () => {
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
          fill: "#ffffff",
          stroke: "#6366f1",
          shape: item.shape,
        };
        p.nodes.push(newNode);
        state.selectedNodeId = newNode.id;
        syncInspector();
      };
      grid.appendChild(card);
    }
    section.appendChild(grid);
    container.appendChild(section);
  }
}
document.getElementById("palette-search").addEventListener("input", (e) => renderPalette(e.target.value));
renderPalette();

// Page tabs
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
      renderPageTabs();
      syncInspector();
    };
    container.appendChild(tab);
  }
  const addBtn = document.createElement("button");
  addBtn.textContent = "+ Page";
  addBtn.onclick = () => {
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
    renderPageTabs();
    syncInspector();
  };
  container.appendChild(addBtn);
}
renderPageTabs();

// Inspector synchronization
function syncInspector() {
  const n = page().nodes.find((item) => item.id === state.selectedNodeId);
  const titleEl = document.getElementById("insp-title");
  const bodyEl = document.getElementById("insp-body");
  const fillEl = document.getElementById("insp-fill");
  const strokeEl = document.getElementById("insp-stroke");
  const xEl = document.getElementById("insp-x");
  const yEl = document.getElementById("insp-y");
  const wEl = document.getElementById("insp-w");
  const hEl = document.getElementById("insp-h");

  if (!n) {
    titleEl.value = "";
    bodyEl.value = "";
    xEl.value = "";
    yEl.value = "";
    wEl.value = "";
    hEl.value = "";
    return;
  }
  titleEl.value = n.title;
  bodyEl.value = n.body || "";
  fillEl.value = n.fill;
  strokeEl.value = n.stroke;
  xEl.value = Math.round(n.x);
  yEl.value = Math.round(n.y);
  wEl.value = Math.round(n.w);
  hEl.value = Math.round(n.h);
}
syncInspector();

for (const [id, key, isNum] of [
  ["insp-title", "title", false],
  ["insp-body", "body", false],
  ["insp-fill", "fill", false],
  ["insp-stroke", "stroke", false],
  ["insp-x", "x", true],
  ["insp-y", "y", true],
  ["insp-w", "w", true],
  ["insp-h", "h", true],
]) {
  document.getElementById(id).addEventListener("input", (e) => {
    const n = page().nodes.find((item) => item.id === state.selectedNodeId);
    if (!n) return;
    n[key] = isNum ? Number(e.target.value) || 40 : e.target.value;
  });
}

// Toolbar buttons & shortcuts
function undo() {
  const p = page();
  if (!p.undoStack.length) return;
  p.redoStack.push(JSON.stringify({ nodes: p.nodes, edges: p.edges }));
  const prev = JSON.parse(p.undoStack.pop());
  p.nodes = prev.nodes;
  p.edges = prev.edges;
  syncInspector();
}

function redo() {
  const p = page();
  if (!p.redoStack.length) return;
  p.undoStack.push(JSON.stringify({ nodes: p.nodes, edges: p.edges }));
  const next = JSON.parse(p.redoStack.pop());
  p.nodes = next.nodes;
  p.edges = next.edges;
  syncInspector();
}

function duplicateSelected() {
  const p = page();
  const n = p.nodes.find((item) => item.id === state.selectedNodeId);
  if (!n) return;
  saveSnapshot();
  const copy = { ...n, id: ++state.nextId, x: n.x + 30, y: n.y + 30 };
  p.nodes.push(copy);
  state.selectedNodeId = copy.id;
  syncInspector();
}

function deleteSelected() {
  const p = page();
  if (!state.selectedNodeId) return;
  saveSnapshot();
  p.nodes = p.nodes.filter((n) => n.id !== state.selectedNodeId);
  p.edges = p.edges.filter((e) => e.source !== state.selectedNodeId && e.target !== state.selectedNodeId);
  state.selectedNodeId = null;
  syncInspector();
}

document.getElementById("btn-undo").onclick = undo;
document.getElementById("btn-redo").onclick = redo;
document.getElementById("btn-duplicate").onclick = duplicateSelected;
document.getElementById("btn-delete").onclick = deleteSelected;
document.getElementById("select-edge-anim").onchange = (e) => {
  state.edgeAnimPreset = e.target.value;
};
document.getElementById("btn-zoom-in").onclick = () => {
  page().zoom = Math.min(8.0, page().zoom * 1.2);
};
document.getElementById("btn-zoom-out").onclick = () => {
  page().zoom = Math.max(0.1, page().zoom / 1.2);
};

document.getElementById("btn-layout-hier").onclick = () => {
  const p = page();
  saveSnapshot();
  const layers = {};
  p.nodes.forEach((n) => (layers[n.id] = 0));
  for (let i = 0; i < p.nodes.length; i++) {
    for (const e of p.edges) {
      if ((layers[e.target] || 0) <= (layers[e.source] || 0)) {
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
      n.x = 120 + Number(layerIdx) * 260;
      n.y = 100 + sibIdx * 130;
    });
  });
  syncInspector();
};

document.getElementById("btn-layout-grid").onclick = () => {
  const p = page();
  saveSnapshot();
  p.nodes.forEach((n, idx) => {
    const col = idx % 3;
    const row = Math.floor(idx / 3);
    n.x = 100 + col * 240;
    n.y = 100 + row * 140;
  });
  syncInspector();
};

function downloadFile(filename, content, mime) {
  const blob = new Blob([content], { type: mime });
  const a = document.createElement("a");
  a.href = URL.createObjectURL(blob);
  a.download = filename;
  a.click();
}

document.getElementById("btn-export-drawio").onclick = () => {
  const p = page();
  let xml = `<mxfile host="RustFlowDraw" version="1.0.0">\n  <diagram id="page-${p.id}" name="${p.name}">\n    <mxGraphModel dx="1280" dy="720" grid="1" gridSize="20">\n      <root>\n        <mxCell id="0"/>\n        <mxCell id="1" parent="0"/>\n`;
  for (const n of p.nodes) {
    xml += `        <mxCell id="node-${n.id}" value="${n.title}" style="rounded=1;fillColor=${n.fill};strokeColor=${n.stroke};" vertex="1" parent="1">\n          <mxGeometry x="${n.x}" y="${n.y}" width="${n.w}" height="${n.h}" as="geometry"/>\n        </mxCell>\n`;
  }
  for (const e of p.edges) {
    xml += `        <mxCell id="edge-${e.id}" value="${e.label}" style="curved=1;rfdAnim=${e.anim};" edge="1" parent="1" source="node-${e.source}" target="node-${e.target}">\n          <mxGeometry relative="1" as="geometry"/>\n        </mxCell>\n`;
  }
  xml += `      </root>\n    </mxGraphModel>\n  </diagram>\n</mxfile>`;
  downloadFile("diagram.drawio", xml, "application/xml");
};

document.getElementById("btn-export-svg").onclick = () => {
  const p = page();
  let svg = `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 1000 600" width="1000" height="600">\n  <style>\n    @keyframes rfd-ants { to { stroke-dashoffset: -28; } }\n    .ants { stroke-dasharray: 9 5; animation: rfd-ants 0.8s linear infinite; }\n    text { font-family: Inter, sans-serif; font-size: 13px; font-weight: 600; fill: #0f172a; }\n  </style>\n`;
  for (const e of p.edges) {
    const src = p.nodes.find((n) => n.id === e.source);
    const dst = p.nodes.find((n) => n.id === e.target);
    if (!src || !dst) continue;
    const x1 = src.x + src.w, y1 = src.y + src.h / 2;
    const x2 = dst.x, y2 = dst.y + dst.h / 2;
    const cx1 = x1 + 80, cx2 = x2 - 80;
    const d = `M ${x1} ${y1} C ${cx1} ${y1}, ${cx2} ${y2}, ${x2} ${y2}`;
    svg += `  <path d="${d}" fill="none" stroke="#6366f1" stroke-width="2" class="${e.anim === "ants" ? "ants" : ""}"/>\n`;
  }
  for (const n of p.nodes) {
    svg += `  <rect x="${n.x}" y="${n.y}" width="${n.w}" height="${n.h}" rx="10" fill="${n.fill}" stroke="${n.stroke}" stroke-width="2"/>\n`;
    svg += `  <text x="${n.x + n.w / 2}" y="${n.y + n.h / 2}" text-anchor="middle" dominant-baseline="middle">${n.title}</text>\n`;
  }
  svg += `</svg>`;
  downloadFile("diagram-animated.svg", svg, "image/svg+xml");
};

window.addEventListener("keydown", (e) => {
  if (e.target.tagName === "INPUT") return;
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
  }
});

// Cubic Bezier helper
function evalBezier(p0, p1, p2, p3, t) {
  const u = 1 - t;
  return {
    x: u * u * u * p0.x + 3 * u * u * t * p1.x + 3 * u * t * t * p2.x + t * t * t * p3.x,
    y: u * u * u * p0.y + 3 * u * u * t * p1.y + 3 * u * t * t * p2.y + t * t * t * p3.y,
  };
}

// Main 60-120 FPS render loop
function renderLoop(timestamp) {
  const timeSec = timestamp * 0.001;
  const p = page();
  ctx.clearRect(0, 0, canvas.width, canvas.height);

  // 1. Infinite pan-zoom dot grid
  const gridStep = 20 * p.zoom;
  if (gridStep >= 6) {
    ctx.fillStyle = "#cbd5e1";
    const startX = ((p.pan.x % gridStep) + gridStep) % gridStep;
    const startY = ((p.pan.y % gridStep) + gridStep) % gridStep;
    for (let x = startX; x < canvas.width; x += gridStep) {
      for (let y = startY; y < canvas.height; y += gridStep) {
        ctx.fillRect(x - 1, y - 1, 2, 2);
      }
    }
  }

  ctx.save();
  ctx.translate(p.pan.x, p.pan.y);
  ctx.scale(p.zoom, p.zoom);

  // 2. Smart Alignment Guides
  ctx.strokeStyle = "#ec4899";
  ctx.lineWidth = 1 / p.zoom;
  ctx.setLineDash([4 / p.zoom, 4 / p.zoom]);
  for (const g of state.smartGuides) {
    ctx.beginPath();
    if (g.type === "v") {
      ctx.moveTo(g.x, g.y1);
      ctx.lineTo(g.x, g.y2);
    } else {
      ctx.moveTo(g.x1, g.y);
      ctx.lineTo(g.x2, g.y);
    }
    ctx.stroke();
  }
  ctx.setLineDash([]);

  // 3. Edges & GPU-style Animations (Marching Ants & Pulse Particles)
  for (const e of p.edges) {
    const src = p.nodes.find((n) => n.id === e.source);
    const dst = p.nodes.find((n) => n.id === e.target);
    if (!src || !dst) continue;

    const p0 = { x: src.x + src.w, y: src.y + src.h / 2 };
    const p3 = { x: dst.x, y: dst.y + dst.h / 2 };
    const span = Math.max(60, Math.abs(p3.x - p0.x) * 0.45);
    const p1 = { x: p0.x + span, y: p0.y };
    const p2 = { x: p3.x - span, y: p3.y };

    ctx.beginPath();
    ctx.moveTo(p0.x, p0.y);
    ctx.bezierCurveTo(p1.x, p1.y, p2.x, p2.y, p3.x, p3.y);
    ctx.strokeStyle = "#6366f1";
    ctx.lineWidth = 2.2;

    if (e.anim === "ants") {
      ctx.setLineDash([9, 5]);
      ctx.lineDashOffset = -timeSec * 36;
    } else {
      ctx.setLineDash([]);
    }
    ctx.stroke();
    ctx.setLineDash([]);

    if (e.anim === "pulse") {
      for (const offset of [0, 0.5]) {
        const t = (timeSec * 0.55 + offset) % 1;
        const pt = evalBezier(p0, p1, p2, p3, t);
        ctx.beginPath();
        ctx.arc(pt.x, pt.y, 4.5, 0, Math.PI * 2);
        ctx.fillStyle = "#4f46e5";
        ctx.shadowColor = "#6366f1";
        ctx.shadowBlur = 10;
        ctx.fill();
        ctx.shadowBlur = 0;
      }
    }
  }

  // Active connection line
  if (state.connectingFromNode) {
    ctx.beginPath();
    ctx.moveTo(state.connectingFromNode.port.x, state.connectingFromNode.port.y);
    ctx.lineTo(state.mouseWorld.x, state.mouseWorld.y);
    ctx.strokeStyle = "#6366f1";
    ctx.setLineDash([5, 5]);
    ctx.stroke();
    ctx.setLineDash([]);
  }

  // 4. React Flow Node Cards & Ports
  for (const n of p.nodes) {
    const selected = n.id === state.selectedNodeId;
    ctx.save();
    ctx.shadowColor = selected ? "rgba(99, 102, 241, 0.28)" : "rgba(15, 23, 42, 0.08)";
    ctx.shadowBlur = selected ? 16 : 10;
    ctx.shadowOffsetY = 4;

    ctx.fillStyle = n.fill;
    ctx.strokeStyle = selected ? "#4f46e5" : n.stroke;
    ctx.lineWidth = selected ? 2.5 : 1.5;

    ctx.beginPath();
    if (n.shape === "diamond") {
      ctx.moveTo(n.x + n.w / 2, n.y);
      ctx.lineTo(n.x + n.w, n.y + n.h / 2);
      ctx.lineTo(n.x + n.w / 2, n.y + n.h);
      ctx.lineTo(n.x, n.y + n.h / 2);
      ctx.closePath();
    } else {
      ctx.roundRect(n.x, n.y, n.w, n.h, 10);
    }
    ctx.fill();
    ctx.stroke();
    ctx.restore();

    // Header bar for React Flow cards
    if (n.shape !== "diamond") {
      ctx.fillStyle = "#f1f5f9";
      ctx.beginPath();
      ctx.roundRect(n.x + 1, n.y + 1, n.w - 2, 26, [9, 9, 0, 0]);
      ctx.fill();
    }

    ctx.fillStyle = "#0f172a";
    ctx.font = "600 13px Inter, sans-serif";
    ctx.textAlign = "center";
    ctx.fillText(n.title, n.x + n.w / 2, n.shape === "diamond" ? n.y + n.h / 2 + 4 : n.y + 18);

    if (n.body && n.shape !== "diamond") {
      ctx.fillStyle = "#64748b";
      ctx.font = "400 11px Inter, sans-serif";
      ctx.fillText(n.body, n.x + n.w / 2, n.y + 52);
    }

    // Ports / Handles
    for (const port of getNodePorts(n)) {
      ctx.beginPath();
      ctx.arc(port.x, port.y, 4.5, 0, Math.PI * 2);
      ctx.fillStyle = "#ffffff";
      ctx.strokeStyle = "#6366f1";
      ctx.lineWidth = 2;
      ctx.fill();
      ctx.stroke();
    }
  }

  ctx.restore();

  // 5. Render Minimap
  minimapCtx.clearRect(0, 0, minimapCanvas.width, minimapCanvas.height);
  const scale = 0.14;
  minimapCtx.save();
  minimapCtx.translate(16, 16);
  minimapCtx.scale(scale, scale);
  for (const n of p.nodes) {
    minimapCtx.fillStyle = n.id === state.selectedNodeId ? "#6366f1" : "#94a3b8";
    minimapCtx.fillRect(n.x, n.y, n.w, n.h);
  }
  const tl = screenToWorld(0, 0);
  const br = screenToWorld(canvas.width, canvas.height);
  minimapCtx.strokeStyle = "#4f46e5";
  minimapCtx.lineWidth = 10;
  minimapCtx.strokeRect(tl.x, tl.y, br.x - tl.x, br.y - tl.y);
  minimapCtx.restore();

  document.getElementById("status-bar").textContent = `Zoom: ${Math.round(p.zoom * 100)}% | Nodes: ${p.nodes.length} | Edges: ${p.edges.length}`;
  requestAnimationFrame(renderLoop);
}
requestAnimationFrame(renderLoop);
