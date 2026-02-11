use std::cell::Cell;

use crate::colors::{ERROR_COLOR, MAIN_COLOR, STROKE_COLOR, UPGRADE_COLOR};
use crate::composites::GridAction;
use crate::egui::{self, CornerRadius, Rect, Stroke};
use crate::molecules::{LabeledValue, TitledBox};

/// A single skill entry for display.
pub struct SkillEntry {
    pub name: String,
    pub dependency: String,
    pub level: i32,
    /// Max level this skill can reach (dependency characteristic level).
    pub max_level: u32,
}

/// Displays the character's learned skills and their levels.
///
/// Renders as a [`TitledBox`] with vertical "Skills" label on the left.
/// The content area contains a 3x4 grid of [`LabeledValue`] rows.
pub struct Skills {
    entries: Vec<SkillEntry>,
    edit_mode: bool,
    available_points: u32,
}

impl Skills {
    pub fn new(entries: Vec<SkillEntry>) -> Self {
        Self {
            entries,
            edit_mode: false,
            available_points: 0,
        }
    }

    pub fn edit_mode(mut self, enabled: bool, available_points: u32) -> Self {
        self.edit_mode = enabled;
        self.available_points = available_points;
        self
    }

    /// Renders the skills grid. Returns `Some(GridAction)` if a skill was clicked in edit mode.
    pub fn show(self, ui: &mut egui::Ui) -> Option<GridAction> {
        let clicked: Cell<Option<GridAction>> = Cell::new(None);
        let edit_mode = self.edit_mode;
        let available_points = self.available_points;

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

                for (i, entry) in self.entries.iter().take(cols * rows).enumerate() {
                    let col = i % cols;
                    let row = i / cols;
                    let x = origin.x + gap + (cell_width + gap) * col as f32;
                    let y = origin.y + gap + (cell_height + gap) * row as f32;
                    let cell_rect =
                        Rect::from_min_size(egui::pos2(x, y), egui::vec2(cell_width, cell_height));

                    let cost = entry.level as u32 + 1;
                    let over_limit = entry.level > 0 && entry.level as u32 > entry.max_level;
                    let can_upgrade = edit_mode
                        && available_points >= cost
                        && (entry.level as u32) < entry.max_level;

                    // Draw highlight background
                    if over_limit {
                        let painter = ui.painter();
                        painter.rect_filled(cell_rect, CornerRadius::same(12), ERROR_COLOR);
                    } else if can_upgrade {
                        let painter = ui.painter();
                        painter.rect_filled(cell_rect, CornerRadius::same(12), UPGRADE_COLOR);
                    }

                    LabeledValue::new(&entry.name, &entry.dependency, &entry.level.to_string())
                        .text_size(14.0)
                        .stroke(Stroke::new(1.0, STROKE_COLOR))
                        .rounding(CornerRadius::same(12))
                        .box_rounding(CornerRadius::same(10))
                        .paint(ui.painter(), cell_rect);

                    // Handle clicks in edit mode
                    if edit_mode {
                        let response = ui.allocate_rect(cell_rect, egui::Sense::click());
                        if can_upgrade && response.clicked() {
                            clicked.set(Some(GridAction::Upgrade(i)));
                        }
                        if response.secondary_clicked() {
                            clicked.set(Some(GridAction::Downgrade(i)));
                        }
                    }
                }
            });

        clicked.get()
    }
}
