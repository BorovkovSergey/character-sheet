use std::cell::Cell;

use crate::colors::{MAIN_COLOR, SECONDARY_COLOR, STROKE_COLOR};
use crate::egui::{self, CornerRadius, RichText, Stroke, TextureId};
use crate::molecules::{TitledBox, WeaponEntry};

const SLOT_COUNT: usize = 3;

/// Data for a single weapon slot passed from the client.
pub struct WeaponSlot {
    pub name: String,
    pub kind: String,
    pub attack: String,
    pub damage: String,
    pub range: String,
    pub condition: String,
}

/// Displays equipped weapon slots.
/// Right-clicking a filled slot shows an "Unequip" context menu.
pub struct Weapon {
    icon: TextureId,
    slots: Vec<WeaponSlot>,
}

impl Weapon {
    pub fn new(icon: TextureId, slots: Vec<WeaponSlot>) -> Self {
        Self { icon, slots }
    }

    /// Renders the weapon slots and returns the index of the weapon the user chose to unequip.
    pub fn show(self, ui: &mut egui::Ui) -> Option<usize> {
        let action = Cell::new(None);
        TitledBox::new("Weapon")
            .fill(SECONDARY_COLOR)
            .rounding(16)
            .content_rounding(14)
            .show(ui, |ui| {
                let result = inner_weapon_slots(ui, self.icon, &self.slots);
                action.set(result);
            });
        action.get()
    }
}

/// Lays out weapon entry slots vertically. Returns index of unequipped weapon if any.
fn inner_weapon_slots(ui: &mut egui::Ui, icon: TextureId, slots: &[WeaponSlot]) -> Option<usize> {
    let count = SLOT_COUNT as f32;
    let spacing = 4.0;
    let available_width = ui.available_width();
    let available_height = ui.available_height();
    let pad = available_height * 0.015;
    let inner_width = available_width - pad;
    let inner_height = available_height - pad * 2.0;
    let item_height = (inner_height - spacing * (count - 1.0)) / count;

    let mut action = None;

    ui.vertical(|ui| {
        ui.add_space(pad);
        ui.spacing_mut().item_spacing = egui::vec2(0.0, spacing);
        for i in 0..SLOT_COUNT {
            ui.allocate_ui_with_layout(
                egui::vec2(inner_width, item_height),
                egui::Layout::left_to_right(egui::Align::Center),
                |ui| {
                    ui.add_space(pad);
                    let available = ui.available_size();
                    let has_weapon = slots.get(i).is_some();
                    let sense = if has_weapon {
                        egui::Sense::click() | egui::Sense::hover()
                    } else {
                        egui::Sense::hover()
                    };
                    let (rect, response) = ui.allocate_exact_size(available, sense);
                    let mut entry = WeaponEntry::new(icon);
                    if let Some(slot) = slots.get(i) {
                        entry = entry
                            .name(&slot.name)
                            .kind(&slot.kind)
                            .attack(&slot.attack)
                            .damage(&slot.damage)
                            .range(&slot.range);
                    }
                    entry.paint(ui.painter(), rect);

                    let mut menu_open = false;
                    if has_weapon {
                        menu_open = response
                            .context_menu(|ui| {
                                if ui.button("Unequip").clicked() {
                                    action = Some(i);
                                    ui.close();
                                }
                            })
                            .is_some();
                    }
                    if let Some(slot) = slots.get(i) {
                        if response.hovered() && !menu_open && !slot.condition.is_empty() {
                            let pos =
                                response.hover_pos().unwrap_or(rect.right_top())
                                    + egui::vec2(8.0, 8.0);
                            egui::Area::new(response.id.with("cond_tip"))
                                .order(egui::Order::Tooltip)
                                .fixed_pos(pos)
                                .show(ui.ctx(), |ui| {
                                    egui::Frame::NONE
                                        .fill(MAIN_COLOR)
                                        .stroke(Stroke::new(0.5, STROKE_COLOR))
                                        .corner_radius(CornerRadius::same(6))
                                        .inner_margin(6.0)
                                        .show(ui, |ui| {
                                            ui.label(
                                                RichText::new(&slot.condition)
                                                    .size(11.0)
                                                    .italics()
                                                    .color(crate::colors::TEXT_COLOR),
                                            );
                                        });
                                });
                        }
                    }
                },
            );
        }
    });

    action
}
