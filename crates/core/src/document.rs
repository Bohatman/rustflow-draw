//! Multi-page workspaces (Draw.io tabbed pages) and interactive Minimap calculations.

use crate::command::CommandHistory;
use crate::scene::{NodeId, PageId, SceneGraph};
use crate::selection::SelectionState;
use crate::{Point, Rect, Viewport};

#[derive(Debug, Clone)]
pub struct Page {
    pub id: PageId,
    pub name: String,
    pub scene: SceneGraph,
    pub viewport: Viewport,
    pub history: CommandHistory,
    pub selection: SelectionState,
}

impl Page {
    pub fn new(id: PageId, name: impl Into<String>, width: f32, height: f32) -> Self {
        Self {
            id,
            name: name.into(),
            scene: SceneGraph::new(),
            viewport: Viewport::new(width, height),
            history: CommandHistory::new(256),
            selection: SelectionState::new(),
        }
    }
}

/// Multi-page diagram document matching Draw.io's multi-tabbed structure.
#[derive(Debug, Clone)]
pub struct Document {
    pub title: String,
    pages: Vec<Page>,
    active_page_id: PageId,
    next_page_id: PageId,
    default_size: (f32, f32),
}

impl Default for Document {
    fn default() -> Self {
        Self::new("Untitled Diagram", 1280.0, 720.0)
    }
}

impl Document {
    pub fn new(title: impl Into<String>, width: f32, height: f32) -> Self {
        let first_page = Page::new(1, "Page-1", width, height);
        Self {
            title: title.into(),
            pages: vec![first_page],
            active_page_id: 1,
            next_page_id: 2,
            default_size: (width, height),
        }
    }

    pub fn pages(&self) -> &[Page] {
        &self.pages
    }

    pub fn pages_mut(&mut self) -> &mut [Page] {
        &mut self.pages
    }

    pub fn active_page_id(&self) -> PageId {
        self.active_page_id
    }

    pub fn active_page(&self) -> &Page {
        self.pages
            .iter()
            .find(|p| p.id == self.active_page_id)
            .unwrap_or(&self.pages[0])
    }

    pub fn active_page_mut(&mut self) -> &mut Page {
        let active_id = self.active_page_id;
        if let Some(pos) = self.pages.iter().position(|p| p.id == active_id) {
            &mut self.pages[pos]
        } else {
            &mut self.pages[0]
        }
    }

    pub fn add_page(&mut self, name: impl Into<String>) -> PageId {
        let id = self.next_page_id;
        self.next_page_id += 1;
        let (w, h) = self.default_size;
        self.pages.push(Page::new(id, name, w, h));
        self.active_page_id = id;
        id
    }

    pub fn switch_page(&mut self, page_id: PageId) -> bool {
        if self.pages.iter().any(|p| p.id == page_id) {
            self.active_page_id = page_id;
            true
        } else {
            false
        }
    }

    pub fn rename_page(&mut self, page_id: PageId, new_name: impl Into<String>) -> bool {
        if let Some(p) = self.pages.iter_mut().find(|p| p.id == page_id) {
            p.name = new_name.into();
            true
        } else {
            false
        }
    }

    pub fn remove_page(&mut self, page_id: PageId) -> bool {
        if self.pages.len() <= 1 {
            return false;
        }
        let Some(pos) = self.pages.iter().position(|p| p.id == page_id) else {
            return false;
        };
        self.pages.remove(pos);
        if self.active_page_id == page_id {
            self.active_page_id = self.pages[pos.saturating_sub(1)].id;
        }
        true
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimapNodeBox {
    pub node_id: NodeId,
    pub rect: Rect,
}

/// Interactive floating Minimap state with viewport frustum box and node thumbnails.
#[derive(Debug, Clone, PartialEq)]
pub struct MinimapSnapshot {
    pub minimap_width: f32,
    pub minimap_height: f32,
    pub world_envelope: Rect,
    pub scale: f32,
    pub viewport_frustum_rect: Rect,
    pub node_boxes: Vec<MinimapNodeBox>,
}

impl MinimapSnapshot {
    pub fn compute(
        scene: &SceneGraph,
        viewport: &Viewport,
        minimap_width: f32,
        minimap_height: f32,
    ) -> Self {
        let visible_world = viewport.visible_world_rect();
        let world_envelope = match scene.world_bounds() {
            Some(bounds) => bounds.union(&visible_world).expand(40.0),
            None => visible_world.expand(40.0),
        };

        let scale_x = minimap_width / world_envelope.width.max(1.0);
        let scale_y = minimap_height / world_envelope.height.max(1.0);
        let scale = scale_x.min(scale_y);

        let map_rect = |r: &Rect| -> Rect {
            Rect::new(
                (r.x - world_envelope.x) * scale,
                (r.y - world_envelope.y) * scale,
                (r.width * scale).max(2.0),
                (r.height * scale).max(2.0),
            )
        };

        let viewport_frustum_rect = map_rect(&visible_world);
        let node_boxes = scene
            .nodes()
            .iter()
            .map(|n| MinimapNodeBox {
                node_id: n.id,
                rect: map_rect(&n.bounds),
            })
            .collect();

        Self {
            minimap_width,
            minimap_height,
            world_envelope,
            scale,
            viewport_frustum_rect,
            node_boxes,
        }
    }

    /// Convert a click/drag point inside the minimap into a world-space center for viewport panning.
    pub fn minimap_point_to_world(&self, minimap_pt: Point) -> Point {
        Point::new(
            self.world_envelope.x + minimap_pt.x / self.scale.max(1e-5),
            self.world_envelope.y + minimap_pt.y / self.scale.max(1e-5),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scene::Node;

    #[test]
    fn document_multi_page_tabs_and_minimap_frustum() {
        let mut doc = Document::new("Architecture Spec", 1280.0, 720.0);
        assert_eq!(doc.pages().len(), 1);
        doc.active_page_mut().scene.insert_node(Node::new_react_flow_card(
            1,
            "Gateway",
            Rect::new(100.0, 100.0, 180.0, 80.0),
        ));

        let page2 = doc.add_page("Page-2: Data Pipeline");
        assert_eq!(doc.pages().len(), 2);
        assert_eq!(doc.active_page_id(), page2);
        assert!(doc.active_page().scene.nodes().is_empty());

        doc.switch_page(1);
        let page1 = doc.active_page();
        let minimap = MinimapSnapshot::compute(&page1.scene, &page1.viewport, 200.0, 140.0);
        assert_eq!(minimap.node_boxes.len(), 1);
        assert!(minimap.viewport_frustum_rect.width > 0.0);
    }
}
