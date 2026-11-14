//! Spatial R-Tree index for 60-120 FPS frustum culling, hit-testing, and magnetic snapping.

use crate::{Point, Rect};

#[derive(Debug, Clone, PartialEq)]
pub struct SpatialEntry<T: Copy + PartialEq> {
    pub id: T,
    pub bounds: Rect,
}

#[derive(Debug, Clone)]
enum RTreeNode<T: Copy + PartialEq> {
    Leaf {
        bounds: Rect,
        entries: Vec<SpatialEntry<T>>,
    },
    Branch {
        bounds: Rect,
        children: Vec<RTreeNode<T>>,
    },
}

impl<T: Copy + PartialEq> RTreeNode<T> {
    fn bounds(&self) -> Rect {
        match self {
            RTreeNode::Leaf { bounds, .. } => *bounds,
            RTreeNode::Branch { bounds, .. } => *bounds,
        }
    }
}

/// Hierarchical R-Tree spatial index using Sort-Tile-Recursive (STR) packing and dynamic updates.
#[derive(Debug, Clone)]
pub struct RTree<T: Copy + PartialEq> {
    entries: Vec<SpatialEntry<T>>,
    root: Option<RTreeNode<T>>,
    max_node_capacity: usize,
    dirty: bool,
}

impl<T: Copy + PartialEq> Default for RTree<T> {
    fn default() -> Self {
        Self::new(16)
    }
}

impl<T: Copy + PartialEq> RTree<T> {
    pub fn new(max_node_capacity: usize) -> Self {
        Self {
            entries: Vec::new(),
            root: None,
            max_node_capacity: max_node_capacity.max(4),
            dirty: false,
        }
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn clear(&mut self) {
        self.entries.clear();
        self.root = None;
        self.dirty = false;
    }

    pub fn upsert(&mut self, id: T, bounds: Rect) {
        if let Some(existing) = self.entries.iter_mut().find(|e| e.id == id) {
            existing.bounds = bounds;
        } else {
            self.entries.push(SpatialEntry { id, bounds });
        }
        self.dirty = true;
    }

    pub fn remove(&mut self, id: T) -> bool {
        let before = self.entries.len();
        self.entries.retain(|e| e.id != id);
        let removed = self.entries.len() != before;
        if removed {
            self.dirty = true;
        }
        removed
    }

    pub fn bulk_load(&mut self, entries: Vec<SpatialEntry<T>>) {
        self.entries = entries;
        self.rebuild();
    }

    pub fn ensure_built(&mut self) {
        if self.dirty || (self.root.is_none() && !self.entries.is_empty()) {
            self.rebuild();
        }
    }

    fn rebuild(&mut self) {
        self.dirty = false;
        if self.entries.is_empty() {
            self.root = None;
            return;
        }
        let mut sorted = self.entries.clone();
        self.root = Some(Self::build_str(&mut sorted, self.max_node_capacity));
    }

    fn compute_bounds_entries(entries: &[SpatialEntry<T>]) -> Rect {
        let mut iter = entries.iter();
        let Some(first) = iter.next() else {
            return Rect::new(0.0, 0.0, 0.0, 0.0);
        };
        let mut acc = first.bounds;
        for item in iter {
            acc = acc.union(&item.bounds);
        }
        acc
    }

    fn compute_bounds_nodes(nodes: &[RTreeNode<T>]) -> Rect {
        let mut iter = nodes.iter();
        let Some(first) = iter.next() else {
            return Rect::new(0.0, 0.0, 0.0, 0.0);
        };
        let mut acc = first.bounds();
        for node in iter {
            acc = acc.union(&node.bounds());
        }
        acc
    }

    fn build_str(entries: &mut [SpatialEntry<T>], cap: usize) -> RTreeNode<T> {
        if entries.len() <= cap {
            let bounds = Self::compute_bounds_entries(entries);
            return RTreeNode::Leaf {
                bounds,
                entries: entries.to_vec(),
            };
        }

        entries.sort_by(|a, b| {
            a.bounds
                .center()
                .x
                .partial_cmp(&b.bounds.center().x)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        let leaf_count = (entries.len() + cap - 1) / cap;
        let slice_count = (leaf_count as f32).sqrt().ceil() as usize;
        let slice_size = (entries.len() + slice_count - 1) / slice_count;

        let mut children = Vec::new();
        for slice in entries.chunks_mut(slice_size.max(1)) {
            slice.sort_by(|a, b| {
                a.bounds
                    .center()
                    .y
                    .partial_cmp(&b.bounds.center().y)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
            for chunk in slice.chunks_mut(cap) {
                children.push(Self::build_str(chunk, cap));
            }
        }

        while children.len() > cap {
            let mut next_level = Vec::new();
            for group in children.chunks(cap) {
                let group_vec = group.to_vec();
                let bounds = Self::compute_bounds_nodes(&group_vec);
                next_level.push(RTreeNode::Branch {
                    bounds,
                    children: group_vec,
                });
            }
            children = next_level;
        }

        let bounds = Self::compute_bounds_nodes(&children);
        RTreeNode::Branch { bounds, children }
    }

    /// Frustum query returning all IDs whose bounding box intersects `frustum`.
    pub fn query_rect(&self, frustum: &Rect) -> Vec<T> {
        let mut out = Vec::new();
        if !self.dirty {
            if let Some(root) = &self.root {
                Self::query_node_rect(root, frustum, &mut out);
                return out;
            }
        }
        for entry in &self.entries {
            if entry.bounds.intersects(frustum) {
                out.push(entry.id);
            }
        }
        out
    }

    fn query_node_rect(node: &RTreeNode<T>, frustum: &Rect, out: &mut Vec<T>) {
        match node {
            RTreeNode::Leaf { bounds, entries } => {
                if !bounds.intersects(frustum) {
                    return;
                }
                for entry in entries {
                    if entry.bounds.intersects(frustum) {
                        out.push(entry.id);
                    }
                }
            }
            RTreeNode::Branch { bounds, children } => {
                if !bounds.intersects(frustum) {
                    return;
                }
                for child in children {
                    Self::query_node_rect(child, frustum, out);
                }
            }
        }
    }

    /// Hit-test query returning all IDs whose bounding box contains `pt`.
    pub fn query_point(&self, pt: Point) -> Vec<T> {
        let tiny = Rect::new(pt.x, pt.y, 0.0, 0.0);
        self.query_rect(&tiny)
    }

    /// Overall bounding box of all indexed elements.
    pub fn total_bounds(&self) -> Option<Rect> {
        if self.entries.is_empty() {
            return None;
        }
        if !self.dirty {
            if let Some(root) = &self.root {
                return Some(root.bounds());
            }
        }
        Some(Self::compute_bounds_entries(&self.entries))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rtree_frustum_culling_and_point_queries() {
        let mut tree = RTree::new(8);
        let mut bulk = Vec::new();
        for i in 0..500u32 {
            let x = (i % 25) as f32 * 100.0;
            let y = (i / 25) as f32 * 100.0;
            bulk.push(SpatialEntry {
                id: i,
                bounds: Rect::new(x, y, 60.0, 40.0),
            });
        }
        tree.bulk_load(bulk);

        let frustum = Rect::new(0.0, 0.0, 180.0, 180.0);
        let visible = tree.query_rect(&frustum);
        assert_eq!(visible.len(), 4);

        let hit = tree.query_point(Point::new(110.0, 110.0));
        assert_eq!(hit, vec![26]);
    }
}
