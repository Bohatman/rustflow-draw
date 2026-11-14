# RFC 0004: Bidirectional `.drawio` XML & Native Serialization

- **Status:** Accepted
- **Phase:** Phase 1 (Architecture & RFC)

## 1. Context
RustFlow Draw must preserve rich React Flow animation metadata in its native format while offering seamless bidirectional `.drawio` (`mxfile` / `mxGraphModel`) compatibility and animated SVG export.

## 2. Decision
- **Native Format (`.rfd.json`):** Versioned schema (`version: "1.0.0"`) serializing multi-page documents, node geometry, style tokens, port definitions, and edge animation configurations.
- **Draw.io XML (`mxfile`):**
  - Each `Page` maps to a `<diagram id="..." name="...">` element containing an `<mxGraphModel>`.
  - Nodes map to `<mxCell vertex="1" style="...">` with `<mxGeometry x="..." y="..." width="..." height="..." as="geometry"/>`.
  - Edges map to `<mxCell edge="1" source="..." target="..." style="...">` including waypoints and custom `rfdAnimation` style keys so round-tripping preserves animations.
- **Animated SVG Export:** Embeds `<style>` with `@keyframes rfd-dash` and `<circle>` motion paths so exported SVGs animate natively in any browser.
