use crate::colors::{MAIN_COLOR, STROKE_COLOR};
use crate::egui::{self, CornerRadius, Rect, Stroke, Widget};
use crate::molecules::{LabeledValue, TitledBox};

/// A single skill entry for display.
pub struct SkillEntry {
    pub name: String,
    pub dependency: String,
    pub level: i32,
}

/// Displays the character's learned skills and their levels.
///
/// Renders as a [`TitledBox`] with vertical "Skills" label on the left.
/// The content area contains a 3x4 grid of [`LabeledValue`] rows.
pub struct Skills {
    entries: Vec<SkillEntry>,
}

impl Skills {
    pub fn new(entries: Vec<SkillEntry>) -> Self {
        Self { entries }
    }
}

impl Widget for Skills {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        TitledBox::new("Skills")
            .rounding(CornerRadius::same(16))
            .header_ratio(0.035)
            .content_fill(MAIN_COLOR)
            .content_rounding(CornerRadius::same(14))
            .show(ui, |ui| {
                let available = ui.available_size();
                let cols = 3;
                let rows = 4;
                let gap = available.y * 0.02;
                let cell_width = (available.x - gap * (cols as f32 + 1.0)) / cols as f32;
                let cell_height = (available.y - gap * (rows as f32 + 1.0)) / rows as f32;
                let origin = ui.min_rect().min;
                let painter = ui.painter();

                for (i, entry) in self.entries.iter().take(cols * rows).enumerate() {
                    let col = i % cols;
                    let row = i / cols;
                    let x = origin.x + gap + (cell_width + gap) * col as f32;
                    let y = origin.y + gap + (cell_height + gap) * row as f32;
                    let cell_rect =
                        Rect::from_min_size(egui::pos2(x, y), egui::vec2(cell_width, cell_height));

                    LabeledValue::new(&entry.name, &entry.dependency, &entry.level.to_string())
                        .text_size(14.0)
                        .stroke(Stroke::new(1.0, STROKE_COLOR))
                        .rounding(CornerRadius::same(12))
                        .box_rounding(CornerRadius::same(10))
                        .paint(painter, cell_rect);
                }
            })
    }
}
