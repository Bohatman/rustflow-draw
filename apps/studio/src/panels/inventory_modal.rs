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


fn draw_drive_folder_icon(painter: &egui::Painter, rect: egui::Rect) {
    let tab = egui::Rect::from_min_max(
        egui::Pos2::new(rect.min.x + 2.0, rect.min.y + 3.0),
        egui::Pos2::new(rect.min.x + rect.width() * 0.46, rect.min.y + rect.height() * 0.34),
    );
    let back = egui::Rect::from_min_max(
        egui::Pos2::new(rect.min.x + 2.0, rect.min.y + rect.height() * 0.16),
        egui::Pos2::new(rect.max.x - 2.0, rect.max.y - 3.0),
    );
    let front = egui::Rect::from_min_max(
        egui::Pos2::new(rect.min.x + 2.0, rect.min.y + rect.height() * 0.28),
        egui::Pos2::new(rect.max.x - 2.0, rect.max.y - 3.0),
    );
    painter.rect_filled(
        front.translate(Vec2::new(0.0, 1.5)),
        4.0,
        Color32::from_rgba_unmultiplied(15, 23, 42, 26),
    );
    painter.rect_filled(tab, 3.0, Color32::from_rgb(56, 152, 236));
    painter.rect_filled(back, 3.5, Color32::from_rgb(56, 152, 236));
    painter.rect_filled(front, 4.0, Color32::from_rgb(90, 186, 250));
    painter.line_segment(
        [
            egui::Pos2::new(front.min.x + 3.0, front.min.y + 1.0),
            egui::Pos2::new(front.max.x - 3.0, front.min.y + 1.0),
        ],
        Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(255, 255, 255, 130)),
    );
    painter.rect_stroke(
        front,
        4.0,
        Stroke::new(0.8_f32, Color32::from_rgb(42, 132, 214)),
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
    let is_compact = screen.width() <= 640.0;
    let win_w = (screen.width() - 28.0).clamp(300.0, 820.0);
    let win_h = (screen.height() - 60.0).clamp(340.0, 560.0);

    let mut open_folder_action: Option<(String, String)> = None;
    let mut jump_crumb_idx: Option<usize> = None;
    let mut open_diagram_id: Option<String> = None;
    let mut create_folder_name: Option<String> = None;
    let mut save_here_name: Option<String> = None;
    let mut rename_commit: Option<(String, String)> = None;
    let mut move_up_item: Option<(String, String, String)> = None;
    let mut delete_item_id: Option<String> = None;
    let mut refresh_now = false;
    let mut open_picker_now = false;

    let accent_blue = Color32::from_rgb(0, 122, 255);
    let sidebar_bg = Color32::from_rgb(236, 238, 243);
    let toolbar_bg = Color32::from_rgb(246, 247, 250);
    let divider_col = Color32::from_rgb(214, 218, 226);
    let text_primary = Color32::from_rgb(28, 30, 36);
    let text_muted = Color32::from_rgb(112, 118, 132);

    egui::Window::new("Drive Inventory")
        .id(egui::Id::new("finder_drive_inventory_window"))
        .title_bar(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .collapsible(false)
        .resizable(true)
        .default_size(Vec2::new(win_w, win_h))
        .frame(
            Frame::new()
                .fill(Color32::WHITE)
                .stroke(Stroke::new(1.0_f32, Color32::from_rgb(196, 200, 212)))
                .corner_radius(11)
                .shadow(egui::Shadow {
                    offset: [0, 10],
                    blur: 28,
                    spread: 2,
                    color: Color32::from_rgba_unmultiplied(15, 23, 42, 62),
                })
                .inner_margin(Margin::same(0)),
        )
        .show(ctx, |ui| {
            let total_size = Vec2::new(win_w, win_h);
            ui.set_min_size(total_size);
            ui.set_max_size(total_size);

            let sidebar_w = if is_compact { 0.0 } else { 176.0 };
            let main_w = (win_w - sidebar_w).max(260.0);

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
            let current_folder_title = app
                .inventory
                .folder_stack
                .last()
                .map(|(_, n)| n.clone())
                .unwrap_or_else(|| app.inventory.root_folder_name.clone());

            let mut preview_requests: Vec<(String, String)> = Vec::new();

            ui.horizontal_top(|ui| {
                ui.spacing_mut().item_spacing = Vec2::ZERO;

                if !is_compact {
                    Frame::new()
                        .fill(sidebar_bg)
                        .corner_radius(egui::CornerRadius {
                            nw: 11,
                            sw: 11,
                            ne: 0,
                            se: 0,
                        })
                        .inner_margin(Margin::same(10))
                        .show(ui, |ui| {
                            ui.set_width(sidebar_w - 20.0);
                            ui.set_min_height(win_h - 20.0);
                            ui.vertical(|ui| {
                                ui.horizontal(|ui| {
                                    ui.spacing_mut().item_spacing.x = 7.0;
                                    let (cr, cresp) =
                                        ui.allocate_exact_size(Vec2::splat(12.0), Sense::click());
                                    ui.painter().circle_filled(
                                        cr.center(),
                                        5.8,
                                        Color32::from_rgb(255, 95, 87),
                                    );
                                    if cresp.hovered() {
                                        let c = cr.center();
                                        ui.painter().line_segment(
                                            [egui::Pos2::new(c.x - 2.4, c.y - 2.4), egui::Pos2::new(c.x + 2.4, c.y + 2.4)],
                                            Stroke::new(1.2_f32, Color32::from_rgb(120, 18, 12)),
                                        );
                                        ui.painter().line_segment(
                                            [egui::Pos2::new(c.x + 2.4, c.y - 2.4), egui::Pos2::new(c.x - 2.4, c.y + 2.4)],
                                            Stroke::new(1.2_f32, Color32::from_rgb(120, 18, 12)),
                                        );
                                    }
                                    if cresp.on_hover_text("Close").clicked() {
                                        open = false;
                                    }
                                    let (yr, _) =
                                        ui.allocate_exact_size(Vec2::splat(12.0), Sense::hover());
                                    ui.painter().circle_filled(
                                        yr.center(),
                                        5.8,
                                        Color32::from_rgb(254, 188, 46),
                                    );
                                    let (gr, _) =
                                        ui.allocate_exact_size(Vec2::splat(12.0), Sense::hover());
                                    ui.painter().circle_filled(
                                        gr.center(),
                                        5.8,
                                        Color32::from_rgb(40, 200, 64),
                                    );
                                });
                                ui.add_space(14.0);

                                ui.label(
                                    RichText::new("LOCATIONS")
                                        .size(9.5)
                                        .strong()
                                        .color(text_muted),
                                );
                                ui.add_space(4.0);
                                let at_root = app.inventory.folder_stack.len() <= 1;
                                let root_btn = ui.add_sized(
                                    [sidebar_w - 22.0, 26.0],
                                    egui::Button::new(
                                        RichText::new(format!("  {}", app.inventory.root_folder_name))
                                            .size(11.5)
                                            .color(if at_root { Color32::WHITE } else { text_primary }),
                                    )
                                    .fill(if at_root { accent_blue } else { Color32::TRANSPARENT })
                                    .corner_radius(6)
                                    .stroke(Stroke::NONE),
                                );
                                if root_btn.clicked() && !at_root {
                                    jump_crumb_idx = Some(0);
                                }
                                if ui
                                    .add_sized(
                                        [sidebar_w - 22.0, 24.0],
                                        egui::Button::new(
                                            RichText::new("  Change Drive Folder...")
                                                .size(11.0)
                                                .color(text_primary),
                                        )
                                        .fill(Color32::TRANSPARENT)
                                        .corner_radius(6)
                                        .stroke(Stroke::NONE),
                                    )
                                    .clicked()
                                {
                                    open_picker_now = true;
                                }

                                ui.add_space(12.0);
                                ui.label(
                                    RichText::new("ACTIONS")
                                        .size(9.5)
                                        .strong()
                                        .color(text_muted),
                                );
                                ui.add_space(4.0);
                                if ui
                                    .add_sized(
                                        [sidebar_w - 22.0, 25.0],
                                        egui::Button::new(
                                            RichText::new("  + New Folder")
                                                .size(11.0)
                                                .color(text_primary),
                                        )
                                        .fill(if app.inventory.show_new_folder_bar {
                                            Color32::from_rgb(218, 224, 236)
                                        } else {
                                            Color32::TRANSPARENT
                                        })
                                        .corner_radius(6)
                                        .stroke(Stroke::NONE),
                                    )
                                    .clicked()
                                {
                                    app.inventory.show_new_folder_bar =
                                        !app.inventory.show_new_folder_bar;
                                }
                                if ui
                                    .add_sized(
                                        [sidebar_w - 22.0, 25.0],
                                        egui::Button::new(
                                            RichText::new("  Refresh Drive")
                                                .size(11.0)
                                                .color(text_primary),
                                        )
                                        .fill(Color32::TRANSPARENT)
                                        .corner_radius(6)
                                        .stroke(Stroke::NONE),
                                    )
                                    .clicked()
                                {
                                    refresh_now = true;
                                }

                                if let Some(ref u) = app.inventory.user {
                                    ui.add_space(16.0);
                                    ui.separator();
                                    ui.add_space(6.0);
                                    ui.label(
                                        RichText::new("ACCOUNT")
                                            .size(9.5)
                                            .strong()
                                            .color(text_muted),
                                    );
                                    ui.label(RichText::new(&u.name).size(11.0).strong().color(text_primary));
                                    ui.label(RichText::new(&u.email).size(9.5).color(text_muted));
                                }
                            });
                        });
                    let (div_r, _) =
                        ui.allocate_exact_size(Vec2::new(1.0, win_h), Sense::hover());
                    ui.painter().rect_filled(div_r, 0.0, divider_col);
                }

                ui.vertical(|ui| {
                    ui.set_width(main_w - if is_compact { 0.0 } else { 1.0 });
                    ui.set_min_height(win_h);

                    Frame::new()
                        .fill(toolbar_bg)
                        .corner_radius(egui::CornerRadius {
                            nw: if is_compact { 11 } else { 0 },
                            ne: 11,
                            sw: 0,
                            se: 0,
                        })
                        .inner_margin(Margin::symmetric(12, 8))
                        .show(ui, |ui| {
                            ui.set_width((main_w - 26.0).max(220.0));
                            ui.horizontal(|ui| {
                                ui.spacing_mut().item_spacing.x = 6.0;
                                if is_compact {
                                    let (cr, cresp) =
                                        ui.allocate_exact_size(Vec2::splat(13.0), Sense::click());
                                    ui.painter().circle_filled(
                                        cr.center(),
                                        6.0,
                                        Color32::from_rgb(255, 95, 87),
                                    );
                                    if cresp.clicked() {
                                        open = false;
                                    }
                                }
                                let can_up = app.inventory.folder_stack.len() > 1;
                                if ui
                                    .add_enabled(
                                        can_up,
                                        egui::Button::new(RichText::new("<").strong().size(12.0))
                                            .fill(Color32::WHITE)
                                            .corner_radius(6)
                                            .stroke(Stroke::new(1.0_f32, divider_col)),
                                    )
                                    .on_hover_text("Back to parent folder")
                                    .clicked()
                                {
                                    jump_crumb_idx = Some(app.inventory.folder_stack.len() - 2);
                                }
                                ui.label(
                                    RichText::new(&current_folder_title)
                                        .strong()
                                        .size(13.5)
                                        .color(text_primary),
                                );

                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Center),
                                    |ui| {
                                        ui.spacing_mut().item_spacing.x = 6.0;
                                        if is_compact
                                            && ui.small_button("Folder...").clicked()
                                        {
                                            open_picker_now = true;
                                        }
                                        ui.add(
                                            egui::TextEdit::singleline(&mut app.inventory.search)
                                                .hint_text("Search...")
                                                .desired_width(if is_compact { 84.0 } else { 130.0 }),
                                        );
                                        Frame::new()
                                            .fill(Color32::from_rgb(228, 231, 238))
                                            .corner_radius(6)
                                            .inner_margin(Margin::same(2))
                                            .show(ui, |ui| {
                                                ui.horizontal(|ui| {
                                                    ui.spacing_mut().item_spacing.x = 2.0;
                                                    let list_sel = app.inventory.list_view;
                                                    if ui
                                                        .add(
                                                            egui::Button::new(
                                                                RichText::new("List")
                                                                    .size(10.5)
                                                                    .color(text_primary),
                                                            )
                                                            .fill(if list_sel {
                                                                Color32::WHITE
                                                            } else {
                                                                Color32::TRANSPARENT
                                                            })
                                                            .corner_radius(5)
                                                            .stroke(Stroke::NONE),
                                                        )
                                                        .clicked()
                                                    {
                                                        app.inventory.list_view = true;
                                                    }
                                                    if ui
                                                        .add(
                                                            egui::Button::new(
                                                                RichText::new("Icons")
                                                                    .size(10.5)
                                                                    .color(text_primary),
                                                            )
                                                            .fill(if !list_sel {
                                                                Color32::WHITE
                                                            } else {
                                                                Color32::TRANSPARENT
                                                            })
                                                            .corner_radius(5)
                                                            .stroke(Stroke::NONE),
                                                        )
                                                        .clicked()
                                                    {
                                                        app.inventory.list_view = false;
                                                    }
                                                });
                                            });
                                    },
                                );
                            });
                        });

                    let (hdiv1, _) = ui.allocate_exact_size(
                        Vec2::new(main_w - if is_compact { 0.0 } else { 1.0 }, 1.0),
                        Sense::hover(),
                    );
                    ui.painter().rect_filled(hdiv1, 0.0, divider_col);

                    Frame::new()
                        .fill(Color32::from_rgb(250, 251, 253))
                        .inner_margin(Margin::symmetric(12, 6))
                        .show(ui, |ui| {
                            ui.set_width((main_w - 26.0).max(220.0));
                            ui.horizontal_wrapped(|ui| {
                                ui.spacing_mut().item_spacing.x = 6.0;
                                ui.add(
                                    egui::TextEdit::singleline(&mut app.inventory.save_as_name)
                                        .hint_text("Diagram name...")
                                        .desired_width(if is_compact { 120.0 } else { 170.0 }),
                                );
                                if ui
                                    .add(
                                        egui::Button::new(
                                            RichText::new("Save Here")
                                                .size(11.0)
                                                .color(Color32::WHITE),
                                        )
                                        .fill(accent_blue)
                                        .corner_radius(6)
                                        .stroke(Stroke::NONE),
                                    )
                                    .clicked()
                                    && !app.inventory.save_as_name.trim().is_empty()
                                {
                                    save_here_name = Some(app.inventory.save_as_name.trim().to_string());
                                }
                                if is_compact && ui.small_button("+ Folder").clicked() {
                                    app.inventory.show_new_folder_bar =
                                        !app.inventory.show_new_folder_bar;
                                }
                                if let Some(sel_id) = app.inventory.selected_item_id.clone() {
                                    if let Some(sel_item) =
                                        filtered.iter().find(|it| it.id == sel_id)
                                    {
                                        ui.separator();
                                        if ui.small_button("Open").clicked() {
                                            if sel_item.is_folder {
                                                open_folder_action = Some((
                                                    sel_item.id.clone(),
                                                    sel_item.name.clone(),
                                                ));
                                            } else {
                                                open_diagram_id = Some(sel_item.id.clone());
                                            }
                                        }
                                        if ui.small_button("Rename").clicked() {
                                            app.inventory.renaming_item = Some((
                                                sel_item.id.clone(),
                                                sel_item.name.clone(),
                                            ));
                                        }
                                        if let Some((ref cur_id, ref up_id)) = parent_above {
                                            if ui.small_button("Move Up").clicked() {
                                                move_up_item = Some((
                                                    sel_item.id.clone(),
                                                    cur_id.clone(),
                                                    up_id.clone(),
                                                ));
                                            }
                                        }
                                        if ui
                                            .small_button(
                                                RichText::new("Trash")
                                                    .color(Color32::from_rgb(210, 40, 40)),
                                            )
                                            .clicked()
                                        {
                                            delete_item_id = Some(sel_item.id.clone());
                                        }
                                    }
                                }
                                if app.inventory.loading {
                                    ui.label(
                                        RichText::new("Syncing...")
                                            .small()
                                            .color(accent_blue),
                                    );
                                }
                            });

                            if app.inventory.show_new_folder_bar {
                                ui.add_space(4.0);
                                ui.horizontal(|ui| {
                                    ui.label(RichText::new("New Folder:").size(11.0).strong());
                                    ui.add(
                                        egui::TextEdit::singleline(
                                            &mut app.inventory.new_folder_name,
                                        )
                                        .hint_text("Folder name...")
                                        .desired_width(160.0),
                                    );
                                    if ui
                                        .add(
                                            egui::Button::new(
                                                RichText::new("Create")
                                                    .size(11.0)
                                                    .color(Color32::WHITE),
                                            )
                                            .fill(accent_blue)
                                            .corner_radius(6)
                                            .stroke(Stroke::NONE),
                                        )
                                        .clicked()
                                        && !app.inventory.new_folder_name.trim().is_empty()
                                    {
                                        create_folder_name =
                                            Some(app.inventory.new_folder_name.trim().to_string());
                                        app.inventory.new_folder_name.clear();
                                        app.inventory.show_new_folder_bar = false;
                                    }
                                    if ui.small_button("Cancel").clicked() {
                                        app.inventory.show_new_folder_bar = false;
                                    }
                                });
                            }

                            if let Some((rid, mut rname)) = app.inventory.renaming_item.clone() {
                                ui.add_space(4.0);
                                let mut cancel_rename = false;
                                ui.horizontal(|ui| {
                                    ui.label(RichText::new("Rename:").size(11.0).strong());
                                    ui.add(
                                        egui::TextEdit::singleline(&mut rname).desired_width(170.0),
                                    );
                                    if ui.small_button("Apply").clicked() {
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

                            if let Some(err) = &app.inventory.last_error {
                                ui.add_space(3.0);
                                ui.label(
                                    RichText::new(err)
                                        .small()
                                        .color(Color32::from_rgb(210, 40, 40)),
                                );
                            }
                        });

                    let (hdiv2, _) = ui.allocate_exact_size(
                        Vec2::new(main_w - if is_compact { 0.0 } else { 1.0 }, 1.0),
                        Sense::hover(),
                    );
                    ui.painter().rect_filled(hdiv2, 0.0, Color32::from_rgb(232, 235, 240));

                    let content_h = (win_h
                        - 118.0
                        - if app.inventory.show_new_folder_bar { 32.0 } else { 0.0 }
                        - if app.inventory.renaming_item.is_some() { 32.0 } else { 0.0 })
                        .max(150.0);

                    Frame::new()
                        .fill(Color32::WHITE)
                        .inner_margin(Margin::symmetric(12, 10))
                        .show(ui, |ui| {
                            ui.set_width((main_w - 26.0).max(220.0));
                            ui.set_min_height(content_h);
                            ScrollArea::vertical()
                                .max_height(content_h)
                                .auto_shrink([false, false])
                                .show(ui, |ui| {
                                    if filtered.is_empty() && !app.inventory.loading {
                                        ui.add_space(40.0);
                                        ui.vertical_centered(|ui| {
                                            ui.label(
                                                RichText::new("Empty Folder")
                                                    .size(14.0)
                                                    .strong()
                                                    .color(text_muted),
                                            );
                                            ui.add_space(4.0);
                                            ui.label(
                                                RichText::new(
                                                    "Click '+ New Folder' or 'Save Here' to add diagrams.",
                                                )
                                                .size(11.5)
                                                .color(text_muted),
                                            );
                                        });
                                        return;
                                    }

                                    if app.inventory.list_view {
                                        ui.horizontal(|ui| {
                                            ui.label(
                                                RichText::new("Name")
                                                    .size(10.0)
                                                    .strong()
                                                    .color(text_muted),
                                            );
                                            ui.with_layout(
                                                egui::Layout::right_to_left(egui::Align::Center),
                                                |ui| {
                                                    ui.label(
                                                        RichText::new("Kind")
                                                            .size(10.0)
                                                            .strong()
                                                            .color(text_muted),
                                                    );
                                                },
                                            );
                                        });
                                        ui.add_space(4.0);
                                        for (row_idx, item) in filtered.iter().enumerate() {
                                            let disp_name = if item.is_folder {
                                                item.name.clone()
                                            } else {
                                                item.name
                                                    .strip_suffix(".bohat.json")
                                                    .unwrap_or(&item.name)
                                                    .to_string()
                                            };
                                            let cache_key = format!(
                                                "{}:{}",
                                                item.id,
                                                item.modified_time.as_deref().unwrap_or("")
                                            );
                                            if !item.is_folder
                                                && !app
                                                    .inventory
                                                    .preview_textures
                                                    .contains_key(&cache_key)
                                                && !app
                                                    .inventory
                                                    .preview_requested
                                                    .contains(&cache_key)
                                            {
                                                preview_requests
                                                    .push((item.id.clone(), cache_key.clone()));
                                            }
                                            let is_sel = app.inventory.selected_item_id.as_deref()
                                                == Some(item.id.as_str());
                                            let row_bg = if is_sel {
                                                accent_blue
                                            } else if row_idx % 2 == 1 {
                                                Color32::from_rgb(247, 248, 251)
                                            } else {
                                                Color32::WHITE
                                            };
                                            let row_fg =
                                                if is_sel { Color32::WHITE } else { text_primary };
                                            let row_w = ui.available_width().max(180.0);
                                            let (rrect, rresp) = ui.allocate_exact_size(
                                                Vec2::new(row_w, 30.0),
                                                Sense::click(),
                                            );
                                            ui.painter().rect_filled(rrect, 5.0, row_bg);
                                            let icon_r = egui::Rect::from_min_size(
                                                egui::Pos2::new(rrect.min.x + 8.0, rrect.min.y + 4.0),
                                                if item.is_folder {
                                                    Vec2::new(26.0, 22.0)
                                                } else {
                                                    Vec2::new(34.0, 22.0)
                                                },
                                            );
                                            if item.is_folder {
                                                draw_drive_folder_icon(ui.painter(), icon_r);
                                            } else if let Some(tex) =
                                                app.inventory.preview_textures.get(&cache_key)
                                            {
                                                let uv = egui::Rect::from_min_max(
                                                    egui::Pos2::ZERO,
                                                    egui::Pos2::new(1.0, 1.0),
                                                );
                                                ui.painter()
                                                    .image(tex.id(), icon_r, uv, Color32::WHITE);
                                                ui.painter().rect_stroke(
                                                    icon_r,
                                                    3.0,
                                                    Stroke::new(
                                                        0.8_f32,
                                                        Color32::from_rgb(190, 196, 212),
                                                    ),
                                                    egui::StrokeKind::Inside,
                                                );
                                            } else {
                                                draw_drive_diagram_icon(ui.painter(), icon_r);
                                            }
                                            ui.painter().text(
                                                egui::Pos2::new(icon_r.max.x + 10.0, rrect.center().y),
                                                Align2::LEFT_CENTER,
                                                &disp_name,
                                                FontId::proportional(12.0),
                                                row_fg,
                                            );
                                            ui.painter().text(
                                                egui::Pos2::new(rrect.max.x - 10.0, rrect.center().y),
                                                Align2::RIGHT_CENTER,
                                                if item.is_folder { "Folder" } else { "Diagram" },
                                                FontId::proportional(10.5),
                                                if is_sel {
                                                    Color32::from_rgba_unmultiplied(255, 255, 255, 210)
                                                } else {
                                                    text_muted
                                                },
                                            );
                                            if rresp.double_clicked()
                                                || (is_compact && rresp.clicked())
                                            {
                                                if item.is_folder {
                                                    open_folder_action = Some((
                                                        item.id.clone(),
                                                        item.name.clone(),
                                                    ));
                                                } else {
                                                    open_diagram_id = Some(item.id.clone());
                                                }
                                            } else if rresp.clicked() {
                                                app.inventory.selected_item_id =
                                                    Some(item.id.clone());
                                            }
                                            rresp.context_menu(|ui| {
                                                if ui.button("Open").clicked() {
                                                    if item.is_folder {
                                                        open_folder_action = Some((
                                                            item.id.clone(),
                                                            item.name.clone(),
                                                        ));
                                                    } else {
                                                        open_diagram_id = Some(item.id.clone());
                                                    }
                                                    ui.close_menu();
                                                }
                                                if ui.button("Rename").clicked() {
                                                    app.inventory.renaming_item = Some((
                                                        item.id.clone(),
                                                        item.name.clone(),
                                                    ));
                                                    ui.close_menu();
                                                }
                                                if let Some((ref cur_id, ref up_id)) = parent_above {
                                                    if ui.button("Move to Parent Folder").clicked() {
                                                        move_up_item = Some((
                                                            item.id.clone(),
                                                            cur_id.clone(),
                                                            up_id.clone(),
                                                        ));
                                                        ui.close_menu();
                                                    }
                                                }
                                                ui.separator();
                                                if ui.button("Move to Trash").clicked() {
                                                    delete_item_id = Some(item.id.clone());
                                                    ui.close_menu();
                                                }
                                            });
                                        }
                                    } else {
                                        let avail_w = ui.available_width().max(220.0);
                                        let cols = ((avail_w / 142.0).floor() as usize).clamp(2, 4);
                                        let gap = 12.0_f32;
                                        let cell_w =
                                            ((avail_w - (cols as f32 - 1.0) * gap) / cols as f32)
                                                .floor()
                                                .max(108.0);
                                        let thumb_w = (cell_w - 14.0).max(90.0);
                                        let thumb_h = (thumb_w * 0.64).clamp(58.0, 82.0);
                                        for chunk in filtered.chunks(cols) {
                                            ui.horizontal(|ui| {
                                                ui.spacing_mut().item_spacing.x = gap;
                                                for item in chunk {
                                                    let disp_name = if item.is_folder {
                                                        item.name.clone()
                                                    } else {
                                                        item.name
                                                            .strip_suffix(".bohat.json")
                                                            .unwrap_or(&item.name)
                                                            .to_string()
                                                    };
                                                    let cache_key = format!(
                                                        "{}:{}",
                                                        item.id,
                                                        item.modified_time.as_deref().unwrap_or("")
                                                    );
                                                    if !item.is_folder
                                                        && !app
                                                            .inventory
                                                            .preview_textures
                                                            .contains_key(&cache_key)
                                                        && !app
                                                            .inventory
                                                            .preview_requested
                                                            .contains(&cache_key)
                                                    {
                                                        preview_requests.push((
                                                            item.id.clone(),
                                                            cache_key.clone(),
                                                        ));
                                                    }
                                                    let is_sel = app
                                                        .inventory
                                                        .selected_item_id
                                                        .as_deref()
                                                        == Some(item.id.as_str());
                                                    let (crect, cresp) = ui.allocate_exact_size(
                                                        Vec2::new(cell_w, thumb_h + 30.0),
                                                        Sense::click(),
                                                    );
                                                    let thumb_r = egui::Rect::from_center_size(
                                                        egui::Pos2::new(
                                                            crect.center().x,
                                                            crect.min.y + thumb_h * 0.5 + 4.0,
                                                        ),
                                                        Vec2::new(thumb_w, thumb_h),
                                                    );
                                                    if is_sel {
                                                        ui.painter().rect_filled(
                                                            thumb_r.expand(4.0),
                                                            7.0,
                                                            Color32::from_rgba_unmultiplied(
                                                                0, 122, 255, 28,
                                                            ),
                                                        );
                                                    } else if cresp.hovered() {
                                                        ui.painter().rect_filled(
                                                            thumb_r.expand(4.0),
                                                            7.0,
                                                            Color32::from_rgba_unmultiplied(
                                                                15, 23, 42, 12,
                                                            ),
                                                        );
                                                    }
                                                    if item.is_folder {
                                                        let f_rect = egui::Rect::from_center_size(
                                                            thumb_r.center(),
                                                            Vec2::new(62.0, 50.0),
                                                        );
                                                        draw_drive_folder_icon(ui.painter(), f_rect);
                                                    } else if let Some(tex) = app
                                                        .inventory
                                                        .preview_textures
                                                        .get(&cache_key)
                                                    {
                                                        ui.painter().rect_filled(
                                                            thumb_r.translate(Vec2::new(0.0, 2.0)),
                                                            6.0,
                                                            Color32::from_rgba_unmultiplied(
                                                                15, 23, 42, 24,
                                                            ),
                                                        );
                                                        let uv = egui::Rect::from_min_max(
                                                            egui::Pos2::ZERO,
                                                            egui::Pos2::new(1.0, 1.0),
                                                        );
                                                        ui.painter().image(
                                                            tex.id(),
                                                            thumb_r,
                                                            uv,
                                                            Color32::WHITE,
                                                        );
                                                        ui.painter().rect_stroke(
                                                            thumb_r,
                                                            6.0,
                                                            Stroke::new(
                                                                if is_sel { 1.6_f32 } else { 1.0_f32 },
                                                                if is_sel {
                                                                    accent_blue
                                                                } else {
                                                                    Color32::from_rgb(206, 211, 222)
                                                                },
                                                            ),
                                                            egui::StrokeKind::Inside,
                                                        );
                                                    } else {
                                                        ui.painter().rect_filled(
                                                            thumb_r,
                                                            6.0,
                                                            Color32::from_rgb(246, 248, 252),
                                                        );
                                                        ui.painter().rect_stroke(
                                                            thumb_r,
                                                            6.0,
                                                            Stroke::new(
                                                                1.0_f32,
                                                                Color32::from_rgb(212, 216, 226),
                                                            ),
                                                            egui::StrokeKind::Inside,
                                                        );
                                                        let d_rect = egui::Rect::from_center_size(
                                                            thumb_r.center(),
                                                            Vec2::new(38.0, 34.0),
                                                        );
                                                        draw_drive_diagram_icon(
                                                            ui.painter(),
                                                            d_rect,
                                                        );
                                                    }

                                                    let short: String =
                                                        if disp_name.chars().count() > 16 {
                                                            format!(
                                                                "{}..",
                                                                disp_name
                                                                    .chars()
                                                                    .take(14)
                                                                    .collect::<String>()
                                                            )
                                                        } else {
                                                            disp_name.clone()
                                                        };
                                                    let lbl_y = thumb_r.max.y + 13.0;
                                                    if is_sel {
                                                        let pill_w =
                                                            (short.len() as f32 * 6.4 + 14.0)
                                                                .min(cell_w - 4.0);
                                                        let pill = egui::Rect::from_center_size(
                                                            egui::Pos2::new(crect.center().x, lbl_y),
                                                            Vec2::new(pill_w, 18.0),
                                                        );
                                                        ui.painter()
                                                            .rect_filled(pill, 4.0, accent_blue);
                                                    }
                                                    ui.painter().text(
                                                        egui::Pos2::new(crect.center().x, lbl_y),
                                                        Align2::CENTER_CENTER,
                                                        short,
                                                        FontId::proportional(11.0),
                                                        if is_sel {
                                                            Color32::WHITE
                                                        } else {
                                                            text_primary
                                                        },
                                                    );

                                                    if cresp.double_clicked()
                                                        || (is_compact && cresp.clicked())
                                                    {
                                                        if item.is_folder {
                                                            open_folder_action = Some((
                                                                item.id.clone(),
                                                                item.name.clone(),
                                                            ));
                                                        } else {
                                                            open_diagram_id = Some(item.id.clone());
                                                        }
                                                    } else if cresp.clicked() {
                                                        app.inventory.selected_item_id =
                                                            Some(item.id.clone());
                                                    }
                                                    cresp.context_menu(|ui| {
                                                        if ui.button("Open").clicked() {
                                                            if item.is_folder {
                                                                open_folder_action = Some((
                                                                    item.id.clone(),
                                                                    item.name.clone(),
                                                                ));
                                                            } else {
                                                                open_diagram_id =
                                                                    Some(item.id.clone());
                                                            }
                                                            ui.close_menu();
                                                        }
                                                        if ui.button("Rename").clicked() {
                                                            app.inventory.renaming_item = Some((
                                                                item.id.clone(),
                                                                item.name.clone(),
                                                            ));
                                                            ui.close_menu();
                                                        }
                                                        if let Some((ref cur_id, ref up_id)) =
                                                            parent_above
                                                        {
                                                            if ui
                                                                .button("Move to Parent Folder")
                                                                .clicked()
                                                            {
                                                                move_up_item = Some((
                                                                    item.id.clone(),
                                                                    cur_id.clone(),
                                                                    up_id.clone(),
                                                                ));
                                                                ui.close_menu();
                                                            }
                                                        }
                                                        ui.separator();
                                                        if ui.button("Move to Trash").clicked() {
                                                            delete_item_id = Some(item.id.clone());
                                                            ui.close_menu();
                                                        }
                                                    });
                                                }
                                            });
                                            ui.add_space(10.0);
                                        }
                                    }
                                });
                        });

                    let (hdiv3, _) = ui.allocate_exact_size(
                        Vec2::new(main_w - if is_compact { 0.0 } else { 1.0 }, 1.0),
                        Sense::hover(),
                    );
                    ui.painter().rect_filled(hdiv3, 0.0, divider_col);

                    Frame::new()
                        .fill(toolbar_bg)
                        .corner_radius(egui::CornerRadius {
                            nw: 0,
                            ne: 0,
                            sw: if is_compact { 11 } else { 0 },
                            se: 11,
                        })
                        .inner_margin(Margin::symmetric(12, 5))
                        .show(ui, |ui| {
                            ui.set_width((main_w - 26.0).max(220.0));
                            ui.horizontal(|ui| {
                                let crumbs = app.inventory.folder_stack.clone();
                                for (idx, (_, name)) in crumbs.iter().enumerate() {
                                    if idx > 0 {
                                        ui.label(
                                            RichText::new(">").size(10.0).color(text_muted),
                                        );
                                    }
                                    let is_last = idx + 1 == crumbs.len();
                                    if ui
                                        .selectable_label(
                                            false,
                                            RichText::new(name)
                                                .size(10.5)
                                                .color(if is_last {
                                                    text_primary
                                                } else {
                                                    text_muted
                                                }),
                                        )
                                        .clicked()
                                        && !is_last
                                    {
                                        jump_crumb_idx = Some(idx);
                                    }
                                }
                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Center),
                                    |ui| {
                                        ui.label(
                                            RichText::new(format!(
                                                "{} item{}",
                                                filtered.len(),
                                                if filtered.len() == 1 { "" } else { "s" }
                                            ))
                                            .size(10.0)
                                            .color(text_muted),
                                        );
                                    },
                                );
                            });
                        });
                });
            });

            for (fid, ckey) in preview_requests.into_iter().take(24) {
                app.inventory.request_diagram_preview(fid, ckey, ctx.clone());
            }
        });

    if open_picker_now {
        app.inventory.show_folder_picker = true;
        app.inventory.picker_stack = vec![("root".to_string(), "My Drive".to_string())];
        app.inventory.picker_search.clear();
        app.inventory
            .browse_drive_folders(Some("root".to_string()), ctx.clone());
    }

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
        let target_file_name = if dname.ends_with(".bohat.json") {
            dname.clone()
        } else {
            format!("{dname}.bohat.json")
        };
        let existing_id = app
            .inventory
            .items
            .iter()
            .find(|it| {
                !it.is_folder
                    && (it.name.eq_ignore_ascii_case(&target_file_name)
                        || it.name.eq_ignore_ascii_case(&dname))
            })
            .map(|it| it.id.clone())
            .or_else(|| {
                if app.inventory.active_folder_id == cur_fid
                    && app.inventory.active_file_name.eq_ignore_ascii_case(&dname)
                {
                    app.inventory.active_file_id.clone()
                } else {
                    None
                }
            });
        app.save_current_to_drive(existing_id, cur_fid, dname, ctx);
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

    if app.inventory.show_folder_picker {
        let mut picker_open = app.inventory.show_folder_picker;
        let pw = (screen.width() - 36.0).clamp(290.0, 520.0);
        let ph = (screen.height() - 88.0).clamp(280.0, 420.0);
        let mut enter_folder: Option<(String, String)> = None;
        let mut jump_picker_idx: Option<usize> = None;
        let mut confirm_current = false;
        let mut reset_default = false;

        egui::Window::new("Select Google Drive Folder")
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .collapsible(false)
            .resizable(true)
            .default_size(Vec2::new(pw, ph))
            .open(&mut picker_open)
            .show(ctx, |ui| {
                ui.horizontal_wrapped(|ui| {
                    if app.inventory.picker_stack.len() > 1 && ui.small_button("<- Up").clicked() {
                        jump_picker_idx = Some(app.inventory.picker_stack.len() - 2);
                    }
                    let crumbs = app.inventory.picker_stack.clone();
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
                            jump_picker_idx = Some(idx);
                        }
                    }
                });
                ui.horizontal(|ui| {
                    ui.add(
                        egui::TextEdit::singleline(&mut app.inventory.picker_search)
                            .hint_text("Filter folders...")
                            .desired_width((ui.available_width() - 70.0).max(120.0)),
                    );
                    if app.inventory.loading {
                        ui.label(RichText::new("Loading...").small().weak());
                    }
                });
                if let Some(err) = &app.inventory.last_error {
                    ui.label(RichText::new(err).small().color(Color32::from_rgb(200, 30, 30)));
                }
                ui.separator();

                let pq = app.inventory.picker_search.trim().to_lowercase();
                let folders: Vec<_> = app
                    .inventory
                    .picker_folders
                    .iter()
                    .filter(|f| pq.is_empty() || f.name.to_lowercase().contains(&pq))
                    .cloned()
                    .collect();

                ScrollArea::vertical()
                    .max_height((ph - 132.0).max(120.0))
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        if folders.is_empty() && !app.inventory.loading {
                            ui.add_space(16.0);
                            ui.vertical_centered(|ui| {
                                ui.label(
                                    RichText::new("No subfolders here. Click 'Use This Folder' below to select this directory.")
                                        .weak(),
                                );
                            });
                        } else {
                            for f in &folders {
                                ui.horizontal(|ui| {
                                    let (ir, _) = ui.allocate_exact_size(
                                        Vec2::new(22.0, 20.0),
                                        Sense::hover(),
                                    );
                                    draw_drive_folder_icon(ui.painter(), ir);
                                    if ui.selectable_label(false, &f.name).clicked() {
                                        enter_folder = Some((f.id.clone(), f.name.clone()));
                                    }
                                });
                            }
                        }
                    });

                ui.separator();
                let cur_label = app
                    .inventory
                    .picker_stack
                    .last()
                    .map(|(_, n)| n.clone())
                    .unwrap_or_else(|| "My Drive".to_string());
                ui.horizontal_wrapped(|ui| {
                    if ui
                        .add(
                            egui::Button::new(
                                RichText::new(format!("Use '{cur_label}'")).color(Color32::WHITE),
                            )
                            .fill(Color32::from_rgb(0, 0, 128)),
                        )
                        .clicked()
                    {
                        confirm_current = true;
                    }
                    if ui.button("Reset Default (Bohat Draw.io)").clicked() {
                        reset_default = true;
                    }
                });
            });

        app.inventory.show_folder_picker = picker_open;
        if let Some((fid, fname)) = enter_folder {
            app.inventory.picker_stack.push((fid.clone(), fname));
            app.inventory.picker_search.clear();
            app.inventory.browse_drive_folders(Some(fid), ctx.clone());
        } else if let Some(idx) = jump_picker_idx {
            app.inventory.picker_stack.truncate(idx + 1);
            let pid = app
                .inventory
                .picker_stack
                .last()
                .map(|(id, _)| id.clone())
                .unwrap_or_else(|| "root".to_string());
            app.inventory.browse_drive_folders(Some(pid), ctx.clone());
        } else if confirm_current {
            if let Some((fid, fname)) = app.inventory.picker_stack.last().cloned() {
                app.inventory
                    .select_root_folder(Some(fid), Some(fname), ctx.clone());
            }
        } else if reset_default {
            app.inventory.select_root_folder(None, None, ctx.clone());
        }
    }
}
