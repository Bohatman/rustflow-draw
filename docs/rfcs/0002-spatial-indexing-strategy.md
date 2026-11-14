# RFC 0002: Spatial Indexing Strategy (R-Tree & Smart Snapping)

- **Status:** Accepted
- **Phase:** Phase 1 (Architecture & RFC)

## 1. Context & Performance Budget
To render 5,000+ nodes and 10,000+ animated edges at 60–120 FPS (8.33–16.66 ms frame budget), the engine must avoid iterating over off-screen elements during rendering, pointer hit-testing, magnetic port connection, and dynamic alignment guide calculation.

## 2. Decision: Hierarchical Bounding Volume R-Tree
`rustflow-core::spatial::RTree` indexes axis-aligned bounding boxes (`Rect`) for both nodes and edge bounding envelopes:

- **Bulk Loading (STR — Sort-Tile-Recursive):** Rebuilds or updates spatial partitions cleanly in $O(N \log N)$ when large layout passes complete, with fast incremental insert/remove/update for interactive dragging.
- **Viewport Frustum Culling:** `query_rect(viewport.visible_world_rect())` returns only visible node and edge IDs for GPU instance buffer generation.
- **Magnetic Port Snapping:** `find_nearest_port(world_pt, radius)` locates candidate connection handles within the magnetic snap threshold.
- **Smart Alignment Guides:** Compares moving selection bounds against nearby visible nodes returned by an expanded viewport query to compute horizontal/vertical center and edge snap lines plus equal-spacing indicators.
