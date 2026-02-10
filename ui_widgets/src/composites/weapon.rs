use crate::colors::SECONDARY_COLOR;
use crate::egui::{self, TextureId, Widget};
use crate::molecules::{TitledBox, WeaponEntry};

const SLOT_COUNT: usize = 3;

/// Data for a single weapon slot passed from the client.
pub struct WeaponSlot {
    pub name: String,
    pub kind: String,
    pub attack: String,
    pub damage: String,
    pub range: String,
}

/// Displays equipped weapon slots.
pub struct Weapon {
    icon: TextureId,
    slots: Vec<WeaponSlot>,
}

impl Weapon {
    pub fn new(icon: TextureId, slots: Vec<WeaponSlot>) -> Self {
        Self { icon, slots }
    }
}

impl Widget for Weapon {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        TitledBox::new("Weapon")
            .fill(SECONDARY_COLOR)
            .rounding(16)
            .content_rounding(14)
            .show(ui, |ui| {
                inner_weapon_slots(ui, self.icon, &self.slots);
            })
    }
}

/// Lays out weapon entry slots vertically.
fn inner_weapon_slots(ui: &mut egui::Ui, icon: TextureId, slots: &[WeaponSlot]) {
    let count = SLOT_COUNT as f32;
    let spacing = 4.0;
    let available_width = ui.available_width();
    let available_height = ui.available_height();
    let pad = available_height * 0.015;
    let inner_width = available_width - pad;
    let inner_height = available_height - pad * 2.0;
    let item_height = (inner_height - spacing * (count - 1.0)) / count;

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
                    let (rect, _response) = ui.allocate_exact_size(available, egui::Sense::hover());
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
                },
            );
        }
    });
}
