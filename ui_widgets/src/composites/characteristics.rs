use std::cell::Cell;

use crate::atoms::{Shape, ShapeBox, Text};
use crate::colors::{MAIN_COLOR, SECONDARY_COLOR, TEXT_COLOR, UPGRADE_COLOR};
use crate::egui::{self, Align2, CornerRadius, Rect, Stroke};
use crate::traits::{Roundable, WithText};

/// Action returned by grid widgets (Characteristics, Skills) in edit mode.
#[derive(Clone, Copy)]
pub enum GridAction {
    Upgrade(usize),
    Downgrade(usize),
}

/// Displays the character's primary characteristics as 8 boxes
/// arranged in 2 rows of 4.
///
/// Each box has the characteristic name on the left and a square ShapeBox
/// with the value on the right.
///
/// Uses `Vec<(String, u32)>` rather than `BTreeMap` to preserve the caller's
/// intended display order (e.g. STR, DEX, END, ...), which does not match
/// alphabetical sorting.
pub struct Characteristics {
    values: Vec<(String, u32)>,
    edit_mode: bool,
    available_points: u32,
}

impl Characteristics {
    pub fn new(values: Vec<(String, u32)>) -> Self {
        Self {
            values,
            edit_mode: false,
            available_points: 0,
        }
    }

    pub fn edit_mode(mut self, enabled: bool, available_points: u32) -> Self {
        self.edit_mode = enabled;
        self.available_points = available_points;
        self
    }

    /// Renders the characteristics grid. Returns `Some(GridAction)` if a
    /// characteristic was clicked in edit mode.
    pub fn show(self, ui: &mut egui::Ui) -> Option<GridAction> {
        let available_width = ui.available_width();
        let available_height = ui.available_height();

        let cols = 4.0;
        let rows = 2.0;
        let spacing_x = 4.0;
        let spacing_y = 4.0;
        let item_width = (available_width - spacing_x * (cols - 1.0)) / cols;
        let item_height = (available_height - spacing_y * (rows - 1.0)) / rows;

        let total = self.values.len();
        let clicked: Cell<Option<GridAction>> = Cell::new(None);

        ui.vertical(|ui| {
            ui.spacing_mut().item_spacing = egui::vec2(0.0, 0.0);

            let mut idx = 0;
            for row in self.values.chunks(4) {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing = egui::vec2(spacing_x, 0.0);
                    for (label, value) in row {
                        let sense = if self.edit_mode {
                            egui::Sense::click()
                        } else {
                            egui::Sense::hover()
                        };
                        let (rect, response) =
                            ui.allocate_exact_size(egui::vec2(item_width, item_height), sense);

                        let cost = value + 1;
                        let can_upgrade = self.edit_mode && self.available_points >= cost;

                        if can_upgrade && response.clicked() {
                            clicked.set(Some(GridAction::Upgrade(idx)));
                        }
                        if self.edit_mode && response.secondary_clicked() {
                            clicked.set(Some(GridAction::Downgrade(idx)));
                        }

                        let painter = ui.painter();

                        // Base rounding 8, with 20 on the four outer corners
                        let mut rounding = CornerRadius::same(8);
                        if idx == 0 {
                            rounding.nw = 20;
                        }
                        if idx == 3 {
                            rounding.ne = 20;
                        }
                        if idx == total - 4 {
                            rounding.sw = 20;
                        }
                        if idx == total - 1 {
                            rounding.se = 20;
                        }

                        // Background — green if upgradeable
                        let bg = if can_upgrade {
                            UPGRADE_COLOR
                        } else {
                            SECONDARY_COLOR
                        };
                        let clipped = painter.with_clip_rect(rect);
                        clipped.rect_filled(rect, rounding, bg);

                        // Label text on the left, 7% padding from left edge
                        let text_x = rect.min.x + rect.width() * 0.07;
                        let text_rect =
                            Rect::from_min_max(egui::pos2(text_x, rect.min.y), rect.max);
                        Text::new(label)
                            .color(TEXT_COLOR)
                            .size(12.0)
                            .align(Align2::LEFT_CENTER)
                            .paint(painter, text_rect);

                        // Square ShapeBox on the right, 2.5% of width padding
                        let pad = rect.width() * 0.025;
                        let box_side = rect.height() - pad * 2.0;
                        let box_rect = Rect::from_min_size(
                            egui::pos2(rect.max.x - pad - box_side, rect.min.y + pad),
                            egui::vec2(box_side, box_side),
                        );

                        let shape = ShapeBox::new(Shape::Rectangle)
                            .fill(MAIN_COLOR)
                            .stroke(Stroke::NONE)
                            .set_text(value.to_string())
                            .set_text_color(TEXT_COLOR)
                            .set_rounding(CornerRadius::same(12));
                        shape.paint(painter, box_rect);

                        idx += 1;
                    }
                });
                ui.add_space(spacing_y);
            }
        });

        clicked.get()
    }
}
