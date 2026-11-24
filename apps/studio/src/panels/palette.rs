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
                            let search_w = (inner_w - 102.0 - clear_w).max(52.0);
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
                                    [50.0, 22.0],
                                    egui::Button::new(RichText::new("+ New").color(Color32::WHITE).size(11.0))
                                        .fill(Color32::from_rgb(0, 0, 128)),
                                )
                                .on_hover_text("Open Symbol Studio")
                                .clicked()
                            {
                                app.show_symbol_studio = true;
                            }
                            let burger_id = ui.make_persistent_id("palette_view_burger_menu");
                            let (brect, bresp) =
                                ui.allocate_exact_size(Vec2::new(24.0, 22.0), Sense::click());
                            let b_open = ui.memory(|m| m.is_popup_open(burger_id));
                            let b_bg = if b_open {
                                Color32::from_rgb(0, 0, 128)
                            } else if bresp.hovered() {
                                Color32::from_rgb(232, 238, 252)
                            } else {
                                Color32::from_rgb(240, 241, 246)
                            };
                            let b_fg = if b_open {
                                Color32::WHITE
                            } else {
                                Color32::from_rgb(24, 28, 42)
                            };
                            ui.painter().rect_filled(brect, 3.0, b_bg);
                            ui.painter().rect_stroke(
                                brect,
                                3.0,
                                Stroke::new(1.0_f32, Color32::from_rgb(148, 152, 168)),
                                egui::StrokeKind::Inside,
                            );
                            let cx = brect.center().x;
                            let cy = brect.center().y;
                            for dy in [-4.0_f32, 0.0, 4.0] {
                                ui.painter().line_segment(
                                    [egui::Pos2::new(cx - 5.5, cy + dy), egui::Pos2::new(cx + 5.5, cy + dy)],
                                    Stroke::new(1.6_f32, b_fg),
                                );
                            }
                            let bresp = bresp.on_hover_text("Symbol View Mode");
                            if bresp.clicked() {
                                ui.memory_mut(|m| m.toggle_popup(burger_id));
                            }
                            egui::popup_below_widget(
                                ui,
                                burger_id,
                                &bresp,
                                egui::PopupCloseBehavior::CloseOnClick,
                                |ui| {
                                    ui.set_min_width(126.0);
                                    ui.label(RichText::new("View Mode").small().strong().weak());
                                    ui.separator();
                                    for (mode, lbl) in [
                                        (PaletteViewMode::IconOnly, "Icon"),
                                        (PaletteViewMode::IconAndName, "Icon + Name"),
                                        (PaletteViewMode::NameList, "Name"),
                                    ] {
                                        let sel = app.palette_view_mode == mode;
                                        if ui.selectable_label(sel, lbl).clicked() {
                                            app.palette_view_mode = mode;
                                        }
                                    }
                                },
                            );
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
                        ui.add_space(2.0);
                        ui.separator();
                        ui.add_space(2.0);

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

                            let row_spacing = 4.0_f32;
                            let header_h = 22.0_f32;
                            let mut row_tops: Vec<f32> = Vec::with_capacity(rows.len() + 1);
                            let mut y_acc = 0.0_f32;
                            for r in &rows {
                                row_tops.push(y_acc);
                                let rh = match r {
                                    PalRow::Header(..) => header_h,
                                    PalRow::Custom(..) | PalRow::Items(..) => tile_h,
                                };
                                y_acc += rh + row_spacing;
                            }
                            row_tops.push(y_acc);

                            ScrollArea::vertical()
                                .drag_to_scroll(true)
                                .auto_shrink([false, false])
                                .show_viewport(ui, |ui, viewport| {
                                    ui.spacing_mut().item_spacing.y = row_spacing;
                                    let vis_min = (viewport.min.y - 80.0).max(0.0);
                                    let vis_max = viewport.max.y + 80.0;
                                    let first = row_tops
                                        .partition_point(|&y| y < vis_min)
                                        .saturating_sub(1)
                                        .min(rows.len());
                                    let last = row_tops
                                        .partition_point(|&y| y <= vis_max)
                                        .min(rows.len());

                                    if first > 0 {
                                        ui.add_space((row_tops[first] - row_spacing).max(0.0));
                                    }
                                    for ridx in first..last {
                                        match rows[ridx] {
                                            PalRow::Header(name, cnt, accent_opt) => {
                                                let (bar, _) = ui.allocate_exact_size(
                                                    Vec2::new(avail_w, header_h),
                                                    Sense::hover(),
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
                                    if last < rows.len() {
                                        ui.add_space((y_acc - row_tops[last]).max(0.0));
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

