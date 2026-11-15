// Developer-Focused Interactive Web App for RustFlow Draw (All 6 SDLC Phases)
const STENCILS = [
  { id: "flowchart.process", label: "Process Card", category: "Flowchart", w: 175, h: 78, shape: "card" },
  { id: "flowchart.decision", label: "Decision Diamond", category: "Flowchart", w: 135, h: 98, shape: "diamond" },
  { id: "flowchart.database", label: "Database Cylinder", category: "Flowchart", w: 150, h: 82, shape: "card" },
  { id: "uml.class", label: "UML Class", category: "UML & BPMN", w: 185, h: 92, shape: "card" },
  { id: "bpmn.task", label: "BPMN Service Task", category: "UML & BPMN", w: 170, h: 78, shape: "card" },
  { id: "cloud.aws.lambda", label: "AWS Lambda", category: "Cloud & K8s", w: 175, h: 78, shape: "card" },
  { id: "cloud.gcp.cloudrun", label: "GCP Cloud Run", category: "Cloud & K8s", w: 175, h: 78, shape: "card" },
  { id: "cloud.k8s.pod", label: "K8s Pod Replica", category: "Cloud & K8s", w: 175, h: 78, shape: "card" },
];

function createDogfoodPages() {
  return [
    {
      id: 1,
      name: "Page-1: RustFlow Architecture",
      pan: { x: 60, y: 50 },
      zoom: 1.0,
      nodes: [
        { id: 1, title: "crates/core", body: "R-Tree + Scene + CRDT", x: 80, y: 150, w: 185, h: 82, fill: "#1e293b", stroke: "#818cf8", shape: "card", z: 1 },
        { id: 2, title: "crates/renderer", body: "wgpu WGSL + Bezier", x: 360, y: 60, w: 185, h: 82, fill: "#1e293b", stroke: "#38bdf8", shape: "card", z: 2 },
        { id: 3, title: "crates/layout", body: "Sugiyama + Force + Spring", x: 360, y: 175, w: 185, h: 82, fill: "#1e293b", stroke: "#34d399", shape: "card", z: 3 },
        { id: 4, title: "crates/io-formats", body: ".drawio XML + SVG + PDF", x: 360, y: 290, w: 185, h: 82, fill: "#1e293b", stroke: "#fbbf24", shape: "card", z: 4 },
        { id: 5, title: "crates/ui-bridge", body: "WASM & Tauri v2 Shell", x: 650, y: 175, w: 190, h: 82, fill: "#1e293b", stroke: "#f472b6", shape: "card", z: 5 },
      ],
      edges: [
        { id: 101, source: 1, target: 2, label: "Frustum Query", anim: "pulse", kind: "bezier" },
        { id: 102, source: 1, target: 3, label: "Graph Moves", anim: "ants", kind: "bezier" },
        { id: 103, source: 1, target: 4, label: "AST Round-Trip", anim: "pulse", kind: "bezier" },
        { id: 104, source: 2, target: 5, label: "Frame Buffers", anim: "pulse", kind: "bezier" },
        { id: 105, source: 3, target: 5, label: "Transitions", anim: "ants", kind: "bezier" },
        { id: 106, source: 4, target: 5, label: "Exporters", anim: "ants", kind: "bezier" },
      ],
      undoStack: [],
      redoStack: [],
    },
    {
      id: 2,
      name: "Page-2: Microservice Flow",
      pan: { x: 80, y: 60 },
      zoom: 1.0,
      nodes: [
        { id: 10, title: "API Gateway", body: "Rate Limiter + TLS", x: 120, y: 140, w: 180, h: 80, fill: "#1e293b", stroke: "#818cf8", shape: "card", z: 1 },
        { id: 11, title: "Billing Worker", body: "Kafka Consumer", x: 400, y: 140, w: 180, h: 80, fill: "#1e293b", stroke: "#34d399", shape: "card", z: 2 },
      ],
      edges: [
        { id: 201, source: 10, target: 11, label: "Events", anim: "pulse", kind: "bezier" },
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
  draggingNode: null,
  resizingNode: null,
  panning: null,
  connectingFromNode: null,
  mouseWorld: { x: 0, y: 0 },
  smartGuides: [],
  nextId: 300,
  edgeAnimPreset: "pulse",
  edgeKindPreset: "bezier",
  theme: "dark",
  fps: 60,
  lastFrameTime: performance.now(),
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
const inlineEditor = document.getElementById("inline-editor");

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

// Draw.io Quick-Add Directional Arrows around the selected node
function getQuickArrows(n) {
  const pad = 22;
  return [
    { dir: "up", x: n.x + n.w / 2, y: n.y - pad, dx: 0, dy: -(n.h + 70), label: "↑" },
    { dir: "right", x: n.x + n.w + pad, y: n.y + n.h / 2, dx: n.w + 90, dy: 0, label: "→" },
    { dir: "down", x: n.x + n.w / 2, y: n.y + n.h + pad, dx: 0, dy: n.h + 70, label: "↓" },
    { dir: "left", x: n.x - pad, y: n.y + n.h / 2, dx: -(n.w + 90), dy: 0, label: "←" },
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

canvas.addEventListener("mousedown", (e) => {
  inlineEditor.style.display = "none";
  const rect = canvas.getBoundingClientRect();
  const sx = e.clientX - rect.left;
  const sy = e.clientY - rect.top;
  const wpt = screenToWorld(sx, sy);
  const p = page();

  const selectedNode = p.nodes.find((n) => n.id === state.selectedNodeId);
  if (selectedNode) {
    // 1. Check bottom-right resize handle
    if (Math.hypot(wpt.x - (selectedNode.x + selectedNode.w), wpt.y - (selectedNode.y + selectedNode.h)) <= 10) {
      saveSnapshot();
      state.resizingNode = { node: selectedNode, aspect: selectedNode.w / Math.max(1, selectedNode.h) };
      return;
    }
    // 2. Check Draw.io quick-add directional arrows
    for (const arrow of getQuickArrows(selectedNode)) {
      if (Math.hypot(wpt.x - arrow.x, wpt.y - arrow.y) <= 11) {
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
          target: newNode.id,
          label: "Flow",
          anim: state.edgeAnimPreset,
          kind: state.edgeKindPreset,
        });
        state.selectedNodeId = newNode.id;
        syncInspector();
        return;
      }
    }
  }

  // 3. Check port handle click (connection drag)
  for (const n of p.nodes) {
    for (const port of getNodePorts(n)) {
      if (Math.hypot(wpt.x - port.x, wpt.y - port.y) <= 9) {
        state.connectingFromNode = { node: n, port };
        return;
      }
    }
  }

  // 4. Check node hit-test (sorted by z-index)
  const sorted = [...p.nodes].sort((a, b) => (a.z || 0) - (b.z || 0));
  const hit = sorted.reverse().find((n) => wpt.x >= n.x && wpt.x <= n.x + n.w && wpt.y >= n.y && wpt.y <= n.y + n.h);
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

// Inline double-click text editing on canvas
canvas.addEventListener("dblclick", (e) => {
  const rect = canvas.getBoundingClientRect();
  const wpt = screenToWorld(e.clientX - rect.left, e.clientY - rect.top);
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
      syncInspector();
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

  if (state.resizingNode) {
    const { node, aspect } = state.resizingNode;
    node.w = Math.max(80, Math.round((state.mouseWorld.x - node.x) / 10) * 10);
    if (e.shiftKey) {
      node.h = Math.max(48, Math.round(node.w / aspect));
    } else {
      node.h = Math.max(48, Math.round((state.mouseWorld.y - node.y) / 10) * 10);
    }
    syncInspector();
  } else if (state.draggingNode) {
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
        state.mouseWorld.x >= n.x - 14 &&
        state.mouseWorld.x <= n.x + n.w + 14 &&
        state.mouseWorld.y >= n.y - 14 &&
        state.mouseWorld.y <= n.y + n.h + 14
    );
    if (dst) {
      saveSnapshot();
      p.edges.push({
        id: ++state.nextId,
        source: src.id,
        target: dst.id,
        label: "Flow",
        anim: state.edgeAnimPreset,
        kind: state.edgeKindPreset,
      });
    }
  }
  state.draggingNode = null;
  state.resizingNode = null;
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
  state.selectedNodeId = newNode.id;
  syncInspector();
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
      card.textContent = "+ " + item.label;
      card.onclick = () => addStencilToCanvas(item);
      grid.appendChild(card);
    }
    section.appendChild(grid);
    container.appendChild(section);
  }
}
document.getElementById("palette-search").addEventListener("input", (e) => renderPalette(e.target.value));
renderPalette();

function renderOutline() {
  const list = document.getElementById("outline-list");
  list.innerHTML = "";
  for (const n of page().nodes) {
    const el = document.createElement("div");
    el.className = "outline-item" + (n.id === state.selectedNodeId ? " active" : "");
    el.innerHTML = `<span>${n.title}</span><span style="opacity:0.6">#${n.id}</span>`;
    el.onclick = () => {
      state.selectedNodeId = n.id;
      syncInspector();
    };
    list.appendChild(el);
  }
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

function syncInspector() {
  renderOutline();
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
    renderOutline();
  });
}

document.getElementById("btn-front").onclick = () => {
  const p = page();
  const n = p.nodes.find((item) => item.id === state.selectedNodeId);
  if (!n) return;
  n.z = Math.max(0, ...p.nodes.map((x) => x.z || 0)) + 1;
};

document.getElementById("btn-back").onclick = () => {
  const p = page();
  const n = p.nodes.find((item) => item.id === state.selectedNodeId);
  if (!n) return;
  n.z = Math.min(0, ...p.nodes.map((x) => x.z || 0)) - 1;
};

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

function runHierarchicalLayout() {
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
      n.x = 100 + Number(layerIdx) * 260;
      n.y = 80 + sibIdx * 125;
    });
  });
  syncInspector();
}

function runGridLayout() {
  const p = page();
  saveSnapshot();
  p.nodes.forEach((n, idx) => {
    const col = idx % 3;
    const row = Math.floor(idx / 3);
    n.x = 100 + col * 240;
    n.y = 90 + row * 135;
  });
  syncInspector();
}

document.getElementById("btn-undo").onclick = undo;
document.getElementById("btn-redo").onclick = redo;
document.getElementById("btn-duplicate").onclick = duplicateSelected;
document.getElementById("btn-delete").onclick = deleteSelected;
document.getElementById("select-edge-anim").onchange = (e) => {
  state.edgeAnimPreset = e.target.value;
};
document.getElementById("select-edge-kind").onchange = (e) => {
  state.edgeKindPreset = e.target.value;
  for (const edge of page().edges) edge.kind = state.edgeKindPreset;
};
document.getElementById("select-theme").onchange = (e) => {
  state.theme = e.target.value;
  document.documentElement.setAttribute("data-theme", state.theme);
};
document.getElementById("btn-layout-hier").onclick = runHierarchicalLayout;
document.getElementById("btn-layout-grid").onclick = runGridLayout;

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
    xml += `        <mxCell id="node-${n.id}" value="${n.title}" style="rounded=1;fillColor=${n.fill};strokeColor=${n.stroke};" vertex="1" parent="1">\n          <mxGeometry x="${n.x}" y="${n.y}" width="${n.w}" height="${n.h}" as="geometry"/>\n        </mxCell>\n`;
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
    const src = p.nodes.find((n) => n.id === e.source);
    const dst = p.nodes.find((n) => n.id === e.target);
    if (!src || !dst) continue;
    const x1 = src.x + src.w, y1 = src.y + src.h / 2;
    const x2 = dst.x, y2 = dst.y + dst.h / 2;
    const d = `M ${x1} ${y1} C ${x1 + 80} ${y1}, ${x2 - 80} ${y2}, ${x2} ${y2}`;
    svg += `  <path d="${d}" fill="none" stroke="#818cf8" stroke-width="2" class="${e.anim === "ants" ? "ants" : ""}"/>\n`;
  }
  for (const n of p.nodes) {
    svg += `  <rect x="${n.x}" y="${n.y}" width="${n.w}" height="${n.h}" rx="10" fill="${n.fill}" stroke="${n.stroke}" stroke-width="2"/>\n`;
    svg += `  <text x="${n.x + n.w / 2}" y="${n.y + n.h / 2}" text-anchor="middle" dominant-baseline="middle">${n.title}</text>\n`;
  }
  svg += `</svg>`;
  downloadFile("diagram-animated.svg", svg, "image/svg+xml");
}

document.getElementById("btn-export-drawio").onclick = exportDrawioXml;
document.getElementById("btn-export-svg").onclick = exportAnimatedSvg;

// Developer Command Palette (Ctrl+K)
const cmdOverlay = document.getElementById("cmd-overlay");
const cmdInput = document.getElementById("cmd-input");
const cmdList = document.getElementById("cmd-list");

const COMMANDS = [
  { label: "Auto-Layout: Sugiyama Hierarchical", shortcut: "Layout", run: runHierarchicalLayout },
  { label: "Auto-Layout: Uniform Grid", shortcut: "Layout", run: runGridLayout },
  { label: "Theme: Switch to Dark Mode", shortcut: "Theme", run: () => { document.getElementById("select-theme").value = "dark"; document.documentElement.setAttribute("data-theme", "dark"); } },
  { label: "Theme: Switch to Light Mode", shortcut: "Theme", run: () => { document.getElementById("select-theme").value = "light"; document.documentElement.setAttribute("data-theme", "light"); } },
  { label: "Theme: Switch to High Contrast", shortcut: "Theme", run: () => { document.getElementById("select-theme").value = "contrast"; document.documentElement.setAttribute("data-theme", "contrast"); } },
  { label: "Export: Download .drawio XML", shortcut: "IO", run: exportDrawioXml },
  { label: "Export: Download Animated SVG", shortcut: "IO", run: exportAnimatedSvg },
  ...STENCILS.map((s) => ({ label: `Add Stencil: ${s.label}`, shortcut: s.category, run: () => addStencilToCanvas(s) })),
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
    return;
  }
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
  } else if (["ArrowUp", "ArrowDown", "ArrowLeft", "ArrowRight"].includes(e.key)) {
    const n = page().nodes.find((item) => item.id === state.selectedNodeId);
    if (!n) return;
    e.preventDefault();
    const step = e.shiftKey ? 20 : 5;
    if (e.key === "ArrowUp") n.y -= step;
    if (e.key === "ArrowDown") n.y += step;
    if (e.key === "ArrowLeft") n.x -= step;
    if (e.key === "ArrowRight") n.x += step;
    syncInspector();
  }
});

function evalBezier(p0, p1, p2, p3, t) {
  const u = 1 - t;
  return {
    x: u * u * u * p0.x + 3 * u * u * t * p1.x + 3 * u * t * t * p2.x + t * t * t * p3.x,
    y: u * u * u * p0.y + 3 * u * u * t * p1.y + 3 * u * t * t * p2.y + t * t * t * p3.y,
  };
}

function renderLoop(timestamp) {
  const dt = Math.max(1, timestamp - state.lastFrameTime);
  state.fps = Math.round(state.fps * 0.9 + (1000 / dt) * 0.1);
  state.lastFrameTime = timestamp;

  const timeSec = timestamp * 0.001;
  const p = page();
  ctx.clearRect(0, 0, canvas.width, canvas.height);

  // 1. Infinite pan-zoom dot grid
  const gridStep = 20 * p.zoom;
  if (gridStep >= 6) {
    ctx.fillStyle = getComputedStyle(document.documentElement).getPropertyValue("--grid-dot").trim() || "#334155";
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
  ctx.strokeStyle = "#f43f5e";
  ctx.lineWidth = 1.2 / p.zoom;
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

  // 3. Edges & GPU-style Animations
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
    if (e.kind === "straight") {
      ctx.lineTo(p3.x, p3.y);
    } else if (e.kind === "orthogonal" || e.kind === "smoothstep") {
      const midX = (p0.x + p3.x) * 0.5;
      ctx.lineTo(midX, p0.y);
      ctx.lineTo(midX, p3.y);
      ctx.lineTo(p3.x, p3.y);
    } else {
      ctx.bezierCurveTo(p1.x, p1.y, p2.x, p2.y, p3.x, p3.y);
    }
    ctx.strokeStyle = "#818cf8";
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
        ctx.fillStyle = "#38bdf8";
        ctx.shadowColor = "#38bdf8";
        ctx.shadowBlur = 12;
        ctx.fill();
        ctx.shadowBlur = 0;
      }
    }
  }

  if (state.connectingFromNode) {
    ctx.beginPath();
    ctx.moveTo(state.connectingFromNode.port.x, state.connectingFromNode.port.y);
    ctx.lineTo(state.mouseWorld.x, state.mouseWorld.y);
    ctx.strokeStyle = "#818cf8";
    ctx.setLineDash([5, 5]);
    ctx.stroke();
    ctx.setLineDash([]);
  }

  // 4. React Flow Node Cards, Ports, Resize Handles & Quick-Add Arrows
  const sortedNodes = [...p.nodes].sort((a, b) => (a.z || 0) - (b.z || 0));
  for (const n of sortedNodes) {
    const selected = n.id === state.selectedNodeId;
    ctx.save();
    ctx.shadowColor = selected ? "rgba(129, 140, 248, 0.35)" : "rgba(0, 0, 0, 0.25)";
    ctx.shadowBlur = selected ? 18 : 10;
    ctx.shadowOffsetY = 4;

    ctx.fillStyle = n.fill;
    ctx.strokeStyle = selected ? "#38bdf8" : n.stroke;
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

    const textColor = state.theme === "light" && n.fill === "#ffffff" ? "#0f172a" : "#f8fafc";
    ctx.fillStyle = textColor;
    ctx.font = "600 13px Inter, sans-serif";
    ctx.textAlign = "center";
    ctx.fillText(n.title, n.x + n.w / 2, n.shape === "diamond" ? n.y + n.h / 2 + 4 : n.y + 30);

    if (n.body && n.shape !== "diamond") {
      ctx.fillStyle = "#94a3b8";
      ctx.font = "400 11px monospace";
      ctx.fillText(n.body, n.x + n.w / 2, n.y + 54);
    }

    for (const port of getNodePorts(n)) {
      ctx.beginPath();
      ctx.arc(port.x, port.y, 4.5, 0, Math.PI * 2);
      ctx.fillStyle = "#0b0f19";
      ctx.strokeStyle = "#818cf8";
      ctx.lineWidth = 2;
      ctx.fill();
      ctx.stroke();
    }

    if (selected) {
      // Bottom-right resize handle
      ctx.fillStyle = "#38bdf8";
      ctx.fillRect(n.x + n.w - 5, n.y + n.h - 5, 10, 10);

      // Draw.io Quick-Add Directional Arrows
      for (const arrow of getQuickArrows(n)) {
        ctx.beginPath();
        ctx.arc(arrow.x, arrow.y, 9, 0, Math.PI * 2);
        ctx.fillStyle = "#1e293b";
        ctx.strokeStyle = "#38bdf8";
        ctx.lineWidth = 1.5;
        ctx.fill();
        ctx.stroke();
        ctx.fillStyle = "#38bdf8";
        ctx.font = "700 11px sans-serif";
        ctx.fillText(arrow.label, arrow.x, arrow.y + 4);
      }
    }
  }

  ctx.restore();

  // 5. Render Minimap
  minimapCtx.clearRect(0, 0, minimapCanvas.width, minimapCanvas.height);
  const scale = 0.14;
  minimapCtx.save();
  minimapCtx.translate(14, 14);
  minimapCtx.scale(scale, scale);
  for (const n of p.nodes) {
    minimapCtx.fillStyle = n.id === state.selectedNodeId ? "#38bdf8" : "#64748b";
    minimapCtx.fillRect(n.x, n.y, n.w, n.h);
  }
  const tl = screenToWorld(0, 0);
  const br = screenToWorld(canvas.width, canvas.height);
  minimapCtx.strokeStyle = "#818cf8";
  minimapCtx.lineWidth = 10;
  minimapCtx.strokeRect(tl.x, tl.y, br.x - tl.x, br.y - tl.y);
  minimapCtx.restore();

  document.getElementById("status-bar").textContent = `FPS: ${state.fps} | Zoom: ${Math.round(p.zoom * 100)}% | Nodes: ${p.nodes.length} | Edges: ${p.edges.length}`;
  requestAnimationFrame(renderLoop);
}
requestAnimationFrame(renderLoop);
