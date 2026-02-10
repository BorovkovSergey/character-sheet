use crate::colors::{MAIN_COLOR, STROKE_COLOR, TEXT_COLOR};
use crate::egui::{self, CornerRadius, RichText, Stroke};

/// Data for displaying an item popup card.
pub struct ItemCard {
    pub name: String,
    pub description: String,
}

impl ItemCard {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: String::new(),
        }
    }

    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = description.into();
        self
    }

    /// Shows the item card as a tooltip popup at the given position.
    pub fn show_at(&self, ctx: &egui::Context, id: egui::Id, pos: egui::Pos2) {
        egui::Area::new(id.with("item_card"))
            .order(egui::Order::Tooltip)
            .fixed_pos(pos)
            .show(ctx, |ui| {
                egui::Frame::NONE
                    .fill(MAIN_COLOR)
                    .stroke(Stroke::new(0.5, STROKE_COLOR))
                    .corner_radius(CornerRadius::same(10))
                    .inner_margin(10.0)
                    .show(ui, |ui| {
                        ui.set_max_width(200.0);

                        ui.label(
                            RichText::new(&self.name)
                                .strong()
                                .size(15.0)
                                .color(TEXT_COLOR),
                        );

                        if !self.description.is_empty() {
                            ui.separator();
                            ui.label(
                                RichText::new(&self.description)
                                    .size(12.0)
                                    .color(TEXT_COLOR),
                            );
                        }
                    });
            });
    }
}
