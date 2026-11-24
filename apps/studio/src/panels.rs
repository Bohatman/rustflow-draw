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

pub fn render_menu_and_toolbar(app: &mut BohatDrawApp, ctx: &egui::Context, narrow: bool) {
    let screen_w = ctx.screen_rect().width();
    let is_phone = screen_w <= 640.0;
    egui::TopBottomPanel::top("platinum_menubar")
        .frame(
            Frame::new()
                .fill(match app.theme_mode {
                    ThemeMode::Dark => Color32::from_rgb(30, 33, 42),
                    _ => Color32::from_rgb(232, 232, 236),
                })
                .stroke(Stroke::new(1.0_f32, Color32::from_rgb(120, 120, 132)))
                .inner_margin(Margin::symmetric(if is_phone { 4 } else { 8 }, 3)),
        )
        .show(ctx, |ui| {
            ScrollArea::horizontal()
                .id_salt("menubar_hscroll")
                .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysHidden)
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        draw_prism_badge(ui);
                        ui.strong(
                            RichText::new(if is_phone { "Bohat" } else { "Bohat Draw" })
                                .color(Color32::from_rgb(0, 0, 140)),
                        );
                        ui.separator();

                        ui.menu_button("File", |ui| {
                            if ui.button("New Empty Space").clicked() {
                                app.new_blank_diagram();
                                ui.close_menu();
                            }
                            if ui.button("Templates / Welcome...").clicked() {
                                app.show_welcome = true;
                                ui.close_menu();
                            }
                            if ui.button("Open (.json / .drawio)...").clicked() {
                                app.open_file();
                                ui.close_menu();
                            }
                            ui.separator();
                            if ui.button("Save Native JSON").clicked() {
                                app.export_json_file();
                                ui.close_menu();
                            }
                            if ui.button("Export .drawio XML").clicked() {
                                app.export_drawio_file();
                                ui.close_menu();
                            }
                            if ui.button("Export Animated SVG").clicked() {
                                app.export_svg_file();
                                ui.close_menu();
                            }
                            if ui.button("Export 2x PNG").clicked() {
                                app.export_png_file();
                                ui.close_menu();
                            }
                            if ui.button("Export Animated GIF").clicked() {
                                app.export_gif_file();
                                ui.close_menu();
                            }
                        });

                        ui.menu_button("Edit", |ui| {
                            if ui.button("Undo (Ctrl+Z)").clicked() {
                                app.undo();
                                ui.close_menu();
                            }
                            if ui.button("Redo (Ctrl+Y)").clicked() {
                                app.redo();
                                ui.close_menu();
                            }
                            ui.separator();
                            if ui.button("Copy (Ctrl+C)").clicked() {
                                app.copy_selection();
                                ui.close_menu();
                            }
                            if ui.button("Paste (Ctrl+V)").clicked() {
                                app.paste_clipboard();
                                ui.close_menu();
                            }
                            if ui.button("Duplicate (Ctrl+D)").clicked() {
                                app.duplicate_selection();
                                ui.close_menu();
                            }
                            if ui.button("Select All (Ctrl+A)").clicked() {
                                app.select_all();
                                ui.close_menu();
                            }
                            if ui.button("Delete (Del)").clicked() {
                                app.delete_selection();
                                ui.close_menu();
                            }
                        });

                        ui.menu_button("View", |ui| {
                            if ui.checkbox(&mut app.show_palette, "Stencils & Symbols").changed()
                                && narrow
                                && app.show_palette
                            {
                                app.show_inspector = false;
                            }
                            if ui.checkbox(&mut app.show_inspector, "Inspector").changed()
                                && narrow
                                && app.show_inspector
                            {
                                app.show_palette = false;
                            }
                            ui.checkbox(&mut app.hand_pan_mode, "Hand Pan Mode (Touch / 1-Finger Pan)");
                            ui.checkbox(&mut app.show_grid, "Show Background Pattern");
                            ui.menu_button("Background Style", |ui| {
                                for (lbl, st) in [
                                    ("Dot Grid", CanvasBgStyle::Dots),
                                    ("Line Grid", CanvasBgStyle::Grid),
                                    ("Crosshair", CanvasBgStyle::Crosshair),
                                    ("Isometric", CanvasBgStyle::Isometric),
                                    ("Solid / None", CanvasBgStyle::None),
                                ] {
                                    if ui.selectable_value(&mut app.bg_style, st, lbl).clicked() {
                                        app.show_grid = !matches!(st, CanvasBgStyle::None);
                                        ui.close_menu();
                                    }
                                }
                            });
                            ui.checkbox(&mut app.snap_to_grid, "Snap to Grid & Centers (Alt bypass)");
                            ui.checkbox(&mut app.show_minimap, "Minimap");
                            ui.separator();
                            if ui.button("Fit Diagram (Ctrl+1)").clicked() {
                                app.fit_view();
                                ui.close_menu();
                            }
                            if ui.button("Zoom In").clicked() {
                                app.zoom_step(1.15);
                                ui.close_menu();
                            }
                            if ui.button("Zoom Out").clicked() {
                                app.zoom_step(0.85);
                                ui.close_menu();
                            }
                        });

                        ui.menu_button("Arrange", |ui| {
                            if ui.button("Auto-Layout: Hierarchical").clicked() {
                                app.push_undo();
                                app.bridge.trigger_auto_layout(LayoutAlgorithm::Hierarchical, 0.35);
                                app.broadcast_scene_snapshot();
                                ui.close_menu();
                            }
                            if ui.button("Auto-Layout: Grid").clicked() {
                                app.push_undo();
                                app.bridge.trigger_auto_layout(LayoutAlgorithm::Grid, 0.35);
                                app.broadcast_scene_snapshot();
                                ui.close_menu();
                            }
                            if ui.button("Auto-Layout: Force Directed").clicked() {
                                app.push_undo();
                                app.bridge.trigger_auto_layout(LayoutAlgorithm::ForceDirected, 0.35);
                                app.broadcast_scene_snapshot();
                                ui.close_menu();
                            }
                        });

                        ui.menu_button("Extras", |ui| {
                            if ui.button("Symbol Studio...").clicked() {
                                app.show_symbol_studio = true;
                                ui.close_menu();
                            }
                            if ui.button("Command Palette (Ctrl+K)").clicked() {
                                app.show_cmd_palette = true;
                                ui.close_menu();
                            }
                            if ui.button("Realtime Co-Op Room...").clicked() {
                                app.show_collab_modal = true;
                                ui.close_menu();
                            }
                        });

                        ui.menu_button("Help", |ui| {
                            if ui.button("Guided Tour...").clicked() {
                                app.guide_tour_step = 0;
                                app.show_guide_tour = true;
                                ui.close_menu();
                            }
                            if ui.button("Keyboard Shortcuts (?)").clicked() {
                                app.show_shortcuts = true;
                                ui.close_menu();
                            }
                            if ui.button("Command Palette (Ctrl+K)").clicked() {
                                app.show_cmd_palette = true;
                                ui.close_menu();
                            }
                            ui.separator();
                            if ui.button("Welcome & Templates").clicked() {
                                app.show_welcome = true;
                                ui.close_menu();
                            }
                            if ui.button("Load Sample Architecture").clicked() {
                                app.load_demo_scene();
                                ui.close_menu();
                            }
                            if ui.button("Load Flowchart Template").clicked() {
                                app.load_flowchart_template();
                                ui.close_menu();
                            }
                        });

                        ui.separator();
                        if let Some(user) = app.inventory.user.clone() {
                            if ui
                                .add(
                                    egui::Button::new(
                                        RichText::new("Drive").size(11.0).color(Color32::WHITE),
                                    )
                                    .fill(Color32::from_rgb(0, 0, 128)),
                                )
                                .on_hover_text("Open Google Drive Folder Inventory")
                                .clicked()
                            {
                                app.inventory.show_modal = true;
                                let fid = app.inventory.current_folder_id();
                                app.inventory.fetch_folder(fid, ctx.clone());
                            }
                            if ui
                                .small_button("Save")
                                .on_hover_text("Save diagram to Google Drive")
                                .clicked()
                            {
                                if let Some(fid) = app.inventory.active_file_id.clone() {
                                    let name = app.inventory.active_file_name.clone();
                                    let folder = app.inventory.active_folder_id.clone();
                                    app.save_current_to_drive(Some(fid), folder, name, ctx);
                                } else {
                                    app.inventory.show_modal = true;
                                    let folder = app.inventory.current_folder_id();
                                    app.inventory.fetch_folder(folder, ctx.clone());
                                }
                            }
                            let short_name: String = user.name.chars().take(14).collect();
                            ui.menu_button(short_name, |ui| {
                                ui.label(RichText::new(&user.email).small().weak());
                                ui.separator();
                                if ui.button("Open Drive Inventory...").clicked() {
                                    app.inventory.show_modal = true;
                                    let fid = app.inventory.current_folder_id();
                                    app.inventory.fetch_folder(fid, ctx.clone());
                                    ui.close_menu();
                                }
                                if ui.button("Save to Google Drive").clicked() {
                                    if let Some(fid) = app.inventory.active_file_id.clone() {
                                        let name = app.inventory.active_file_name.clone();
                                        let folder = app.inventory.active_folder_id.clone();
                                        app.save_current_to_drive(Some(fid), folder, name, ctx);
                                    } else {
                                        app.inventory.show_modal = true;
                                        let folder = app.inventory.current_folder_id();
                                        app.inventory.fetch_folder(folder, ctx.clone());
                                    }
                                    ui.close_menu();
                                }
                                ui.separator();
                                if ui.button("Sign Out").clicked() {
                                    app.inventory.logout(ctx.clone());
                                    ui.close_menu();
                                }
                            });
                        } else {
                            let btn_lbl = if is_phone {
                                "Sign In"
                            } else {
                                "Sign in with Google"
                            };
                            if ui
                                .add(
                                    egui::Button::new(
                                        RichText::new(btn_lbl).size(11.0).color(Color32::WHITE),
                                    )
                                    .fill(Color32::from_rgb(37, 99, 235)),
                                )
                                .on_hover_text(
                                    "Sign in with Google to unlock Google Drive Folder Inventory (Guest Mode works without login)",
                                )
                                .clicked()
                            {
                                app.inventory.start_google_login(ctx.clone());
                            }
                        }

                        if !is_phone {
                            ui.separator();
                            let theme_lbl = match app.theme_mode {
                                ThemeMode::Light => "Platinum 90s",
                                ThemeMode::Dark => "Classic Dark",
                                ThemeMode::HighContrast => "High Contrast",
                            };
                            if ui.small_button(theme_lbl).clicked() {
                                app.theme_mode = match app.theme_mode {
                                    ThemeMode::Light => ThemeMode::Dark,
                                    ThemeMode::Dark => ThemeMode::HighContrast,
                                    ThemeMode::HighContrast => ThemeMode::Light,
                                };
                                crate::theme::apply_theme(ctx, app.theme_mode);
                            }
                            if !narrow {
                                ui.separator();
                                ui.label(RichText::new("100% Rust").monospace().size(10.5));
                            }
                        }
                    });
                });
        });

    egui::TopBottomPanel::top("platinum_toolbar")
        .frame(
            Frame::new()
                .fill(match app.theme_mode {
                    ThemeMode::Dark => Color32::from_rgb(36, 39, 48),
                    _ => Color32::from_rgb(220, 220, 224),
                })
                .stroke(Stroke::new(1.0_f32, Color32::from_rgb(132, 132, 144)))
                .inner_margin(Margin::symmetric(if is_phone { 4 } else { 8 }, 4)),
        )
        .show(ctx, |ui| {
            ScrollArea::horizontal()
                .id_salt("toolbar_hscroll")
                .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysHidden)
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        if ui
                            .selectable_label(app.show_palette, "Symbols")
                            .on_hover_text("Toggle Left Symbols Panel")
                            .clicked()
                        {
                            app.show_palette = !app.show_palette;
                            if narrow && app.show_palette {
                                app.show_inspector = false;
                            }
                        }
                        if ui
                            .selectable_label(app.show_inspector, "Inspector")
                            .on_hover_text("Toggle Right Inspector Panel")
                            .clicked()
                        {
                            app.show_inspector = !app.show_inspector;
                            if narrow && app.show_inspector {
                                app.show_palette = false;
                            }
                        }
                        let pan_lbl = if app.hand_pan_mode { "Pan" } else { "Select" };
                        if ui
                            .selectable_label(app.hand_pan_mode, pan_lbl)
                            .on_hover_text(
                                "Toggle Hand Pan vs Select mode (1-finger drag pans canvas when Pan is active; pinch-to-zoom works anytime)",
                            )
                            .clicked()
                        {
                            app.hand_pan_mode = !app.hand_pan_mode;
                        }
                        ui.separator();

                if icon_button(ui, ToolIcon::NewFile, Some("New"), "Create New Empty Space", true, false).clicked() {
                    app.new_blank_diagram();
                }
                if icon_button(ui, ToolIcon::OpenFile, Some("Open"), "Open .json / .drawio", true, false).clicked() {
                    app.open_file();
                }
                ui.separator();

                if icon_button(ui, ToolIcon::Undo, None, "Undo (Ctrl+Z)", !app.undo_stack.is_empty(), false).clicked() {
                    app.undo();
                }
                if icon_button(ui, ToolIcon::Redo, None, "Redo (Ctrl+Y)", !app.redo_stack.is_empty(), false).clicked() {
                    app.redo();
                }
                ui.separator();

                if icon_button(ui, ToolIcon::Copy, None, "Copy (Ctrl+C)", true, false).clicked() {
                    app.copy_selection();
                }
                if icon_button(ui, ToolIcon::Paste, None, "Paste (Ctrl+V)", !app.clipboard_nodes.is_empty(), false).clicked() {
                    app.paste_clipboard();
                }
                if icon_button(ui, ToolIcon::Duplicate, None, "Duplicate (Ctrl+D)", true, false).clicked() {
                    app.duplicate_selection();
                }
                if icon_button(ui, ToolIcon::Delete, None, "Delete Selected (Del)", true, false).clicked() {
                    app.delete_selection();
                }
                ui.separator();

                if icon_button(ui, ToolIcon::ZoomOut, None, "Zoom Out (Ctrl+-)", true, false).clicked() {
                    app.zoom_step(0.85);
                }
                ui.label(
                    RichText::new(format!("{:.0}%", app.bridge.viewport.zoom * 100.0))
                        .monospace()
                        .size(11.0),
                );
                if icon_button(ui, ToolIcon::ZoomIn, None, "Zoom In (Ctrl++)", true, false).clicked() {
                    app.zoom_step(1.15);
                }
                if icon_button(ui, ToolIcon::FitView, Some("Fit"), "Fit View (Ctrl+1)", true, false).clicked() {
                    app.fit_view();
                }
                egui::ComboBox::from_id_salt("tb_bg_style")
                    .width(74.0)
                    .selected_text(match app.bg_style {
                        CanvasBgStyle::Dots => "Dots",
                        CanvasBgStyle::Grid => "Grid",
                        CanvasBgStyle::Crosshair => "Cross",
                        CanvasBgStyle::Isometric => "Iso",
                        CanvasBgStyle::None => "Solid",
                    })
                    .show_ui(ui, |ui| {
                        for (lbl, st) in [
                            ("Dot Grid", CanvasBgStyle::Dots),
                            ("Line Grid", CanvasBgStyle::Grid),
                            ("Crosshair", CanvasBgStyle::Crosshair),
                            ("Isometric", CanvasBgStyle::Isometric),
                            ("Solid / None", CanvasBgStyle::None),
                        ] {
                            if ui.selectable_value(&mut app.bg_style, st, lbl).changed() {
                                app.show_grid = !matches!(st, CanvasBgStyle::None);
                            }
                        }
                    });
                ui.separator();

                let mut changed_kind = false;
                egui::ComboBox::from_id_salt("tb_edge_kind")
                    .width(86.0)
                    .selected_text(format!("{:?}", app.default_edge_kind))
                    .show_ui(ui, |ui| {
                        for k in [
                            EdgeKind::Bezier,
                            EdgeKind::SmoothStep,
                            EdgeKind::Orthogonal,
                            EdgeKind::Straight,
                        ] {
                            if ui
                                .selectable_value(&mut app.default_edge_kind, k, format!("{k:?}"))
                                .changed()
                            {
                                changed_kind = true;
                            }
                        }
                    });
                if changed_kind && !app.bridge.selection.selected_edges.is_empty() {
                    app.push_undo();
                    let kind = app.default_edge_kind;
                    let eids = app.bridge.selection.selected_edges.clone();
                    for eid in eids {
                        if let Some(e) = app.bridge.scene.get_edge_mut(eid) {
                            e.kind = kind;
                        }
                    }
                    app.broadcast_scene_snapshot();
                }

                let mut changed_arrow = false;
                egui::ComboBox::from_id_salt("tb_edge_arrow")
                    .width(74.0)
                    .selected_text(match app.default_edge_arrow {
                        EdgeArrowStyle::OneWay => "1-Way",
                        EdgeArrowStyle::TwoWay => "2-Way",
                        EdgeArrowStyle::None => "No Head",
                        EdgeArrowStyle::Open => "Open",
                        EdgeArrowStyle::Diamond => "Diamond",
                    })
                    .show_ui(ui, |ui| {
                        for (lbl, arr) in [
                            ("1-Way", EdgeArrowStyle::OneWay),
                            ("2-Way", EdgeArrowStyle::TwoWay),
                            ("No Head", EdgeArrowStyle::None),
                            ("Open", EdgeArrowStyle::Open),
                            ("Diamond", EdgeArrowStyle::Diamond),
                        ] {
                            if ui
                                .selectable_value(&mut app.default_edge_arrow, arr, lbl)
                                .changed()
                            {
                                changed_arrow = true;
                            }
                        }
                    });
                if changed_arrow && !app.bridge.selection.selected_edges.is_empty() {
                    app.push_undo();
                    let arrow = app.default_edge_arrow;
                    let eids = app.bridge.selection.selected_edges.clone();
                    for eid in eids {
                        if let Some(e) = app.bridge.scene.get_edge_mut(eid) {
                            e.arrow = arrow;
                        }
                    }
                    app.broadcast_scene_snapshot();
                }

                let mut changed_anim = false;
                egui::ComboBox::from_id_salt("tb_edge_anim")
                    .width(74.0)
                    .selected_text(match app.default_edge_anim {
                        1 => "Ants",
                        2 => "Pulse",
                        _ => "Static",
                    })
                    .show_ui(ui, |ui| {
                        changed_anim |= ui.selectable_value(&mut app.default_edge_anim, 0, "Static").changed();
                        changed_anim |= ui.selectable_value(&mut app.default_edge_anim, 1, "Ants").changed();
                        changed_anim |= ui.selectable_value(&mut app.default_edge_anim, 2, "Pulse").changed();
                    });
                if changed_anim && !app.bridge.selection.selected_edges.is_empty() {
                    app.push_undo();
                    let eids = app.bridge.selection.selected_edges.clone();
                    for eid in eids {
                        let anim = app
                            .bridge
                            .scene
                            .get_edge(eid)
                            .map(|e| app.default_edge_animation(e.stroke));
                        if let (Some(e), Some(a)) = (app.bridge.scene.get_edge_mut(eid), anim) {
                            e.animation = a;
                        }
                    }
                    app.broadcast_scene_snapshot();
                }
                ui.separator();

                if icon_button(ui, ToolIcon::Image, Some("Image"), "Add Image or Set Selected Image", true, false).clicked() {
                    let target = app
                        .bridge
                        .selection
                        .selected_nodes
                        .first()
                        .copied()
                        .filter(|&id| {
                            app.bridge
                                .scene
                                .get_node(id)
                                .is_some_and(|n| matches!(n.shape, ShapeKind::ImageNode))
                        });
                    app.trigger_image_upload(target, ctx);
                }
                if icon_button(ui, ToolIcon::Symbol, Some("Symbol"), "Open Symbol Studio", true, false).clicked() {
                    app.show_symbol_studio = true;
                }
                if icon_button(ui, ToolIcon::Png, Some("PNG"), "Copy as 2x PNG", true, false).clicked() {
                    app.copy_diagram_png();
                }
                if icon_button(ui, ToolIcon::Gif, Some("GIF"), "Copy as Animated GIF", true, true).clicked() {
                    app.copy_diagram_gif();
                }
                if icon_button(ui, ToolIcon::Layout, Some("Layout"), "Hierarchical Auto-Layout", true, false).clicked() {
                    app.push_undo();
                    app.bridge.trigger_auto_layout(LayoutAlgorithm::Hierarchical, 0.35);
                    app.broadcast_scene_snapshot();
                }
                let share_lbl = if app.collab.connected {
                    format!("Copy Link ({})", app.collab.peers.len())
                } else {
                    "Copy Link".to_string()
                };
                let link_resp = icon_button(
                    ui,
                    ToolIcon::Share,
                    Some(&share_lbl),
                    "Copy room link to clipboard (Right-click for Room settings)",
                    true,
                    app.collab.connected,
                );
                if link_resp.clicked() {
                    let room = if app.collab.connected {
                        app.collab.room_id.clone()
                    } else {
                        let current = app.collab_room_input.trim().to_string();
                        let unique = if current.is_empty() || current == "lobby" {
                            crate::platform::generate_unique_room_id()
                        } else {
                            current
                        };
                        app.collab_room_input = unique.clone();
                        app.collab_ws_url = crate::platform::default_ws_url(&unique);
                        app.collab.connect(&app.collab_ws_url, &unique, ctx.clone());
                        unique
                    };
                    let share_url = crate::platform::share_room_url(&room);
                    ctx.copy_text(share_url.clone());
                    app.set_status(format!("Copied unique link: {share_url}"));
                }
                if link_resp.secondary_clicked() {
                    app.show_collab_modal = !app.show_collab_modal;
                }
                if icon_button(ui, ToolIcon::Help, None, "Keyboard Shortcuts (?)", true, false).clicked() {
                    app.show_shortcuts = true;
                }
                if icon_button(ui, ToolIcon::Search, Some("Ctrl+K"), "Command Palette (Ctrl+K)", true, false).clicked() {
                    app.show_cmd_palette = true;
                }
            });
            });
        });
}

pub fn render_bottom_bar(app: &mut BohatDrawApp, ctx: &egui::Context) {
    let screen_w = ctx.screen_rect().width();
    let is_phone = screen_w <= 640.0;
    egui::TopBottomPanel::bottom("platinum_bottom")
        .frame(
            Frame::new()
                .fill(match app.theme_mode {
                    ThemeMode::Dark => Color32::from_rgb(28, 31, 40),
                    _ => Color32::from_rgb(220, 220, 224),
                })
                .stroke(Stroke::new(1.0_f32, Color32::from_rgb(124, 124, 136)))
                .inner_margin(if is_phone {
                    Margin {
                        left: 6,
                        right: 6,
                        top: 5,
                        bottom: 18,
                    }
                } else {
                    Margin::symmetric(8, 3)
                }),
        )
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                let active_id = app.bridge.document.active_page_id();
                let pages: Vec<(u64, String)> = app
                    .bridge
                    .document
                    .pages()
                    .iter()
                    .map(|p| (p.id, p.name.clone()))
                    .collect();
                for (pid, pname) in pages {
                    let selected = pid == active_id;
                    let resp = if is_phone {
                        ui.add_sized(
                            [68.0, 24.0],
                            egui::Button::new(
                                RichText::new(&pname).size(11.0).color(if selected {
                                    Color32::WHITE
                                } else {
                                    Color32::from_rgb(20, 24, 36)
                                }),
                            )
                            .fill(if selected {
                                Color32::from_rgb(0, 0, 128)
                            } else {
                                Color32::from_rgb(238, 239, 244)
                            }),
                        )
                    } else {
                        ui.selectable_label(selected, &pname)
                    };
                    if resp.clicked() && !selected {
                        app.bridge.switch_page(pid);
                    }
                }
                let add_resp = if is_phone {
                    ui.add_sized([28.0, 24.0], egui::Button::new(RichText::new("+").strong()))
                } else {
                    ui.small_button("+")
                };
                if add_resp.on_hover_text("Add Page").clicked() {
                    let next_num = app.bridge.document.pages().len() + 1;
                    app.bridge.add_page(format!("Page-{next_num}"));
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let sel_count = app.bridge.selection.selected_nodes.len()
                        + app.bridge.selection.selected_edges.len();
                    if is_phone {
                        ui.label(
                            RichText::new(format!(
                                "{:.0}% | {}n {}e",
                                app.bridge.viewport.zoom * 100.0,
                                app.bridge.scene.nodes().len(),
                                app.bridge.scene.edges().len()
                            ))
                            .monospace()
                            .size(10.0),
                        );
                    } else {
                        ui.label(
                            RichText::new(format!(
                                "FPS: {:.0} | Zoom: {:.0}% | Selected: {} | Nodes: {} | Edges: {}",
                                app.fps,
                                app.bridge.viewport.zoom * 100.0,
                                sel_count,
                                app.bridge.scene.nodes().len(),
                                app.bridge.scene.edges().len()
                            ))
                            .monospace()
                            .size(10.5),
                        );
                        ui.separator();
                        ui.label(RichText::new(&app.status_message).size(10.5).weak());
                    }
                });
            });
        });
}

pub fn render_palette_panel(app: &mut BohatDrawApp, ctx: &egui::Context) {
    if !app.show_palette {
        return;
    }
    let screen_w = ctx.screen_rect().width();
    let is_phone = screen_w <= 640.0;
    let (def_w, min_w, max_w) = if is_phone {
        (
            (screen_w * 0.80).clamp(196.0, 256.0),
            188.0,
            (screen_w * 0.86).clamp(210.0, 280.0),
        )
    } else {
        (256.0, 220.0, 340.0)
    };
    egui::SidePanel::left("palette_panel")
        .default_width(def_w)
        .min_width(min_w)
        .max_width(max_w)
        .resizable(true)
        .show_separator_line(false)
        .frame(
            Frame::new()
                .fill(desktop_bg_color(app.theme_mode))
                .inner_margin(Margin {
                    left: 4,
                    right: 0,
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
                    let query = app.palette_search.trim().to_lowercase();
                    let all_cats: [(StencilCategory, &str, &str, Option<Color32>); 10] = [
                        (StencilCategory::Aws, "AWS Cloud", "AWS", Some(Color32::from_rgb(237, 113, 0))),
                        (StencilCategory::Azure, "Microsoft Azure", "Azure", Some(Color32::from_rgb(0, 120, 212))),
                        (StencilCategory::Gcp, "Google Cloud (GCP)", "GCP", Some(Color32::from_rgb(66, 133, 244))),
                        (StencilCategory::InfraDevOps, "Infra & DevOps", "Infra", Some(Color32::from_rgb(50, 108, 229))),
                        (StencilCategory::Standard, "Standard Icons", "Std", Some(Color32::from_rgb(15, 118, 110))),
                        (StencilCategory::Flowchart, "Flowchart & ISO", "Flow", None),
                        (StencilCategory::Network, "Network & Security", "Net", None),
                        (StencilCategory::Uml, "UML Diagrams", "UML", None),
                        (StencilCategory::Bpmn, "BPMN Workflows", "BPMN", None),
                        (StencilCategory::Wireframes, "Wireframes & Containers", "UI", None),
                    ];
                    let mut cat_mask = 0u16;
                    for (bit, &(cat, ..)) in all_cats.iter().enumerate() {
                        if app.palette_selected_cats.contains(&cat) {
                            cat_mask |= 1 << bit;
                        }
                    }
                    let sub_key = app.palette_subgroup.clone().unwrap_or_default();
                    if app.palette_cache.is_empty()
                        || app.palette_cache_key.0 != query
                        || app.palette_cache_key.1 != cat_mask
                        || app.palette_cache_key.2 != sub_key
                    {
                        app.palette_cache = app
                            .bridge
                            .palette
                            .search(&query)
                            .into_iter()
                            .filter(|item| {
                                (app.palette_selected_cats.is_empty()
                                    || app.palette_selected_cats.contains(&item.category))
                                    && app
                                        .palette_subgroup
                                        .as_deref()
                                        .is_none_or(|sg| item.subgroup == sg)
                            })
                            .cloned()
                            .collect();
                        app.palette_cache_key = (query.clone(), cat_mask, sub_key);
                    }
                    let count_badge =
                        format!("{} items", app.palette_cache.len() + app.custom_stencils.len());
                    if draw_platinum_titlebar(ui, app.theme_mode, "Symbols & Cloud", Some(&count_badge)) {
                        app.show_palette = false;
                    }

                    Frame::new().inner_margin(Margin::same(8)).show(ui, |ui| {
                        let inner_w = ui.available_width();
                        ui.set_max_width(inner_w);
                        ui.horizontal(|ui| {
                            let has_q = !app.palette_search.is_empty();
                            let clear_w = if has_q { 24.0 } else { 0.0 };
                            let search_w = (inner_w - 74.0 - clear_w).max(60.0);
                            ui.add(
                                egui::TextEdit::singleline(&mut app.palette_search)
                                    .hint_text("Search 130+ symbols...")
                                    .desired_width(search_w),
                            );
                            if has_q && ui.small_button("x").on_hover_text("Clear search").clicked() {
                                app.palette_search.clear();
                            }
                            if ui
                                .add_sized(
                                    [54.0, 22.0],
                                    egui::Button::new(RichText::new("+ New").color(Color32::WHITE).size(11.0))
                                        .fill(Color32::from_rgb(0, 0, 128)),
                                )
                                .on_hover_text("Open Symbol Studio")
                                .clicked()
                            {
                                app.show_symbol_studio = true;
                            }
                        });
                        ui.add_space(5.0);

                        // Filterable Multi-Select Category Box
                        let summary = if app.palette_selected_cats.is_empty() {
                            format!("All Categories ({})", app.bridge.palette.items().len())
                        } else if app.palette_selected_cats.len() == 1 {
                            let c = app.palette_selected_cats[0];
                            let name = all_cats
                                .iter()
                                .find(|(k, ..)| *k == c)
                                .map(|(_, n, ..)| *n)
                                .unwrap_or("Selected");
                            format!("{name} ({})", app.bridge.palette.by_category(c).len())
                        } else if app.palette_selected_cats.len() <= 3 {
                            let names: Vec<&str> = app
                                .palette_selected_cats
                                .iter()
                                .filter_map(|c| {
                                    all_cats.iter().find(|(k, ..)| k == c).map(|(_, _, s, _)| *s)
                                })
                                .collect();
                            format!("{} ({})", names.join(", "), app.palette_cache.len())
                        } else {
                            format!(
                                "{} categories ({})",
                                app.palette_selected_cats.len(),
                                app.palette_cache.len()
                            )
                        };

                        let (sel_rect, sel_resp) =
                            ui.allocate_exact_size(Vec2::new(inner_w, 24.0), Sense::click());
                        let sel_bg = if app.palette_cat_dropdown_open {
                            Color32::from_rgb(232, 238, 255)
                        } else if sel_resp.hovered() {
                            Color32::from_rgb(244, 246, 252)
                        } else {
                            Color32::WHITE
                        };
                        let sel_border = if app.palette_cat_dropdown_open || sel_resp.hovered() {
                            Color32::from_rgb(0, 0, 140)
                        } else {
                            Color32::from_rgb(132, 136, 152)
                        };
                        ui.painter().rect_filled(sel_rect, 3.0, sel_bg);
                        ui.painter().rect_stroke(
                            sel_rect,
                            3.0,
                            Stroke::new(1.0_f32, sel_border),
                            egui::StrokeKind::Inside,
                        );
                        ui.painter().text(
                            egui::Pos2::new(sel_rect.min.x + 8.0, sel_rect.center().y),
                            Align2::LEFT_CENTER,
                            summary,
                            FontId::proportional(11.0),
                            Color32::from_rgb(20, 24, 38),
                        );
                        let arrow_txt = if app.palette_cat_dropdown_open { "^" } else { "v" };
                        ui.painter().text(
                            egui::Pos2::new(sel_rect.max.x - 10.0, sel_rect.center().y),
                            Align2::CENTER_CENTER,
                            arrow_txt,
                            FontId::monospace(10.0),
                            Color32::from_rgb(60, 66, 86),
                        );
                        if sel_resp.clicked() {
                            app.palette_cat_dropdown_open = !app.palette_cat_dropdown_open;
                        }

                        if app.palette_cat_dropdown_open {
                            let popup_pos = egui::Pos2::new(sel_rect.min.x, sel_rect.max.y + 3.0);
                            let popup_w = inner_w.max(220.0);
                            let area_resp = egui::Area::new(egui::Id::new("palette_cat_multiselect_popup"))
                                .order(egui::Order::Foreground)
                                .fixed_pos(popup_pos)
                                .show(ctx, |ui| {
                                    egui::Frame::popup(ui.style())
                                        .inner_margin(Margin::same(6))
                                        .show(ui, |ui| {
                                            ui.set_width(popup_w - 12.0);
                                            ui.horizontal(|ui| {
                                                let has_f = !app.palette_cat_filter.is_empty();
                                                let fw = (popup_w - 28.0 - if has_f { 22.0 } else { 0.0 }).max(80.0);
                                                ui.add(
                                                    egui::TextEdit::singleline(&mut app.palette_cat_filter)
                                                        .hint_text("Filter categories...")
                                                        .desired_width(fw),
                                                );
                                                if has_f && ui.small_button("x").clicked() {
                                                    app.palette_cat_filter.clear();
                                                }
                                            });
                                            ui.add_space(4.0);
                                            ui.horizontal(|ui| {
                                                let is_all = app.palette_selected_cats.is_empty();
                                                if ui
                                                    .selectable_label(is_all, RichText::new("All").small().strong())
                                                    .clicked()
                                                {
                                                    app.palette_selected_cats.clear();
                                                    app.palette_subgroup = None;
                                                }
                                                if !is_all && ui.small_button("Clear").clicked() {
                                                    app.palette_selected_cats.clear();
                                                    app.palette_subgroup = None;
                                                }
                                                ui.with_layout(
                                                    egui::Layout::right_to_left(egui::Align::Center),
                                                    |ui| {
                                                        if ui.small_button("Done").clicked() {
                                                            app.palette_cat_dropdown_open = false;
                                                        }
                                                    },
                                                );
                                            });
                                            ui.separator();
                                            let cf = app.palette_cat_filter.trim().to_lowercase();
                                            ScrollArea::vertical().max_height(220.0).show(ui, |ui| {
                                                let mut any_match = false;
                                                for &(cat, full_name, short_name, dot_col) in &all_cats {
                                                    if !cf.is_empty()
                                                        && !full_name.to_lowercase().contains(&cf)
                                                        && !short_name.to_lowercase().contains(&cf)
                                                    {
                                                        continue;
                                                    }
                                                    any_match = true;
                                                    let mut checked = app.palette_selected_cats.contains(&cat);
                                                    let cnt = app.bridge.palette.by_category(cat).len();
                                                    ui.horizontal(|ui| {
                                                        if ui.checkbox(&mut checked, "").changed() {
                                                            if checked {
                                                                if !app.palette_selected_cats.contains(&cat) {
                                                                    app.palette_selected_cats.push(cat);
                                                                }
                                                            } else {
                                                                app.palette_selected_cats.retain(|c| *c != cat);
                                                            }
                                                            app.palette_subgroup = None;
                                                        }
                                                        if let Some(dc) = dot_col {
                                                            let (dr, _) = ui.allocate_exact_size(
                                                                Vec2::splat(8.0),
                                                                Sense::hover(),
                                                            );
                                                            ui.painter().circle_filled(dr.center(), 3.2, dc);
                                                        }
                                                        let lbl_resp = ui.selectable_label(
                                                            checked,
                                                            format!("{full_name} ({cnt})"),
                                                        );
                                                        if lbl_resp.clicked() {
                                                            if checked {
                                                                app.palette_selected_cats.retain(|c| *c != cat);
                                                            } else {
                                                                app.palette_selected_cats.push(cat);
                                                            }
                                                            app.palette_subgroup = None;
                                                        }
                                                    });
                                                }
                                                if !any_match {
                                                    ui.label(RichText::new("No matching category").small().weak());
                                                }
                                            });
                                        });
                                });
                            if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
                                app.palette_cat_dropdown_open = false;
                            } else if ctx.input(|i| i.pointer.any_pressed()) {
                                if let Some(p) = ctx.input(|i| i.pointer.interact_pos()) {
                                    if !area_resp.response.rect.contains(p) && !sel_rect.contains(p) {
                                        app.palette_cat_dropdown_open = false;
                                    }
                                }
                            }
                        }
                        ui.add_space(4.0);

                        // 1-Row Quick Provider Multi-Toggle Shortcuts
                        let quick_row: [(Option<StencilCategory>, &str, Option<Color32>); 5] = [
                            (None, "All", None),
                            (Some(StencilCategory::Aws), "AWS", Some(Color32::from_rgb(237, 113, 0))),
                            (Some(StencilCategory::Azure), "Azure", Some(Color32::from_rgb(0, 120, 212))),
                            (Some(StencilCategory::Gcp), "GCP", Some(Color32::from_rgb(66, 133, 244))),
                            (Some(StencilCategory::Flowchart), "Flow", None),
                        ];
                        let gap = 3.0_f32;
                        let pill_w = ((inner_w - gap * 4.0 - 2.0) / 5.0).floor().max(32.0);
                        ui.horizontal(|ui| {
                            ui.spacing_mut().item_spacing.x = gap;
                            for &(cat_opt, label, dot_col) in &quick_row {
                                let selected = match cat_opt {
                                    None => app.palette_selected_cats.is_empty(),
                                    Some(c) => app.palette_selected_cats.contains(&c),
                                };
                                let (rect, resp) = ui.allocate_exact_size(
                                    Vec2::new(pill_w, 22.0),
                                    Sense::click(),
                                );
                                let bg = if selected {
                                    Color32::from_rgb(0, 0, 128)
                                } else if resp.hovered() {
                                    Color32::from_rgb(232, 238, 252)
                                } else {
                                    Color32::from_rgb(240, 241, 246)
                                };
                                let border = if selected {
                                    Color32::from_rgb(0, 0, 110)
                                } else {
                                    Color32::from_rgb(164, 168, 182)
                                };
                                let fg = if selected {
                                    Color32::WHITE
                                } else {
                                    Color32::from_rgb(24, 28, 42)
                                };
                                ui.painter().rect_filled(rect, 3.0, bg);
                                ui.painter().rect_stroke(
                                    rect,
                                    3.0,
                                    Stroke::new(1.0_f32, border),
                                    egui::StrokeKind::Inside,
                                );
                                if let Some(dc) = dot_col {
                                    ui.painter().circle_filled(
                                        egui::Pos2::new(rect.min.x + 6.0, rect.center().y),
                                        2.5,
                                        if selected { Color32::WHITE } else { dc },
                                    );
                                    ui.painter().text(
                                        egui::Pos2::new(rect.center().x + 3.0, rect.center().y),
                                        Align2::CENTER_CENTER,
                                        label,
                                        FontId::proportional(10.5),
                                        fg,
                                    );
                                } else {
                                    ui.painter().text(
                                        rect.center(),
                                        Align2::CENTER_CENTER,
                                        label,
                                        FontId::proportional(10.5),
                                        fg,
                                    );
                                }
                                if resp.clicked() {
                                    match cat_opt {
                                        None => app.palette_selected_cats.clear(),
                                        Some(c) => {
                                            if let Some(pos) =
                                                app.palette_selected_cats.iter().position(|x| *x == c)
                                            {
                                                app.palette_selected_cats.remove(pos);
                                            } else {
                                                app.palette_selected_cats.push(c);
                                            }
                                        }
                                    }
                                    app.palette_subgroup = None;
                                }
                            }
                        });

                        // Sub-group Filter Chips when AWS or Azure is selected
                        let has_cloud_sub = app
                            .palette_selected_cats
                            .iter()
                            .any(|c| matches!(c, StencilCategory::Aws | StencilCategory::Azure));
                        if has_cloud_sub {
                            ui.add_space(4.0);
                            let subgroups = [
                                (None, "All"),
                                (Some("Compute"), "Compute"),
                                (Some("Storage"), "Storage"),
                                (Some("Database"), "DB"),
                                (Some("Network"), "Net"),
                                (Some("Messaging"), "Msg"),
                                (Some("Analytics"), "Data"),
                                (Some("Security"), "Sec"),
                                (Some("DevOps"), "Ops"),
                                (Some("AI & Data"), "AI"),
                            ];
                            ui.horizontal_wrapped(|ui| {
                                ui.spacing_mut().item_spacing = Vec2::new(3.0, 3.0);
                                for (sg_opt, short) in subgroups {
                                    let sel = app.palette_subgroup.as_deref() == sg_opt;
                                    if ui.selectable_label(sel, RichText::new(short).size(10.5)).clicked() {
                                        app.palette_subgroup = sg_opt.map(|s| s.to_string());
                                    }
                                }
                            });
                        }
                        // View Mode Switcher (Icon / Icon+Name Finder style / Name List)
                        ui.add_space(4.0);
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("View").size(10.5).weak());
                            let vw = ((inner_w - 38.0 - 10.0) / 3.0).floor().max(42.0);
                            for (mode, lbl) in [
                                (PaletteViewMode::IconOnly, "Icon"),
                                (PaletteViewMode::IconAndName, "Icon+Name"),
                                (PaletteViewMode::NameList, "Name"),
                            ] {
                                let sel = app.palette_view_mode == mode;
                                let (rect, resp) =
                                    ui.allocate_exact_size(Vec2::new(vw, 20.0), Sense::click());
                                let bg = if sel {
                                    Color32::from_rgb(0, 0, 128)
                                } else if resp.hovered() {
                                    Color32::from_rgb(232, 238, 252)
                                } else {
                                    Color32::from_rgb(240, 241, 246)
                                };
                                let fg = if sel {
                                    Color32::WHITE
                                } else {
                                    Color32::from_rgb(24, 28, 42)
                                };
                                ui.painter().rect_filled(rect, 3.0, bg);
                                ui.painter().rect_stroke(
                                    rect,
                                    3.0,
                                    Stroke::new(1.0_f32, Color32::from_rgb(160, 164, 178)),
                                    egui::StrokeKind::Inside,
                                );
                                ui.painter().text(
                                    rect.center(),
                                    Align2::CENTER_CENTER,
                                    lbl,
                                    FontId::proportional(10.0),
                                    fg,
                                );
                                if resp.clicked() {
                                    app.palette_view_mode = mode;
                                }
                            }
                        });
                        ui.add_space(3.0);
                        ui.separator();
                        ui.add_space(3.0);

                        enum PalRow {
                            Header(&'static str, usize, Option<Color32>),
                            Custom([usize; 4], u8),
                            Items([usize; 4], u8),
                        }

                        let view_mode = app.palette_view_mode;
                        let avail_w = (ui.available_width() - 10.0).max(180.0);
                        let cols: usize = match view_mode {
                            PaletteViewMode::IconOnly => {
                                if avail_w >= 210.0 { 3 } else { 2 }
                            }
                            PaletteViewMode::IconAndName | PaletteViewMode::NameList => 1,
                        };
                        let tile_gap = 5.0_f32;
                        let tile_w = ((avail_w - tile_gap * (cols as f32 - 1.0)) / cols as f32)
                            .floor()
                            .max(46.0);
                        let tile_h = match view_mode {
                            PaletteViewMode::IconOnly => 62.0_f32,
                            PaletteViewMode::IconAndName => 30.0_f32,
                            PaletteViewMode::NameList => 24.0_f32,
                        };

                        let mut rows: Vec<PalRow> = Vec::with_capacity(64);
                        if !app.custom_stencils.is_empty() && app.palette_selected_cats.is_empty() {
                            rows.push(PalRow::Header("Custom Symbols", app.custom_stencils.len(), None));
                            let mut i = 0;
                            while i < app.custom_stencils.len() {
                                let mut arr = [0usize; 4];
                                let mut cnt = 0u8;
                                while (cnt as usize) < cols && i < app.custom_stencils.len() {
                                    arr[cnt as usize] = i;
                                    cnt += 1;
                                    i += 1;
                                }
                                rows.push(PalRow::Custom(arr, cnt));
                            }
                        }

                        let sections: [(StencilCategory, &'static str, Option<Color32>); 10] = [
                            (StencilCategory::Aws, "AWS Cloud", Some(Color32::from_rgb(237, 113, 0))),
                            (StencilCategory::Azure, "Microsoft Azure", Some(Color32::from_rgb(0, 120, 212))),
                            (StencilCategory::Gcp, "Google Cloud (GCP)", Some(Color32::from_rgb(66, 133, 244))),
                            (StencilCategory::InfraDevOps, "Infra & DevOps", Some(Color32::from_rgb(50, 108, 229))),
                            (StencilCategory::Standard, "Standard Icons", Some(Color32::from_rgb(15, 118, 110))),
                            (StencilCategory::Flowchart, "Flowchart & ISO", None),
                            (StencilCategory::Network, "Network & Security", None),
                            (StencilCategory::Uml, "UML Diagrams", None),
                            (StencilCategory::Bpmn, "BPMN Workflows", None),
                            (StencilCategory::Wireframes, "Wireframes & Containers", None),
                        ];

                        for (cat, name, accent_opt) in sections {
                            let mut count = 0usize;
                            for item in &app.palette_cache {
                                if item.category == cat {
                                    count += 1;
                                }
                            }
                            if count == 0 {
                                continue;
                            }
                            rows.push(PalRow::Header(name, count, accent_opt));
                            let mut arr = [0usize; 4];
                            let mut cnt = 0u8;
                            for (idx, item) in app.palette_cache.iter().enumerate() {
                                if item.category != cat {
                                    continue;
                                }
                                arr[cnt as usize] = idx;
                                cnt += 1;
                                if (cnt as usize) == cols {
                                    rows.push(PalRow::Items(arr, cnt));
                                    arr = [0usize; 4];
                                    cnt = 0;
                                }
                            }
                            if cnt > 0 {
                                rows.push(PalRow::Items(arr, cnt));
                            }
                        }

                        if rows.is_empty() {
                            ui.add_space(16.0);
                            ui.vertical_centered(|ui| {
                                ui.label(RichText::new("No matching symbols").weak());
                                if ui.small_button("Reset filter").clicked() {
                                    app.palette_search.clear();
                                    app.palette_selected_cats.clear();
                                    app.palette_subgroup = None;
                                }
                            });
                        } else {
                            let mut spawn_custom_idx: Option<usize> = None;
                            let mut drag_custom_idx: Option<usize> = None;
                            let mut spawn_item_idx: Option<usize> = None;
                            let mut drag_item_idx: Option<usize> = None;
                            let is_touch_or_mobile =
                                screen_w <= 960.0 || ctx.input(|i| i.any_touches());
                            let tile_sense = if is_touch_or_mobile {
                                Sense::click()
                            } else {
                                Sense::click_and_drag()
                            };

                            ScrollArea::vertical()
                                .drag_to_scroll(true)
                                .auto_shrink([false, false])
                                .show_rows(ui, tile_h + 4.0, rows.len(), |ui, row_range| {
                                    for ridx in row_range {
                                        match rows[ridx] {
                                            PalRow::Header(name, cnt, accent_opt) => {
                                                let (hrect, _) = ui.allocate_exact_size(
                                                    Vec2::new(avail_w, tile_h),
                                                    Sense::hover(),
                                                );
                                                let bar_h = 22.0_f32.min(hrect.height());
                                                let bar = egui::Rect::from_min_max(
                                                    egui::Pos2::new(hrect.min.x, hrect.max.y - bar_h - 1.0),
                                                    egui::Pos2::new(hrect.max.x, hrect.max.y - 1.0),
                                                );
                                                ui.painter().rect_filled(
                                                    bar,
                                                    3.0,
                                                    Color32::from_rgb(224, 226, 234),
                                                );
                                                if let Some(ac) = accent_opt {
                                                    let dot = egui::Rect::from_min_size(
                                                        bar.min,
                                                        Vec2::new(4.0, bar.height()),
                                                    );
                                                    ui.painter().rect_filled(dot, 2.0, ac);
                                                }
                                                ui.painter().text(
                                                    egui::Pos2::new(bar.min.x + 10.0, bar.center().y),
                                                    Align2::LEFT_CENTER,
                                                    format!("{name} ({cnt})"),
                                                    FontId::proportional(11.0),
                                                    accent_opt.unwrap_or(Color32::from_rgb(24, 28, 42)),
                                                );
                                            }
                                            PalRow::Custom(arr, cnt) => {
                                                ui.horizontal(|ui| {
                                                    ui.spacing_mut().item_spacing.x = tile_gap;
                                                    for k in 0..(cnt as usize) {
                                                        let cidx = arr[k];
                                                        let c = &app.custom_stencils[cidx];
                                                        let (rect, resp) = ui.allocate_exact_size(
                                                            Vec2::new(tile_w, tile_h),
                                                            tile_sense,
                                                        );
                                                        let hovered = resp.hovered();
                                                        ui.painter().rect_filled(
                                                            rect,
                                                            3.0,
                                                            if hovered {
                                                                Color32::from_rgb(234, 240, 255)
                                                            } else {
                                                                Color32::WHITE
                                                            },
                                                        );
                                                        ui.painter().rect_stroke(
                                                            rect,
                                                            3.0,
                                                            Stroke::new(
                                                                if hovered { 1.4_f32 } else { 1.0_f32 },
                                                                if hovered {
                                                                    Color32::from_rgb(0, 0, 140)
                                                                } else {
                                                                    Color32::from_rgb(184, 188, 200)
                                                                },
                                                            ),
                                                            egui::StrokeKind::Inside,
                                                        );
                                                        match view_mode {
                                                            PaletteViewMode::NameList => {
                                                                ui.painter().text(
                                                                    egui::Pos2::new(rect.min.x + 8.0, rect.center().y),
                                                                    Align2::LEFT_CENTER,
                                                                    &c.label,
                                                                    FontId::proportional(11.0),
                                                                    Color32::from_rgb(24, 28, 42),
                                                                );
                                                            }
                                                            PaletteViewMode::IconAndName => {
                                                                let icon_rect = egui::Rect::from_center_size(
                                                                    egui::Pos2::new(rect.min.x + 17.0, rect.center().y),
                                                                    Vec2::new(22.0, 18.0),
                                                                );
                                                                for layer in &c.layers {
                                                                    let lr = egui::Rect::from_min_size(
                                                                        egui::Pos2::new(
                                                                            icon_rect.min.x + layer.x * icon_rect.width(),
                                                                            icon_rect.min.y + layer.y * icon_rect.height(),
                                                                        ),
                                                                        Vec2::new(
                                                                            (layer.w * icon_rect.width()).max(2.0),
                                                                            (layer.h * icon_rect.height()).max(2.0),
                                                                        ),
                                                                    );
                                                                    draw_simple_layer_shape(
                                                                        ui.painter(),
                                                                        lr,
                                                                        &layer.shape,
                                                                        to_color32(layer.fill),
                                                                        Stroke::new(1.0_f32, to_color32(layer.stroke)),
                                                                    );
                                                                }
                                                                ui.painter().text(
                                                                    egui::Pos2::new(rect.min.x + 34.0, rect.center().y),
                                                                    Align2::LEFT_CENTER,
                                                                    &c.label,
                                                                    FontId::proportional(11.0),
                                                                    Color32::from_rgb(24, 28, 42),
                                                                );
                                                            }
                                                            PaletteViewMode::IconOnly => {
                                                                let icon_rect = egui::Rect::from_center_size(
                                                                    egui::Pos2::new(rect.center().x, rect.min.y + 22.0),
                                                                    Vec2::new(34.0, 26.0),
                                                                );
                                                                for layer in &c.layers {
                                                                    let lr = egui::Rect::from_min_size(
                                                                        egui::Pos2::new(
                                                                            icon_rect.min.x + layer.x * icon_rect.width(),
                                                                            icon_rect.min.y + layer.y * icon_rect.height(),
                                                                        ),
                                                                        Vec2::new(
                                                                            (layer.w * icon_rect.width()).max(3.0),
                                                                            (layer.h * icon_rect.height()).max(3.0),
                                                                        ),
                                                                    );
                                                                    draw_simple_layer_shape(
                                                                        ui.painter(),
                                                                        lr,
                                                                        &layer.shape,
                                                                        to_color32(layer.fill),
                                                                        Stroke::new(1.1_f32, to_color32(layer.stroke)),
                                                                    );
                                                                }
                                                                let short_lbl: String = c.label.chars().take(10).collect();
                                                                ui.painter().text(
                                                                    egui::Pos2::new(rect.center().x, rect.max.y - 10.0),
                                                                    Align2::CENTER_CENTER,
                                                                    short_lbl,
                                                                    FontId::proportional(9.5),
                                                                    Color32::from_rgb(24, 28, 42),
                                                                );
                                                            }
                                                        }
                                                        if resp.drag_started() {
                                                            drag_custom_idx = Some(cidx);
                                                        } else if resp.clicked() {
                                                            spawn_custom_idx = Some(cidx);
                                                        }
                                                        if hovered {
                                                            resp.on_hover_text(&c.label);
                                                        }
                                                    }
                                                });
                                            }
                                            PalRow::Items(arr, cnt) => {
                                                ui.horizontal(|ui| {
                                                    ui.spacing_mut().item_spacing.x = tile_gap;
                                                    for k in 0..(cnt as usize) {
                                                        let item_idx = arr[k];
                                                        let item = &app.palette_cache[item_idx];
                                                        let (rect, resp) = ui.allocate_exact_size(
                                                            Vec2::new(tile_w, tile_h),
                                                            tile_sense,
                                                        );
                                                        let hovered = resp.hovered();
                                                        ui.painter().rect_filled(
                                                            rect,
                                                            3.0,
                                                            if hovered {
                                                                Color32::from_rgb(234, 240, 255)
                                                            } else {
                                                                Color32::WHITE
                                                            },
                                                        );
                                                        if view_mode == PaletteViewMode::IconOnly {
                                                            if let Some(icon) = item.icon {
                                                                let strip = egui::Rect::from_min_size(
                                                                    rect.min,
                                                                    Vec2::new(rect.width(), 2.5),
                                                                );
                                                                ui.painter().rect_filled(
                                                                    strip,
                                                                    1.0,
                                                                    symbol_icon_accent(icon),
                                                                );
                                                            }
                                                        }
                                                        ui.painter().rect_stroke(
                                                            rect,
                                                            3.0,
                                                            Stroke::new(
                                                                if hovered { 1.4_f32 } else { 1.0_f32 },
                                                                if hovered {
                                                                    Color32::from_rgb(0, 0, 140)
                                                                } else {
                                                                    Color32::from_rgb(184, 188, 200)
                                                                },
                                                            ),
                                                            egui::StrokeKind::Inside,
                                                        );
                                                        match view_mode {
                                                            PaletteViewMode::IconOnly => {
                                                                if let Some(icon) = item.icon {
                                                                    let icon_rect = egui::Rect::from_center_size(
                                                                        egui::Pos2::new(rect.center().x, rect.min.y + 24.0),
                                                                        Vec2::splat(28.0),
                                                                    );
                                                                    draw_symbol_icon(ui.painter(), icon_rect, icon);
                                                                } else {
                                                                    let icon_rect = egui::Rect::from_center_size(
                                                                        egui::Pos2::new(rect.center().x, rect.min.y + 23.0),
                                                                        Vec2::new(36.0, 28.0),
                                                                    );
                                                                    draw_stencil_preview(ui.painter(), icon_rect, &item.shape);
                                                                }
                                                                let max_c = if cols == 3 { 10 } else { 14 };
                                                                let short_lbl: String = if item.label.chars().count() > max_c {
                                                                    format!(
                                                                        "{}..",
                                                                        item.label.chars().take(max_c - 2).collect::<String>()
                                                                    )
                                                                } else {
                                                                    item.label.clone()
                                                                };
                                                                ui.painter().text(
                                                                    egui::Pos2::new(rect.center().x, rect.max.y - 10.0),
                                                                    Align2::CENTER_CENTER,
                                                                    short_lbl,
                                                                    FontId::proportional(9.5),
                                                                    Color32::from_rgb(24, 28, 42),
                                                                );
                                                            }
                                                            PaletteViewMode::IconAndName => {
                                                                if let Some(icon) = item.icon {
                                                                    let icon_rect = egui::Rect::from_center_size(
                                                                        egui::Pos2::new(rect.min.x + 16.0, rect.center().y),
                                                                        Vec2::splat(20.0),
                                                                    );
                                                                    draw_symbol_icon(ui.painter(), icon_rect, icon);
                                                                } else {
                                                                    let icon_rect = egui::Rect::from_center_size(
                                                                        egui::Pos2::new(rect.min.x + 17.0, rect.center().y),
                                                                        Vec2::new(24.0, 19.0),
                                                                    );
                                                                    draw_stencil_preview(ui.painter(), icon_rect, &item.shape);
                                                                }
                                                                ui.painter().text(
                                                                    egui::Pos2::new(rect.min.x + 33.0, rect.center().y),
                                                                    Align2::LEFT_CENTER,
                                                                    &item.label,
                                                                    FontId::proportional(11.0),
                                                                    Color32::from_rgb(20, 24, 38),
                                                                );
                                                                if !item.subgroup.is_empty() {
                                                                    ui.painter().text(
                                                                        egui::Pos2::new(rect.max.x - 7.0, rect.center().y),
                                                                        Align2::RIGHT_CENTER,
                                                                        &item.subgroup,
                                                                        FontId::proportional(9.0),
                                                                        Color32::from_rgb(108, 114, 134),
                                                                    );
                                                                }
                                                            }
                                                            PaletteViewMode::NameList => {
                                                                ui.painter().text(
                                                                    egui::Pos2::new(rect.min.x + 8.0, rect.center().y),
                                                                    Align2::LEFT_CENTER,
                                                                    &item.label,
                                                                    FontId::proportional(11.0),
                                                                    Color32::from_rgb(20, 24, 38),
                                                                );
                                                                if !item.subgroup.is_empty() {
                                                                    ui.painter().text(
                                                                        egui::Pos2::new(rect.max.x - 7.0, rect.center().y),
                                                                        Align2::RIGHT_CENTER,
                                                                        &item.subgroup,
                                                                        FontId::proportional(9.0),
                                                                        Color32::from_rgb(108, 114, 134),
                                                                    );
                                                                }
                                                            }
                                                        }
                                                        if resp.drag_started() {
                                                            drag_item_idx = Some(item_idx);
                                                        } else if resp.clicked() {
                                                            spawn_item_idx = Some(item_idx);
                                                        }
                                                        if hovered {
                                                            resp.on_hover_text(&item.label);
                                                        }
                                                    }
                                                });
                                            }
                                        }
                                    }
                                });

                            if let Some(cidx) = drag_custom_idx {
                                let c = &app.custom_stencils[cidx];
                                app.dragging_stencil = Some(DragStencilPayload {
                                    label: c.label.clone(),
                                    width: c.width,
                                    height: c.height,
                                    shape: c.shape.clone(),
                                    icon: None,
                                    layers: c.layers.clone(),
                                });
                            } else if let Some(cidx) = spawn_custom_idx {
                                let c = app.custom_stencils[cidx].clone();
                                let center = app.bridge.viewport.screen_to_world(
                                    rustflow_core::Point::new(
                                        app.bridge.viewport.screen_width * 0.5,
                                        app.bridge.viewport.screen_height * 0.5,
                                    ),
                                );
                                app.spawn_stencil_at_world(
                                    &c.label,
                                    c.width,
                                    c.height,
                                    c.shape,
                                    None,
                                    c.layers,
                                    center,
                                );
                                if is_phone {
                                    app.show_palette = false;
                                }
                            }
                            if let Some(idx) = drag_item_idx {
                                let item = &app.palette_cache[idx];
                                app.dragging_stencil = Some(DragStencilPayload {
                                    label: item.label.clone(),
                                    width: item.default_width,
                                    height: item.default_height,
                                    shape: item.shape.clone(),
                                    icon: item.icon,
                                    layers: Vec::new(),
                                });
                            } else if let Some(idx) = spawn_item_idx {
                                let item = app.palette_cache[idx].clone();
                                app.spawn_stencil_at_center(
                                    &item.label,
                                    item.default_width,
                                    item.default_height,
                                    item.shape,
                                    item.icon,
                                );
                                if is_phone {
                                    app.show_palette = false;
                                }
                            }
                        }
                    });
                });
            });
        });
}

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
}

fn draw_drive_folder_icon(painter: &egui::Painter, rect: egui::Rect) {
    let tab = egui::Rect::from_min_max(
        egui::Pos2::new(rect.min.x + 2.0, rect.min.y + 3.0),
        egui::Pos2::new(rect.min.x + rect.width() * 0.48, rect.min.y + 10.0),
    );
    let body = egui::Rect::from_min_max(
        egui::Pos2::new(rect.min.x + 2.0, rect.min.y + 8.0),
        egui::Pos2::new(rect.max.x - 2.0, rect.max.y - 3.0),
    );
    painter.rect_filled(tab, 2.0, Color32::from_rgb(234, 179, 8));
    painter.rect_filled(body, 2.5, Color32::from_rgb(250, 204, 21));
    painter.rect_stroke(
        body,
        2.5,
        Stroke::new(1.1_f32, Color32::from_rgb(161, 98, 7)),
        egui::StrokeKind::Inside,
    );
}

fn draw_drive_diagram_icon(painter: &egui::Painter, rect: egui::Rect) {
    let page = rect.shrink2(Vec2::new(5.0, 2.0));
    painter.rect_filled(page, 2.0, Color32::WHITE);
    painter.rect_stroke(
        page,
        2.0,
        Stroke::new(1.1_f32, Color32::from_rgb(0, 0, 140)),
        egui::StrokeKind::Inside,
    );
    let top_box = egui::Rect::from_center_size(
        egui::Pos2::new(page.center().x, page.min.y + page.height() * 0.30),
        Vec2::new(page.width() * 0.56, page.height() * 0.22),
    );
    let bot_box = egui::Rect::from_center_size(
        egui::Pos2::new(page.center().x, page.min.y + page.height() * 0.72),
        Vec2::new(page.width() * 0.56, page.height() * 0.22),
    );
    painter.rect_filled(top_box, 1.5, Color32::from_rgb(219, 234, 254));
    painter.rect_stroke(
        top_box,
        1.5,
        Stroke::new(1.0_f32, Color32::from_rgb(37, 99, 235)),
        egui::StrokeKind::Inside,
    );
    painter.rect_filled(bot_box, 1.5, Color32::from_rgb(220, 252, 231));
    painter.rect_stroke(
        bot_box,
        1.5,
        Stroke::new(1.0_f32, Color32::from_rgb(16, 185, 129)),
        egui::StrokeKind::Inside,
    );
    painter.line_segment(
        [
            egui::Pos2::new(page.center().x, top_box.max.y),
            egui::Pos2::new(page.center().x, bot_box.min.y),
        ],
        Stroke::new(1.1_f32, Color32::from_rgb(0, 0, 140)),
    );
}

pub fn render_inventory_modals(app: &mut BohatDrawApp, ctx: &egui::Context) {
    if app.inventory.show_setup_help {
        let mut open = app.inventory.show_setup_help;
        let mut close_now = false;
        let w = (ctx.screen_rect().width() - 28.0).clamp(280.0, 480.0);
        egui::Window::new("Google Sign-In & Drive Inventory Setup")
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .collapsible(false)
            .resizable(false)
            .fixed_size(Vec2::new(w, 290.0))
            .open(&mut open)
            .show(ctx, |ui| {
                ui.label(
                    RichText::new("Guest Mode is active — you can draw and export freely without signing in.")
                        .strong()
                        .size(12.0),
                );
                ui.add_space(6.0);
                ui.label("To enable Google Sign-In & Google Drive Folder Inventory on this server:");
                ui.add_space(4.0);
                ui.label("1. In Google Cloud Console, enable Google Drive API and create a Web OAuth 2.0 Client ID.");
                ui.label("2. Add Redirect URI: <PUBLIC_BASE_URL>/api/auth/google/callback");
                ui.label("3. Create a local .env file in the project root:");
                ui.add_space(4.0);
                Frame::new()
                    .fill(Color32::from_rgb(236, 240, 248))
                    .stroke(Stroke::new(1.0_f32, Color32::from_rgb(160, 170, 196)))
                    .corner_radius(3)
                    .inner_margin(Margin::same(6))
                    .show(ui, |ui| {
                        ui.label(
                            RichText::new(
                                "GOOGLE_CLIENT_ID=your-id.apps.googleusercontent.com\n\
                                 GOOGLE_CLIENT_SECRET=GOCSPX-your-secret\n\
                                 PUBLIC_BASE_URL=http://localhost:6129",
                            )
                            .monospace()
                            .size(10.5),
                        );
                    });
                ui.add_space(6.0);
                ui.label(RichText::new("4. Restart the server: systemctl --user restart bohat-drawio").small());
                ui.add_space(8.0);
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui
                        .add(
                            egui::Button::new(RichText::new("Continue as Guest").color(Color32::WHITE))
                                .fill(Color32::from_rgb(0, 0, 128)),
                        )
                        .clicked()
                    {
                        close_now = true;
                    }
                });
            });
        app.inventory.show_setup_help = open && !close_now;
    }

    if !app.inventory.show_modal {
        return;
    }
    if app.inventory.user.is_none() {
        app.inventory.show_modal = false;
        return;
    }

    let mut open = app.inventory.show_modal;
    let screen = ctx.screen_rect();
    let win_w = (screen.width() - 28.0).clamp(300.0, 680.0);
    let win_h = (screen.height() - 64.0).clamp(320.0, 520.0);

    let mut open_folder_action: Option<(String, String)> = None;
    let mut jump_crumb_idx: Option<usize> = None;
    let mut open_diagram_id: Option<String> = None;
    let mut create_folder_name: Option<String> = None;
    let mut save_here_name: Option<String> = None;
    let mut rename_commit: Option<(String, String)> = None;
    let mut move_up_item: Option<(String, String, String)> = None;
    let mut delete_item_id: Option<String> = None;
    let mut refresh_now = false;

    egui::Window::new("Google Drive Inventory — Bohat Draw.io")
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .collapsible(false)
        .resizable(true)
        .default_size(Vec2::new(win_w, win_h))
        .open(&mut open)
        .show(ctx, |ui| {
            ui.horizontal_wrapped(|ui| {
                if app.inventory.folder_stack.len() > 1 && ui.small_button("<- Up").clicked() {
                    jump_crumb_idx = Some(app.inventory.folder_stack.len() - 2);
                }
                let crumbs = app.inventory.folder_stack.clone();
                for (idx, (_, name)) in crumbs.iter().enumerate() {
                    if idx > 0 {
                        ui.label(RichText::new("/").weak());
                    }
                    let is_last = idx + 1 == crumbs.len();
                    if ui
                        .selectable_label(is_last, RichText::new(name).strong().size(11.5))
                        .clicked()
                        && !is_last
                    {
                        jump_crumb_idx = Some(idx);
                    }
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.small_button("Refresh").clicked() {
                        refresh_now = true;
                    }
                    let view_lbl = if app.inventory.list_view { "Icons" } else { "List" };
                    if ui.small_button(view_lbl).clicked() {
                        app.inventory.list_view = !app.inventory.list_view;
                    }
                });
            });
            ui.separator();

            ui.horizontal_wrapped(|ui| {
                ui.add(
                    egui::TextEdit::singleline(&mut app.inventory.new_folder_name)
                        .hint_text("New folder name...")
                        .desired_width(130.0),
                );
                if ui.button("+ Folder").clicked() && !app.inventory.new_folder_name.trim().is_empty()
                {
                    create_folder_name = Some(app.inventory.new_folder_name.trim().to_string());
                    app.inventory.new_folder_name.clear();
                }
                ui.separator();
                ui.add(
                    egui::TextEdit::singleline(&mut app.inventory.save_as_name)
                        .hint_text("Diagram name...")
                        .desired_width(140.0),
                );
                if ui
                    .add(
                        egui::Button::new(
                            RichText::new("Save Current Here").color(Color32::WHITE),
                        )
                        .fill(Color32::from_rgb(0, 0, 128)),
                    )
                    .clicked()
                    && !app.inventory.save_as_name.trim().is_empty()
                {
                    save_here_name = Some(app.inventory.save_as_name.trim().to_string());
                }
            });

            ui.horizontal(|ui| {
                ui.add(
                    egui::TextEdit::singleline(&mut app.inventory.search)
                        .hint_text("Search in this folder...")
                        .desired_width((ui.available_width() - 90.0).max(120.0)),
                );
                if app.inventory.loading {
                    ui.label(RichText::new("Syncing Drive...").small().color(Color32::from_rgb(0, 0, 140)));
                }
            });

            if let Some(err) = &app.inventory.last_error {
                ui.label(RichText::new(err).small().color(Color32::from_rgb(200, 30, 30)));
            }

            if let Some((rid, mut rname)) = app.inventory.renaming_item.clone() {
                let mut cancel_rename = false;
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Rename:").strong());
                    ui.add(egui::TextEdit::singleline(&mut rname).desired_width(180.0));
                    if ui.small_button("Save").clicked() {
                        rename_commit = Some((rid.clone(), rname.clone()));
                        cancel_rename = true;
                    }
                    if ui.small_button("Cancel").clicked() {
                        cancel_rename = true;
                    }
                });
                if cancel_rename {
                    app.inventory.renaming_item = None;
                } else {
                    app.inventory.renaming_item = Some((rid, rname));
                }
            }

            ui.separator();

            let q = app.inventory.search.trim().to_lowercase();
            let filtered: Vec<_> = app
                .inventory
                .items
                .iter()
                .filter(|it| q.is_empty() || it.name.to_lowercase().contains(&q))
                .cloned()
                .collect();

            let parent_above: Option<(String, String)> = if app.inventory.folder_stack.len() >= 2 {
                let cur_id = app.inventory.folder_stack.last().unwrap().0.clone();
                let up_id = app.inventory.folder_stack[app.inventory.folder_stack.len() - 2]
                    .0
                    .clone();
                Some((cur_id, up_id))
            } else {
                None
            };

            ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    if filtered.is_empty() && !app.inventory.loading {
                        ui.add_space(24.0);
                        ui.vertical_centered(|ui| {
                            ui.label(
                                RichText::new("This folder is empty. Create a folder or click 'Save Current Here'.")
                                    .weak(),
                            );
                        });
                        return;
                    }

                    if app.inventory.list_view {
                        for item in &filtered {
                            let disp_name = if item.is_folder {
                                item.name.clone()
                            } else {
                                item.name
                                    .strip_suffix(".bohat.json")
                                    .unwrap_or(&item.name)
                                    .to_string()
                            };
                            ui.horizontal(|ui| {
                                let (icon_r, _) =
                                    ui.allocate_exact_size(Vec2::new(22.0, 20.0), Sense::hover());
                                if item.is_folder {
                                    draw_drive_folder_icon(ui.painter(), icon_r);
                                } else {
                                    draw_drive_diagram_icon(ui.painter(), icon_r);
                                }
                                let resp = ui.selectable_label(false, &disp_name);
                                if resp.clicked() || resp.double_clicked() {
                                    if item.is_folder {
                                        open_folder_action = Some((item.id.clone(), item.name.clone()));
                                    } else {
                                        open_diagram_id = Some(item.id.clone());
                                    }
                                }
                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Center),
                                    |ui| {
                                        if ui.small_button("Trash").clicked() {
                                            delete_item_id = Some(item.id.clone());
                                        }
                                        if ui.small_button("Rename").clicked() {
                                            app.inventory.renaming_item =
                                                Some((item.id.clone(), item.name.clone()));
                                        }
                                        if let Some((ref cur_id, ref up_id)) = parent_above {
                                            if ui.small_button("Move Up").clicked() {
                                                move_up_item = Some((
                                                    item.id.clone(),
                                                    cur_id.clone(),
                                                    up_id.clone(),
                                                ));
                                            }
                                        }
                                    },
                                );
                            });
                            ui.separator();
                        }
                    } else {
                        let avail_w = ui.available_width().max(220.0);
                        let cols = ((avail_w / 128.0).floor() as usize).clamp(2, 5);
                        let card_w = ((avail_w - (cols as f32 - 1.0) * 8.0) / cols as f32)
                            .floor()
                            .max(96.0);
                        for chunk in filtered.chunks(cols) {
                            ui.horizontal(|ui| {
                                ui.spacing_mut().item_spacing.x = 8.0;
                                for item in chunk {
                                    let disp_name = if item.is_folder {
                                        item.name.clone()
                                    } else {
                                        item.name
                                            .strip_suffix(".bohat.json")
                                            .unwrap_or(&item.name)
                                            .to_string()
                                    };
                                    Frame::new()
                                        .fill(Color32::WHITE)
                                        .stroke(Stroke::new(1.0_f32, Color32::from_rgb(174, 180, 198)))
                                        .corner_radius(3)
                                        .inner_margin(Margin::same(6))
                                        .show(ui, |ui| {
                                            ui.set_width(card_w - 14.0);
                                            ui.vertical_centered(|ui| {
                                                let (ir, resp) = ui.allocate_exact_size(
                                                    Vec2::new(40.0, 34.0),
                                                    Sense::click(),
                                                );
                                                if item.is_folder {
                                                    draw_drive_folder_icon(ui.painter(), ir);
                                                } else {
                                                    draw_drive_diagram_icon(ui.painter(), ir);
                                                }
                                                let short: String = if disp_name.chars().count() > 14 {
                                                    format!(
                                                        "{}..",
                                                        disp_name.chars().take(12).collect::<String>()
                                                    )
                                                } else {
                                                    disp_name.clone()
                                                };
                                                let lbl_resp = ui.selectable_label(false, short);
                                                if resp.clicked() || lbl_resp.clicked() {
                                                    if item.is_folder {
                                                        open_folder_action =
                                                            Some((item.id.clone(), item.name.clone()));
                                                    } else {
                                                        open_diagram_id = Some(item.id.clone());
                                                    }
                                                }
                                                ui.horizontal(|ui| {
                                                    if ui.small_button("Ren").on_hover_text("Rename").clicked() {
                                                        app.inventory.renaming_item =
                                                            Some((item.id.clone(), item.name.clone()));
                                                    }
                                                    if let Some((ref cur_id, ref up_id)) = parent_above {
                                                        if ui.small_button("Up").on_hover_text("Move to parent folder").clicked() {
                                                            move_up_item = Some((
                                                                item.id.clone(),
                                                                cur_id.clone(),
                                                                up_id.clone(),
                                                            ));
                                                        }
                                                    }
                                                    if ui.small_button("x").on_hover_text("Move to Trash").clicked() {
                                                        delete_item_id = Some(item.id.clone());
                                                    }
                                                });
                                            });
                                        });
                                }
                            });
                            ui.add_space(6.0);
                        }
                    }
                });
        });

    app.inventory.show_modal = open;

    if let Some((fid, fname)) = open_folder_action {
        app.inventory.folder_stack.push((fid.clone(), fname));
        app.inventory.fetch_folder(Some(fid), ctx.clone());
    } else if let Some(idx) = jump_crumb_idx {
        app.inventory.folder_stack.truncate(idx + 1);
        let fid = app.inventory.current_folder_id();
        app.inventory.fetch_folder(fid, ctx.clone());
    } else if let Some(did) = open_diagram_id {
        app.inventory.load_diagram(did, ctx.clone());
    } else if let Some(fname) = create_folder_name {
        app.inventory.create_folder(fname, ctx.clone());
    } else if let Some(dname) = save_here_name {
        let cur_fid = app.inventory.current_folder_id();
        app.save_current_to_drive(None, cur_fid, dname, ctx);
    } else if let Some((id, new_name)) = rename_commit {
        app.inventory.rename_item(id, new_name, ctx.clone());
    } else if let Some((id, old_p, new_p)) = move_up_item {
        app.inventory.move_item_to_folder(id, old_p, new_p, ctx.clone());
    } else if let Some(id) = delete_item_id {
        app.inventory.delete_item(id, ctx.clone());
    } else if refresh_now {
        let fid = app.inventory.current_folder_id();
        app.inventory.fetch_folder(fid, ctx.clone());
    }
}
