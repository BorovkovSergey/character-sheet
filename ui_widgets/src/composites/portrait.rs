use std::collections::BTreeMap;

use crate::atoms::{Shape, ShapeBox};
use crate::colors::{MAIN_COLOR, TEXT_COLOR};
use crate::egui::{self, Align2, FontId, Stroke};
use crate::molecules::InventoryTooltip;

/// State stored in egui temp data for the "Add Experience" popup.
#[derive(Clone)]
struct AddExpPopupState {
    open: bool,
    input_text: String,
}

/// Data for the "Add item" context submenu.
pub struct AddItemMenu {
    pub items: Vec<InventoryTooltip>,
    pub equipment: BTreeMap<String, Vec<InventoryTooltip>>,
    pub weapons: BTreeMap<String, Vec<InventoryTooltip>>,
}

/// What the user selected from the "Add item" menu.
pub enum AddItemSelection {
    Item(String),
    Equipment(String),
    Weapon(String),
}

/// Response from portrait rendering.
pub struct PortraitResponse {
    pub add_exp: Option<u32>,
    pub toggle_edit: bool,
    pub open_learn_ability: bool,
    pub open_learn_trait: bool,
    pub open_create_item: bool,
    pub add_item: Option<AddItemSelection>,
    pub save: bool,
    pub back: bool,
    pub upload_portrait: bool,
    pub toggle_auth: bool,
}

/// Character portrait display area.
pub struct Portrait {
    border_1: egui::TextureId,
    border_2: egui::TextureId,
    avatar: egui::TextureId,
    shield: Option<egui::TextureId>,
    level: u32,
    xp_current: u32,
    xp_next: u32,
    xp_fraction: f32,
    edit_mode: bool,
    authenticated: bool,
    ability_points: u32,
    trait_points: u32,
    armor: i32,
    add_item_menu: Option<AddItemMenu>,
    avatar_size: Option<[f32; 2]>,
}

impl Portrait {
    pub fn new(
        border_1: egui::TextureId,
        border_2: egui::TextureId,
        avatar: egui::TextureId,
        level: u32,
        xp_current: u32,
        xp_next: u32,
        xp_fraction: f32,
        edit_mode: bool,
    ) -> Self {
        Self {
            border_1,
            border_2,
            avatar,
            level,
            xp_current,
            xp_next,
            xp_fraction,
            edit_mode,
            authenticated: false,
            shield: None,
            ability_points: 0,
            trait_points: 0,
            armor: 0,
            add_item_menu: None,
            avatar_size: None,
        }
    }

    pub fn shield(mut self, texture: egui::TextureId, armor: i32) -> Self {
        self.shield = Some(texture);
        self.armor = armor;
        self
    }

    pub fn ability_points(mut self, points: u32) -> Self {
        self.ability_points = points;
        self
    }

    pub fn trait_points(mut self, points: u32) -> Self {
        self.trait_points = points;
        self
    }

    pub fn add_item_menu(mut self, menu: AddItemMenu) -> Self {
        self.add_item_menu = Some(menu);
        self
    }

    pub fn authenticated(mut self, authenticated: bool) -> Self {
        self.authenticated = authenticated;
        self
    }

    pub fn avatar_size(mut self, size: Option<[f32; 2]>) -> Self {
        self.avatar_size = size;
        self
    }

    /// Renders the portrait and returns actions from the context menu.
    pub fn show(self, ui: &mut egui::Ui) -> PortraitResponse {
        let size = ui.available_size();
        let (rect, response) = ui.allocate_exact_size(size, egui::Sense::click());

        let portrait_w = size.x * 0.65;
        let inset = (size.x - portrait_w) / 2.0;
        let portrait_rect = egui::Rect::from_min_size(
            rect.min + egui::vec2(inset, 0.0),
            egui::vec2(portrait_w, size.y),
        );

        let uv = egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0));
        let painter = ui.painter_at(rect);

        painter.image(self.border_1, portrait_rect, uv, egui::Color32::WHITE);
        painter.image(self.border_2, portrait_rect, uv, egui::Color32::WHITE);

        let border = size.y * 0.025;
        let clip_rect = portrait_rect.shrink(border);
        // Compute cover-mode image_rect so uploaded portraits don't stretch.
        let image_rect = if let Some([iw, ih]) = self.avatar_size {
            if ih == 0.0 || clip_rect.height() == 0.0 {
                clip_rect
            } else {
                let img_aspect = iw / ih;
                let clip_aspect = clip_rect.width() / clip_rect.height();
                if img_aspect > clip_aspect {
                    let h = clip_rect.height();
                    let w = h * img_aspect;
                    egui::Rect::from_center_size(clip_rect.center(), egui::vec2(w, h))
                } else {
                    let w = clip_rect.width();
                    let h = w / img_aspect;
                    egui::Rect::from_center_size(clip_rect.center(), egui::vec2(w, h))
                }
            }
        } else {
            rect
        };
        paint_ellipse_image(&painter, self.avatar, clip_rect, image_rect);

        // Level circle in the top-left corner of the portrait
        let circle_size = portrait_w * 0.3;
        let circle_rect = egui::Rect::from_min_size(
            portrait_rect.min + egui::vec2(0.0, 10.0),
            egui::vec2(circle_size, circle_size),
        );
        // Border images clipped to circle
        paint_ellipse_image(&painter, self.border_1, circle_rect, circle_rect);
        paint_ellipse_image(&painter, self.border_2, circle_rect, circle_rect);
        let circle_border = border / 2.0;
        let arc_thickness = circle_border;
        let arc_rect = circle_rect.shrink(circle_border);

        // Inner fill (covers arc zone so unfilled area looks thin)
        ShapeBox::new(Shape::Circle)
            .fill(MAIN_COLOR)
            .stroke(Stroke::NONE)
            .paint(&painter, arc_rect);

        // XP arc drawn on top
        paint_xp_arc(&painter, arc_rect, arc_thickness, self.xp_fraction);

        // Level text
        let level_text = self.level.to_string();
        let font_size = arc_rect.height() * 0.5;
        painter.text(
            arc_rect.center(),
            Align2::CENTER_CENTER,
            level_text,
            FontId::proportional(font_size),
            TEXT_COLOR,
        );

        // Hover tooltip for XP on level circle
        if let Some(pos) = ui.ctx().pointer_hover_pos() {
            if circle_rect.contains(pos) {
                egui::Area::new(egui::Id::new("level_xp_tooltip"))
                    .order(egui::Order::Tooltip)
                    .fixed_pos(pos + egui::vec2(12.0, 12.0))
                    .show(ui.ctx(), |ui| {
                        egui::Frame::new()
                            .fill(MAIN_COLOR)
                            .corner_radius(4.0)
                            .inner_margin(egui::Margin::symmetric(8, 4))
                            .show(ui, |ui| {
                                ui.label(
                                    egui::RichText::new(format!(
                                        "EXP: {} / {}",
                                        self.xp_current, self.xp_next
                                    ))
                                    .color(TEXT_COLOR)
                                    .size(13.0),
                                );
                            });
                    });
            }
        }

        // Armor shield in the bottom-right corner of the portrait
        if let Some(shield_tex) = self.shield {
            let shield_size = portrait_w * 0.3;
            let shield_rect = egui::Rect::from_min_size(
                egui::pos2(
                    portrait_rect.max.x - shield_size,
                    portrait_rect.max.y - shield_size - 10.0,
                ),
                egui::vec2(shield_size, shield_size),
            );
            painter.image(shield_tex, shield_rect, uv, egui::Color32::WHITE);
            let armor_text = self.armor.to_string();
            let armor_font_size = shield_size * 0.35;
            painter.text(
                shield_rect.center(),
                Align2::CENTER_CENTER,
                armor_text,
                FontId::proportional(armor_font_size),
                TEXT_COLOR,
            );
        }

        // Context menu on right-click
        let popup_id = response.id.with("add_exp");
        let mut toggle_edit = false;
        let mut open_learn_ability = false;
        let mut open_learn_trait = false;
        let mut open_create_item = false;
        let mut add_item_selection = None;
        let mut save_clicked = false;
        let mut back_clicked = false;
        let mut upload_portrait = false;
        let mut toggle_auth = false;
        let add_item_menu = self.add_item_menu;
        response.context_menu(|ui| {
            if self.authenticated && ui.button("Save").clicked() {
                save_clicked = true;
                ui.close();
            }
            if ui.button("Back").clicked() {
                back_clicked = true;
                ui.close();
            }
            ui.separator();
            if ui.button("Upload Portrait").clicked() {
                upload_portrait = true;
                ui.close();
            }
            if ui.button("Add EXP").clicked() {
                ui.data_mut(|d| {
                    d.insert_temp(
                        popup_id,
                        AddExpPopupState {
                            open: true,
                            input_text: String::new(),
                        },
                    );
                });
                ui.close();
            }
            let edit_label = if self.edit_mode {
                "Confirm changes"
            } else {
                "Edit"
            };
            if ui.button(edit_label).clicked() {
                toggle_edit = true;
                ui.close();
            }
            let has_points = self.ability_points > 0;
            if ui
                .add_enabled(has_points, egui::Button::new("Learn ability"))
                .clicked()
            {
                open_learn_ability = true;
                ui.close();
            }
            let has_trait_points = self.trait_points > 0;
            if ui
                .add_enabled(has_trait_points, egui::Button::new("Learn trait"))
                .clicked()
            {
                open_learn_trait = true;
                ui.close();
            }
            if ui.button("Create item").clicked() {
                open_create_item = true;
                ui.close();
            }
            ui.separator();
            let auth_label = if self.authenticated {
                "\u{1F513} Lock"
            } else {
                "\u{1F512} Unlock"
            };
            if ui.button(auth_label).clicked() {
                toggle_auth = true;
                ui.close();
            }
            if let Some(menu) = &add_item_menu {
                ui.menu_button("Add item", |ui| {
                    if !menu.items.is_empty() {
                        ui.menu_button("Item", |ui| {
                            for tooltip in &menu.items {
                                let resp = ui.button(tooltip.name());
                                if resp.clicked() {
                                    add_item_selection =
                                        Some(AddItemSelection::Item(tooltip.name().to_owned()));
                                    ui.close();
                                }
                                show_menu_tooltip(ui, &resp, tooltip);
                            }
                        });
                    }
                    if !menu.equipment.is_empty() {
                        ui.menu_button("Equipment", |ui| {
                            for (slot, tooltips) in &menu.equipment {
                                ui.menu_button(slot, |ui| {
                                    for tooltip in tooltips {
                                        let resp = ui.button(tooltip.name());
                                        if resp.clicked() {
                                            add_item_selection = Some(AddItemSelection::Equipment(
                                                tooltip.name().to_owned(),
                                            ));
                                            ui.close();
                                        }
                                        show_menu_tooltip(ui, &resp, tooltip);
                                    }
                                });
                            }
                        });
                    }
                    if !menu.weapons.is_empty() {
                        ui.menu_button("Weapon", |ui| {
                            for (kind, tooltips) in &menu.weapons {
                                ui.menu_button(kind, |ui| {
                                    for tooltip in tooltips {
                                        let resp = ui.button(tooltip.name());
                                        if resp.clicked() {
                                            add_item_selection = Some(AddItemSelection::Weapon(
                                                tooltip.name().to_owned(),
                                            ));
                                            ui.close();
                                        }
                                        show_menu_tooltip(ui, &resp, tooltip);
                                    }
                                });
                            }
                        });
                    }
                });
            }
        });

        // "Add Experience" popup window
        let mut add_exp = None;
        let mut state: AddExpPopupState =
            ui.data(|d| d.get_temp(popup_id))
                .unwrap_or(AddExpPopupState {
                    open: false,
                    input_text: String::new(),
                });

        if state.open {
            let mut open = true;
            egui::Window::new("Add Experience")
                .collapsible(false)
                .resizable(false)
                .open(&mut open)
                .show(ui.ctx(), |ui| {
                    ui.horizontal(|ui| {
                        ui.label("EXP:");
                        ui.add(
                            egui::TextEdit::singleline(&mut state.input_text).desired_width(80.0),
                        );
                    });
                    state.input_text.retain(|c| c.is_ascii_digit());

                    let enter_pressed = ui.input(|i| i.key_pressed(egui::Key::Enter));

                    ui.horizontal(|ui| {
                        if ui.button("OK").clicked() || enter_pressed {
                            if let Ok(value) = state.input_text.parse::<u32>() {
                                if value > 0 {
                                    add_exp = Some(value);
                                }
                            }
                            state.open = false;
                        }
                        if ui.button("Cancel").clicked() {
                            state.open = false;
                        }
                    });
                });

            // Handle X button closing the window
            if !open {
                state.open = false;
            }

            ui.data_mut(|d| d.insert_temp(popup_id, state));
        }

        PortraitResponse {
            add_exp,
            toggle_edit,
            open_learn_ability,
            open_learn_trait,
            open_create_item,
            add_item: add_item_selection,
            save: save_clicked,
            back: back_clicked,
            upload_portrait,
            toggle_auth,
        }
    }
}

/// Paints `texture` clipped to an ellipse defined by `clip_rect`.
/// UV coordinates are computed relative to `image_rect` so the image
/// keeps its proportions and is simply cropped by the ellipse.
fn paint_ellipse_image(
    painter: &egui::Painter,
    texture: egui::TextureId,
    clip_rect: egui::Rect,
    image_rect: egui::Rect,
) {
    const SEGMENTS: usize = 64;

    let center = clip_rect.center();
    let rx = clip_rect.width() / 2.0;
    let ry = clip_rect.height() / 2.0;

    let pos_to_uv = |pos: egui::Pos2| -> egui::Pos2 {
        egui::pos2(
            (pos.x - image_rect.min.x) / image_rect.width(),
            (pos.y - image_rect.min.y) / image_rect.height(),
        )
    };

    let mut mesh = egui::Mesh::with_texture(texture);

    // Center vertex
    mesh.vertices.push(egui::epaint::Vertex {
        pos: center,
        uv: pos_to_uv(center),
        color: egui::Color32::WHITE,
    });

    // Perimeter vertices
    for i in 0..=SEGMENTS {
        let angle = std::f32::consts::TAU * (i as f32) / (SEGMENTS as f32);
        let pos = egui::pos2(center.x + rx * angle.cos(), center.y + ry * angle.sin());
        mesh.vertices.push(egui::epaint::Vertex {
            pos,
            uv: pos_to_uv(pos),
            color: egui::Color32::WHITE,
        });
    }

    // Triangle fan
    for i in 0..SEGMENTS {
        mesh.indices.push(0);
        mesh.indices.push((i + 1) as u32);
        mesh.indices.push((i + 2) as u32);
    }

    painter.add(egui::Shape::mesh(mesh));
}

/// Draws a gradient green-to-blue arc representing XP progress.
/// The arc starts at the top (12 o'clock) and sweeps clockwise.
/// Rendered as a series of short colored strokes for reliable display.
fn paint_xp_arc(painter: &egui::Painter, outer_rect: egui::Rect, thickness: f32, fraction: f32) {
    if fraction <= 0.0 {
        return;
    }

    const SEGMENTS: usize = 64;
    let fraction = fraction.min(1.0);
    let seg_count = ((SEGMENTS as f32) * fraction).ceil() as usize;

    let center = outer_rect.center();
    let radius = outer_rect.width().min(outer_rect.height()) / 2.0 - thickness / 2.0;

    let start_color = egui::Color32::from_rgb(0x00, 0xE5, 0x76); // green
    let end_color = egui::Color32::from_rgb(0x00, 0xB4, 0xD8); // cyan

    let start_angle = -std::f32::consts::FRAC_PI_2;

    for i in 0..seg_count {
        let t0 = (i as f32) / (SEGMENTS as f32);
        let t1 = ((i + 1) as f32) / (SEGMENTS as f32);
        let a0 = start_angle + std::f32::consts::TAU * t0;
        let a1 = start_angle + std::f32::consts::TAU * t1;

        let color_t = t0 / fraction;
        let color = lerp_color(start_color, end_color, color_t);

        let p0 = egui::pos2(center.x + radius * a0.cos(), center.y + radius * a0.sin());
        let p1 = egui::pos2(center.x + radius * a1.cos(), center.y + radius * a1.sin());

        painter.line_segment([p0, p1], egui::Stroke::new(thickness, color));
    }
}

fn lerp_color(a: egui::Color32, b: egui::Color32, t: f32) -> egui::Color32 {
    let t = t.clamp(0.0, 1.0);
    let lerp = |a: u8, b: u8| -> u8 { (a as f32 + (b as f32 - a as f32) * t) as u8 };
    egui::Color32::from_rgb(lerp(a.r(), b.r()), lerp(a.g(), b.g()), lerp(a.b(), b.b()))
}

/// Shows a tooltip card next to a hovered menu button.
fn show_menu_tooltip(ui: &egui::Ui, resp: &egui::Response, tooltip: &InventoryTooltip) {
    if !resp.hovered() {
        return;
    }
    let pos = resp.rect.right_top() + egui::vec2(8.0, 0.0);
    tooltip.show_at(ui.ctx(), resp.id, pos);
}
