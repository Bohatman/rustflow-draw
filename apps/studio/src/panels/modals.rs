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


pub fn render_collab_and_duel_modals(app: &mut BohatDrawApp, ctx: &egui::Context) {
    if app.show_collab_modal {
        let mut open = app.show_collab_modal;
        egui::Window::new("Realtime Co-Editing Room")
            .open(&mut open)
            .resizable(false)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Name");
                    ui.text_edit_singleline(&mut app.collab.display_name);
                });
                ui.horizontal(|ui| {
                    ui.label("Room");
                    ui.text_edit_singleline(&mut app.collab_room_input);
                    if ui.small_button("New ID").on_hover_text("Generate new unique room ID").clicked() {
                        let fresh = crate::platform::generate_unique_room_id();
                        app.collab_room_input = fresh.clone();
                        app.collab_ws_url = crate::platform::default_ws_url(&fresh);
                    }
                });
                ui.horizontal(|ui| {
                    ui.label("Server WS");
                    ui.text_edit_singleline(&mut app.collab_ws_url);
                });
                ui.horizontal(|ui| {
                    if ui.button("Connect").clicked() {
                        let room = app.collab_room_input.trim().to_string();
                        let url = if app.collab_ws_url.contains("?room=") {
                            app.collab_ws_url.clone()
                        } else {
                            crate::platform::default_ws_url(&room)
                        };
                        app.collab.connect(&url, &room, ctx.clone());
                    }
                    if app.collab.connected && ui.button("Disconnect").clicked() {
                        app.collab.disconnect();
                    }
                    if ui.button("Copy Link").clicked() {
                        let room = if app.collab_room_input.trim().is_empty() {
                            "lobby".to_string()
                        } else {
                            app.collab_room_input.trim().to_string()
                        };
                        let share_url = crate::platform::share_room_url(&room);
                        ctx.copy_text(share_url.clone());
                        app.set_status(format!("Copied link: {share_url}"));
                    }
                });
                ui.separator();
                ui.label(format!(
                    "Status: {} | Peers: {}",
                    if app.collab.connected { "Connected" } else { "Offline" },
                    app.collab.peers.len()
                ));
                for peer in &app.collab.peers {
                    ui.label(format!("- {} ({})", peer.name, peer.id));
                }
            });
        app.show_collab_modal = open;
    }

    if let Some(duel) = app.collab.active_duel.clone() {
        let mash_key =
            ctx.input(|i| i.key_pressed(egui::Key::Space) || i.key_pressed(egui::Key::Enter));
        if mash_key {
            app.collab.send(&ClientMsg::DuelMash {
                item_id: duel.item_id,
            });
        }
        egui::Window::new("[DUEL] OWNERSHIP DUEL!")
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .collapsible(false)
            .resizable(false)
            .show(ctx, |ui| {
                ui.heading(format!("Contesting: {}", duel.item_label));
                ui.label(format!("Time left: {:.1}s", duel.remaining_ms as f32 / 1000.0));
                ui.separator();
                ui.horizontal(|ui| {
                    ui.strong(format!("{}: {}", duel.challenger_name, duel.challenger_taps));
                    ui.label("  VS  ");
                    ui.strong(format!("{}: {}", duel.defender_name, duel.defender_taps));
                });
                let total = (duel.challenger_taps + duel.defender_taps).max(1) as f32;
                let ratio = duel.challenger_taps as f32 / total;
                ui.add(egui::ProgressBar::new(ratio).text("Tug-of-War"));
                ui.add_space(6.0);
                if ui
                    .add_sized(
                        [240.0, 44.0],
                        egui::Button::new(
                            RichText::new("MASH! (Space / Enter / Click)")
                                .strong()
                                .color(Color32::WHITE),
                        )
                        .fill(Color32::from_rgb(220, 38, 38)),
                    )
                    .clicked()
                {
                    app.collab.send(&ClientMsg::DuelMash {
                        item_id: duel.item_id,
                    });
                }
            });
        ctx.request_repaint();
    }
}

pub fn render_extra_modals(app: &mut BohatDrawApp, ctx: &egui::Context) {
    if app.show_cmd_palette {
        let mut open = app.show_cmd_palette;
        let cmd_w = (ctx.screen_rect().width() - 44.0).clamp(240.0, 340.0);
        egui::Window::new("Command Palette (Ctrl+K)")
            .anchor(egui::Align2::CENTER_TOP, [0.0, 72.0])
            .collapsible(false)
            .open(&mut open)
            .show(ctx, |ui| {
                ui.add(
                    egui::TextEdit::singleline(&mut app.cmd_query)
                        .hint_text("Type a command or shape...")
                        .desired_width(cmd_w),
                );
                let q = app.cmd_query.trim().to_ascii_lowercase();
                ui.separator();
                ScrollArea::vertical().max_height(240.0).show(ui, |ui| {
                    let cmds = [
                        ("Fit View", "fit"),
                        ("Auto-Layout Hierarchical", "hier"),
                        ("Auto-Layout Grid", "grid"),
                        ("Copy as 2x PNG", "png"),
                        ("Copy as Animated GIF", "gif"),
                        ("New Blank Diagram", "new"),
                        ("Open Symbol Studio", "sym"),
                        ("Share Co-Op Room", "share"),
                    ];
                    for (label, id) in cmds {
                        if q.is_empty() || label.to_ascii_lowercase().contains(&q) {
                            if ui.button(label).clicked() {
                                match id {
                                    "fit" => app.fit_view(),
                                    "hier" => {
                                        app.push_undo();
                                        app.bridge.trigger_auto_layout(LayoutAlgorithm::Hierarchical, 0.35);
                                    }
                                    "grid" => {
                                        app.push_undo();
                                        app.bridge.trigger_auto_layout(LayoutAlgorithm::Grid, 0.35);
                                    }
                                    "png" => app.copy_diagram_png(),
                                    "gif" => app.copy_diagram_gif(),
                                    "new" => app.new_blank_diagram(),
                                    "sym" => app.show_symbol_studio = true,
                                    "share" => app.show_collab_modal = true,
                                    _ => {}
                                }
                                app.show_cmd_palette = false;
                            }
                        }
                    }
                    let items: Vec<_> = app.bridge.palette.search(&q).into_iter().take(12).cloned().collect();
                    for item in items {
                        if ui.button(format!("+ Add Shape: {}", item.label)).clicked() {
                            app.spawn_stencil_at_center(
                                &item.label,
                                item.default_width,
                                item.default_height,
                                item.shape.clone(),
                                item.icon,
                            );
                            app.show_cmd_palette = false;
                        }
                    }
                });
            });
        app.show_cmd_palette = open;
    }

    if app.show_shortcuts {
        let mut open = app.show_shortcuts;
        egui::Window::new("Keyboard Shortcuts")
            .open(&mut open)
            .resizable(false)
            .show(ctx, |ui| {
                for (k, d) in [
                    ("Double-click Node", "Inline rename title"),
                    ("Drag from Port Dot", "Connect edge (hover node to reveal ports)"),
                    ("Right-click Canvas/Item", "Context menu (Layers, Border, Copy PNG/GIF)"),
                    ("Ctrl/Cmd + S", "Save to same Google Drive file (or export JSON in Guest Mode)"),
                    ("Ctrl/Cmd + Z / Y", "Undo / Redo"),
                    ("Ctrl/Cmd + C / V / D", "Copy / Paste / Duplicate"),
                    ("Ctrl/Cmd + A", "Select All"),
                    ("Ctrl/Cmd + K", "Command Palette"),
                    ("Ctrl/Cmd + 1 / 0", "Fit Diagram to Screen"),
                    ("Space + Drag / Wheel", "Pan & Zoom Canvas"),
                    ("Arrows / Shift+Arrows", "Nudge Selection (2px / 10px)"),
                    ("Del / Backspace", "Delete Selection"),
                ] {
                    ui.horizontal(|ui| {
                        ui.strong(RichText::new(k).monospace());
                        ui.label("—");
                        ui.label(d);
                    });
                }
            });
        app.show_shortcuts = open;
    }

    if app.show_welcome {
        let mut open = app.show_welcome;
        egui::Window::new("Welcome to Bohat Draw.io (100% Rust)")
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .open(&mut open)
            .resizable(false)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Document Name");
                    ui.text_edit_singleline(&mut app.welcome_doc_name);
                });
                ui.add_space(4.0);
                ui.label("Choose a starting template:");
                ui.add_space(6.0);
                ui.horizontal(|ui| {
                    if ui.button("Blank Canvas").clicked() {
                        app.new_blank_diagram();
                        app.show_welcome = false;
                    }
                    if ui.button("Sample Architecture").clicked() {
                        app.load_demo_scene();
                        app.show_welcome = false;
                    }
                    if ui.button("Flowchart").clicked() {
                        app.load_flowchart_template();
                        app.show_welcome = false;
                    }
                });
            });
        app.show_welcome = open;
    }

    if app.show_symbol_studio {
        let mut open = app.show_symbol_studio;
        egui::Window::new("Symbol Studio - Multi-Layer Composer")
            .open(&mut open)
            .resizable(false)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Name");
                    ui.text_edit_singleline(&mut app.sym_name);
                    ui.label("W");
                    ui.add(egui::DragValue::new(&mut app.sym_w).range(40.0..=400.0));
                    ui.label("H");
                    ui.add(egui::DragValue::new(&mut app.sym_h).range(40.0..=300.0));
                });
                ui.separator();

                let shapes = [
                    ("Rounded", ShapeKind::RoundedRect),
                    ("Rect", ShapeKind::Rectangle),
                    ("Diamond", ShapeKind::Diamond),
                    ("Ellipse", ShapeKind::Ellipse),
                    ("Hexagon", ShapeKind::Hexagon),
                    ("Shield", ShapeKind::Shield),
                ];

                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.label(RichText::new("Interactive Canvas (drag layers)").strong().size(11.0));
                        let (crect, cresp) = ui.allocate_exact_size(Vec2::new(240.0, 160.0), Sense::click_and_drag());
                        ui.painter().rect_filled(crect, 2.0, Color32::from_rgb(246, 247, 250));
                        ui.painter().rect_stroke(
                            crect,
                            2.0,
                            Stroke::new(1.0_f32, Color32::from_rgb(60, 60, 72)),
                            egui::StrokeKind::Inside,
                        );

                        if cresp.clicked() || cresp.drag_started() {
                            if let Some(pos) = cresp.interact_pointer_pos() {
                                let nx = (pos.x - crect.min.x) / crect.width();
                                let ny = (pos.y - crect.min.y) / crect.height();
                                let mut hit = None;
                                for (idx, l) in app.sym_layers.iter().enumerate().rev() {
                                    if nx >= l.x && nx <= l.x + l.w && ny >= l.y && ny <= l.y + l.h {
                                        hit = Some(idx);
                                        break;
                                    }
                                }
                                if let Some(h) = hit {
                                    app.sym_selected_layer = Some(h);
                                }
                            }
                        }

                        if cresp.dragged() {
                            if let Some(sel) = app.sym_selected_layer {
                                if let Some(layer) = app.sym_layers.get_mut(sel) {
                                    let d = cresp.drag_delta();
                                    layer.x = (layer.x + d.x / crect.width()).clamp(0.0, 1.0 - layer.w);
                                    layer.y = (layer.y + d.y / crect.height()).clamp(0.0, 1.0 - layer.h);
                                }
                            }
                        }

                        for (idx, layer) in app.sym_layers.iter().enumerate() {
                            let lr = egui::Rect::from_min_size(
                                egui::Pos2::new(
                                    crect.min.x + layer.x * crect.width(),
                                    crect.min.y + layer.y * crect.height(),
                                ),
                                Vec2::new(
                                    (layer.w * crect.width()).max(6.0),
                                    (layer.h * crect.height()).max(6.0),
                                ),
                            );
                            draw_simple_layer_shape(
                                ui.painter(),
                                lr,
                                &layer.shape,
                                to_color32(layer.fill),
                                Stroke::new(1.8_f32, to_color32(layer.stroke)),
                            );
                            if app.sym_selected_layer == Some(idx) {
                                ui.painter().rect_stroke(
                                    lr.expand(2.0),
                                    1.0,
                                    Stroke::new(1.2_f32, Color32::from_rgb(220, 38, 38)),
                                    egui::StrokeKind::Outside,
                                );
                            }
                        }

                        ui.add_space(4.0);
                        ui.horizontal(|ui| {
                            egui::ComboBox::from_id_salt("sym_add_shape")
                                .width(88.0)
                                .selected_text(shapes[app.sym_shape_idx % shapes.len()].0)
                                .show_ui(ui, |ui| {
                                    for (idx, (lbl, _)) in shapes.iter().enumerate() {
                                        ui.selectable_value(&mut app.sym_shape_idx, idx, *lbl);
                                    }
                                });
                            if ui.button("+ Add Layer").clicked() {
                                let offset = ((app.sym_layers.len() % 5) as f32) * 0.06;
                                app.sym_layers.push(SymbolLayer {
                                    shape: shapes[app.sym_shape_idx % shapes.len()].1.clone(),
                                    x: (0.2 + offset).min(0.6),
                                    y: (0.2 + offset).min(0.6),
                                    w: 0.42,
                                    h: 0.42,
                                    fill: from_color32(app.sym_fill),
                                    stroke: from_color32(app.sym_stroke),
                                });
                                app.sym_selected_layer = Some(app.sym_layers.len() - 1);
                            }
                        });
                    });

                    ui.separator();

                    ui.vertical(|ui| {
                        ui.label(RichText::new(format!("Layers ({})", app.sym_layers.len())).strong().size(11.0));
                        ui.horizontal(|ui| {
                            let sel = app.sym_selected_layer;
                            if ui.small_button("Up").clicked() {
                                if let Some(idx) = sel {
                                    if idx + 1 < app.sym_layers.len() {
                                        app.sym_layers.swap(idx, idx + 1);
                                        app.sym_selected_layer = Some(idx + 1);
                                    }
                                }
                            }
                            if ui.small_button("Down").clicked() {
                                if let Some(idx) = sel {
                                    if idx > 0 {
                                        app.sym_layers.swap(idx, idx - 1);
                                        app.sym_selected_layer = Some(idx - 1);
                                    }
                                }
                            }
                            if ui.small_button("Dup").clicked() {
                                if let Some(idx) = sel {
                                    if let Some(mut copy) = app.sym_layers.get(idx).cloned() {
                                        copy.x = (copy.x + 0.05).min(1.0 - copy.w);
                                        copy.y = (copy.y + 0.05).min(1.0 - copy.h);
                                        app.sym_layers.push(copy);
                                        app.sym_selected_layer = Some(app.sym_layers.len() - 1);
                                    }
                                }
                            }
                            if ui.small_button("Del").clicked() {
                                if let Some(idx) = sel {
                                    if app.sym_layers.len() > 1 {
                                        app.sym_layers.remove(idx);
                                        app.sym_selected_layer = Some(idx.min(app.sym_layers.len() - 1));
                                    }
                                }
                            }
                        });

                        ScrollArea::vertical().max_height(82.0).show(ui, |ui| {
                            for idx in (0..app.sym_layers.len()).rev() {
                                let lbl = format!("#{} {:?}", idx + 1, app.sym_layers[idx].shape);
                                if ui.selectable_label(app.sym_selected_layer == Some(idx), lbl).clicked() {
                                    app.sym_selected_layer = Some(idx);
                                }
                            }
                        });

                        if let Some(sel) = app.sym_selected_layer {
                            if let Some(layer) = app.sym_layers.get_mut(sel) {
                                ui.separator();
                                ui.add(egui::Slider::new(&mut layer.w, 0.1..=1.0).text("W"));
                                ui.add(egui::Slider::new(&mut layer.h, 0.1..=1.0).text("H"));
                                ui.horizontal(|ui| {
                                    ui.label("Fill");
                                    let mut fc = to_color32(layer.fill);
                                    if ui.color_edit_button_srgba(&mut fc).changed() {
                                        layer.fill = from_color32(fc);
                                    }
                                    ui.label("Stroke");
                                    let mut sc = to_color32(layer.stroke);
                                    if ui.color_edit_button_srgba(&mut sc).changed() {
                                        layer.stroke = from_color32(sc);
                                    }
                                });
                            }
                        }
                    });
                });

                ui.separator();
                if ui
                    .add(
                        egui::Button::new(RichText::new("Save Symbol & Add to Palette").color(Color32::WHITE))
                            .fill(Color32::from_rgb(0, 0, 128)),
                    )
                    .clicked()
                {
                    let base_shape = app
                        .sym_layers
                        .first()
                        .map(|l| l.shape.clone())
                        .unwrap_or(ShapeKind::RoundedRect);
                    app.custom_stencils.push(CustomStencil {
                        label: app.sym_name.clone(),
                        width: app.sym_w,
                        height: app.sym_h,
                        shape: base_shape,
                        layers: app.sym_layers.clone(),
                    });
                    app.set_status(format!("Added custom symbol '{}'", app.sym_name));
                    app.show_symbol_studio = false;
                }
            });
        app.show_symbol_studio = open;
    }

    if app.show_guide_tour {
        let steps: [(&str, &str, &str); 6] = [
            (
                "1. Welcome to Bohat Draw.io",
                "You start on a clean, blank canvas ready for architecture, cloud, or flowchart diagrams.\n\nUse the bottom tabs to add or switch between pages, or load a sample template anytime from File or Help.",
                "Tip: Press Ctrl+K anytime to open the Command Palette.",
            ),
            (
                "2. Symbols & Cloud Icons (Left Panel)",
                "Browse 165+ vector symbols including 76 AWS services (EC2, Lambda, MSK, Managed Flink, Bedrock...), 33 Azure services, GCP, Kubernetes, Docker, Standard Icons, and ISO Flowcharts.\n\nUse the filterable multi-select Category Box to pick one or multiple providers, and click or drag any symbol onto the canvas.",
                "Tip: Click '+ New' to build custom multi-layer symbols in Symbol Studio.",
            ),
            (
                "3. Smart Edges & Curved Labels",
                "Hover over any node to reveal connection ports, then drag from a port to another node to connect them.\n\nSelect an edge to switch routing (Bezier, Smooth, Ortho, Straight), arrowheads, flow animations (Marching Ants / Pulse Particles), or add curved Along-Path labels in the Text tab.",
                "Tip: Hold Alt while dragging a node to temporarily disable magnetic snapping.",
            ),
            (
                "4. Double-Click Editing & Right Inspector",
                "Double-click any node to edit both its Title and Body inline (double-click the top for Title, or lower area for Body).\n\nUse the Right Inspector tabs (Style, Text, Arrange, Layers) to style colors, border patterns, port density, align/distribute nodes, or drag-and-drop layers into Sections.",
                "Tip: Press Ctrl+Enter to save multiline body text, or Esc to cancel.",
            ),
            (
                "5. Interactive Minimap & Smooth Navigation",
                "Click or drag inside the bottom-right Minimap to pan across large diagrams, or scroll over it to zoom.\n\nOn the canvas: hold Space + drag (or middle-mouse drag) to pan, Ctrl + scroll to zoom around your cursor, and Ctrl+0 to Fit View.",
                "Tip: You can toggle Grid / Dots / Crosshair / Isometric backgrounds from the toolbar.",
            ),
            (
                "6. Export & Unique Realtime Co-Op Links",
                "Export crisp 2x PNGs, Animated GIFs (with Marching Ants & Pulse Particles), SVGs, or .drawio XML from the toolbar or File menu.\n\nClick 'Copy Link' in the toolbar to copy a unique shareable room link (?room=...) for live multi-cursor collaboration and Ownership Duels!",
                "Tip: Right-click 'Copy Link' to open Room Settings or mint a new Room ID.",
            ),
        ];
        let total = steps.len();
        let step = app.guide_tour_step.min(total - 1);
        let (title, body, tip) = steps[step];
        let mut open = app.show_guide_tour;
        let mut close_now = false;
        let tour_w = (ctx.screen_rect().width() - 28.0).clamp(276.0, 430.0);
        egui::Window::new("Bohat Draw.io — Guided Tour")
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .collapsible(false)
            .resizable(false)
            .fixed_size(Vec2::new(tour_w, 260.0))
            .open(&mut open)
            .show(ctx, |ui| {
                ui.add(
                    egui::ProgressBar::new((step + 1) as f32 / total as f32)
                        .text(format!("Step {} of {}", step + 1, total)),
                );
                ui.add_space(8.0);
                ui.label(
                    RichText::new(title)
                        .strong()
                        .size(15.0)
                        .color(Color32::from_rgb(0, 0, 128)),
                );
                ui.add_space(6.0);
                ui.label(RichText::new(body).size(12.0));
                ui.add_space(8.0);
                Frame::new()
                    .fill(Color32::from_rgb(236, 240, 252))
                    .stroke(Stroke::new(1.0_f32, Color32::from_rgb(160, 174, 212)))
                    .corner_radius(3)
                    .inner_margin(Margin::same(6))
                    .show(ui, |ui| {
                        ui.label(RichText::new(tip).small().color(Color32::from_rgb(24, 36, 86)));
                    });
                ui.add_space(10.0);

                ui.horizontal(|ui| {
                    if step == 0 && ui.button("Load Sample Architecture").clicked() {
                        app.load_demo_scene();
                    } else if step == 1 && ui.button("Show AWS Icons").clicked() {
                        app.show_palette = true;
                        app.palette_selected_cats = vec![StencilCategory::Aws];
                        app.palette_subgroup = None;
                    }
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if step + 1 < total {
                            if ui
                                .add(
                                    egui::Button::new(RichText::new("Next ->").color(Color32::WHITE))
                                        .fill(Color32::from_rgb(0, 0, 128)),
                                )
                                .clicked()
                            {
                                app.guide_tour_step = (step + 1).min(total - 1);
                            }
                        } else if ui
                            .add(
                                egui::Button::new(RichText::new("Finish").color(Color32::WHITE))
                                    .fill(Color32::from_rgb(0, 0, 128)),
                            )
                            .clicked()
                        {
                            close_now = true;
                        }
                        if step > 0 && ui.button("<- Back").clicked() {
                            app.guide_tour_step = step.saturating_sub(1);
                        }
                    });
                });
            });
        app.show_guide_tour = open && !close_now;
    }

    // Mermaid Flowchart Import / Export Modal
    if app.show_mermaid_modal {
        let mut open = app.show_mermaid_modal;
        let mw = (ctx.screen_rect().width() - 32.0).clamp(320.0, 560.0);
        egui::Window::new("Mermaid Flowchart — Import / Export")
            .anchor(Align2::CENTER_CENTER, [0.0, 0.0])
            .collapsible(false)
            .resizable(true)
            .default_size(Vec2::new(mw, 380.0))
            .open(&mut open)
            .show(ctx, |ui| {
                ui.label(
                    RichText::new(
                        "Paste Mermaid flowchart syntax (flowchart LR/TD, subgraph, -->, -.->, ==>, [], (), {}, [()], (())) or copy your current diagram as Mermaid:",
                    )
                    .size(11.0)
                    .weak(),
                );
                ui.add_space(6.0);
                ScrollArea::vertical().max_height(240.0).show(ui, |ui| {
                    ui.add(
                        egui::TextEdit::multiline(&mut app.mermaid_text)
                            .font(egui::TextStyle::Monospace)
                            .desired_rows(12)
                            .desired_width(f32::INFINITY),
                    );
                });
                if let Some(err) = &app.mermaid_error {
                    ui.add_space(4.0);
                    ui.label(RichText::new(err).small().color(Color32::from_rgb(220, 38, 38)));
                }
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    if ui
                        .add(
                            egui::Button::new(RichText::new("Import to Canvas").color(Color32::WHITE))
                                .fill(Color32::from_rgb(37, 99, 235)),
                        )
                        .clicked()
                    {
                        app.apply_mermaid_import();
                    }
                    if ui.button("Refresh from Canvas").clicked() {
                        app.mermaid_text =
                            rustflow_io_formats::export_mermaid_flowchart(&app.bridge.scene);
                        app.mermaid_error = None;
                    }
                    if ui.button("Copy Mermaid").clicked() {
                        ctx.copy_text(app.mermaid_text.clone());
                        app.set_status("Copied Mermaid flowchart to clipboard");
                    }
                });
            });
        app.show_mermaid_modal = open;
    }
}

