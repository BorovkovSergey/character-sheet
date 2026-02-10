use crate::colors::{MAIN_COLOR, STROKE_COLOR, TEXT_COLOR};
use crate::egui::{self, CornerRadius, RichText, Stroke};

/// Data for displaying an equipment popup card.
pub struct EquipmentCard {
    pub name: String,
    pub slot: String,
    pub description: String,
    pub armor: i32,
    pub effects: Vec<String>,
}

impl EquipmentCard {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            slot: String::new(),
            description: String::new(),
            armor: 0,
            effects: Vec::new(),
        }
    }

    pub fn slot(mut self, slot: impl Into<String>) -> Self {
        self.slot = slot.into();
        self
    }

    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = description.into();
        self
    }

    pub fn armor(mut self, armor: i32) -> Self {
        self.armor = armor;
        self
    }

    pub fn effects(mut self, effects: Vec<String>) -> Self {
        self.effects = effects;
        self
    }

    /// Shows the equipment card as a tooltip popup at the given position.
    pub fn show_at(&self, ctx: &egui::Context, id: egui::Id, pos: egui::Pos2) {
        egui::Area::new(id.with("equipment_card"))
            .order(egui::Order::Tooltip)
            .fixed_pos(pos)
            .show(ctx, |ui| {
                egui::Frame::NONE
                    .fill(MAIN_COLOR)
                    .stroke(Stroke::new(0.5, STROKE_COLOR))
                    .corner_radius(CornerRadius::same(10))
                    .inner_margin(10.0)
                    .show(ui, |ui| {
                        ui.set_max_width(220.0);

                        // Name (bold) + slot (right)
                        ui.horizontal(|ui| {
                            ui.label(
                                RichText::new(&self.name)
                                    .strong()
                                    .size(15.0)
                                    .color(TEXT_COLOR),
                            );
                            if !self.slot.is_empty() {
                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Center),
                                    |ui| {
                                        ui.label(
                                            RichText::new(&self.slot)
                                                .size(12.0)
                                                .color(STROKE_COLOR),
                                        );
                                    },
                                );
                            }
                        });

                        ui.separator();

                        // Description
                        if !self.description.is_empty() {
                            ui.label(
                                RichText::new(&self.description)
                                    .size(12.0)
                                    .color(TEXT_COLOR),
                            );
                            ui.add_space(4.0);
                        }

                        // Armor
                        if self.armor != 0 {
                            ui.label(
                                RichText::new(format!("Armor: {}", self.armor))
                                    .size(12.0)
                                    .color(TEXT_COLOR),
                            );
                        }

                        // Effects
                        if !self.effects.is_empty() {
                            ui.add_space(2.0);
                            for effect in &self.effects {
                                ui.label(
                                    RichText::new(effect.as_str())
                                        .size(11.0)
                                        .color(TEXT_COLOR),
                                );
                            }
                        }
                    });
            });
    }
}
