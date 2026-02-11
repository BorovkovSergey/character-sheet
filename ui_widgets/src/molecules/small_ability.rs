use crate::atoms::{Shape, ShapeBox, Text};
use crate::colors::{MAIN_COLOR, STROKE_COLOR, TEXT_COLOR};
use crate::egui::{self, Align2, Color32, CornerRadius, Rect, Stroke, TextureId};
use crate::traits::{Roundable, WithText};

/// A compact ability card rendered as a square with icon, mana badge, and name.
pub struct SmallAbility<'a> {
    name: &'a str,
    image: TextureId,
    mp_cost: Option<u32>,
    fill: Color32,
    learned: bool,
}

impl<'a> SmallAbility<'a> {
    pub fn new(name: &'a str, image: TextureId) -> Self {
        Self {
            name,
            image,
            mp_cost: None,
            fill: MAIN_COLOR,
            learned: false,
        }
    }

    pub fn mp_cost(mut self, cost: Option<u32>) -> Self {
        self.mp_cost = cost;
        self
    }

    pub fn fill(mut self, color: Color32) -> Self {
        self.fill = color;
        self
    }

    pub fn learned(mut self, learned: bool) -> Self {
        self.learned = learned;
        self
    }

    pub fn paint(&self, painter: &egui::Painter, rect: Rect) {
        let side = rect.height();
        let square = Rect::from_center_size(rect.center(), egui::vec2(side, side));

        // Learned glow: soft bluish shadow behind the card
        if self.learned {
            let glow_color = Color32::from_rgba_premultiplied(0x6C, 0x5C, 0xE7, 0x30);
            let shadow = square.expand(3.0);
            painter.rect_filled(shadow, CornerRadius::same(19), glow_color);
        }

        // Card background
        let stroke_color = if self.learned {
            Color32::from_rgb(0x6C, 0x5C, 0xE7)
        } else {
            STROKE_COLOR
        };
        let stroke_width = if self.learned { 2.0 } else { 1.0 };
        ShapeBox::new(Shape::Rectangle)
            .fill(self.fill)
            .stroke(Stroke::new(stroke_width, stroke_color))
            .set_rounding(CornerRadius::same(16))
            .paint(painter, square);

        let pad = square.width() * 0.08;
        let inner = square.shrink(pad);

        // Top portion: icon (60% height), bottom: name (40% height)
        let icon_h = inner.height() * 0.6;
        let icon_size = icon_h;
        let icon_rect = Rect::from_min_size(
            egui::pos2(inner.center().x - icon_size / 2.0, inner.min.y),
            egui::vec2(icon_size, icon_size),
        );

        // Icon image
        let uv = Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0));
        painter.add(
            egui::epaint::RectShape::filled(icon_rect, CornerRadius::same(12), Color32::WHITE)
                .with_texture(self.image, uv),
        );

        // Mana cost badge (bottom-right of icon)
        if let Some(mp) = self.mp_cost {
            let badge_side = icon_size * 0.40;
            let badge_left = icon_rect.right() - badge_side * 0.9;
            let badge_top = icon_rect.bottom() - badge_side;
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

        // Name text below icon
        let text_top = icon_rect.bottom() + pad * 0.5;
        let text_rect = Rect::from_min_max(egui::pos2(inner.min.x, text_top), inner.max);
        let font_size = (text_rect.height() * 0.4).min(14.0);
        Text::new(self.name)
            .color(TEXT_COLOR)
            .size(font_size)
            .align(Align2::CENTER_CENTER)
            .paint(painter, text_rect);
    }
}
