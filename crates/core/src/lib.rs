//! Core types, spatial R-Tree, scene graph, command stack, selection, multi-page document, and viewport primitives.

pub mod command;
pub mod crdt;
pub mod document;
pub mod scene;
pub mod selection;
pub mod spatial;
pub mod telemetry;

pub use command::{Command, CommandHistory};
pub use crdt::{CrdtOp, CrdtReplica, HlcTimestamp, ThemeTokens};
pub use document::{Document, MinimapNodeBox, MinimapSnapshot, Page};
pub use scene::{
    ColorRgba, Edge, EdgeAnimation, EdgeId, EdgeKind, GroupId, Node, NodeId, NodeStyle, PageId,
    Port, PortDirection, PortId, PortSide, QuickConnectDirection, SceneGraph, ShapeKind, TextAlign,
    TextStyle, ThemeMode,
};
pub use selection::{
    compute_smart_guides, AlignmentGuide, GuideOrientation, SelectionState, SmartSnapResult,
    SpacingIndicator,
};
pub use spatial::{RTree, SpatialEntry};
pub use telemetry::{build_dogfood_architecture_scene, DiagnosticEvent, TelemetryCollector};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

impl Point {
    pub const ZERO: Self = Self { x: 0.0, y: 0.0 };

    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub fn distance_to(self, other: Self) -> f32 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        (dx * dx + dy * dy).sqrt()
    }

    pub fn lerp(self, other: Self, t: f32) -> Self {
        Self {
            x: self.x + (other.x - self.x) * t,
            y: self.y + (other.y - self.y) * t,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Rect {
    pub const fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    pub fn right(&self) -> f32 {
        self.x + self.width
    }

    pub fn bottom(&self) -> f32 {
        self.y + self.height
    }

    pub fn center(&self) -> Point {
        Point::new(self.x + self.width * 0.5, self.y + self.height * 0.5)
    }

    pub fn contains_point(&self, pt: Point) -> bool {
        pt.x >= self.x && pt.x <= self.right() && pt.y >= self.y && pt.y <= self.bottom()
    }

    pub fn intersects(&self, other: &Rect) -> bool {
        self.x <= other.right()
            && self.right() >= other.x
            && self.y <= other.bottom()
            && self.bottom() >= other.y
    }

    pub fn contains_rect(&self, other: &Rect) -> bool {
        other.x >= self.x
            && other.right() <= self.right()
            && other.y >= self.y
            && other.bottom() <= self.bottom()
    }

    pub fn union(&self, other: &Rect) -> Rect {
        let min_x = self.x.min(other.x);
        let min_y = self.y.min(other.y);
        let max_x = self.right().max(other.right());
        let max_y = self.bottom().max(other.bottom());
        Rect::new(min_x, min_y, max_x - min_x, max_y - min_y)
    }

    pub fn expand(&self, margin: f32) -> Rect {
        Rect::new(
            self.x - margin,
            self.y - margin,
            (self.width + margin * 2.0).max(0.0),
            (self.height + margin * 2.0).max(0.0),
        )
    }

    pub fn area(&self) -> f32 {
        self.width.max(0.0) * self.height.max(0.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GridStyle {
    Dots,
    Lines,
    Hidden,
}

/// Infinite canvas viewport with pan and cursor-centered zoom (10% to 800%).
#[derive(Debug, Clone, PartialEq)]
pub struct Viewport {
    pub pan: Point,
    pub zoom: f32,
    pub screen_width: f32,
    pub screen_height: f32,
    pub grid_size: f32,
    pub grid_style: GridStyle,
    pub snap_to_grid: bool,
}

impl Default for Viewport {
    fn default() -> Self {
        Self {
            pan: Point::ZERO,
            zoom: 1.0,
            screen_width: 1280.0,
            screen_height: 720.0,
            grid_size: 20.0,
            grid_style: GridStyle::Dots,
            snap_to_grid: true,
        }
    }
}

impl Viewport {
    pub const MIN_ZOOM: f32 = 0.10;
    pub const MAX_ZOOM: f32 = 8.00;

    pub fn new(screen_width: f32, screen_height: f32) -> Self {
        Self {
            screen_width,
            screen_height,
            ..Self::default()
        }
    }

    pub fn pan_by(&mut self, delta_screen_x: f32, delta_screen_y: f32) {
        self.pan.x += delta_screen_x;
        self.pan.y += delta_screen_y;
    }

    pub fn center_on_world(&mut self, world_pt: Point) {
        self.pan.x = self.screen_width * 0.5 - world_pt.x * self.zoom;
        self.pan.y = self.screen_height * 0.5 - world_pt.y * self.zoom;
    }

    /// Cursor-centered smooth zoom clamped between 10% (`0.10`) and 800% (`8.00`).
    pub fn zoom_at(&mut self, target_zoom: f32, cursor_screen: Point) {
        let world_before = self.screen_to_world(cursor_screen);
        self.zoom = target_zoom.clamp(Self::MIN_ZOOM, Self::MAX_ZOOM);
        self.pan.x = cursor_screen.x - world_before.x * self.zoom;
        self.pan.y = cursor_screen.y - world_before.y * self.zoom;
    }

    pub fn screen_to_world(&self, screen: Point) -> Point {
        Point::new(
            (screen.x - self.pan.x) / self.zoom,
            (screen.y - self.pan.y) / self.zoom,
        )
    }

    pub fn world_to_screen(&self, world: Point) -> Point {
        Point::new(
            world.x * self.zoom + self.pan.x,
            world.y * self.zoom + self.pan.y,
        )
    }

    pub fn visible_world_rect(&self) -> Rect {
        let top_left = self.screen_to_world(Point::ZERO);
        let bottom_right = self.screen_to_world(Point::new(self.screen_width, self.screen_height));
        Rect::new(
            top_left.x,
            top_left.y,
            bottom_right.x - top_left.x,
            bottom_right.y - top_left.y,
        )
    }

    pub fn snap_point(&self, pt: Point) -> Point {
        if !self.snap_to_grid || self.grid_size <= 0.0 {
            return pt;
        }
        Point::new(
            (pt.x / self.grid_size).round() * self.grid_size,
            (pt.y / self.grid_size).round() * self.grid_size,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn viewport_cursor_centered_zoom_invariants() {
        let mut vp = Viewport::new(1280.0, 720.0);
        let cursor = Point::new(400.0, 300.0);
        let before = vp.screen_to_world(cursor);
        vp.zoom_at(2.5, cursor);
        let after = vp.screen_to_world(cursor);
        assert!((before.x - after.x).abs() < 1e-4);
        assert!((before.y - after.y).abs() < 1e-4);

        vp.zoom_at(0.01, cursor);
        assert_eq!(vp.zoom, Viewport::MIN_ZOOM);
        vp.zoom_at(20.0, cursor);
        assert_eq!(vp.zoom, Viewport::MAX_ZOOM);
    }
}
