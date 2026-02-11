use crate::colors::{MAIN_COLOR, STROKE_COLOR, TEXT_COLOR};
use crate::egui::{self, CornerRadius, RichText, Stroke};

/// Data for displaying a weapon popup card.
pub struct WeaponCard {
    pub name: String,
    pub kind: String,
    pub attack: String,
    pub damage: String,
    pub range: String,
    pub condition: String,
}

impl WeaponCard {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            kind: String::new(),
            attack: String::new(),
            damage: String::new(),
            range: String::new(),
            condition: String::new(),
        }
    }

    pub fn kind(mut self, kind: impl Into<String>) -> Self {
        self.kind = kind.into();
        self
    }

    pub fn attack(mut self, attack: impl Into<String>) -> Self {
        self.attack = attack.into();
        self
    }

    pub fn damage(mut self, damage: impl Into<String>) -> Self {
        self.damage = damage.into();
        self
    }

    pub fn range(mut self, range: impl Into<String>) -> Self {
        self.range = range.into();
        self
    }

    pub fn condition(mut self, condition: impl Into<String>) -> Self {
        self.condition = condition.into();
        self
    }

    /// Shows the weapon card as a tooltip popup at the given position.
    pub fn show_at(&self, ctx: &egui::Context, id: egui::Id, pos: egui::Pos2) {
        egui::Area::new(id.with("weapon_card"))
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

                        // Name (bold) + type (right)
                        ui.horizontal(|ui| {
                            ui.label(
                                RichText::new(&self.name)
                                    .strong()
                                    .size(15.0)
                                    .color(TEXT_COLOR),
                            );
                            if !self.kind.is_empty() {
                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Center),
                                    |ui| {
                                        ui.label(
                                            RichText::new(&self.kind)
                                                .size(12.0)
                                                .color(STROKE_COLOR),
                                        );
                                    },
                                );
                            }
                        });

                        ui.separator();

                        if !self.attack.is_empty() {
                            ui.label(
                                RichText::new(format!("Attack: {}", self.attack))
                                    .size(12.0)
                                    .color(TEXT_COLOR),
                            );
                        }
                        if !self.damage.is_empty() {
                            ui.label(
                                RichText::new(format!("Damage: {}", self.damage))
                                    .size(12.0)
                                    .color(TEXT_COLOR),
                            );
                        }
                        if !self.range.is_empty() {
                            ui.label(
                                RichText::new(format!("Range: {}", self.range))
                                    .size(12.0)
                                    .color(TEXT_COLOR),
                            );
                        }
                        if !self.condition.is_empty() {
                            ui.add_space(2.0);
                            ui.label(
                                RichText::new(&self.condition)
                                    .size(11.0)
                                    .italics()
                                    .color(STROKE_COLOR),
                            );
                        }
                    });
            });
    }
}
