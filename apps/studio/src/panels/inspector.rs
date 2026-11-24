#![allow(unused_imports)]
use crate::app::{
    BohatDrawApp, CanvasBgStyle, CustomStencil, DragStencilPayload, PaletteViewMode,
    StudioInspectorTab,
};
use crate::canvas::draw_simple_layer_shape;
use crate::theme::{
    desktop_bg_color, draw_eye_toggle, draw_lock_toggle, draw_platinum_titlebar, draw_prism_badge,
    draw_stencil_preview, draw_symbol_icon, from_color32, icon_button, platinum_window_frame,
    symbol_icon_accent, to_color32, ToolIcon, COLOR_SWATCHES,
};
use egui::{Align2, Color32, FontId, Frame, Margin, RichText, ScrollArea, Sense, Stroke, Vec2};
use rustflow_collab_protocol::ClientMsg;
use rustflow_core::{
    BorderStyle, EdgeAnimation, EdgeArrowStyle, EdgeKind, PortDensity, Rect, ShapeKind,
    SymbolLayer, ThemeMode,
};
use rustflow_layout::{
    align_nodes, distribute_nodes, layout_cascade_pile, layout_heap_pile, layout_overlap_stack,
    layout_separate_overlaps, AlignMode, DistributeAxis, LayoutAlgorithm,
};
use rustflow_ui_bridge::StencilCategory;


fn inspector_card(ui: &mut egui::Ui, title: &str, add_contents: impl FnOnce(&mut egui::Ui)) {
    let card_inner_w = (ui.available_width() - 22.0).max(180.0);
    Frame::new()
        .fill(Color32::from_rgb(232, 233, 238))
        .stroke(Stroke::new(1.0_f32, Color32::from_rgb(156, 160, 174)))
        .corner_radius(3)
        .inner_margin(Margin::same(8))
        .show(ui, |ui| {
            ui.set_min_width(card_inner_w);
            ui.set_max_width(card_inner_w);
            ui.spacing_mut().slider_width = (card_inner_w - 116.0).clamp(70.0, 145.0);
            ui.spacing_mut().slider_rail_height = 10.0;
            ui.visuals_mut().slider_trailing_fill = true;
            ui.label(
                RichText::new(title)
                    .strong()
                    .size(11.5)
                    .color(Color32::from_rgb(20, 24, 38)),
            );
            ui.add_space(4.0);
            add_contents(ui);
        });
    ui.add_space(6.0);
}

fn seg_btn(ui: &mut egui::Ui, selected: bool, label: &str, width: f32) -> egui::Response {
    let btn = egui::Button::new(
        RichText::new(label)
            .size(11.0)
            .color(if selected { Color32::WHITE } else { Color32::from_rgb(22, 24, 32) }),
    )
    .fill(if selected {
        Color32::from_rgb(0, 0, 128)
    } else {
        Color32::from_rgb(242, 243, 247)
    });
    ui.add_sized([width, 24.0], btn)
}

pub fn render_inspector_panel(app: &mut BohatDrawApp, ctx: &egui::Context) {
    if !app.show_inspector {
        return;
    }
    let screen_w = ctx.screen_rect().width();
    let is_phone = screen_w <= 640.0;
    let (def_w, min_w, max_w) = if is_phone {
        (
            (screen_w * 0.82).clamp(210.0, 276.0),
            196.0,
            (screen_w * 0.88).clamp(220.0, 296.0),
        )
    } else {
        (304.0, 260.0, 380.0)
    };
    egui::SidePanel::right("inspector_panel")
        .default_width(def_w)
        .min_width(min_w)
        .max_width(max_w)
        .resizable(true)
        .show_separator_line(false)
        .frame(
            Frame::new()
                .fill(desktop_bg_color(app.theme_mode))
                .inner_margin(Margin {
                    left: 0,
                    right: 4,
                    top: 4,
                    bottom: 4,
                }),
        )
        .show(ctx, |ui| {
            let max_panel_w = ui.max_rect().width();
            ui.set_max_width(max_panel_w);
            platinum_window_frame(app.theme_mode).show(ui, |ui| {
                let frame_w = ui.available_width();
                ui.set_max_width(frame_w);
                ui.set_min_size(ui.available_size());
                ui.vertical(|ui| {
                    let badge = if app.bridge.selection.selected_nodes.len() > 1 {
                        format!("Multi ({})", app.bridge.selection.selected_nodes.len())
                    } else if !app.bridge.selection.selected_nodes.is_empty() {
                        "Node".to_string()
                    } else if !app.bridge.selection.selected_edges.is_empty() {
                        "Edge".to_string()
                    } else {
                        "Diagram".to_string()
                    };
                    if draw_platinum_titlebar(ui, app.theme_mode, "Inspector", Some(&badge)) {
                        app.show_inspector = false;
                    }

                    Frame::new().inner_margin(Margin::same(8)).show(ui, |ui| {
                        let inner_w = ui.available_width();
                        ui.set_max_width(inner_w);
                        let gap = ui.spacing().item_spacing.x;
                        let tab_w = ((inner_w - gap * 3.0 - 2.0) / 4.0).floor().max(42.0);
                        ui.horizontal(|ui| {
                            for (tab, label) in [
                                (StudioInspectorTab::Style, "Style"),
                                (StudioInspectorTab::Text, "Text"),
                                (StudioInspectorTab::Arrange, "Arrange"),
                                (StudioInspectorTab::Layers, "Layers"),
                            ] {
                                if seg_btn(ui, app.inspector_tab == tab, label, tab_w).clicked() {
                                    app.inspector_tab = tab;
                                }
                            }
                        });
                        ui.add_space(6.0);

                        ScrollArea::vertical()
                            .auto_shrink([false, false])
                            .show(ui, |ui| match app.inspector_tab {
                                StudioInspectorTab::Style => render_style_tab(app, ui),
                                StudioInspectorTab::Text => render_text_tab(app, ui),
                                StudioInspectorTab::Arrange => render_arrange_tab(app, ui),
                                StudioInspectorTab::Layers => render_layers_tab(app, ui),
                            });
                    });
                });
            });
        });
}

fn render_color_swatches(ui: &mut egui::Ui) -> Option<Color32> {
    let mut picked = None;
    let gap = ui.spacing().item_spacing.x;
    let sw = ((ui.available_width() - gap * 7.0) / 8.0).floor().clamp(16.0, 26.0);
    for row in COLOR_SWATCHES.chunks(8) {
        ui.horizontal(|ui| {
            for &[r, g, b] in row {
                let col = Color32::from_rgb(r, g, b);
                let (rect, resp) = ui.allocate_exact_size(Vec2::splat(sw), Sense::click());
                ui.painter().rect_filled(rect, 3.0, col);
                let border = if resp.hovered() {
                    Color32::from_rgb(0, 0, 140)
                } else {
                    Color32::from_rgb(90, 94, 108)
                };
                ui.painter().rect_stroke(
                    rect,
                    3.0,
                    Stroke::new(if resp.hovered() { 1.8_f32 } else { 1.0_f32 }, border),
                    egui::StrokeKind::Inside,
                );
                if resp.clicked() {
                    picked = Some(col);
                }
            }
        });
    }
    picked
}

fn render_border_style_grid(ui: &mut egui::Ui, current: &mut BorderStyle) -> bool {
    let mut changed = false;
    let gap = ui.spacing().item_spacing.x;
    let btn_w = ((ui.available_width() - gap * 2.0) / 3.0).floor().max(52.0);
    let styles = [
        ("Solid", BorderStyle::Solid),
        ("Dashed", BorderStyle::Dashed),
        ("Dotted", BorderStyle::Dotted),
        ("DashDot", BorderStyle::DashDot),
        ("Double", BorderStyle::Double),
        ("None", BorderStyle::None),
    ];
    for chunk in styles.chunks(3) {
        ui.horizontal(|ui| {
            for &(tip, bs) in chunk {
                let selected = *current == bs;
                let (rect, resp) = ui.allocate_exact_size(Vec2::new(btn_w, 26.0), Sense::click());
                let bg = if selected {
                    Color32::from_rgb(0, 0, 128)
                } else if resp.hovered() {
                    Color32::from_rgb(246, 248, 255)
                } else {
                    Color32::WHITE
                };
                let fg = if selected { Color32::WHITE } else { Color32::from_rgb(24, 28, 40) };
                ui.painter().rect_filled(rect, 2.0, bg);
                ui.painter().rect_stroke(
                    rect,
                    2.0,
                    Stroke::new(1.0_f32, Color32::from_rgb(80, 84, 98)),
                    egui::StrokeKind::Inside,
                );
                let y = rect.center().y;
                let x0 = rect.min.x + 8.0;
                let x1 = rect.max.x - 8.0;
                match bs {
                    BorderStyle::Solid => {
                        ui.painter().line_segment([egui::Pos2::new(x0, y), egui::Pos2::new(x1, y)], Stroke::new(2.0_f32, fg));
                    }
                    BorderStyle::Dashed => {
                        let mut x = x0;
                        while x < x1 - 4.0 {
                            ui.painter().line_segment([egui::Pos2::new(x, y), egui::Pos2::new((x + 8.0).min(x1), y)], Stroke::new(2.0_f32, fg));
                            x += 13.0;
                        }
                    }
                    BorderStyle::Dotted => {
                        let mut x = x0 + 2.0;
                        while x < x1 {
                            ui.painter().circle_filled(egui::Pos2::new(x, y), 1.6, fg);
                            x += 6.0;
                        }
                    }
                    BorderStyle::DashDot => {
                        let mut x = x0;
                        while x < x1 - 3.0 {
                            ui.painter().line_segment([egui::Pos2::new(x, y), egui::Pos2::new((x + 8.0).min(x1), y)], Stroke::new(2.0_f32, fg));
                            if x + 12.0 < x1 {
                                ui.painter().circle_filled(egui::Pos2::new(x + 12.0, y), 1.5, fg);
                            }
                            x += 17.0;
                        }
                    }
                    BorderStyle::Double => {
                        ui.painter().line_segment([egui::Pos2::new(x0, y - 2.0), egui::Pos2::new(x1, y - 2.0)], Stroke::new(1.4_f32, fg));
                        ui.painter().line_segment([egui::Pos2::new(x0, y + 2.0), egui::Pos2::new(x1, y + 2.0)], Stroke::new(1.4_f32, fg));
                    }
                    BorderStyle::None => {
                        ui.painter().text(rect.center(), Align2::CENTER_CENTER, "None", FontId::proportional(10.5), fg);
                    }
                }
                if resp.on_hover_text(tip).clicked() {
                    *current = bs;
                    changed = true;
                }
            }
        });
    }
    changed
}

fn render_style_tab(app: &mut BohatDrawApp, ui: &mut egui::Ui) {
    if let Some(&nid) = app.bridge.selection.selected_nodes.first() {
        let mut changed = false;
        let mut trigger_img_upload = false;
        let mut clear_img = false;
        if let Some(node) = app.bridge.scene.get_node_mut(nid) {
            if matches!(node.shape, ShapeKind::ImageNode) {
                inspector_card(ui, "Image", |ui| {
                    ui.horizontal(|ui| {
                        if ui.add_sized([120.0, 26.0], egui::Button::new("Upload Image...")).clicked() {
                            trigger_img_upload = true;
                        }
                        if node.image_data.is_some()
                            && ui.add_sized([68.0, 26.0], egui::Button::new("Clear")).clicked()
                        {
                            node.image_data = None;
                            clear_img = true;
                            changed = true;
                        }
                    });
                });
            }

            inspector_card(ui, "Colors", |ui| {
                if let Some(sw) = render_color_swatches(ui) {
                    if ui.input(|i| i.modifiers.shift) {
                        node.style.stroke = from_color32(sw);
                    } else {
                        node.style.fill = from_color32(sw);
                    }
                    changed = true;
                }
                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    ui.label("Fill");
                    let mut c = to_color32(node.style.fill);
                    if ui.color_edit_button_srgba(&mut c).changed() {
                        node.style.fill = from_color32(c);
                        changed = true;
                    }
                    ui.add_space(10.0);
                    ui.label("Stroke");
                    let mut s = to_color32(node.style.stroke);
                    if ui.color_edit_button_srgba(&mut s).changed() {
                        node.style.stroke = from_color32(s);
                        changed = true;
                    }
                });
            });

            inspector_card(ui, "Border & Shape", |ui| {
                if render_border_style_grid(ui, &mut node.style.border_style) {
                    changed = true;
                }
                ui.add_space(4.0);
                if ui.add(egui::Slider::new(&mut node.style.stroke_width, 0.5..=8.0).text("Width")).changed() {
                    changed = true;
                }
                if ui.add(egui::Slider::new(&mut node.style.corner_radius, 0.0..=28.0).text("Radius")).changed() {
                    changed = true;
                }
                if ui.add(egui::Slider::new(&mut node.style.opacity, 0.1..=1.0).text("Opacity")).changed() {
                    changed = true;
                }
                if ui.checkbox(&mut node.style.border_anim, "Animated Border").changed() {
                    changed = true;
                }
            });

            inspector_card(ui, "Connect Points / Side", |ui| {
                let gap = ui.spacing().item_spacing.x;
                let bw = ((ui.available_width() - gap * 4.0) / 5.0).floor().max(32.0);
                ui.horizontal(|ui| {
                    for (lbl, val) in [
                        ("Auto", PortDensity::Auto),
                        ("1", PortDensity::Fixed(1)),
                        ("3", PortDensity::Fixed(3)),
                        ("5", PortDensity::Fixed(5)),
                        ("7", PortDensity::Fixed(7)),
                    ] {
                        if seg_btn(ui, node.port_density == val, lbl, bw).clicked() {
                            node.port_density = val;
                            node.rebuild_ports();
                            changed = true;
                        }
                    }
                });
            });
        }
        if clear_img {
            app.image_textures.remove(&nid);
        }
        if trigger_img_upload {
            let ctx = ui.ctx().clone();
            app.trigger_image_upload(Some(nid), &ctx);
        }
        if changed {
            app.broadcast_scene_snapshot();
        }
    } else if let Some(&eid) = app.bridge.selection.selected_edges.first() {
        let mut changed = false;
        if let Some(edge) = app.bridge.scene.get_edge_mut(eid) {
            inspector_card(ui, "Line & Color", |ui| {
                if let Some(sw) = render_color_swatches(ui) {
                    edge.stroke = from_color32(sw);
                    changed = true;
                }
                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    ui.label("Color");
                    let mut c = to_color32(edge.stroke);
                    if ui.color_edit_button_srgba(&mut c).changed() {
                        edge.stroke = from_color32(c);
                        changed = true;
                    }
                });
                if ui.add(egui::Slider::new(&mut edge.stroke_width, 1.0..=6.0).text("Width")).changed() {
                    changed = true;
                }
            });

            inspector_card(ui, "Routing", |ui| {
                let gap = ui.spacing().item_spacing.x;
                let bw = ((ui.available_width() - gap) / 2.0).floor().max(64.0);
                ui.horizontal(|ui| {
                    if seg_btn(ui, edge.kind == EdgeKind::Bezier, "Bezier", bw).clicked() {
                        edge.kind = EdgeKind::Bezier;
                        changed = true;
                    }
                    if seg_btn(ui, edge.kind == EdgeKind::SmoothStep, "Smooth", bw).clicked() {
                        edge.kind = EdgeKind::SmoothStep;
                        changed = true;
                    }
                });
                ui.horizontal(|ui| {
                    if seg_btn(ui, edge.kind == EdgeKind::Orthogonal, "Ortho", bw).clicked() {
                        edge.kind = EdgeKind::Orthogonal;
                        changed = true;
                    }
                    if seg_btn(ui, edge.kind == EdgeKind::Straight, "Straight", bw).clicked() {
                        edge.kind = EdgeKind::Straight;
                        changed = true;
                    }
                });
                ui.add_space(4.0);
                if !edge.waypoints.is_empty() {
                    ui.horizontal(|ui| {
                        ui.label(
                            RichText::new(format!("Manual ({} pt)", edge.waypoints.len()))
                                .small()
                                .strong()
                                .color(Color32::from_rgb(0, 0, 140)),
                        );
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.small_button("Reset Auto").clicked() {
                                edge.waypoints.clear();
                                changed = true;
                            }
                        });
                    });
                } else {
                    ui.label(
                        RichText::new("Drag edge midpoint handles to override auto-route")
                            .small()
                            .weak(),
                    );
                }
            });

            inspector_card(ui, "Arrowhead", |ui| {
                let gap = ui.spacing().item_spacing.x;
                let bw = ((ui.available_width() - gap * 4.0) / 5.0).floor().max(32.0);
                ui.horizontal(|ui| {
                    for (lbl, arr) in [
                        ("->", EdgeArrowStyle::OneWay),
                        ("<->", EdgeArrowStyle::TwoWay),
                        ("---", EdgeArrowStyle::None),
                        (">", EdgeArrowStyle::Open),
                        ("<>", EdgeArrowStyle::Diamond),
                    ] {
                        if seg_btn(ui, edge.arrow == arr, lbl, bw).clicked() {
                            edge.arrow = arr;
                            changed = true;
                        }
                    }
                });
            });

            inspector_card(ui, "Flow Animation", |ui| {
                let gap = ui.spacing().item_spacing.x;
                let bw = ((ui.available_width() - gap * 2.0) / 3.0).floor().max(52.0);
                let is_none = matches!(edge.animation, EdgeAnimation::None);
                let is_ants = matches!(edge.animation, EdgeAnimation::MarchingAnts { .. });
                let is_pulse = matches!(edge.animation, EdgeAnimation::PulseParticles { .. });
                ui.horizontal(|ui| {
                    if seg_btn(ui, is_none, "Static", bw).clicked() {
                        edge.animation = EdgeAnimation::None;
                        changed = true;
                    }
                    if seg_btn(ui, is_ants, "Ants", bw).clicked() {
                        edge.animation = EdgeAnimation::MarchingAnts {
                            speed: 45.0,
                            dash_length: 8.0,
                            gap_length: 5.0,
                            reverse: false,
                        };
                        changed = true;
                    }
                    if seg_btn(ui, is_pulse, "Pulse", bw).clicked() {
                        edge.animation = EdgeAnimation::PulseParticles {
                            speed: 65.0,
                            particle_count: 3,
                            radius: 4.0,
                            color: edge.stroke,
                        };
                        changed = true;
                    }
                });
            });
        }
        if changed {
            app.broadcast_scene_snapshot();
        }
    } else {
        inspector_card(ui, "Canvas Background", |ui| {
            let gap = ui.spacing().item_spacing.x;
            let bw = ((ui.available_width() - gap * 2.0) / 3.0).floor().max(52.0);
            let bgs = [
                ("Dots", CanvasBgStyle::Dots),
                ("Grid", CanvasBgStyle::Grid),
                ("Cross", CanvasBgStyle::Crosshair),
                ("Iso", CanvasBgStyle::Isometric),
                ("Solid", CanvasBgStyle::None),
            ];
            for chunk in bgs.chunks(3) {
                ui.horizontal(|ui| {
                    for &(lbl, st) in chunk {
                        if seg_btn(ui, app.bg_style == st, lbl, bw).clicked() {
                            app.bg_style = st;
                            app.show_grid = !matches!(st, CanvasBgStyle::None);
                        }
                    }
                });
            }
        });
        inspector_card(ui, "Workspace", |ui| {
            ui.checkbox(&mut app.snap_to_grid, "Snap to Grid on Drop");
            ui.checkbox(&mut app.show_minimap, "Show Minimap");
        });
    }
}

fn render_text_tab(app: &mut BohatDrawApp, ui: &mut egui::Ui) {
    if let Some(&nid) = app.bridge.selection.selected_nodes.first() {
        let mut changed = false;
        if let Some(node) = app.bridge.scene.get_node_mut(nid) {
            ui.label("Title");
            if ui.text_edit_singleline(&mut node.title).changed() {
                changed = true;
            }
            if crate::canvas::node_supports_body(&node.shape) {
                ui.label("Body");
                if ui.text_edit_multiline(&mut node.body).changed() {
                    changed = true;
                }
            }
            if matches!(node.shape, ShapeKind::FolderTree) {
                ui.separator();
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Folder Tree Items").strong());
                    if ui.small_button("+ Folder").clicked() {
                        node.tree_items.push(rustflow_core::FolderTreeItem {
                            label: "new_folder".to_string(),
                            depth: 0,
                            is_folder: true,
                            expanded: true,
                        });
                        changed = true;
                    }
                    if ui.small_button("+ File").clicked() {
                        node.tree_items.push(rustflow_core::FolderTreeItem {
                            label: "file.rs".to_string(),
                            depth: 1,
                            is_folder: false,
                            expanded: false,
                        });
                        changed = true;
                    }
                });
                let mut remove_idx = None;
                let mut swap_pair = None;
                let len = node.tree_items.len();
                for (idx, item) in node.tree_items.iter_mut().enumerate() {
                    ui.horizontal(|ui| {
                        if ui
                            .small_button(if item.is_folder { "D" } else { "F" })
                            .on_hover_text("Toggle Folder / File")
                            .clicked()
                        {
                            item.is_folder = !item.is_folder;
                            item.expanded = item.is_folder;
                            changed = true;
                        }
                        if ui.small_button("<").on_hover_text("Outdent").clicked() {
                            item.depth = item.depth.saturating_sub(1);
                            changed = true;
                        }
                        if ui.small_button(">").on_hover_text("Indent").clicked() {
                            item.depth = (item.depth + 1).min(4);
                            changed = true;
                        }
                        if ui
                            .add(egui::TextEdit::singleline(&mut item.label).desired_width(96.0))
                            .changed()
                        {
                            changed = true;
                        }
                        if idx > 0 && ui.small_button("^").on_hover_text("Move Up").clicked() {
                            swap_pair = Some((idx, idx - 1));
                        }
                        if idx + 1 < len && ui.small_button("v").on_hover_text("Move Down").clicked() {
                            swap_pair = Some((idx, idx + 1));
                        }
                        if ui.small_button("x").on_hover_text("Remove").clicked() {
                            remove_idx = Some(idx);
                        }
                    });
                }
                if let Some((a, b)) = swap_pair {
                    node.tree_items.swap(a, b);
                    changed = true;
                }
                if let Some(rm) = remove_idx {
                    node.tree_items.remove(rm);
                    changed = true;
                }
            }
            if ui
                .add(egui::Slider::new(&mut node.text_style.font_size, 9.0..=32.0).text("Font px"))
                .changed()
            {
                changed = true;
            }
            ui.horizontal(|ui| {
                ui.label("Text Color");
                let mut c = to_color32(node.text_style.color);
                if ui.color_edit_button_srgba(&mut c).changed() {
                    node.text_style.color = from_color32(c);
                    changed = true;
                }
            });
        }
        if changed {
            app.broadcast_scene_snapshot();
        }
    } else if let Some(&eid) = app.bridge.selection.selected_edges.first() {
        let mut changed = false;
        if let Some(edge) = app.bridge.scene.get_edge_mut(eid) {
            ui.label(RichText::new("Edge Label").strong());
            if ui.text_edit_singleline(&mut edge.label).changed() {
                changed = true;
            }
            egui::ComboBox::from_label("Placement")
                .selected_text(match edge.label_style.placement {
                    rustflow_core::EdgeLabelPlacement::Center => "Center on Line",
                    rustflow_core::EdgeLabelPlacement::Above => "Top (Above Line)",
                    rustflow_core::EdgeLabelPlacement::Below => "Bottom (Below Line)",
                    rustflow_core::EdgeLabelPlacement::AlongPath => "Along Path",
                })
                .show_ui(ui, |ui| {
                    for (lbl, pl) in [
                        ("Center on Line", rustflow_core::EdgeLabelPlacement::Center),
                        ("Top (Above Line)", rustflow_core::EdgeLabelPlacement::Above),
                        ("Bottom (Below Line)", rustflow_core::EdgeLabelPlacement::Below),
                        ("Along Path", rustflow_core::EdgeLabelPlacement::AlongPath),
                    ] {
                        if ui.selectable_value(&mut edge.label_style.placement, pl, lbl).changed() {
                            changed = true;
                        }
                    }
                });
            if ui
                .add(egui::Slider::new(&mut edge.label_style.font_size, 8.0..=28.0).text("Font px"))
                .changed()
            {
                changed = true;
            }
            if ui
                .add(egui::Slider::new(&mut edge.label_style.position, 0.1..=0.9).text("Position"))
                .changed()
            {
                changed = true;
            }
            if ui
                .add(egui::Slider::new(&mut edge.label_style.offset_y, -36.0..=36.0).text("Offset Y"))
                .changed()
            {
                changed = true;
            }
            if ui
                .checkbox(&mut edge.label_style.show_bg, "Show Pill Background")
                .changed()
            {
                changed = true;
            }
            ui.horizontal(|ui| {
                ui.label("Text");
                let mut tc = to_color32(edge.label_style.text_color);
                if ui.color_edit_button_srgba(&mut tc).changed() {
                    edge.label_style.text_color = from_color32(tc);
                    changed = true;
                }
                if edge.label_style.show_bg {
                    ui.label("Pill");
                    let mut bc = to_color32(edge.label_style.bg_color);
                    if ui.color_edit_button_srgba(&mut bc).changed() {
                        edge.label_style.bg_color = from_color32(bc);
                        changed = true;
                    }
                }
            });
        }
        if changed {
            app.broadcast_scene_snapshot();
        }
    } else {
        ui.label("Select a node or edge to edit text.");
    }
}

fn render_arrange_tab(app: &mut BohatDrawApp, ui: &mut egui::Ui) {
    let ids = app.bridge.selection.selected_nodes.clone();
    if ids.is_empty() {
        ui.label("Select one or more nodes to arrange.");
        return;
    }

    if let Some(&first_id) = ids.first() {
        if let Some(n) = app.bridge.scene.get_node(first_id).cloned() {
            let mut x = n.bounds.x;
            let mut y = n.bounds.y;
            let mut w = n.bounds.width;
            let mut h = n.bounds.height;
            let mut changed = false;
            ui.label(RichText::new("Geometry").strong());
            ui.horizontal(|ui| {
                ui.label("X");
                changed |= ui.add(egui::DragValue::new(&mut x).speed(1.0)).changed();
                ui.label("Y");
                changed |= ui.add(egui::DragValue::new(&mut y).speed(1.0)).changed();
            });
            ui.horizontal(|ui| {
                ui.label("W");
                changed |= ui.add(egui::DragValue::new(&mut w).range(24.0..=2000.0)).changed();
                ui.label("H");
                changed |= ui.add(egui::DragValue::new(&mut h).range(24.0..=2000.0)).changed();
            });
            if changed {
                app.bridge
                    .scene
                    .resize_node(first_id, Rect::new(x, y, w, h), false);
                app.broadcast_scene_snapshot();
            }
        }
    }

    ui.separator();
    ui.label(RichText::new("Align & Distribute").strong());
    ui.horizontal_wrapped(|ui| {
        for (icon, tip, mode) in [
            (ToolIcon::AlignLeft, "Align Left", AlignMode::Left),
            (ToolIcon::AlignCenterH, "Align Horizontal Center", AlignMode::CenterH),
            (ToolIcon::AlignRight, "Align Right", AlignMode::Right),
            (ToolIcon::AlignTop, "Align Top", AlignMode::Top),
            (ToolIcon::AlignCenterV, "Align Vertical Middle", AlignMode::CenterV),
            (ToolIcon::AlignBottom, "Align Bottom", AlignMode::Bottom),
        ] {
            if icon_button(ui, icon, None, tip, true, false).clicked() {
                app.push_undo();
                align_nodes(&mut app.bridge.scene, &ids, mode);
                app.broadcast_scene_snapshot();
            }
        }
        if icon_button(ui, ToolIcon::DistH, None, "Distribute Horizontally", true, false).clicked() {
            app.push_undo();
            distribute_nodes(&mut app.bridge.scene, &ids, DistributeAxis::Horizontal);
            app.broadcast_scene_snapshot();
        }
        if icon_button(ui, ToolIcon::DistV, None, "Distribute Vertically", true, false).clicked() {
            app.push_undo();
            distribute_nodes(&mut app.bridge.scene, &ids, DistributeAxis::Vertical);
            app.broadcast_scene_snapshot();
        }
    });

    ui.separator();
    ui.label(RichText::new("Overlap & Pile").strong());
    ui.horizontal_wrapped(|ui| {
        if ui.button("Stack").clicked() {
            app.push_undo();
            layout_overlap_stack(&mut app.bridge.scene, &ids);
            app.broadcast_scene_snapshot();
        }
        if ui.button("Cascade").clicked() {
            app.push_undo();
            layout_cascade_pile(&mut app.bridge.scene, &ids, 22.0, 22.0);
            app.broadcast_scene_snapshot();
        }
        if ui.button("Heap").clicked() {
            app.push_undo();
            layout_heap_pile(&mut app.bridge.scene, &ids);
            app.broadcast_scene_snapshot();
        }
        if ui.button("Separate").clicked() {
            app.push_undo();
            layout_separate_overlaps(&mut app.bridge.scene, &ids, 18.0);
            app.broadcast_scene_snapshot();
        }
    });

    ui.separator();
    ui.label(RichText::new("Sections & Hierarchy").strong());
    ui.horizontal_wrapped(|ui| {
        if ui.button("Group Section").clicked() {
            app.push_undo();
            app.bridge.scene.group_into_section(&ids, "Section");
            app.broadcast_scene_snapshot();
        }
        if ui.button("Fit Section").clicked() {
            app.push_undo();
            for &id in &ids {
                app.bridge.scene.fit_section_to_children(id);
            }
            app.broadcast_scene_snapshot();
        }
        if ui.button("Detach").clicked() {
            app.push_undo();
            app.bridge.scene.detach_from_parent(&ids);
            app.broadcast_scene_snapshot();
        }
    });
}

fn render_layers_tab(app: &mut BohatDrawApp, ui: &mut egui::Ui) {
    let ids = app.bridge.selection.selected_nodes.clone();
    ui.horizontal_wrapped(|ui| {
        let has_sel = !ids.is_empty();
        if icon_button(ui, ToolIcon::BringFront, None, "Bring to Front", has_sel, false).clicked() {
            app.push_undo();
            app.bridge.scene.bring_to_front(&ids);
            app.broadcast_scene_snapshot();
        }
        if icon_button(ui, ToolIcon::BringForward, None, "Bring Forward", has_sel, false).clicked() {
            app.push_undo();
            app.bridge.scene.bring_forward(&ids);
            app.broadcast_scene_snapshot();
        }
        if icon_button(ui, ToolIcon::SendBackward, None, "Send Backward", has_sel, false).clicked() {
            app.push_undo();
            app.bridge.scene.send_backward(&ids);
            app.broadcast_scene_snapshot();
        }
        if icon_button(ui, ToolIcon::SendBack, None, "Send to Back", has_sel, false).clicked() {
            app.push_undo();
            app.bridge.scene.send_to_back(&ids);
            app.broadcast_scene_snapshot();
        }
        if ui
            .add_enabled(has_sel, egui::Button::new("Group"))
            .on_hover_text("Group selected into Section")
            .clicked()
        {
            app.push_undo();
            app.bridge.scene.group_into_section(&ids, "Section");
            app.broadcast_scene_snapshot();
        }
        if ui
            .add_enabled(has_sel, egui::Button::new("Detach"))
            .on_hover_text("Unparent selected from Section/Parent")
            .clicked()
        {
            app.push_undo();
            app.bridge.scene.detach_from_parent(&ids);
            app.broadcast_scene_snapshot();
        }
    });
    ui.label(
        RichText::new("Drag row to reorder; drop on Section (or Shift+drop) to nest")
            .size(9.5)
            .weak(),
    );
    ui.separator();

    let rows = app.bridge.scene.hierarchical_layers_for_panel();
    let mut toggle_vis = None;
    let mut toggle_lock = None;
    let mut select_action: Option<(u64, bool)> = None;
    let mut drop_action: Option<(u64, u64, bool, bool)> = None;
    let pointer_pos = ui.input(|i| i.pointer.hover_pos().or(i.pointer.interact_pos()));
    let primary_down = ui.input(|i| i.pointer.primary_down());
    let shift_held = ui.input(|i| i.modifiers.shift);

    for row in rows {
        let Some(node) = app.bridge.scene.get_node(row.node_id) else {
            continue;
        };
        let node_id = node.id;
        let node_vis = node.visible;
        let node_lock = node.locked;
        let node_is_section = node.is_section;
        let node_title = node.title.clone();
        let is_sel = app.bridge.selection.selected_nodes.contains(&node_id);

        ui.horizontal(|ui| {
            ui.add_space(row.depth as f32 * 14.0);
            if draw_eye_toggle(ui, node_vis)
                .on_hover_text(if node_vis { "Hide layer" } else { "Show layer" })
                .clicked()
            {
                toggle_vis = Some(node_id);
            }
            if draw_lock_toggle(ui, node_lock)
                .on_hover_text(if node_lock { "Unlock layer" } else { "Lock layer" })
                .clicked()
            {
                toggle_lock = Some(node_id);
            }
            let prefix = if node_is_section { "[S] " } else if row.depth > 0 { "|- " } else { "" };
            let label = format!("{prefix}{node_title}");
            let row_w = ui.available_width().max(80.0);
            let (rect, resp) = ui.allocate_exact_size(Vec2::new(row_w, 20.0), Sense::click_and_drag());
            let bg = if app.dragging_layer_id == Some(node_id) {
                Color32::from_rgb(198, 212, 248)
            } else if is_sel {
                Color32::from_rgb(0, 0, 128)
            } else if resp.hovered() {
                Color32::from_rgb(232, 236, 248)
            } else {
                Color32::TRANSPARENT
            };
            let fg = if is_sel && app.dragging_layer_id != Some(node_id) {
                Color32::WHITE
            } else {
                Color32::from_rgb(18, 20, 28)
            };
            ui.painter().rect_filled(rect, 2.0, bg);
            ui.painter().text(
                egui::Pos2::new(rect.min.x + 5.0, rect.center().y),
                Align2::LEFT_CENTER,
                label,
                FontId::proportional(11.0),
                fg,
            );

            if resp.clicked() {
                select_action = Some((node_id, shift_held || ui.input(|i| i.modifiers.command || i.modifiers.ctrl)));
            }
            if resp.drag_started() {
                app.dragging_layer_id = Some(node_id);
            }

            if let (Some(drag_id), Some(p)) = (app.dragging_layer_id, pointer_pos) {
                if drag_id != node_id && rect.contains(p) {
                    let rel_y = (p.y - rect.min.y) / rect.height().max(1.0);
                    let nest = (node_is_section && rel_y > 0.25 && rel_y < 0.75) || shift_held;
                    if nest {
                        ui.painter().rect_stroke(
                            rect,
                            2.0,
                            Stroke::new(2.0_f32, Color32::from_rgb(16, 185, 129)),
                            egui::StrokeKind::Inside,
                        );
                    } else {
                        let y_line = if rel_y < 0.5 { rect.min.y } else { rect.max.y };
                        ui.painter().line_segment(
                            [egui::Pos2::new(rect.min.x, y_line), egui::Pos2::new(rect.max.x, y_line)],
                            Stroke::new(2.0_f32, Color32::from_rgb(0, 0, 180)),
                        );
                    }
                    if !primary_down {
                        drop_action = Some((drag_id, node_id, nest, rel_y < 0.5));
                    }
                }
            }
        });
    }

    if let Some((drag_id, target_id, nest, place_above)) = drop_action {
        let is_descendant = app.bridge.scene.descendant_ids(drag_id).contains(&target_id);
        if !is_descendant && drag_id != target_id {
            app.push_undo();
            if nest {
                if let Some(n) = app.bridge.scene.get_node_mut(drag_id) {
                    n.parent_id = Some(target_id);
                }
            } else if let Some(target_node) = app.bridge.scene.get_node(target_id).cloned() {
                if let Some(n) = app.bridge.scene.get_node_mut(drag_id) {
                    n.parent_id = target_node.parent_id;
                    n.z_index = if place_above {
                        target_node.z_index + 1
                    } else {
                        target_node.z_index - 1
                    };
                }
            }
            app.bridge.scene.normalize_layer_order();
            app.broadcast_scene_snapshot();
        }
        app.dragging_layer_id = None;
    } else if !primary_down && app.dragging_layer_id.is_some() {
        app.dragging_layer_id = None;
    }

    if let Some(id) = toggle_vis {
        if let Some(n) = app.bridge.scene.get_node_mut(id) {
            n.visible = !n.visible;
        }
        app.broadcast_scene_snapshot();
    }
    if let Some(id) = toggle_lock {
        if let Some(n) = app.bridge.scene.get_node_mut(id) {
            n.locked = !n.locked;
        }
        app.broadcast_scene_snapshot();
    }
    if let Some((id, additive)) = select_action {
        app.bridge.selection.select_node(id, additive);
    }
}

