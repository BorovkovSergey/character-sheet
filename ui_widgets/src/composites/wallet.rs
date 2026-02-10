use crate::atoms::{Shape, ShapeBox};
use crate::colors::{MAIN_COLOR, SECONDARY_COLOR, STROKE_COLOR, TEXT_COLOR};
use crate::egui::{self, Color32, CornerRadius, Stroke, TextureId};
use crate::molecules::{TitlePosition, TitledBox};
use crate::traits::{Roundable, WithText};

/// Result of rendering the wallet. Each field is `Some(delta)` when
/// the corresponding currency cell was clicked.
/// Left-click = positive delta, right-click = negative delta.
pub struct WalletResponse {
    pub gold: Option<i64>,
    pub silver: Option<i64>,
    pub copper: Option<i64>,
}

/// A single currency entry (label, amount, icon).
struct CurrencyEntry {
    label: &'static str,
    amount: u32,
    icon: TextureId,
    /// Value of one unit in base currency.
    unit_value: i64,
}

/// Displays the character's currency and wealth.
pub struct Wallet {
    gold: u32,
    silver: u32,
    copper: u32,
    gold_icon: TextureId,
    silver_icon: TextureId,
    copper_icon: TextureId,
}

impl Wallet {
    pub fn new(
        gold: u32,
        silver: u32,
        copper: u32,
        gold_icon: TextureId,
        silver_icon: TextureId,
        copper_icon: TextureId,
    ) -> Self {
        Self {
            gold,
            silver,
            copper,
            gold_icon,
            silver_icon,
            copper_icon,
        }
    }

    pub fn show(self, ui: &mut egui::Ui) -> WalletResponse {
        let mut response = WalletResponse {
            gold: None,
            silver: None,
            copper: None,
        };

        let entries = [
            CurrencyEntry {
                label: "Gold",
                amount: self.gold,
                icon: self.gold_icon,
                unit_value: 1000,
            },
            CurrencyEntry {
                label: "Silver",
                amount: self.silver,
                icon: self.silver_icon,
                unit_value: 10,
            },
            CurrencyEntry {
                label: "Copper",
                amount: self.copper,
                icon: self.copper_icon,
                unit_value: 1,
            },
        ];

        TitledBox::new("Wallet")
            .fill(SECONDARY_COLOR)
            .rounding(16)
            .content_fill(MAIN_COLOR)
            .content_rounding(14)
            .show(ui, |ui| {
                let results = inner_currency_boxes(ui, &entries);
                response.gold = results[0];
                response.silver = results[1];
                response.copper = results[2];
            });

        response
    }
}

/// Lays out a row of equally-spaced currency [`TitledBox`] widgets.
/// Returns a delta for each entry: `Some(+unit)` on left-click, `Some(-unit)` on right-click.
fn inner_currency_boxes(ui: &mut egui::Ui, entries: &[CurrencyEntry]) -> Vec<Option<i64>> {
    let count = entries.len() as f32;
    let spacing = 4.0;
    let available_width = ui.available_width();
    let available_height = ui.available_height();
    let pad_x = available_width * 0.015;
    let pad_y = available_height * 0.015;
    let inner_width = available_width - pad_x * 2.0;
    let inner_height = available_height - pad_y * 2.0;
    let item_width = (inner_width - spacing * (count - 1.0)) / count;

    let mut results = vec![None; entries.len()];

    ui.vertical(|ui| {
        ui.add_space(pad_y);
        ui.horizontal(|ui| {
            ui.add_space(pad_x);
            ui.spacing_mut().item_spacing = egui::vec2(spacing, 0.0);
            for (i, entry) in entries.iter().enumerate() {
                let text = entry.amount.to_string();

                ui.allocate_ui_with_layout(
                    egui::vec2(item_width, inner_height),
                    egui::Layout::left_to_right(egui::Align::Center),
                    |ui| {
                        TitledBox::new(entry.label)
                            .title_position(TitlePosition::Top)
                            .fill(Color32::TRANSPARENT)
                            .rounding(8)
                            .show(ui, |ui| {
                                let available = ui.available_size();
                                let (rect, resp) =
                                    ui.allocate_exact_size(available, egui::Sense::click());

                                let shape = ShapeBox::new(Shape::Rectangle)
                                    .fill(Color32::TRANSPARENT)
                                    .stroke(Stroke::new(1.0, STROKE_COLOR))
                                    .icon(entry.icon)
                                    .set_text(text.clone())
                                    .set_text_color(TEXT_COLOR)
                                    .set_rounding(CornerRadius::same(12));
                                shape.paint(ui.painter(), rect);

                                if resp.clicked() {
                                    results[i] = Some(entry.unit_value);
                                } else if resp.secondary_clicked() {
                                    results[i] = Some(-entry.unit_value);
                                }
                            });
                    },
                );
            }
        });
    });

    results
}
