use crate::egui::{self, CornerRadius, TextureId};
use crate::molecules::{AbilityCard, TitledBox};
use crate::styles::UiStyle;

/// A single ability entry for display.
pub struct AbilityEntry {
    pub name: String,
    pub description: String,
    pub image: TextureId,
    pub mp_cost: Option<u32>,
    pub ap_cost: Option<u32>,
    pub self_only: bool,
    pub range: Option<u32>,
    pub ability_type: String,
    pub check: String,
    pub enemy_check: String,
}

/// Displays the character's learned abilities as a 2-column grid of cards.
///
/// Returns `Some(new_mp)` when an ability is clicked and enough mana is available.
pub struct Abilities {
    entries: Vec<AbilityEntry>,
    current_mp: u32,
}

impl Abilities {
    pub fn new(entries: Vec<AbilityEntry>, current_mp: u32) -> Self {
        Self {
            entries,
            current_mp,
        }
    }

    pub fn show(self, ui: &mut egui::Ui) -> Option<u32> {
        let mut result = None;

        TitledBox::new("Abilities")
            .rounding(CornerRadius::same(16))
            .header_ratio(0.035)
            .show(ui, |ui| {
                if self.entries.is_empty() {
                    return;
                }

                let available = ui.available_size();
                let pad = UiStyle::content_padding(ui);
                let cols = 2usize;
                let visible_rows = 2usize;
                let card_width =
                    (available.x - pad * (cols as f32 + 1.0)) / cols as f32;
                let card_height =
                    (available.y - pad * (visible_rows as f32 + 1.0)) / visible_rows as f32;

                let actual_rows = self.entries.len().div_ceil(cols);
                let total_height = pad + (card_height + pad) * actual_rows as f32;

                egui::ScrollArea::vertical()
                    .id_salt("abilities_scroll")
                    .auto_shrink(false)
                    .show(ui, |ui| {
                        let (content_rect, _) = ui.allocate_exact_size(
                            egui::vec2(available.x, total_height),
                            egui::Sense::hover(),
                        );
                        let origin = content_rect.min;
                        let painter = ui.painter();

                        for (i, entry) in self.entries.into_iter().enumerate() {
                            let col = i % cols;
                            let row = i / cols;
                            let x = origin.x + pad + (card_width + pad) * col as f32;
                            let y = origin.y + pad + (card_height + pad) * row as f32;
                            let rect = egui::Rect::from_min_size(
                                egui::pos2(x, y),
                                egui::vec2(card_width, card_height),
                            );

                            let mp_cost = entry.mp_cost.unwrap_or(0);

                            AbilityCard::new(entry.image, entry.description)
                                .name(entry.name)
                                .mp_cost(entry.mp_cost)
                                .ap_cost(entry.ap_cost)
                                .self_only(entry.self_only)
                                .range(entry.range)
                                .ability_type(entry.ability_type)
                                .check(entry.check)
                                .enemy_check(entry.enemy_check)
                                .paint(painter, rect);

                            let card_id = ui.id().with("ability_card").with(i);
                            let response =
                                ui.interact(rect, card_id, egui::Sense::click());
                            if response.clicked() && self.current_mp >= mp_cost {
                                result = Some(self.current_mp - mp_cost);
                            }
                        }
                    });
            });

        result
    }
}
