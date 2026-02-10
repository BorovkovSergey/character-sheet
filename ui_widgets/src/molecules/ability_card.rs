use crate::atoms::{Shape, ShapeBox, Text};
use crate::colors::{MAIN_COLOR, STROKE_COLOR, TEXT_COLOR};
use crate::egui::{self, Align2, Color32, CornerRadius, Rect, Stroke, TextureId};
use crate::traits::{Roundable, WithText};

/// A single ability card split into two equal horizontal rows:
/// - Top row: square ability image (left-aligned) + info column to the right
/// - Bottom row: ability description text (wrapped, clipped)
///
/// A mana-cost badge sits at the bottom-right of the image, protruding
/// 10% beyond the image's right edge.
pub struct AbilityCard {
    image: TextureId,
    description: String,
    name: String,
    mp_cost: Option<u32>,
    ap_cost: Option<u32>,
    self_only: bool,
    range: Option<u32>,
    ability_type: String,
    check: String,
    enemy_check: String,
}

impl AbilityCard {
    pub fn new(image: TextureId, description: impl Into<String>) -> Self {
        Self {
            image,
            description: description.into(),
            name: String::new(),
            mp_cost: None,
            ap_cost: None,
            self_only: false,
            range: None,
            ability_type: String::new(),
            check: String::new(),
            enemy_check: String::new(),
        }
    }

    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }

    pub fn mp_cost(mut self, cost: Option<u32>) -> Self {
        self.mp_cost = cost;
        self
    }

    pub fn ap_cost(mut self, cost: Option<u32>) -> Self {
        self.ap_cost = cost;
        self
    }

    pub fn self_only(mut self, self_only: bool) -> Self {
        self.self_only = self_only;
        self
    }

    pub fn range(mut self, range: Option<u32>) -> Self {
        self.range = range;
        self
    }

    pub fn ability_type(mut self, t: impl Into<String>) -> Self {
        self.ability_type = t.into();
        self
    }

    pub fn check(mut self, check: impl Into<String>) -> Self {
        self.check = check.into();
        self
    }

    pub fn enemy_check(mut self, enemy_check: impl Into<String>) -> Self {
        self.enemy_check = enemy_check.into();
        self
    }

    /// Paints the ability card into the given rect.
    pub fn paint(self, painter: &egui::Painter, rect: Rect) {
        // Card background
        ShapeBox::new(Shape::Rectangle)
            .fill(Color32::WHITE)
            .stroke(Stroke::NONE)
            .set_rounding(CornerRadius::same(12))
            .paint(painter, rect);

        let half_h = rect.height() * 0.5;
        let pad = rect.width() * 0.02;

        // Top row: square image, left-aligned with padding
        let inner_top = Rect::from_min_max(
            egui::pos2(rect.min.x + pad, rect.min.y + pad),
            egui::pos2(rect.max.x - pad, rect.min.y + half_h - pad * 0.5),
        );
        let icon_size = inner_top.height();
        let inner_image = Rect::from_min_size(inner_top.min, egui::vec2(icon_size, icon_size));
        let uv = Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0));
        painter.add(
            egui::epaint::RectShape::filled(inner_image, CornerRadius::same(12), Color32::WHITE)
                .with_texture(self.image, uv),
        );

        // Mana cost badge (bottom-right of visible image, bottom aligned with image)
        if let Some(mp) = self.mp_cost {
            let badge_side = icon_size * 0.40;
            let badge_left = inner_image.right() - badge_side * 0.9;
            let badge_top = inner_image.bottom() - badge_side;
            let badge_rect = Rect::from_min_size(
                egui::pos2(badge_left, badge_top),
                egui::vec2(badge_side, badge_side),
            );

            ShapeBox::new(Shape::Rectangle)
                .fill(MAIN_COLOR)
                .stroke(Stroke::new(1.0, STROKE_COLOR))
                .set_rounding(CornerRadius::same(8))
                .set_text(mp.to_string())
                .set_text_color(TEXT_COLOR)
                .set_text_align(Align2::CENTER_CENTER)
                .paint(painter, badge_rect);
        }

        // Info column: to the right of the image
        let info_left = inner_image.right() + pad;
        let info_rect = Rect::from_min_max(egui::pos2(info_left, inner_top.min.y), inner_top.max);
        let info_clipped = painter.with_clip_rect(info_rect);
        let line_h = info_rect.height() / 3.0;

        // Line 1: ability name (bold)
        let name_rect = Rect::from_min_size(info_rect.min, egui::vec2(info_rect.width(), line_h));
        let font_size = line_h * 0.62;
        Text::new(&self.name)
            .color(TEXT_COLOR)
            .size(font_size + font_size * 0.25)
            .bold()
            .align(Align2::LEFT_CENTER)
            .paint(&info_clipped, name_rect);

        // Line 2: AP cost + ability type
        let line2_y = info_rect.min.y + line_h;
        let line2_rect = Rect::from_min_size(
            egui::pos2(info_rect.min.x, line2_y),
            egui::vec2(info_rect.width(), line_h),
        );
        let is_passive = self.ability_type == "Passive";
        let mut line2_parts = Vec::new();
        if let Some(ap) = self.ap_cost {
            line2_parts.push(format!("AP: {ap}"));
        }
        if self.self_only {
            line2_parts.push("Self".to_string());
        } else if let Some(r) = self.range {
            line2_parts.push(format!("Range: {r}"));
        }
        if !self.ability_type.is_empty() {
            line2_parts.push(self.ability_type);
        }
        let line2_text = line2_parts.join("     ");
        Text::new(line2_text)
            .color(TEXT_COLOR)
            .size(font_size)
            .align(Align2::LEFT_CENTER)
            .paint(&info_clipped, line2_rect);

        // Line 3: "Check: " (regular) + value (bold) — skip for Passive
        if !is_passive {
            let line3_y = info_rect.min.y + line_h * 2.0;
            let line3_rect = Rect::from_min_size(
                egui::pos2(info_rect.min.x, line3_y),
                egui::vec2(info_rect.width(), line_h),
            );
            let prefix = "Check: ";
            Text::new(prefix)
                .color(TEXT_COLOR)
                .size(font_size)
                .align(Align2::LEFT_CENTER)
                .paint(&info_clipped, line3_rect);
            let prefix_width = info_clipped
                .layout_no_wrap(
                    prefix.to_string(),
                    egui::FontId::proportional(font_size),
                    TEXT_COLOR,
                )
                .size()
                .x;
            let value_rect = Rect::from_min_size(
                egui::pos2(line3_rect.min.x + prefix_width, line3_rect.min.y),
                egui::vec2(line3_rect.width() - prefix_width, line3_rect.height()),
            );
            let check_value = if self.check.is_empty() {
                "None".to_string()
            } else if self.enemy_check.is_empty() {
                self.check
            } else {
                format!("{} vs {}", self.check, self.enemy_check)
            };
            Text::new(check_value)
                .color(TEXT_COLOR)
                .size(font_size)
                .bold()
                .align(Align2::LEFT_CENTER)
                .paint(&info_clipped, value_rect);
        }

        // Bottom row: description with padding
        let inner_desc = Rect::from_min_max(
            egui::pos2(rect.min.x + pad, rect.min.y + half_h + pad * 0.5),
            egui::pos2(rect.max.x - pad, rect.max.y - pad),
        );
        let font = egui::FontId::proportional(12.0);
        let galley = painter.layout(self.description, font, TEXT_COLOR, inner_desc.width());
        let clipped = painter.with_clip_rect(inner_desc);
        clipped.galley(inner_desc.min, galley, TEXT_COLOR);
    }
}
