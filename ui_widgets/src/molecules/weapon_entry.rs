use crate::atoms::{Shape, ShapeBox, Text};
use crate::colors::{MAIN_COLOR, TEXT_COLOR};
use crate::egui::{self, pos2, vec2, Align2, CornerRadius, Rect, Stroke, TextureId};
use crate::traits::Roundable;

const FONT_SIZE: f32 = 14.0;
const TITLE_FONT_SIZE: f32 = 16.0;

/// A single weapon slot rendered as a rounded rectangle with icon and stats.
pub struct WeaponEntry {
    icon: TextureId,
    name: String,
    kind: String,
    attack: String,
    damage: String,
    range: String,
}

impl WeaponEntry {
    pub fn new(icon: TextureId) -> Self {
        Self {
            icon,
            name: String::new(),
            kind: String::new(),
            attack: String::new(),
            damage: String::new(),
            range: String::new(),
        }
    }

    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }

    pub fn kind(mut self, kind: impl Into<String>) -> Self {
        self.kind = kind.into();
        self
    }

    pub fn attack(mut self, attack: impl Into<String>) -> Self {
        self.attack = attack.into();
        self
    }

    pub fn damage(mut self, damage: impl Into<String>) -> Self {
        self.damage = damage.into();
        self
    }

    pub fn range(mut self, range: impl Into<String>) -> Self {
        self.range = range.into();
        self
    }

    pub fn paint(&self, painter: &egui::Painter, rect: Rect) {
        let shape = ShapeBox::new(Shape::Rectangle)
            .fill(MAIN_COLOR)
            .stroke(Stroke::NONE)
            .set_rounding(CornerRadius::same(14));
        shape.paint(painter, rect);

        let pad = rect.height() * 0.037;
        let icon_size = rect.height() - pad * 2.0;
        let icon_rect = Rect::from_min_size(
            pos2(rect.min.x + pad, rect.min.y + pad),
            vec2(icon_size, icon_size),
        );
        let uv = Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0));
        painter.add(
            egui::epaint::RectShape::filled(
                icon_rect,
                CornerRadius::same(12),
                egui::Color32::WHITE,
            )
            .with_texture(self.icon, uv),
        );

        // Text area to the right of icon (gap = 8x icon padding)
        let text_left = icon_rect.max.x + pad * 8.0;
        let text_rect = Rect::from_min_max(
            pos2(text_left, rect.min.y + pad),
            pos2(rect.max.x - pad * 8.0, rect.max.y - pad),
        );

        if self.name.is_empty() {
            Text::new("Not selected")
                .color(TEXT_COLOR)
                .size(FONT_SIZE)
                .align(Align2::LEFT_CENTER)
                .paint(painter, text_rect);
        } else {
            let half_h = text_rect.height() * 0.5;

            // Row 1: name (left, bold) + type (right, normal)
            let row1_rect = Rect::from_min_size(text_rect.min, vec2(text_rect.width(), half_h));
            Text::new(&self.name)
                .color(TEXT_COLOR)
                .size(TITLE_FONT_SIZE)
                .bold()
                .align(Align2::LEFT_CENTER)
                .paint(painter, row1_rect);
            if !self.kind.is_empty() {
                Text::new(&format!("Type: {}", self.kind))
                    .color(TEXT_COLOR)
                    .size(FONT_SIZE)
                    .align(Align2::RIGHT_CENTER)
                    .paint(painter, row1_rect);
            }

            // Row 2: attack (left)  damage (center)  range (right)
            let stats_rect = Rect::from_min_size(
                pos2(text_rect.min.x, text_rect.min.y + half_h),
                vec2(text_rect.width(), half_h),
            );
            if !self.attack.is_empty() {
                Text::new(&format!("Attack: {}", self.attack))
                    .color(TEXT_COLOR)
                    .size(FONT_SIZE)
                    .align(Align2::LEFT_CENTER)
                    .paint(painter, stats_rect);
            }
            if !self.damage.is_empty() {
                Text::new(&format!("Damage: {}", self.damage))
                    .color(TEXT_COLOR)
                    .size(FONT_SIZE)
                    .align(Align2::CENTER_CENTER)
                    .paint(painter, stats_rect);
            }
            if !self.range.is_empty() {
                Text::new(&format!("Range: {}", self.range))
                    .color(TEXT_COLOR)
                    .size(FONT_SIZE)
                    .align(Align2::RIGHT_CENTER)
                    .paint(painter, stats_rect);
            }
        }
    }
}
