use crate::atoms::{Shape, ShapeBox};
use crate::colors::{MAIN_COLOR, TEXT_COLOR};
use crate::egui::{self, Align2, FontId, Stroke, Widget};

/// XP required per level (placeholder constant).
const XP_PER_LEVEL: u32 = 1000;

/// Character portrait display area.
pub struct Portrait {
    border_1: egui::TextureId,
    border_2: egui::TextureId,
    avatar: egui::TextureId,
    level: u32,
    experience: u32,
}

impl Portrait {
    pub fn new(
        border_1: egui::TextureId,
        border_2: egui::TextureId,
        avatar: egui::TextureId,
        level: u32,
        experience: u32,
    ) -> Self {
        Self {
            border_1,
            border_2,
            avatar,
            level,
            experience,
        }
    }
}

impl Widget for Portrait {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let size = ui.available_size();
        let (rect, response) = ui.allocate_exact_size(size, egui::Sense::hover());

        let portrait_w = size.x * 0.65;
        let inset = (size.x - portrait_w) / 2.0;
        let portrait_rect = egui::Rect::from_min_size(
            rect.min + egui::vec2(inset, 0.0),
            egui::vec2(portrait_w, size.y),
        );

        let uv = egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0));
        let painter = ui.painter_at(rect);

        painter.image(self.border_1, portrait_rect, uv, egui::Color32::WHITE);
        painter.image(self.border_2, portrait_rect, uv, egui::Color32::WHITE);

        let border = size.y * 0.025;
        let clip_rect = portrait_rect.shrink(border);
        paint_ellipse_image(&painter, self.avatar, clip_rect, rect);

        // Level circle in the top-left corner of the portrait
        let circle_size = portrait_w * 0.3;
        let circle_rect = egui::Rect::from_min_size(
            portrait_rect.min + egui::vec2(0.0, 10.0),
            egui::vec2(circle_size, circle_size),
        );
        // Border images clipped to circle
        paint_ellipse_image(&painter, self.border_1, circle_rect, circle_rect);
        paint_ellipse_image(&painter, self.border_2, circle_rect, circle_rect);
        let circle_border = border / 2.0;
        let arc_thickness = circle_border;
        let arc_rect = circle_rect.shrink(circle_border);

        // Inner fill (covers arc zone so unfilled area looks thin)
        ShapeBox::new(Shape::Circle)
            .fill(MAIN_COLOR)
            .stroke(Stroke::NONE)
            .paint(&painter, arc_rect);

        // XP arc drawn on top
        let xp_fraction = (self.experience % XP_PER_LEVEL) as f32 / XP_PER_LEVEL as f32;
        paint_xp_arc(&painter, arc_rect, arc_thickness, xp_fraction);

        // Level text
        let level_text = self.level.to_string();
        let font_size = arc_rect.height() * 0.5;
        painter.text(
            arc_rect.center(),
            Align2::CENTER_CENTER,
            level_text,
            FontId::proportional(font_size),
            TEXT_COLOR,
        );

        response
    }
}

/// Paints `texture` clipped to an ellipse defined by `clip_rect`.
/// UV coordinates are computed relative to `image_rect` so the image
/// keeps its proportions and is simply cropped by the ellipse.
fn paint_ellipse_image(
    painter: &egui::Painter,
    texture: egui::TextureId,
    clip_rect: egui::Rect,
    image_rect: egui::Rect,
) {
    const SEGMENTS: usize = 64;

    let center = clip_rect.center();
    let rx = clip_rect.width() / 2.0;
    let ry = clip_rect.height() / 2.0;

    let pos_to_uv = |pos: egui::Pos2| -> egui::Pos2 {
        egui::pos2(
            (pos.x - image_rect.min.x) / image_rect.width(),
            (pos.y - image_rect.min.y) / image_rect.height(),
        )
    };

    let mut mesh = egui::Mesh::with_texture(texture);

    // Center vertex
    mesh.vertices.push(egui::epaint::Vertex {
        pos: center,
        uv: pos_to_uv(center),
        color: egui::Color32::WHITE,
    });

    // Perimeter vertices
    for i in 0..=SEGMENTS {
        let angle = std::f32::consts::TAU * (i as f32) / (SEGMENTS as f32);
        let pos = egui::pos2(center.x + rx * angle.cos(), center.y + ry * angle.sin());
        mesh.vertices.push(egui::epaint::Vertex {
            pos,
            uv: pos_to_uv(pos),
            color: egui::Color32::WHITE,
        });
    }

    // Triangle fan
    for i in 0..SEGMENTS {
        mesh.indices.push(0);
        mesh.indices.push((i + 1) as u32);
        mesh.indices.push((i + 2) as u32);
    }

    painter.add(egui::Shape::mesh(mesh));
}

/// Draws a gradient green-to-blue arc representing XP progress.
/// The arc starts at the top (12 o'clock) and sweeps clockwise.
/// Rendered as a series of short colored strokes for reliable display.
fn paint_xp_arc(painter: &egui::Painter, outer_rect: egui::Rect, thickness: f32, fraction: f32) {
    if fraction <= 0.0 {
        return;
    }

    const SEGMENTS: usize = 64;
    let fraction = fraction.min(1.0);
    let seg_count = ((SEGMENTS as f32) * fraction).ceil() as usize;

    let center = outer_rect.center();
    let radius = outer_rect.width().min(outer_rect.height()) / 2.0 - thickness / 2.0;

    let start_color = egui::Color32::from_rgb(0x00, 0xE5, 0x76); // green
    let end_color = egui::Color32::from_rgb(0x00, 0xB4, 0xD8); // cyan

    let start_angle = -std::f32::consts::FRAC_PI_2;

    for i in 0..seg_count {
        let t0 = (i as f32) / (SEGMENTS as f32);
        let t1 = ((i + 1) as f32) / (SEGMENTS as f32);
        let a0 = start_angle + std::f32::consts::TAU * t0;
        let a1 = start_angle + std::f32::consts::TAU * t1;

        let color_t = t0 / fraction;
        let color = lerp_color(start_color, end_color, color_t);

        let p0 = egui::pos2(center.x + radius * a0.cos(), center.y + radius * a0.sin());
        let p1 = egui::pos2(center.x + radius * a1.cos(), center.y + radius * a1.sin());

        painter.line_segment([p0, p1], egui::Stroke::new(thickness, color));
    }
}

fn lerp_color(a: egui::Color32, b: egui::Color32, t: f32) -> egui::Color32 {
    let t = t.clamp(0.0, 1.0);
    let lerp = |a: u8, b: u8| -> u8 { (a as f32 + (b as f32 - a as f32) * t) as u8 };
    egui::Color32::from_rgb(lerp(a.r(), b.r()), lerp(a.g(), b.g()), lerp(a.b(), b.b()))
}
