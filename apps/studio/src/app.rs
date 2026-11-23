use crate::canvas::render_canvas;
use crate::collab::CollabClient;
use crate::panels::{
    render_bottom_bar, render_collab_and_duel_modals, render_extra_modals, render_inspector_panel,
    render_menu_and_toolbar, render_palette_panel,
};
use crate::platform;
use crate::theme::{apply_theme, canvas_clear_rgba, desktop_bg_color, draw_platinum_titlebar, platinum_window_frame};
use egui::{Color32, Margin, Pos2};
use rustflow_collab_protocol::{ClientMsg, ServerMsg};
use rustflow_core::{
    build_dogfood_architecture_scene, ColorRgba, Edge, EdgeAnimation, EdgeArrowStyle, EdgeId,
    EdgeKind, Node, NodeId, Point, PortId, Rect, SceneGraph, ShapeKind, SymbolIcon, SymbolLayer,
    ThemeMode,
};
use rustflow_io_formats::{
    export_animated_gif_bytes_with_bg, export_animated_svg, export_drawio_xml,
    export_png_bytes_with_bg, export_scene_serde_json, import_drawio_xml, import_scene_serde_json,
};
use rustflow_renderer::ExportBgStyle;
use rustflow_ui_bridge::{EngineBridge, StencilCategory};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StudioInspectorTab {
    Style,
    Text,
    Arrange,
    Layers,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PaletteViewMode {
    IconOnly,
    IconAndName,
    NameList,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CanvasBgStyle {
    Dots,
    Grid,
    Crosshair,
    Isometric,
    None,
}

#[derive(Debug, Clone)]
pub struct CustomStencil {
    pub label: String,
    pub width: f32,
    pub height: f32,
    pub shape: ShapeKind,
    pub layers: Vec<SymbolLayer>,
}

#[derive(Debug, Clone)]
pub struct DragStencilPayload {
    pub label: String,
    pub width: f32,
    pub height: f32,
    pub shape: ShapeKind,
    pub icon: Option<SymbolIcon>,
    pub layers: Vec<SymbolLayer>,
}

pub struct BohatDrawApp {
    pub bridge: EngineBridge,
    pub theme_mode: ThemeMode,
    pub show_grid: bool,
    pub bg_style: CanvasBgStyle,
    pub snap_to_grid: bool,
    pub show_minimap: bool,
    pub show_palette: bool,
    pub show_inspector: bool,
    pub hand_pan_mode: bool,
    pub layout_initialized: bool,
    pub last_was_narrow: bool,
    pub inspector_tab: StudioInspectorTab,
    pub palette_search: String,
    pub palette_selected_cats: Vec<StencilCategory>,
    pub palette_cat_dropdown_open: bool,
    pub palette_cat_filter: String,
    pub palette_subgroup: Option<String>,
    pub palette_view_mode: PaletteViewMode,
    pub panning: bool,
    pub dragging_minimap: bool,
    pub dragging_nodes: bool,
    pub drag_origin_world: Point,
    pub drag_start_positions: Vec<(NodeId, Point)>,
    pub active_guides: Vec<(bool, f32, bool)>,
    pub resizing_node: Option<NodeId>,
    pub connecting_from: Option<(NodeId, PortId)>,
    pub dragging_waypoint: Option<(EdgeId, usize)>,
    pub marquee_screen: Option<(Pos2, Pos2)>,
    pub editing_node: Option<NodeId>,
    pub editing_focus_body: bool,
    pub editing_needs_focus: bool,
    pub editing_snapshot: Option<(String, String)>,
    pub default_edge_kind: EdgeKind,
    pub default_edge_arrow: EdgeArrowStyle,
    pub default_edge_anim: u8,
    pub undo_stack: Vec<SceneGraph>,
    pub redo_stack: Vec<SceneGraph>,
    pub clipboard_nodes: Vec<Node>,
    pub clipboard_edges: Vec<Edge>,
    pub collab: CollabClient,
    pub show_collab_modal: bool,
    pub show_cmd_palette: bool,
    pub cmd_query: String,
    pub show_shortcuts: bool,
    pub show_guide_tour: bool,
    pub guide_tour_step: usize,
    pub show_welcome: bool,
    pub welcome_doc_name: String,
    pub show_symbol_studio: bool,
    pub sym_name: String,
    pub sym_w: f32,
    pub sym_h: f32,
    pub sym_fill: Color32,
    pub sym_stroke: Color32,
    pub sym_shape_idx: usize,
    pub sym_layers: Vec<SymbolLayer>,
    pub sym_selected_layer: Option<usize>,
    pub dragging_stencil: Option<DragStencilPayload>,
    pub dragging_layer_id: Option<NodeId>,
    pub custom_stencils: Vec<CustomStencil>,
    pub image_textures: HashMap<NodeId, egui::TextureHandle>,
    pub bg_tile_tex: Option<(CanvasBgStyle, ThemeMode, egui::TextureHandle)>,
    pub glyph_cache: HashMap<(char, u16, Color32), (std::sync::Arc<egui::Galley>, f32)>,
    pub palette_cache_key: (String, u16, String),
    pub palette_cache: Vec<rustflow_ui_bridge::StencilItem>,
    pub pending_image_upload: Rc<RefCell<Option<(Option<NodeId>, String, Vec<u8>)>>>,
    pub collab_room_input: String,
    pub collab_ws_url: String,
    pub status_message: String,
    pub toast: Option<(String, f64)>,
    pub fps: f32,
    pub last_time: f64,
    pub target_zoom: Option<(f32, f32, f32)>,
    last_cursor_sent: f64,
    theme_applied: bool,
}

impl BohatDrawApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let mut bridge = EngineBridge::new(1280.0, 720.0);
        bridge.scene = SceneGraph::new();
        bridge.document.active_page_mut().name = "Page-1".to_string();
        bridge.sync_active_page_to_document();

        let initial_room = platform::initial_room_from_url();
        let room_str = initial_room
            .clone()
            .unwrap_or_else(platform::generate_unique_room_id);
        let ws_url = platform::default_ws_url(&room_str);

        let mut app = Self {
            bridge,
            theme_mode: ThemeMode::Light,
            show_grid: true,
            bg_style: CanvasBgStyle::Dots,
            snap_to_grid: true,
            show_minimap: true,
            show_palette: true,
            show_inspector: true,
            hand_pan_mode: false,
            layout_initialized: false,
            last_was_narrow: false,
            inspector_tab: StudioInspectorTab::Style,
            palette_search: String::new(),
            palette_selected_cats: Vec::new(),
            palette_cat_dropdown_open: false,
            palette_cat_filter: String::new(),
            palette_subgroup: None,
            palette_view_mode: PaletteViewMode::IconOnly,
            panning: false,
            dragging_minimap: false,
            dragging_nodes: false,
            drag_origin_world: Point::ZERO,
            drag_start_positions: Vec::new(),
            active_guides: Vec::new(),
            resizing_node: None,
            connecting_from: None,
            dragging_waypoint: None,
            marquee_screen: None,
            editing_node: None,
            editing_focus_body: false,
            editing_needs_focus: false,
            editing_snapshot: None,
            default_edge_kind: EdgeKind::Bezier,
            default_edge_arrow: EdgeArrowStyle::OneWay,
            default_edge_anim: 2,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            clipboard_nodes: Vec::new(),
            clipboard_edges: Vec::new(),
            collab: CollabClient::default(),
            show_collab_modal: false,
            show_cmd_palette: false,
            cmd_query: String::new(),
            show_shortcuts: false,
            show_guide_tour: false,
            guide_tour_step: 0,
            show_welcome: false,
            welcome_doc_name: "Untitled Diagram".to_string(),
            show_symbol_studio: false,
            sym_name: "Custom Service".to_string(),
            sym_w: 170.0,
            sym_h: 80.0,
            sym_fill: Color32::WHITE,
            sym_stroke: Color32::from_rgb(0, 0, 128),
            sym_shape_idx: 0,
            sym_layers: vec![SymbolLayer {
                shape: ShapeKind::RoundedRect,
                x: 0.1,
                y: 0.15,
                w: 0.8,
                h: 0.7,
                fill: ColorRgba::WHITE,
                stroke: ColorRgba::new(0.0, 0.0, 0.5, 1.0),
            }],
            sym_selected_layer: Some(0),
            dragging_stencil: None,
            dragging_layer_id: None,
            custom_stencils: Vec::new(),
            image_textures: HashMap::new(),
            bg_tile_tex: None,
            glyph_cache: HashMap::new(),
            palette_cache_key: (String::new(), 0, String::new()),
            palette_cache: Vec::new(),
            pending_image_upload: Rc::new(RefCell::new(None)),
            collab_room_input: room_str.clone(),
            collab_ws_url: ws_url.clone(),
            status_message: "Ready — 100% Rust Platinum Engine".to_string(),
            toast: None,
            fps: 60.0,
            last_time: 0.0,
            target_zoom: None,
            last_cursor_sent: 0.0,
            theme_applied: false,
        };

        apply_theme(&cc.egui_ctx, app.theme_mode);
        cc.egui_ctx.options_mut(|o| {
            o.input_options.max_click_dist = 4.0;
        });
        app.theme_applied = true;

        if let Some(room) = initial_room {
            app.collab.connect(&ws_url, &room, cc.egui_ctx.clone());
        }
        app
    }

    pub fn set_status(&mut self, msg: impl Into<String>) {
        let s = msg.into();
        self.status_message = s.clone();
        self.toast = Some((s, self.last_time + 2.6));
    }

    pub fn start_inline_edit(&mut self, nid: NodeId, focus_body: bool) {
        if let Some(node) = self.bridge.scene.get_node(nid) {
            self.editing_snapshot = Some((node.title.clone(), node.body.clone()));
            self.push_undo();
            self.editing_node = Some(nid);
            self.editing_focus_body = focus_body;
            self.editing_needs_focus = true;
        }
    }

    pub fn finish_inline_edit(&mut self, commit: bool) {
        if let Some(edit_id) = self.editing_node.take() {
            let snap = self.editing_snapshot.take();
            self.editing_needs_focus = false;
            self.editing_focus_body = false;
            if commit {
                self.broadcast_scene_snapshot();
            } else if let Some((orig_title, orig_body)) = snap {
                if let Some(node) = self.bridge.scene.get_node_mut(edit_id) {
                    node.title = orig_title;
                    node.body = orig_body;
                }
            }
        }
    }

    pub fn push_undo(&mut self) {
        self.undo_stack.push(self.bridge.scene.clone());
        if self.undo_stack.len() > 80 {
            self.undo_stack.remove(0);
        }
        self.redo_stack.clear();
    }

    pub fn undo(&mut self) {
        if let Some(prev) = self.undo_stack.pop() {
            self.redo_stack.push(self.bridge.scene.clone());
            self.bridge.scene = prev;
            self.bridge.selection.clear();
            self.status_message = "Undo".to_string();
            self.broadcast_scene_snapshot();
        }
    }

    pub fn redo(&mut self) {
        if let Some(next) = self.redo_stack.pop() {
            self.undo_stack.push(self.bridge.scene.clone());
            self.bridge.scene = next;
            self.bridge.selection.clear();
            self.status_message = "Redo".to_string();
            self.broadcast_scene_snapshot();
        }
    }

    pub fn copy_selection(&mut self) {
        let mut selected_ids: Vec<NodeId> = self.bridge.selection.selected_nodes.clone();
        for &id in &self.bridge.selection.selected_nodes {
            for desc in self.bridge.scene.descendant_ids(id) {
                if !selected_ids.contains(&desc) {
                    selected_ids.push(desc);
                }
            }
        }
        let copied: Vec<Node> = selected_ids
            .iter()
            .filter_map(|&id| self.bridge.scene.get_node(id).cloned())
            .collect();
        let copied_edges: Vec<Edge> = self
            .bridge
            .scene
            .edges()
            .iter()
            .filter(|e| {
                selected_ids.contains(&e.source_node) && selected_ids.contains(&e.target_node)
            })
            .cloned()
            .collect();

        if !copied.is_empty() {
            let mut sub_scene = SceneGraph::new();
            let mut bounds: Option<Rect> = None;
            for n in &copied {
                if n.visible {
                    bounds = Some(match bounds {
                        Some(acc) => acc.union(&n.bounds),
                        None => n.bounds,
                    });
                }
                sub_scene.insert_node(n.clone());
            }
            for e in &copied_edges {
                sub_scene.insert_edge(e.clone());
            }
            sub_scene.normalize_layer_order();

            let rect = bounds.unwrap_or_else(|| Rect::new(0.0, 0.0, 400.0, 260.0));
            let padded = rect.expand(36.0);
            let export_w = ((padded.width * 2.0).round() as u32).clamp(320, 2560);
            let export_h = ((padded.height * 2.0).round() as u32).clamp(200, 1920);
            let clear = match self.theme_mode {
                ThemeMode::Dark => canvas_clear_rgba(self.theme_mode),
                _ => [255, 255, 255, 255],
            };
            if let Ok(png_bytes) = export_png_bytes_with_bg(
                &sub_scene,
                export_w,
                export_h,
                0.0,
                clear,
                ExportBgStyle::None,
            ) {
                let _ = platform::copy_png_to_system_clipboard(&png_bytes);
            }

            let count = copied.len();
            self.clipboard_nodes = copied;
            self.clipboard_edges = copied_edges;
            self.set_status(format!(
                "Copied {count} item(s) as 2x PNG picture (paste into PowerPoint, Slides, or canvas)"
            ));
        } else if !self.bridge.scene.nodes().is_empty() {
            self.copy_diagram_png();
        }
    }

    pub fn paste_clipboard(&mut self) {
        if self.clipboard_nodes.is_empty() {
            return;
        }
        self.push_undo();
        let mut id_map: HashMap<NodeId, NodeId> = HashMap::new();
        let mut new_ids = Vec::new();
        for orig in self.clipboard_nodes.clone() {
            let nid = self.bridge.scene.allocate_id();
            id_map.insert(orig.id, nid);
            let mut n = orig;
            n.id = nid;
            n.bounds.x += 32.0;
            n.bounds.y += 32.0;
            n.rebuild_ports();
            self.bridge.scene.insert_node(n);
            new_ids.push(nid);
        }
        for &nid in &new_ids {
            let mapped_parent = self
                .bridge
                .scene
                .get_node(nid)
                .and_then(|n| n.parent_id)
                .and_then(|old_p| id_map.get(&old_p).copied());
            if let Some(n) = self.bridge.scene.get_node_mut(nid) {
                n.parent_id = mapped_parent;
            }
        }
        let mut new_edge_ids = Vec::new();
        for orig_e in self.clipboard_edges.clone() {
            if let (Some(&src), Some(&dst)) = (
                id_map.get(&orig_e.source_node),
                id_map.get(&orig_e.target_node),
            ) {
                let eid = self.bridge.scene.allocate_id();
                let mut e = orig_e;
                e.id = eid;
                e.source_node = src;
                e.target_node = dst;
                for wp in &mut e.waypoints {
                    wp.x += 32.0;
                    wp.y += 32.0;
                }
                self.bridge.scene.insert_edge(e);
                new_edge_ids.push(eid);
            }
        }
        self.bridge.scene.normalize_layer_order();
        self.bridge.selection.clear();
        self.bridge.selection.selected_nodes = new_ids;
        self.bridge.selection.selected_edges = new_edge_ids;
        self.status_message = "Pasted selection".to_string();
        self.broadcast_scene_snapshot();
    }

    pub fn select_all(&mut self) {
        self.bridge.selection.clear();
        self.bridge.selection.selected_nodes = self
            .bridge
            .scene
            .nodes()
            .iter()
            .filter(|n| n.visible)
            .map(|n| n.id)
            .collect();
        self.bridge.selection.selected_edges = self
            .bridge
            .scene
            .edges()
            .iter()
            .map(|e| e.id)
            .collect();
        let n_cnt = self.bridge.selection.selected_nodes.len();
        let e_cnt = self.bridge.selection.selected_edges.len();
        self.set_status(format!(
            "Selected all ({n_cnt} nodes, {e_cnt} edges) — press Ctrl+C to copy as picture for PowerPoint"
        ));
    }

    pub fn default_edge_animation(&self, stroke: ColorRgba) -> EdgeAnimation {
        match self.default_edge_anim {
            1 => EdgeAnimation::MarchingAnts {
                speed: 45.0,
                dash_length: 8.0,
                gap_length: 5.0,
                reverse: false,
            },
            2 => EdgeAnimation::PulseParticles {
                speed: 65.0,
                particle_count: 3,
                radius: 4.0,
                color: stroke,
            },
            _ => EdgeAnimation::None,
        }
    }

    pub fn is_interacting(&self) -> bool {
        self.panning
            || self.dragging_minimap
            || self.dragging_nodes
            || self.resizing_node.is_some()
            || self.connecting_from.is_some()
            || self.dragging_waypoint.is_some()
            || self.marquee_screen.is_some()
            || self.dragging_stencil.is_some()
    }

    pub fn has_live_animations(&self) -> bool {
        if self.collab.active_duel.is_some() || self.bridge.active_transition.is_some() {
            return true;
        }
        for e in self.bridge.scene.edges() {
            if !matches!(e.animation, EdgeAnimation::None) {
                return true;
            }
        }
        for n in self.bridge.scene.nodes() {
            if n.style.border_anim {
                return true;
            }
        }
        false
    }

    pub fn new_blank_diagram(&mut self) {
        let w = self.bridge.viewport.screen_width.max(1280.0);
        let h = self.bridge.viewport.screen_height.max(720.0);
        self.bridge.document = rustflow_core::Document::new("Untitled Diagram", w, h);
        self.bridge.scene = SceneGraph::new();
        self.bridge.selection.clear();
        self.bridge.viewport.zoom = 1.0;
        self.bridge.viewport.pan = Point::ZERO;
        self.bridge.sync_active_page_to_document();
        self.image_textures.clear();
        self.undo_stack.clear();
        self.redo_stack.clear();
        if !self.collab.connected {
            let fresh = platform::generate_unique_room_id();
            self.collab_ws_url = platform::default_ws_url(&fresh);
            self.collab_room_input = fresh;
        }
        self.set_status("Created blank diagram");
        self.broadcast_scene_snapshot();
    }

    pub fn load_demo_scene(&mut self) {
        let w = self.bridge.viewport.screen_width.max(1280.0);
        let h = self.bridge.viewport.screen_height.max(720.0);
        self.bridge.document = rustflow_core::Document::new("RustFlow Architecture", w, h);
        self.bridge.document.active_page_mut().name = "Page-1: RustFlow Architecture".to_string();
        self.bridge.scene = build_dogfood_architecture_scene();
        self.bridge.scene.normalize_layer_order();
        self.bridge.selection.clear();
        self.fit_view();
        self.bridge.sync_active_page_to_document();
        self.undo_stack.clear();
        self.redo_stack.clear();
        self.set_status("Loaded sample architecture");
        self.broadcast_scene_snapshot();
    }

    pub fn load_flowchart_template(&mut self) {
        let w = self.bridge.viewport.screen_width.max(1280.0);
        let h = self.bridge.viewport.screen_height.max(720.0);
        self.bridge.document = rustflow_core::Document::new("Flowchart", w, h);
        self.bridge.document.active_page_mut().name = "Page-1: Flowchart".to_string();
        let mut s = SceneGraph::new();
        let a = Node::new_shape(1, "Start", Rect::new(140.0, 80.0, 140.0, 56.0), ShapeKind::RoundedRect);
        let b = Node::new_shape(2, "Valid?", Rect::new(140.0, 190.0, 140.0, 86.0), ShapeKind::Diamond);
        let c = Node::new_shape(3, "Process Data", Rect::new(360.0, 200.0, 160.0, 66.0), ShapeKind::ReactFlowCard);
        let d = Node::new_shape(4, "Retry", Rect::new(140.0, 330.0, 140.0, 56.0), ShapeKind::RoundedRect);
        for n in [a, b, c, d] {
            s.insert_node(n);
        }
        let mut e1 = Edge::new_bezier(10, 1, 2);
        e1.kind = EdgeKind::SmoothStep;
        let mut e2 = Edge::new_bezier(11, 2, 3);
        e2.kind = EdgeKind::SmoothStep;
        e2.label = "Yes".into();
        e2.animation = EdgeAnimation::MarchingAnts {
            speed: 45.0,
            dash_length: 8.0,
            gap_length: 5.0,
            reverse: false,
        };
        let mut e3 = Edge::new_bezier(12, 2, 4);
        e3.kind = EdgeKind::SmoothStep;
        e3.label = "No".into();
        for e in [e1, e2, e3] {
            s.insert_edge(e);
        }
        s.normalize_layer_order();
        self.bridge.scene = s;
        self.bridge.selection.clear();
        self.fit_view();
        self.bridge.sync_active_page_to_document();
        self.undo_stack.clear();
        self.redo_stack.clear();
        self.set_status("Loaded Flowchart template");
        self.broadcast_scene_snapshot();
    }

    pub fn spawn_stencil_at_world(
        &mut self,
        label: &str,
        w: f32,
        h: f32,
        shape: ShapeKind,
        icon: Option<SymbolIcon>,
        layers: Vec<SymbolLayer>,
        world_center: Point,
    ) {
        self.push_undo();
        let id = self.bridge.scene.allocate_id();
        let mut x = world_center.x - w * 0.5;
        let mut y = world_center.y - h * 0.5;
        if self.snap_to_grid {
            let g = self.bridge.viewport.grid_size.max(4.0);
            x = (x / g).round() * g;
            y = (y / g).round() * g;
        }
        let mut node = Node::new_shape(id, label, Rect::new(x, y, w, h), shape);
        node.icon = icon;
        node.custom_layers = layers;
        self.bridge.scene.insert_node(node);
        self.bridge.scene.normalize_layer_order();
        self.bridge.selection.select_node(id, false);
        self.broadcast_scene_snapshot();
    }

    pub fn spawn_stencil_at_center(
        &mut self,
        label: &str,
        w: f32,
        h: f32,
        shape: ShapeKind,
        icon: Option<SymbolIcon>,
    ) {
        let center = self.bridge.viewport.screen_to_world(Point::new(
            self.bridge.viewport.screen_width * 0.5,
            self.bridge.viewport.screen_height * 0.5,
        ));
        let offset = ((self.bridge.scene.nodes().len() % 6) as f32) * 18.0;
        self.spawn_stencil_at_world(
            label,
            w,
            h,
            shape,
            icon,
            Vec::new(),
            Point::new(center.x + offset, center.y + offset),
        );
    }

    pub fn fit_view(&mut self) {
        let mut bounds: Option<Rect> = None;
        for n in self.bridge.scene.nodes() {
            if !n.visible {
                continue;
            }
            bounds = Some(match bounds {
                Some(acc) => acc.union(&n.bounds),
                None => n.bounds,
            });
        }
        let Some(rect) = bounds else {
            self.bridge.viewport.zoom = 1.0;
            self.bridge.viewport.pan = Point::ZERO;
            return;
        };
        let padded = rect.expand(48.0);
        let sw = self.bridge.viewport.screen_width.max(320.0);
        let sh = self.bridge.viewport.screen_height.max(240.0);
        let zoom = (sw / padded.width.max(1.0))
            .min(sh / padded.height.max(1.0))
            .clamp(0.15, 2.0);
        self.bridge.viewport.zoom = zoom;
        self.bridge.viewport.pan = Point::new(
            (sw - padded.width * zoom) * 0.5 - padded.x * zoom,
            (sh - padded.height * zoom) * 0.5 - padded.y * zoom,
        );
    }

    pub fn zoom_step(&mut self, factor: f32) {
        let cx = self.bridge.viewport.screen_width * 0.5;
        let cy = self.bridge.viewport.screen_height * 0.5;
        let nz = (self.bridge.viewport.zoom * factor).clamp(0.15, 4.0);
        self.bridge.zoom_at(nz, cx, cy);
    }

    pub fn duplicate_selection(&mut self) {
        let ids = self.bridge.selection.selected_nodes.clone();
        if ids.is_empty() {
            return;
        }
        self.push_undo();
        let mut new_ids = Vec::new();
        for id in ids {
            if let Some(orig) = self.bridge.scene.get_node(id).cloned() {
                let nid = self.bridge.scene.allocate_id();
                let mut dup = orig;
                dup.id = nid;
                dup.bounds.x += 28.0;
                dup.bounds.y += 28.0;
                dup.rebuild_ports();
                self.bridge.scene.insert_node(dup);
                new_ids.push(nid);
            }
        }
        self.bridge.scene.normalize_layer_order();
        self.bridge.selection.clear();
        self.bridge.selection.selected_nodes = new_ids;
        self.broadcast_scene_snapshot();
    }

    pub fn delete_selection(&mut self) {
        let nids = self.bridge.selection.selected_nodes.clone();
        let eids = self.bridge.selection.selected_edges.clone();
        if nids.is_empty() && eids.is_empty() {
            return;
        }
        self.push_undo();
        for id in nids {
            self.bridge.scene.remove_node(id);
        }
        for id in eids {
            self.bridge.scene.remove_edge(id);
        }
        self.bridge.selection.clear();
        self.broadcast_scene_snapshot();
    }

    fn current_export_bg(&self) -> ExportBgStyle {
        if !self.show_grid {
            return ExportBgStyle::None;
        }
        match self.bg_style {
            CanvasBgStyle::Dots => ExportBgStyle::Dots,
            CanvasBgStyle::Grid => ExportBgStyle::Grid,
            CanvasBgStyle::Crosshair => ExportBgStyle::Crosshair,
            CanvasBgStyle::Isometric => ExportBgStyle::Isometric,
            CanvasBgStyle::None => ExportBgStyle::None,
        }
    }

    pub fn copy_diagram_png(&mut self) {
        let clear = canvas_clear_rgba(self.theme_mode);
        let bg = self.current_export_bg();
        match export_png_bytes_with_bg(&self.bridge.scene, 1440, 840, 0.0, clear, bg) {
            Ok(bytes) => match platform::copy_png_to_clipboard(&bytes) {
                Ok(msg) => self.set_status(msg),
                Err(err) => self.set_status(err),
            },
            Err(err) => self.set_status(format!("PNG error: {err}")),
        }
    }

    pub fn copy_diagram_gif(&mut self) {
        let clear = canvas_clear_rgba(self.theme_mode);
        let bg = self.current_export_bg();
        match export_animated_gif_bytes_with_bg(&self.bridge.scene, 720, 420, 16, 10, clear, bg) {
            Ok(bytes) => match platform::copy_animated_gif_or_save(&bytes) {
                Ok(msg) => self.set_status(msg),
                Err(err) => self.set_status(err),
            },
            Err(err) => self.set_status(format!("GIF error: {err}")),
        }
    }

    pub fn export_json_file(&mut self) {
        match export_scene_serde_json(&self.bridge.scene) {
            Ok(json) => match platform::save_bytes_dialog("diagram.bohat.json", json.as_bytes()) {
                Ok(msg) => self.status_message = msg,
                Err(err) => self.status_message = err,
            },
            Err(err) => self.status_message = format!("JSON error: {err}"),
        }
    }

    pub fn export_drawio_file(&mut self) {
        let xml = export_drawio_xml(&self.bridge.scene, "Page-1");
        match platform::save_bytes_dialog("diagram.drawio", xml.as_bytes()) {
            Ok(msg) => self.status_message = msg,
            Err(err) => self.status_message = err,
        }
    }

    pub fn export_svg_file(&mut self) {
        let svg = export_animated_svg(&self.bridge.scene);
        match platform::save_bytes_dialog("diagram.svg", svg.as_bytes()) {
            Ok(msg) => self.status_message = msg,
            Err(err) => self.status_message = err,
        }
    }

    pub fn export_png_file(&mut self) {
        let clear = canvas_clear_rgba(self.theme_mode);
        let bg = self.current_export_bg();
        match export_png_bytes_with_bg(&self.bridge.scene, 1600, 960, 0.0, clear, bg) {
            Ok(bytes) => match platform::save_bytes_dialog("diagram-2x.png", &bytes) {
                Ok(msg) => self.set_status(msg),
                Err(err) => self.set_status(err),
            },
            Err(err) => self.set_status(format!("PNG error: {err}")),
        }
    }

    pub fn export_gif_file(&mut self) {
        let clear = canvas_clear_rgba(self.theme_mode);
        let bg = self.current_export_bg();
        match export_animated_gif_bytes_with_bg(&self.bridge.scene, 760, 460, 18, 10, clear, bg) {
            Ok(bytes) => match platform::save_bytes_dialog("diagram-animated.gif", &bytes) {
                Ok(msg) => self.set_status(msg),
                Err(err) => self.set_status(err),
            },
            Err(err) => self.set_status(format!("GIF error: {err}")),
        }
    }

    pub fn trigger_image_upload(&mut self, target_node: Option<NodeId>, ctx: &egui::Context) {
        if let Some((name, bytes)) = platform::pick_image_file_sync() {
            self.apply_image_bytes(target_node, name, bytes, ctx);
        } else {
            platform::trigger_wasm_image_picker(
                target_node,
                Rc::clone(&self.pending_image_upload),
                ctx.clone(),
            );
        }
    }

    pub fn apply_image_bytes(
        &mut self,
        target_node: Option<NodeId>,
        name: String,
        bytes: Vec<u8>,
        ctx: &egui::Context,
    ) {
        let decoded = match image::load_from_memory(&bytes) {
            Ok(img) => img.to_rgba8(),
            Err(err) => {
                self.set_status(format!("Invalid image: {err}"));
                return;
            }
        };
        let (iw, ih) = decoded.dimensions();
        let color_img = egui::ColorImage::from_rgba_unmultiplied(
            [iw as usize, ih as usize],
            decoded.as_raw(),
        );
        self.push_undo();
        let nid = if let Some(existing_id) = target_node {
            if let Some(node) = self.bridge.scene.get_node_mut(existing_id) {
                node.shape = ShapeKind::ImageNode;
                node.image_data = Some(bytes);
                if node.title == "Image Node" || node.title == "Image" {
                    node.title = name.clone();
                }
            }
            existing_id
        } else {
            let center = self.bridge.viewport.screen_to_world(Point::new(
                self.bridge.viewport.screen_width * 0.5,
                self.bridge.viewport.screen_height * 0.5,
            ));
            let aspect = (iw as f32 / ih.max(1) as f32).clamp(0.35, 3.0);
            let w = 220.0_f32;
            let h = (w / aspect).clamp(90.0, 260.0);
            let id = self.bridge.scene.allocate_id();
            let mut node = Node::new_shape(
                id,
                &name,
                Rect::new(center.x - w * 0.5, center.y - h * 0.5, w, h),
                ShapeKind::ImageNode,
            );
            node.image_data = Some(bytes);
            self.bridge.scene.insert_node(node);
            self.bridge.scene.normalize_layer_order();
            self.bridge.selection.select_node(id, false);
            id
        };
        let tex = ctx.load_texture(format!("img_node_{nid}"), color_img, egui::TextureOptions::LINEAR);
        self.image_textures.insert(nid, tex);
        self.set_status(format!("Loaded image '{name}' ({iw}x{ih})"));
        self.broadcast_scene_snapshot();
    }

    fn sync_image_uploads_and_textures(&mut self, ctx: &egui::Context) {
        let pending = self.pending_image_upload.borrow_mut().take();
        if let Some((target_node, name, bytes)) = pending {
            self.apply_image_bytes(target_node, name, bytes, ctx);
        }

        let dropped: Vec<(String, Vec<u8>)> = ctx.input(|i| {
            i.raw
                .dropped_files
                .iter()
                .filter_map(|f| {
                    let bytes = if let Some(b) = &f.bytes {
                        b.to_vec()
                    } else if let Some(p) = &f.path {
                        std::fs::read(p).ok()?
                    } else {
                        return None;
                    };
                    let name = if !f.name.is_empty() {
                        f.name.clone()
                    } else {
                        f.path
                            .as_ref()
                            .and_then(|p| p.file_name())
                            .map(|s| s.to_string_lossy().to_string())
                            .unwrap_or_else(|| "Image".to_string())
                    };
                    Some((name, bytes))
                })
                .collect()
        });
        for (name, bytes) in dropped {
            let target = self
                .bridge
                .selection
                .selected_nodes
                .first()
                .copied()
                .filter(|&id| {
                    self.bridge
                        .scene
                        .get_node(id)
                        .is_some_and(|n| matches!(n.shape, ShapeKind::ImageNode))
                });
            self.apply_image_bytes(target, name, bytes, ctx);
        }

        let missing: Vec<(NodeId, Vec<u8>)> = self
            .bridge
            .scene
            .nodes()
            .iter()
            .filter_map(|n| {
                let data = n.image_data.as_ref()?;
                if !self.image_textures.contains_key(&n.id) {
                    Some((n.id, data.clone()))
                } else {
                    None
                }
            })
            .collect();
        for (nid, data) in missing {
            if let Ok(img) = image::load_from_memory(&data) {
                let rgba = img.to_rgba8();
                let (iw, ih) = rgba.dimensions();
                let ci = egui::ColorImage::from_rgba_unmultiplied(
                    [iw as usize, ih as usize],
                    rgba.as_raw(),
                );
                let tex = ctx.load_texture(format!("img_node_{nid}"), ci, egui::TextureOptions::LINEAR);
                self.image_textures.insert(nid, tex);
            }
        }
    }

    pub fn open_file(&mut self) {
        let Some((name, content)) = platform::open_text_dialog() else {
            return;
        };
        let loaded = if content.trim_start().starts_with('<') {
            import_drawio_xml(&content)
        } else {
            import_scene_serde_json(&content)
        };
        match loaded {
            Ok(scene) => {
                let w = self.bridge.viewport.screen_width.max(1280.0);
                let h = self.bridge.viewport.screen_height.max(720.0);
                self.bridge.document = rustflow_core::Document::new(&name, w, h);
                self.bridge.document.active_page_mut().name = format!("Page-1: {name}");
                self.bridge.scene = scene;
                self.bridge.selection.clear();
                self.fit_view();
                self.bridge.sync_active_page_to_document();
                self.undo_stack.clear();
                self.redo_stack.clear();
                self.set_status(format!("Opened {name}"));
                self.broadcast_scene_snapshot();
            }
            Err(err) => self.status_message = format!("Open error: {err}"),
        }
    }

    pub fn broadcast_scene_snapshot(&mut self) {
        if !self.collab.connected {
            return;
        }
        if let Ok(scene_json) = export_scene_serde_json(&self.bridge.scene) {
            self.collab.send(&ClientMsg::Snapshot { scene_json });
        }
    }

    pub fn maybe_send_cursor(&mut self, world_pt: Point) {
        if !self.collab.connected {
            return;
        }
        self.last_cursor_sent += 1.0;
        if (self.last_cursor_sent as u64) % 3 == 0 {
            self.collab.send(&ClientMsg::Cursor {
                x: world_pt.x,
                y: world_pt.y,
            });
        }
    }

    fn handle_shortcuts(&mut self, ctx: &egui::Context) {
        if ctx.wants_keyboard_input() || self.editing_node.is_some() {
            return;
        }
        ctx.input(|i| {
            let cmd = i.modifiers.command || i.modifiers.ctrl;
            if i.key_pressed(egui::Key::Delete) || i.key_pressed(egui::Key::Backspace) {
                self.delete_selection();
            }
            if cmd && i.key_pressed(egui::Key::Z) {
                if i.modifiers.shift {
                    self.redo();
                } else {
                    self.undo();
                }
            }
            if cmd && i.key_pressed(egui::Key::Y) {
                self.redo();
            }
            let has_copy_ev = i.events.iter().any(|ev| matches!(ev, egui::Event::Copy));
            let has_cut_ev = i.events.iter().any(|ev| matches!(ev, egui::Event::Cut));
            let has_paste_ev = i
                .events
                .iter()
                .any(|ev| matches!(ev, egui::Event::Paste(_)));

            if has_copy_ev || (cmd && i.key_pressed(egui::Key::C)) {
                self.copy_selection();
            }
            if has_cut_ev || (cmd && i.key_pressed(egui::Key::X)) {
                self.copy_selection();
                self.delete_selection();
            }
            if has_paste_ev || (cmd && i.key_pressed(egui::Key::V)) {
                self.paste_clipboard();
            }
            if cmd && i.key_pressed(egui::Key::A) {
                self.select_all();
            }
            if cmd && i.key_pressed(egui::Key::D) {
                self.duplicate_selection();
            }
            if cmd && i.key_pressed(egui::Key::K) {
                self.show_cmd_palette = !self.show_cmd_palette;
            }
            if cmd && (i.key_pressed(egui::Key::Num0) || i.key_pressed(egui::Key::Num1)) {
                self.fit_view();
            }
            if cmd && i.key_pressed(egui::Key::Plus) {
                self.zoom_step(1.15);
            }
            if cmd && i.key_pressed(egui::Key::Minus) {
                self.zoom_step(0.85);
            }
            if i.key_pressed(egui::Key::Questionmark) {
                self.show_shortcuts = !self.show_shortcuts;
            }
            let step = if i.modifiers.shift { 10.0 } else { 2.0 };
            let mut dx = 0.0;
            let mut dy = 0.0;
            if i.key_pressed(egui::Key::ArrowLeft) {
                dx -= step;
            }
            if i.key_pressed(egui::Key::ArrowRight) {
                dx += step;
            }
            if i.key_pressed(egui::Key::ArrowUp) {
                dy -= step;
            }
            if i.key_pressed(egui::Key::ArrowDown) {
                dy += step;
            }
            if dx != 0.0 || dy != 0.0 {
                let ids = self.bridge.selection.selected_nodes.clone();
                for id in ids {
                    self.bridge.scene.move_node_with_descendants(id, dx, dy);
                }
            }
        });
    }

    fn poll_collab(&mut self) {
        for msg in self.collab.poll() {
            match msg {
                ServerMsg::Welcome {
                    snapshot: Some(json),
                    ..
                }
                | ServerMsg::Snapshot {
                    scene_json: json, ..
                } => {
                    if let Ok(scene) = import_scene_serde_json(&json) {
                        self.bridge.scene = scene;
                    }
                }
                ServerMsg::DuelEnded {
                    winner_name,
                    challenger_taps,
                    defender_taps,
                    ..
                } => {
                    self.status_message = format!(
                        "Duel winner: {winner_name} ({challenger_taps} vs {defender_taps})"
                    );
                }
                _ => {}
            }
        }
    }
}

impl eframe::App for BohatDrawApp {
    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        desktop_bg_color(self.theme_mode).to_normalized_gamma_f32()
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if !self.theme_applied {
            apply_theme(ctx, self.theme_mode);
            self.theme_applied = true;
        }

        let (dt, now) = ctx.input(|i| (i.stable_dt.clamp(0.002, 0.1), i.time));
        let frame_dt = if self.last_time > 0.0 {
            ((now - self.last_time) as f32).clamp(0.002, 0.1)
        } else {
            dt
        };
        self.last_time = now;
        self.fps = self.fps * 0.92 + (1.0 / frame_dt).clamp(1.0, 240.0) * 0.08;

        if let Some((tz, sx, sy)) = self.target_zoom {
            let cur = self.bridge.viewport.zoom;
            let diff = tz - cur;
            if diff.abs() < 0.002 {
                self.bridge.zoom_at(tz, sx, sy);
                self.target_zoom = None;
            } else {
                let alpha = 1.0 - (-26.0 * dt).exp();
                self.bridge.zoom_at(cur + diff * alpha, sx, sy);
                ctx.request_repaint();
            }
        }

        if let Some(mut tr) = self.bridge.active_transition.take() {
            if !tr.step(dt, &mut self.bridge.scene) {
                self.bridge.active_transition = Some(tr);
            }
        }
        self.sync_image_uploads_and_textures(ctx);
        self.poll_collab();
        self.handle_shortcuts(ctx);

        let screen_w = ctx.screen_rect().width();
        let narrow = screen_w <= 960.0;
        let is_phone = screen_w <= 640.0;
        if !self.layout_initialized && screen_w > 64.0 {
            self.layout_initialized = true;
            self.last_was_narrow = narrow;
            if narrow {
                self.show_palette = false;
                self.show_inspector = false;
                self.hand_pan_mode = true;
            }
        } else if narrow && !self.last_was_narrow {
            self.last_was_narrow = true;
            if is_phone {
                self.show_palette = false;
                self.show_inspector = false;
            } else if self.show_palette && self.show_inspector {
                self.show_inspector = false;
            }
            self.hand_pan_mode = true;
        } else if !narrow && self.last_was_narrow {
            self.last_was_narrow = false;
            self.show_palette = true;
            self.show_inspector = true;
        }
        if narrow && self.show_palette && self.show_inspector {
            self.show_inspector = false;
        }

        render_menu_and_toolbar(self, ctx, narrow);
        render_bottom_bar(self, ctx);
        render_palette_panel(self, ctx);
        render_inspector_panel(self, ctx);

        egui::CentralPanel::default()
            .frame(
                egui::Frame::new()
                    .fill(desktop_bg_color(self.theme_mode))
                    .inner_margin(Margin::same(if is_phone { 2 } else { 4 })),
            )
            .show(ctx, |ui| {
                platinum_window_frame(self.theme_mode).show(ui, |ui| {
                    ui.vertical(|ui| {
                        let canvas_title = if is_phone {
                            "Bohat Draw"
                        } else {
                            match self.theme_mode {
                                ThemeMode::Light => "Bohat Draw Canvas — Platinum 90s",
                                ThemeMode::Dark => "Bohat Draw Canvas — Classic Dark",
                                ThemeMode::HighContrast => "Bohat Draw Canvas — High Contrast",
                            }
                        };
                        let _ = draw_platinum_titlebar(ui, self.theme_mode, canvas_title, None);
                        render_canvas(self, ui);
                    });
                });
            });

        render_collab_and_duel_modals(self, ctx);
        render_extra_modals(self, ctx);
    }
}
