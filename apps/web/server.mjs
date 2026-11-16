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

function shutdown(signal) {
  console.log(`Received ${signal}, shutting down bohat-drawio gracefully...`);
  server.close(() => {
    process.exit(0);
  });
  setTimeout(() => process.exit(0), 3000).unref();
}

process.on("SIGTERM", () => shutdown("SIGTERM"));
process.on("SIGINT", () => shutdown("SIGINT"));
