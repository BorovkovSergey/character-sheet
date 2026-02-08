use crate::colors::{MAIN_COLOR, SECONDARY_COLOR};
use crate::egui::{self, Widget};
use crate::molecules::TitledBox;

/// Displays the character's derived stats (HP, MP, etc.).
pub struct Stats;

impl Stats {
    pub fn new() -> Self {
        Self
    }
}

impl Widget for Stats {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let width = ui.available_width();
        let height = ui.available_height();

        let top_h = height * 0.54;
        let bottom_h = height * 0.44;
        let gap = height * 0.02;

        ui.vertical(|ui| {
            ui.spacing_mut().item_spacing = egui::vec2(0.0, 0.0);

            ui.add_sized(
                [width, top_h],
                TitledBox::new("Stats")
                    .fill(SECONDARY_COLOR)
                    .rounding(16)
                    .content_fill(MAIN_COLOR)
                    .content_rounding(14),
            );

            ui.add_space(gap);

            ui.add_sized(
                [width, bottom_h],
                TitledBox::new("Stats 2")
                    .fill(SECONDARY_COLOR)
                    .rounding(16)
                    .content_fill(MAIN_COLOR)
                    .content_rounding(14),
            );
        })
        .response
    }
}
