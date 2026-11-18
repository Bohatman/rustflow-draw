import fs from "node:fs";
import path from "node:path";
import crypto from "node:crypto";
import zlib from "node:zlib";
import { fileURLToPath } from "node:url";
const __dirname = path.dirname(fileURLToPath(import.meta.url));
const distDir = path.join(__dirname, "dist");
const assetsDir = path.join(distDir, "assets");
fs.rmSync(distDir, { recursive: true, force: true });
fs.mkdirSync(assetsDir, { recursive: true });
const rawJs = fs.readFileSync(path.join(__dirname, "src", "app.js"), "utf8");
const prodJs = rawJs
  .split("\n")
  .map((line) => {
    const trimmed = line.trim();
    if (trimmed.startsWith("//")) return "";
    return line.replace(/^\s+/, "");
  })
  .filter((line) => line.length > 0)
  .join("\n");
const hash = crypto.createHash("sha256").update(prodJs).digest("hex").slice(0, 10);
const bundleFilename = `app.${hash}.js`;
const bundlePath = path.join(assetsDir, bundleFilename);
fs.writeFileSync(bundlePath, prodJs, "utf8");
fs.writeFileSync(`${bundlePath}.gz`, zlib.gzipSync(Buffer.from(prodJs, "utf8"), { level: 9 }));
fs.writeFileSync(
  `${bundlePath}.br`,
  zlib.brotliCompressSync(Buffer.from(prodJs, "utf8"), {
    params: { [zlib.constants.BROTLI_PARAM_QUALITY]: 11 },
  })
);
fs.mkdirSync(path.join(distDir, "src"), { recursive: true });
fs.writeFileSync(path.join(distDir, "src", "app.js"), prodJs, "utf8");
const rawHtml = fs.readFileSync(path.join(__dirname, "index.html"), "utf8");
const prodHtml = rawHtml
  .replace(/\/src\/app\.js(\?v=[^"']*)?/, `/assets/${bundleFilename}`)
  .replace(/<!--[\s\S]*?-->/g, "")
  .replace(/\n\s+\n/g, "\n");
const htmlPath = path.join(distDir, "index.html");
fs.writeFileSync(htmlPath, prodHtml, "utf8");
fs.writeFileSync(`${htmlPath}.gz`, zlib.gzipSync(Buffer.from(prodHtml, "utf8"), { level: 9 }));
fs.writeFileSync(
  `${htmlPath}.br`,
  zlib.brotliCompressSync(Buffer.from(prodHtml, "utf8"), {
    params: { [zlib.constants.BROTLI_PARAM_QUALITY]: 11 },
  })
);
const manifest = {
  name: "bohat-drawio",
  version: "1.0.0-prod",
  builtAt: new Date().toISOString(),
  bundle: `/assets/${bundleFilename}`,
  hash,
  rawBytes: Buffer.byteLength(prodJs, "utf8"),
  gzipBytes: fs.statSync(`${bundlePath}.gz`).size,
  brotliBytes: fs.statSync(`${bundlePath}.br`).size,
};
fs.writeFileSync(path.join(distDir, "build-manifest.json"), JSON.stringify(manifest, null, 2), "utf8");
console.log(
  `[build] Production bundle ready: /assets/${bundleFilename} (${(manifest.rawBytes / 1024).toFixed(1)} KB raw, ${(manifest.gzipBytes / 1024).toFixed(1)} KB gzip, ${(manifest.brotliBytes / 1024).toFixed(1)} KB brotli)`
);
