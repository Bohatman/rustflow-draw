import http from "node:http";
import fs from "node:fs";
import path from "node:path";
import crypto from "node:crypto";
import { fileURLToPath } from "node:url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const port = Number(process.env.PORT || process.argv[2] || 6129);
const isProd = process.env.NODE_ENV === "production";
const distDir = path.join(__dirname, "dist");
const rootDir = isProd && fs.existsSync(distDir) ? distDir : __dirname;

let buildManifest = { name: "bohat-drawio", version: "0.1.0", mode: isProd ? "production" : "development" };
const manifestPath = path.join(distDir, "build-manifest.json");
if (fs.existsSync(manifestPath)) {
  try {
    buildManifest = JSON.parse(fs.readFileSync(manifestPath, "utf8"));
  } catch {
    // Ignore parse error
  }
}

const MIME_TYPES = {
  ".html": "text/html; charset=utf-8",
  ".js": "text/javascript; charset=utf-8",
  ".mjs": "text/javascript; charset=utf-8",
  ".css": "text/css; charset=utf-8",
  ".svg": "image/svg+xml",
  ".png": "image/png",
  ".gif": "image/gif",
  ".jpg": "image/jpeg",
  ".jpeg": "image/jpeg",
  ".webp": "image/webp",
  ".ico": "image/x-icon",
  ".json": "application/json; charset=utf-8",
  ".wasm": "application/wasm",
};

const SECURITY_HEADERS = {
  "X-Content-Type-Options": "nosniff",
  "X-Frame-Options": "SAMEORIGIN",
  "Referrer-Policy": "strict-origin-when-cross-origin",
};

// In-memory cache for production assets (raw + pre-compressed .br and .gz)
const memoryCache = new Map();

function loadAsset(filePath) {
  if (isProd && memoryCache.has(filePath)) {
    return memoryCache.get(filePath);
  }
  if (!fs.existsSync(filePath) || fs.statSync(filePath).isDirectory()) {
    return null;
  }
  const raw = fs.readFileSync(filePath);
  const gz = fs.existsSync(`${filePath}.gz`) ? fs.readFileSync(`${filePath}.gz`) : null;
  const br = fs.existsSync(`${filePath}.br`) ? fs.readFileSync(`${filePath}.br`) : null;
  const etag = `"${crypto.createHash("sha1").update(raw).digest("hex").slice(0, 16)}"`;
  const entry = { raw, gz, br, etag };
  if (isProd) {
    memoryCache.set(filePath, entry);
  }
  return entry;
}

const startedAt = Date.now();

const server = http.createServer((req, res) => {
  const urlPath = (req.url || "/").split("?")[0];

  if (urlPath === "/healthz" || urlPath === "/api/version") {
    const body = JSON.stringify({
      status: "ok",
      service: "bohat-drawio",
      mode: isProd ? "production" : "development",
      port,
      uptimeSec: Math.round((Date.now() - startedAt) / 1000),
      ...buildManifest,
    });
    res.writeHead(200, {
      "Content-Type": "application/json; charset=utf-8",
      "Cache-Control": "no-store",
      ...SECURITY_HEADERS,
    });
    res.end(body);
    return;
  }

  const relPath = urlPath === "/" ? "/index.html" : urlPath;
  const filePath = path.resolve(rootDir, "." + relPath);

  if (!filePath.startsWith(rootDir)) {
    res.writeHead(403, SECURITY_HEADERS);
    res.end("Forbidden");
    return;
  }

  const asset = loadAsset(filePath);
  if (!asset) {
    res.writeHead(404, { "Content-Type": "text/plain; charset=utf-8", ...SECURITY_HEADERS });
    res.end("Not found: " + relPath);
    return;
  }

  if (isProd && req.headers["if-none-match"] === asset.etag) {
    res.writeHead(304, SECURITY_HEADERS);
    res.end();
    return;
  }

  const ext = path.extname(filePath);
  const isImmutableAsset = isProd && relPath.startsWith("/assets/");
  const headers = {
    "Content-Type": MIME_TYPES[ext] || "application/octet-stream",
    ETag: asset.etag,
    "Cache-Control": isImmutableAsset
      ? "public, max-age=31536000, immutable"
      : "no-store, no-cache, must-revalidate",
    ...SECURITY_HEADERS,
  };

  const acceptEncoding = String(req.headers["accept-encoding"] || "");
  if (asset.br && acceptEncoding.includes("br")) {
    headers["Content-Encoding"] = "br";
    headers["Vary"] = "Accept-Encoding";
    res.writeHead(200, headers);
    res.end(asset.br);
  } else if (asset.gz && acceptEncoding.includes("gzip")) {
    headers["Content-Encoding"] = "gzip";
    headers["Vary"] = "Accept-Encoding";
    res.writeHead(200, headers);
    res.end(asset.gz);
  } else {
    res.writeHead(200, headers);
    res.end(asset.raw);
  }
});

server.listen(port, "0.0.0.0", () => {
  console.log(
    `\n  Bohat Draw.io (${isProd ? "PRODUCTION" : "DEV"}) running at: http://0.0.0.0:${port} (root: ${rootDir})\n`
  );
});

// ============================================================================
// REALTIME CO-OP WEBSOCKET HUB (RFC 6455, ZERO EXTERNAL DEPENDENCIES)
// ============================================================================
const WS_GUID = "258EAFA5-E914-47DA-95CA-C5AB0DC85B11";
const rooms = new Map();

function getRoom(roomId) {
  let room = rooms.get(roomId);
  if (!room) {
    room = {
      id: roomId,
      clients: new Map(),
      snapshot: null,
      locks: new Map(),
      duels: new Map(),
    };
    rooms.set(roomId, room);
  }
  return room;
}

function encodeWsFrame(text, opcode = 0x1) {
  const payload = Buffer.from(text, "utf8");
  const len = payload.length;
  let header;
  if (len < 126) {
    header = Buffer.alloc(2);
    header[0] = 0x80 | opcode;
    header[1] = len;
  } else if (len < 65536) {
    header = Buffer.alloc(4);
    header[0] = 0x80 | opcode;
    header[1] = 126;
    header.writeUInt16BE(len, 2);
  } else {
    header = Buffer.alloc(10);
    header[0] = 0x80 | opcode;
    header[1] = 127;
    header.writeBigUInt64BE(BigInt(len), 2);
  }
  return Buffer.concat([header, payload]);
}

function sendWs(client, msgObj) {
  if (!client?.socket || client.socket.destroyed) return;
  try {
    client.socket.write(encodeWsFrame(JSON.stringify(msgObj)));
  } catch {
    // Ignore write error on closing socket
  }
}

function broadcastRoom(room, msgObj, excludeId = null) {
  const frame = encodeWsFrame(JSON.stringify(msgObj));
  for (const [cid, c] of room.clients.entries()) {
    if (cid === excludeId || c.socket.destroyed) continue;
    try {
      c.socket.write(frame);
    } catch {
      // Ignore write error
    }
  }
}

function serializePeers(room) {
  return [...room.clients.values()].map((c) => ({
    id: c.id,
    name: c.name,
    color: c.color,
    cursor: c.cursor,
  }));
}

function serializeLocks(room) {
  return [...room.locks.entries()].map(([nodeId, l]) => ({ nodeId, ...l }));
}

function serializeDuel(duel) {
  return {
    duelId: duel.duelId,
    nodeId: duel.nodeId,
    nodeTitle: duel.nodeTitle,
    endsAt: duel.endsAt,
    fighters: [...duel.fighters.values()],
  };
}

function finishDuel(room, nodeId) {
  const duel = room.duels.get(nodeId);
  if (!duel) return;
  clearTimeout(duel.timer);
  room.duels.delete(nodeId);

  const fighters = [...duel.fighters.values()].sort((a, b) => b.clicks - a.clicks);
  const winner = fighters[0];
  if (winner) {
    const lock = {
      ownerId: winner.id,
      ownerName: winner.name,
      ownerColor: winner.color,
      updatedAt: Date.now(),
    };
    room.locks.set(nodeId, lock);
    broadcastRoom(room, {
      type: "duel_end",
      duelId: duel.duelId,
      nodeId,
      nodeTitle: duel.nodeTitle,
      winnerId: winner.id,
      winnerName: winner.name,
      winnerColor: winner.color,
      fighters,
      lock,
    });
  }
}

function handleRoomMessage(room, client, msg) {
  if (!msg || typeof msg.type !== "string") return;

  if (msg.type === "join") {
    client.name = String(msg.name || "Guest").slice(0, 24);
    client.color = String(msg.color || "#0077b5").slice(0, 16);
    if (!room.snapshot && msg.initialSnapshot) {
      room.snapshot = msg.initialSnapshot;
    }
    sendWs(client, {
      type: "room_state",
      roomId: room.id,
      snapshot: room.snapshot,
      peers: serializePeers(room),
      locks: serializeLocks(room),
      duels: [...room.duels.values()].map(serializeDuel),
    });
    broadcastRoom(
      room,
      {
        type: "peer_joined",
        peer: { id: client.id, name: client.name, color: client.color, cursor: client.cursor },
      },
      client.id
    );
    return;
  }

  if (msg.type === "rename") {
    client.name = String(msg.name || "Guest").slice(0, 24);
    broadcastRoom(room, { type: "peer_renamed", id: client.id, name: client.name });
    return;
  }

  if (msg.type === "cursor") {
    client.cursor = { x: Number(msg.x) || 0, y: Number(msg.y) || 0, pageId: msg.pageId || 1 };
    broadcastRoom(
      room,
      { type: "cursor", id: client.id, name: client.name, color: client.color, cursor: client.cursor },
      client.id
    );
    return;
  }

  if (msg.type === "sync_state") {
    const lock = msg.editingNodeId !== undefined && msg.editingNodeId !== null ? room.locks.get(msg.editingNodeId) : null;
    if (lock && lock.ownerId !== client.id) {
      return;
    }
    if (msg.snapshot) {
      room.snapshot = msg.snapshot;
      broadcastRoom(room, { type: "remote_state", fromId: client.id, snapshot: msg.snapshot }, client.id);
    }
    return;
  }

  if (msg.type === "claim_node") {
    const nodeId = msg.nodeId;
    if (nodeId === undefined || nodeId === null) return;
    const nodeTitle = String(msg.nodeTitle || `Node #${nodeId}`).slice(0, 40);

    const activeDuel = room.duels.get(nodeId);
    if (activeDuel) {
      if (!activeDuel.fighters.has(client.id)) {
        activeDuel.fighters.set(client.id, { id: client.id, name: client.name, color: client.color, clicks: 1 });
      }
      broadcastRoom(room, { type: "duel_tick", ...serializeDuel(activeDuel) });
      return;
    }

    const existing = room.locks.get(nodeId);
    const stale = existing && Date.now() - existing.updatedAt > 15000;
    if (!existing || stale || existing.ownerId === client.id || !room.clients.has(existing.ownerId)) {
      const lock = {
        ownerId: client.id,
        ownerName: client.name,
        ownerColor: client.color,
        updatedAt: Date.now(),
      };
      room.locks.set(nodeId, lock);
      broadcastRoom(room, { type: "node_lock", nodeId, ...lock });
      return;
    }

    // Conflict! Two different users trying to edit/move the same item -> Start Button-Mashing Duel!
    const defender = room.clients.get(existing.ownerId);
    const duelId = crypto.randomBytes(4).toString("hex");
    const durationMs = 5000;
    const endsAt = Date.now() + durationMs;
    const fighters = new Map();
    fighters.set(defender.id, { id: defender.id, name: defender.name, color: defender.color, clicks: 1 });
    fighters.set(client.id, { id: client.id, name: client.name, color: client.color, clicks: 1 });

    const duel = {
      duelId,
      nodeId,
      nodeTitle,
      endsAt,
      fighters,
      timer: setTimeout(() => finishDuel(room, nodeId), durationMs),
    };
    room.duels.set(nodeId, duel);
    broadcastRoom(room, { type: "duel_start", ...serializeDuel(duel) });
    return;
  }

  if (msg.type === "duel_mash") {
    const duel = room.duels.get(msg.nodeId);
    if (!duel || duel.duelId !== msg.duelId) return;
    let fighter = duel.fighters.get(client.id);
    if (!fighter) {
      fighter = { id: client.id, name: client.name, color: client.color, clicks: 0 };
      duel.fighters.set(client.id, fighter);
    }
    fighter.clicks += 1;
    broadcastRoom(room, { type: "duel_tick", ...serializeDuel(duel) });
    return;
  }

  if (msg.type === "release_node") {
    const nodeId = msg.nodeId;
    if (room.duels.has(nodeId)) return;
    const lock = room.locks.get(nodeId);
    if (lock && lock.ownerId === client.id) {
      room.locks.delete(nodeId);
      broadcastRoom(room, { type: "node_unlock", nodeId });
    }
  }
}

server.on("upgrade", (req, socket) => {
  const reqUrl = new URL(req.url || "/", `http://${req.headers.host || "localhost"}`);
  if (reqUrl.pathname !== "/ws") {
    socket.destroy();
    return;
  }
  const key = req.headers["sec-websocket-key"];
  if (!key) {
    socket.destroy();
    return;
  }

  const accept = crypto.createHash("sha1").update(key + WS_GUID).digest("base64");
  socket.write(
    [
      "HTTP/1.1 101 Switching Protocols",
      "Upgrade: websocket",
      "Connection: Upgrade",
      `Sec-WebSocket-Accept: ${accept}`,
      "",
      "",
    ].join("\r\n")
  );

  const roomId = (reqUrl.searchParams.get("room") || "default").replace(/[^a-zA-Z0-9_-]/g, "").slice(0, 40) || "default";
  const clientId = crypto.randomBytes(6).toString("hex");
  const room = getRoom(roomId);
  const client = { id: clientId, name: "Guest", color: "#0077b5", cursor: null, socket };
  room.clients.set(clientId, client);
  sendWs(client, { type: "welcome", clientId, roomId });

  let rxBuf = Buffer.alloc(0);
  socket.on("data", (chunk) => {
    rxBuf = Buffer.concat([rxBuf, chunk]);
    while (rxBuf.length >= 2) {
      const b0 = rxBuf[0];
      const b1 = rxBuf[1];
      const opcode = b0 & 0x0f;
      const masked = (b1 & 0x80) !== 0;
      let payloadLen = b1 & 0x7f;
      let offset = 2;
      if (payloadLen === 126) {
        if (rxBuf.length < 4) return;
        payloadLen = rxBuf.readUInt16BE(2);
        offset = 4;
      } else if (payloadLen === 127) {
        if (rxBuf.length < 10) return;
        payloadLen = Number(rxBuf.readBigUInt64BE(2));
        offset = 10;
      }
      const maskLen = masked ? 4 : 0;
      if (rxBuf.length < offset + maskLen + payloadLen) return;

      const maskKey = masked ? rxBuf.subarray(offset, offset + 4) : null;
      const payload = Buffer.from(rxBuf.subarray(offset + maskLen, offset + maskLen + payloadLen));
      rxBuf = rxBuf.subarray(offset + maskLen + payloadLen);

      if (masked && maskKey) {
        for (let i = 0; i < payload.length; i++) payload[i] ^= maskKey[i & 3];
      }

      if (opcode === 0x8) {
        socket.end();
        return;
      }
      if (opcode === 0x9) {
        try {
          socket.write(encodeWsFrame(payload.toString("utf8"), 0xA));
        } catch {
          // Ignore
        }
        continue;
      }
      if (opcode === 0x1) {
        try {
          const msg = JSON.parse(payload.toString("utf8"));
          handleRoomMessage(room, client, msg);
        } catch {
          // Ignore malformed JSON
        }
      }
    }
  });

  const cleanup = () => {
    room.clients.delete(clientId);
    for (const [nid, lock] of room.locks.entries()) {
      if (lock.ownerId === clientId) {
        room.locks.delete(nid);
        broadcastRoom(room, { type: "node_unlock", nodeId: nid });
      }
    }
    broadcastRoom(room, { type: "peer_left", id: clientId });
    if (room.clients.size === 0) {
      for (const duel of room.duels.values()) clearTimeout(duel.timer);
      room.duels.clear();
      setTimeout(() => {
        if (room.clients.size === 0) rooms.delete(roomId);
      }, 600000).unref();
    }
  };
  socket.on("close", cleanup);
  socket.on("error", cleanup);
});

function shutdown(signal) {
  console.log(`Received ${signal}, shutting down bohat-drawio gracefully...`);
  server.close(() => {
    process.exit(0);
  });
  setTimeout(() => process.exit(0), 3000).unref();
}

process.on("SIGTERM", () => shutdown("SIGTERM"));
process.on("SIGINT", () => shutdown("SIGINT"));
