use crate::atoms::{Shape, ShapeBox};
use crate::colors::{MAIN_COLOR, TEXT_COLOR};
use crate::egui::{self, Align2, FontId, Stroke};

/// XP required to advance from `level` to `level + 1`.
fn xp_to_next_level(level: u32) -> u32 {
    (level + 1) * 10
}

/// State stored in egui temp data for the "Add Experience" popup.
#[derive(Clone)]
struct AddExpPopupState {
    open: bool,
    input_text: String,
}

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

    /// Renders the portrait and returns `Some(exp)` when the user confirms
    /// adding experience through the context-menu popup.
    pub fn show(self, ui: &mut egui::Ui) -> Option<u32> {
        let size = ui.available_size();
        let (rect, response) = ui.allocate_exact_size(size, egui::Sense::click());

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
        let xp_fraction = self.experience as f32 / xp_to_next_level(self.level) as f32;
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

        // Context menu on right-click
        let popup_id = response.id.with("add_exp");
        response.context_menu(|ui| {
            if ui.button("Add EXP").clicked() {
                ui.data_mut(|d| {
                    d.insert_temp(
                        popup_id,
                        AddExpPopupState {
                            open: true,
                            input_text: String::new(),
                        },
                    );
                });
                ui.close();
            }
        });

        // "Add Experience" popup window
        let mut result = None;
        let mut state: AddExpPopupState =
            ui.data(|d| d.get_temp(popup_id))
                .unwrap_or(AddExpPopupState {
                    open: false,
                    input_text: String::new(),
                });

        if state.open {
            let mut open = true;
            egui::Window::new("Add Experience")
                .collapsible(false)
                .resizable(false)
                .open(&mut open)
                .show(ui.ctx(), |ui| {
                    ui.horizontal(|ui| {
                        ui.label("EXP:");
                        ui.add(
                            egui::TextEdit::singleline(&mut state.input_text).desired_width(80.0),
                        );
                    });
                    state.input_text.retain(|c| c.is_ascii_digit());

                    let enter_pressed = ui.input(|i| i.key_pressed(egui::Key::Enter));

                    ui.horizontal(|ui| {
                        if ui.button("OK").clicked() || enter_pressed {
                            if let Ok(value) = state.input_text.parse::<u32>() {
                                if value > 0 {
                                    result = Some(value);
                                }
                            }
                            state.open = false;
                        }
                        if ui.button("Cancel").clicked() {
                            state.open = false;
                        }
                    });
                });

            // Handle X button closing the window
            if !open {
                state.open = false;
            }

            ui.data_mut(|d| d.insert_temp(popup_id, state));
        }

        result
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
