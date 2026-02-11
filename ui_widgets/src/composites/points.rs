use crate::atoms::{Shape, ShapeBox, Text};
use crate::colors::{ERROR_COLOR, MAIN_COLOR, SECONDARY_COLOR, STROKE_COLOR, TEXT_COLOR};
use crate::egui::{self, Align2, CornerRadius, Rect, Stroke, Widget};
use crate::traits::{Roundable, WithText};

/// Response from an editable Points widget.
pub struct PointsResponse {
    pub characteristic_points: u32,
    pub skill_points: i32,
}

/// Displays two point pools side by side: Characteristic points and Skill points.
pub struct Points {
    characteristic_points: u32,
    skill_points: i32,
    editable: bool,
}

impl Points {
    pub fn new(characteristic_points: u32, skill_points: i32) -> Self {
        Self {
            characteristic_points,
            skill_points,
            editable: false,
        }
    }

    pub fn editable(mut self, editable: bool) -> Self {
        self.editable = editable;
        self
    }

    /// Renders the widget and returns the (possibly edited) values.
    pub fn show(self, ui: &mut egui::Ui) -> PointsResponse {
        let available_width = ui.available_width();
        let available_height = ui.available_height();

        let spacing_x = 4.0;
        let item_width = (available_width - spacing_x) / 2.0;

        let mut char_pts = self.characteristic_points as i32;
        let mut skill_pts = self.skill_points;

        let items: [(&str, &mut i32, CornerRadius); 2] = [
            (
                "Characteristic points",
                &mut char_pts,
                CornerRadius::same(12),
            ),
            ("Skill points", &mut skill_pts, CornerRadius::same(12)),
        ];

        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing = egui::vec2(spacing_x, 0.0);

            for (label, value, bg_rounding) in items {
                let (rect, _) = ui.allocate_exact_size(
                    egui::vec2(item_width, available_height),
                    egui::Sense::hover(),
                );

                let painter = ui.painter();

                // Background
                let clipped = painter.with_clip_rect(rect);
                clipped.rect_filled(rect, bg_rounding, SECONDARY_COLOR);

                // Label text on the left
                let text_x = rect.min.x + rect.width() * 0.07;
                let text_rect = Rect::from_min_max(egui::pos2(text_x, rect.min.y), rect.max);
                Text::new(label)
                    .color(TEXT_COLOR)
                    .size(12.0)
                    .align(Align2::LEFT_CENTER)
                    .paint(painter, text_rect);

                // Square box on the right with the value
                let pad = rect.width() * 0.025;
                let box_side = rect.height() - pad * 2.0;
                let box_rect = Rect::from_min_size(
                    egui::pos2(rect.max.x - pad - box_side, rect.min.y + pad),
                    egui::vec2(box_side, box_side),
                );

                if self.editable {
                    let id = ui.id().with("pts_edit").with(label);
                    let prev_id = id.with("prev_val");

                    // Detect external value changes by comparing with last-stored value
                    let prev_val: i32 = ui.data(|d| d.get_temp(prev_id)).unwrap_or(*value);
                    let mut text: String = if prev_val != *value {
                        // External change — sync text to new value
                        value.to_string()
                    } else {
                        ui.data(|d| d.get_temp::<String>(id))
                            .unwrap_or_else(|| value.to_string())
                    };

                    // Measure text width to size the box dynamically
                    let font = egui::FontId::proportional(12.0);
                    let galley = painter.layout_no_wrap(text.clone(), font.clone(), TEXT_COLOR);
                    let text_w = galley.size().x;
                    let h_pad = 8.0;
                    let box_w = text_w.max(box_side) + h_pad * 2.0;
                    let box_rect = Rect::from_min_size(
                        egui::pos2(rect.max.x - pad - box_w, rect.min.y + pad),
                        egui::vec2(box_w, box_side),
                    );

                    // Draw ShapeBox-style background
                    let box_fill = if *value < 0 { ERROR_COLOR } else { MAIN_COLOR };
                    painter.rect_filled(box_rect, CornerRadius::same(8), box_fill);
                    painter.rect_stroke(
                        box_rect,
                        CornerRadius::same(8),
                        Stroke::new(1.0, STROKE_COLOR),
                        egui::StrokeKind::Inside,
                    );

                    // Text input inside the box
                    let inset = box_rect.shrink(3.0);
                    let mut child = ui.new_child(egui::UiBuilder::new().max_rect(inset).layout(
                        egui::Layout::centered_and_justified(egui::Direction::LeftToRight),
                    ));

                    let edit = egui::TextEdit::singleline(&mut text)
                        .horizontal_align(egui::Align::Center)
                        .font(font)
                        .text_color(TEXT_COLOR)
                        .frame(false)
                        .margin(egui::Margin::ZERO);

                    if child.add(edit).changed() {
                        text.retain(|c| c.is_ascii_digit() || c == '-');
                        if let Ok(v) = text.parse::<i32>() {
                            *value = v;
                        } else if text.is_empty() {
                            *value = 0;
                        }
                    }

                    ui.data_mut(|d| {
                        d.insert_temp(id, text);
                        d.insert_temp(prev_id, *value);
                    });
                } else {
                    let shape = ShapeBox::new(Shape::Rectangle)
                        .fill(MAIN_COLOR)
                        .stroke(Stroke::new(1.0, STROKE_COLOR))
                        .set_text(value.to_string())
                        .set_text_color(TEXT_COLOR)
                        .set_rounding(CornerRadius::same(8));
                    shape.paint(painter, box_rect);
                }
            }
        });

        PointsResponse {
            characteristic_points: char_pts.max(0) as u32,
            skill_points: skill_pts,
        }
    }
}

impl Widget for Points {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let size = egui::vec2(ui.available_width(), ui.available_height());
        let (rect, response) = ui.allocate_exact_size(size, egui::Sense::hover());
        let mut child = ui.new_child(
            egui::UiBuilder::new()
                .max_rect(rect)
                .layout(egui::Layout::top_down(egui::Align::Min)),
        );
        self.show(&mut child);
        response
    }
}
