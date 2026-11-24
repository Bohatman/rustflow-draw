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
                            ui.menu_button("New from Template", |ui| {
                                if ui.button("AWS 3-Tier Web App").clicked() {
                                    app.load_starter_template(
                                        "AWS 3-Tier Web App",
                                        "flowchart LR\n  U((Users)) -->|HTTPS| CF[CloudFront CDN]\n  CF --> ALB[Application LB]\n  subgraph VPC Public & App\n    ALB --> ECS1(ECS Service A)\n    ALB --> ECS2(ECS Service B)\n  end\n  subgraph Data Tier\n    ECS1 --> RDS[(RDS Postgres)]\n    ECS2 --> RDS\n    ECS1 -.->|Cache| RED[(ElastiCache Redis)]\n  end\n",
                                    );
                                    ui.close_menu();
                                }
                                if ui.button("Event-Driven Microservices").clicked() {
                                    app.load_starter_template(
                                        "Event-Driven Microservices",
                                        "flowchart LR\n  C((Client)) -->|REST| APIGW[API Gateway]\n  APIGW --> AUTH(Auth Lambda)\n  APIGW --> ORD(Order Service)\n  ORD -.->|OrderCreated| MQ[SQS / Kafka]\n  MQ -.-> BILL(Billing Worker)\n  MQ -.-> NOTIF(Notify Worker)\n  ORD --> DB[(DynamoDB)]\n",
                                    );
                                    ui.close_menu();
                                }
                                if ui.button("Kubernetes Cluster").clicked() {
                                    app.load_starter_template(
                                        "Kubernetes Cluster",
                                        "flowchart LR\n  IN((Internet)) --> ING[Ingress Controller]\n  subgraph K8s Cluster\n    ING --> FE[Web Pods]\n    FE --> API[API Pods]\n    API --> WORK[Worker Pods]\n  end\n  API --> PG[(Managed DB)]\n  WORK -.-> S3[(Object Storage)]\n",
                                    );
                                    ui.close_menu();
                                }
                                if ui.button("CI/CD Pipeline").clicked() {
                                    app.load_starter_template(
                                        "CI/CD Pipeline",
                                        "flowchart LR\n  DEV((Developer)) -->|git push| GH[GitHub Repo]\n  GH --> CI[Build & Test]\n  CI --> SEC{Security Scan}\n  SEC -->|Pass| REG[(Container Registry)]\n  REG --> STG[Staging]\n  STG -->|Approve| PROD[Production]\n",
                                    );
                                    ui.close_menu();
                                }
                            });
                            if ui.button("Import / Export Mermaid...").clicked() {
                                app.open_mermaid_modal();
                                ui.close_menu();
                            }
                            if ui.button("Open Local (.json / .drawio)...").clicked() {
                                app.open_file(ctx);
                                ui.close_menu();
                            }
                            if ui.button("Open from Google Drive...").clicked() {
                                if app.inventory.user.is_some() {
                                    app.inventory.show_modal = true;
                                    let fid = app.inventory.current_folder_id();
                                    app.inventory.fetch_folder(fid, ctx.clone());
                                } else {
                                    app.inventory.start_google_login(ctx.clone());
                                }
                                ui.close_menu();
                            }
                            ui.separator();
                            if ui.button("Save to Google Drive (Ctrl+S)").clicked() {
                                if app.inventory.user.is_some() {
                                    app.quick_save(ctx);
                                } else {
                                    app.inventory.start_google_login(ctx.clone());
                                }
                                ui.close_menu();
                            }
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
                                .on_hover_text("Save to Google Drive (Ctrl+S — updates current file)")
                                .clicked()
                            {
                                app.quick_save(ctx);
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
                                ui.checkbox(
                                    &mut app.autosave_to_drive,
                                    "Auto-Save to Google Drive",
                                );
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
                            let (status_txt, status_col) = if app.is_dirty {
                                ("● Unsaved", Color32::from_rgb(217, 119, 6))
                            } else {
                                ("✓ Saved", Color32::from_rgb(22, 163, 74))
                            };
                            ui.label(RichText::new(status_txt).size(10.5).strong().color(status_col))
                                .on_hover_text(if app.is_dirty {
                                    "Unsaved changes — press Ctrl+S to save (local crash recovery is active)"
                                } else {
                                    "All changes saved"
                                });
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
                if icon_button(ui, ToolIcon::OpenFile, Some("Open"), "Open Local .json / .drawio", true, false).clicked() {
                    app.open_file(ctx);
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

    if app.recovered_draft.is_some() {
        egui::TopBottomPanel::top("draft_recovery_banner")
            .frame(
                Frame::new()
                    .fill(Color32::from_rgb(254, 249, 195))
                    .stroke(Stroke::new(1.0_f32, Color32::from_rgb(234, 179, 8)))
                    .inner_margin(Margin::symmetric(12, 5)),
            )
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new("Recovered an unsaved local draft from your last session.")
                            .size(11.5)
                            .strong()
                            .color(Color32::from_rgb(113, 63, 18)),
                    );
                    if ui
                        .add(
                            egui::Button::new(
                                RichText::new("Restore Draft").size(11.0).color(Color32::WHITE),
                            )
                            .fill(Color32::from_rgb(37, 99, 235)),
                        )
                        .clicked()
                    {
                        app.restore_recovered_draft();
                    }
                    if ui.small_button("Dismiss").clicked() {
                        app.dismiss_recovered_draft();
                    }
                });
            });
    }
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
                    let page_count = app.bridge.document.pages().len();
                    resp.context_menu(|ui| {
                        if ui.button("Rename Page...").clicked() {
                            app.renaming_page = Some((pid, pname.clone()));
                            ui.close_menu();
                        }
                        if ui.button("Duplicate Page").clicked() {
                            app.bridge.sync_active_page_to_document();
                            let cloned_scene = app
                                .bridge
                                .document
                                .pages()
                                .iter()
                                .find(|p| p.id == pid)
                                .map(|p| p.scene.clone());
                            if let Some(sc) = cloned_scene {
                                let new_id = app.bridge.add_page(format!("{pname} Copy"));
                                app.bridge.switch_page(new_id);
                                app.bridge.scene = sc;
                                app.bridge.sync_active_page_to_document();
                                app.mark_dirty();
                            }
                            ui.close_menu();
                        }
                        if page_count > 1 && ui.button("Delete Page").clicked() {
                            app.bridge.sync_active_page_to_document();
                            if app.bridge.document.remove_page(pid) {
                                app.bridge.load_active_page_from_document();
                                app.mark_dirty();
                            }
                            ui.close_menu();
                        }
                    });
                }
                if let Some((rpid, mut draft_name)) = app.renaming_page.clone() {
                    let mut done = false;
                    let mut cancel = false;
                    egui::Window::new("Rename Page")
                        .collapsible(false)
                        .resizable(false)
                        .anchor(Align2::CENTER_CENTER, [0.0, 0.0])
                        .show(ctx, |ui| {
                            let ed = ui.text_edit_singleline(&mut draft_name);
                            if ed.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                                done = true;
                            }
                            ui.horizontal(|ui| {
                                if ui.button("Save").clicked() {
                                    done = true;
                                }
                                if ui.button("Cancel").clicked() {
                                    cancel = true;
                                }
                            });
                        });
                    if done {
                        let clean = draft_name.trim();
                        if !clean.is_empty() {
                            app.bridge.document.rename_page(rpid, clean.to_string());
                            app.mark_dirty();
                        }
                        app.renaming_page = None;
                    } else if cancel {
                        app.renaming_page = None;
                    } else {
                        app.renaming_page = Some((rpid, draft_name));
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

