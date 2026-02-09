use crate::atoms::{Shape, ShapeBox, Text};
use crate::colors::{MAIN_COLOR, STROKE_COLOR, TEXT_COLOR};
use crate::egui::{self, Align2, CornerRadius, Stroke, Widget};
use crate::molecules::TitledBox;
use crate::traits::Roundable;

/// A single trait entry for display.
pub struct TraitEntry {
    pub name: String,
    pub description: String,
    pub effects: Vec<String>,
}

/// Displays the character's passive traits and perks.
///
/// Renders as a [`TitledBox`] with vertical "Traits" label on the left.
/// The content area contains horizontally scrollable [`ShapeBox`] cards,
/// with at most 3 visible at a time. Each card shows the trait name at
/// the top and its description below.
pub struct Traits {
    entries: Vec<TraitEntry>,
}

impl Traits {
    pub fn new(entries: Vec<TraitEntry>) -> Self {
        Self { entries }
    }
}

impl Widget for Traits {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        TitledBox::new("Traits")
            .rounding(CornerRadius::same(16))
            .header_ratio(0.035)
            .show(ui, |ui| {
                let available = ui.available_size();
                let pad = available.y * 0.02;
                let max_visible = 3.0_f32;
                let card_width = (available.x - pad * (max_visible + 1.0)) / max_visible;
                let card_height = available.y;

                egui::ScrollArea::horizontal().show(ui, |ui| {
                    ui.spacing_mut().item_spacing = egui::vec2(pad, 0.0);
                    ui.add_space(pad);

                    for entry in &self.entries {
                        let (rect, response) = ui.allocate_exact_size(
                            egui::vec2(card_width, card_height),
                            egui::Sense::hover(),
                        );

                        let painter = ui.painter();

                        // Card background (no stroke)
                        ShapeBox::new(Shape::Rectangle)
                            .fill(MAIN_COLOR)
                            .stroke(Stroke::NONE)
                            .set_rounding(CornerRadius::same(12))
                            .paint(painter, rect);

                        let inner_pad = 8.0;
                        let inner = rect.shrink(inner_pad);

                        // Trait name at top
                        let name_height = 22.0;
                        let name_rect = egui::Rect::from_min_size(
                            inner.min,
                            egui::vec2(inner.width(), name_height),
                        );
                        Text::new(&entry.name)
                            .color(TEXT_COLOR)
                            .size(16.0)
                            .bold()
                            .align(Align2::LEFT_TOP)
                            .paint(painter, name_rect);

                        // Description with text wrapping
                        let desc_top = name_rect.max.y + 6.0;
                        let desc_rect =
                            egui::Rect::from_min_max(egui::pos2(inner.min.x, desc_top), inner.max);
                        let font = egui::FontId::proportional(14.0);
                        let galley = painter.layout(
                            entry.description.clone(),
                            font,
                            TEXT_COLOR,
                            desc_rect.width(),
                        );
                        let clipped = painter.with_clip_rect(desc_rect);
                        clipped.galley(desc_rect.min, galley, TEXT_COLOR);

                        // Effects tooltip on hover
                        if response.hovered() && !entry.effects.is_empty() {
                            let tooltip_pos = response.hover_pos().unwrap_or(rect.right_bottom())
                                + egui::vec2(8.0, 8.0);
                            egui::Area::new(response.id.with("tooltip"))
                                .order(egui::Order::Tooltip)
                                .fixed_pos(tooltip_pos)
                                .show(ui.ctx(), |ui| {
                                    egui::Frame::NONE
                                        .fill(MAIN_COLOR)
                                        .stroke(Stroke::new(0.5, STROKE_COLOR))
                                        .corner_radius(CornerRadius::same(8))
                                        .inner_margin(8.0)
                                        .show(ui, |ui| {
                                            ui.label(
                                                egui::RichText::new(&entry.name)
                                                    .strong()
                                                    .size(14.0)
                                                    .color(TEXT_COLOR),
                                            );
                                            ui.separator();
                                            for effect in &entry.effects {
                                                ui.label(
                                                    egui::RichText::new(effect.as_str())
                                                        .size(12.0)
                                                        .color(TEXT_COLOR),
                                                );
                                            }
                                        });
                                });
                        }
                    }
                });
            })
    }
}
