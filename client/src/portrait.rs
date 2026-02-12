use std::io::Cursor;
use std::sync::{Arc, Mutex};

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use ui_widgets::colors::{MAIN_COLOR, STROKE_COLOR, TEXT_COLOR};
use uuid::Uuid;

use crate::components::{ActiveCharacter, CharacterId, PortraitTexture};

/// Maximum portrait size in bytes (512KB).
const MAX_PORTRAIT_SIZE: usize = 512 * 1024;

// ---------------------------------------------------------------------------
// Resources
// ---------------------------------------------------------------------------

/// Shared buffer for receiving file bytes from the async file dialog.
#[derive(Resource, Clone)]
pub struct PortraitPickerResult(pub Arc<Mutex<Option<Vec<u8>>>>);

impl Default for PortraitPickerResult {
    fn default() -> Self {
        Self(Arc::new(Mutex::new(None)))
    }
}

/// Buffered portrait data received from the server, waiting for texture creation.
#[derive(Resource)]
pub struct PendingPortraitData {
    pub id: Uuid,
    pub png_data: Vec<u8>,
}

/// Holds portrait PNG bytes during character creation flow
/// (between "Create" click and receiving CharacterCreated response).
#[derive(Resource, Default)]
pub struct PendingCreationPortrait(pub Option<Vec<u8>>);

/// Unified crop editor popup state.
/// Callers set `open = true` to show the popup. When the user confirms,
/// the cropped PNG bytes are placed in `result` for the caller to consume.
#[derive(Resource, Default)]
pub struct CropEditorSlot {
    pub open: bool,
    pub editor: Option<PortraitCropEditor>,
    pub result: Option<Vec<u8>>,
}

/// State for the interactive crop/adjust editor.
pub struct PortraitCropEditor {
    full_image: image::DynamicImage,
    preview_texture: egui::TextureHandle,
    img_w: f32,
    img_h: f32,
    pub zoom: f32,
    pub pan: egui::Vec2,
}

impl PortraitCropEditor {
    /// Decode raw image bytes and create the editor state.
    pub fn from_raw_bytes(ctx: &egui::Context, raw_bytes: &[u8]) -> Option<Self> {
        if raw_bytes.len() > MAX_PORTRAIT_SIZE * 10 {
            warn!(
                "Uploaded image too large ({} bytes), skipping",
                raw_bytes.len()
            );
            return None;
        }
        let img = image::load_from_memory(raw_bytes)
            .inspect_err(|e| warn!("Failed to decode image: {}", e))
            .ok()?;
        let (w, h) = (img.width(), img.height());
        let rgba = img.to_rgba8();
        let size = [w as usize, h as usize];
        let color_image = egui::ColorImage::from_rgba_unmultiplied(size, rgba.as_raw());
        let texture = ctx.load_texture("crop_preview", color_image, egui::TextureOptions::LINEAR);
        Some(Self {
            full_image: img,
            preview_texture: texture,
            img_w: w as f32,
            img_h: h as f32,
            zoom: 1.0,
            pan: egui::Vec2::ZERO,
        })
    }
}

// ---------------------------------------------------------------------------
// File dialog
// ---------------------------------------------------------------------------

/// Opens the file dialog and stores the result in [`PortraitPickerResult`].
pub fn spawn_portrait_picker(result: &PortraitPickerResult) {
    let shared = result.0.clone();

    #[cfg(target_arch = "wasm32")]
    {
        let task = async move {
            let file = rfd::AsyncFileDialog::new()
                .add_filter("Images", &["png", "jpg", "jpeg", "webp", "bmp"])
                .pick_file()
                .await;
            if let Some(file) = file {
                let bytes = file.read().await;
                if let Ok(mut guard) = shared.lock() {
                    *guard = Some(bytes);
                }
            }
        };
        wasm_bindgen_futures::spawn_local(task);
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        std::thread::spawn(move || {
            let file = rfd::FileDialog::new()
                .add_filter("Images", &["png", "jpg", "jpeg", "webp", "bmp"])
                .pick_file();
            if let Some(path) = file {
                if let Ok(bytes) = std::fs::read(&path) {
                    if let Ok(mut guard) = shared.lock() {
                        *guard = Some(bytes);
                    }
                }
            }
        });
    }
}

// ---------------------------------------------------------------------------
// Image helpers
// ---------------------------------------------------------------------------

/// Creates an egui texture handle from PNG bytes.
pub fn png_to_texture(
    ctx: &egui::Context,
    name: &str,
    png_data: &[u8],
) -> Option<egui::TextureHandle> {
    let img = image::load_from_memory(png_data).ok()?;
    let rgba = img.to_rgba8();
    let size = [rgba.width() as usize, rgba.height() as usize];
    let color_image = egui::ColorImage::from_rgba_unmultiplied(size, &rgba);
    Some(ctx.load_texture(name, color_image, egui::TextureOptions::LINEAR))
}

/// Computes the portrait ellipse radii for a given preview rect,
/// matching the proportions used by the portrait widget.
fn portrait_ellipse(rect: egui::Rect) -> (f32, f32) {
    let portrait_w = rect.width() * 0.65;
    let border = rect.height() * 0.025;
    let rx = (portrait_w / 2.0 - border).max(1.0);
    let ry = (rect.height() / 2.0 - border).max(1.0);
    (rx, ry)
}

/// Crop `img` using the crop-editor viewport parameters and encode as 256×256 PNG.
fn crop_and_encode(
    img: &image::DynamicImage,
    img_w: f32,
    img_h: f32,
    preview_rect: egui::Rect,
    zoom: f32,
    pan: egui::Vec2,
) -> Option<Vec<u8>> {
    let (rx, ry) = portrait_ellipse(preview_rect);
    let preview_w = preview_rect.width();
    let preview_h = preview_rect.height();

    let base_scale = (preview_w / img_w).max(preview_h / img_h);
    let scale = base_scale * zoom;

    // Ellipse bounding box in image coordinates.
    let center_x = img_w / 2.0 - pan.x / scale;
    let center_y = img_h / 2.0 - pan.y / scale;
    let half_w = rx / scale;
    let half_h = ry / scale;

    let x = (center_x - half_w).max(0.0) as u32;
    let y = (center_y - half_h).max(0.0) as u32;
    let w = ((half_w * 2.0) as u32).min(img.width().saturating_sub(x));
    let h = ((half_h * 2.0) as u32).min(img.height().saturating_sub(y));
    if w == 0 || h == 0 {
        return None;
    }

    let cropped = img.crop_imm(x, y, w, h);
    let resized = cropped.resize_exact(256, 256, image::imageops::FilterType::Lanczos3);

    let mut png_bytes = Vec::new();
    resized
        .write_to(&mut Cursor::new(&mut png_bytes), image::ImageFormat::Png)
        .inspect_err(|e| warn!("Failed to encode PNG: {}", e))
        .ok()?;
    Some(png_bytes)
}

// ---------------------------------------------------------------------------
// Crop popup
// ---------------------------------------------------------------------------

/// Polls the picker for new file bytes and opens the crop popup when ready.
/// Call this every frame from any screen that uses portrait upload.
pub fn poll_and_render_crop_popup(
    ctx: &egui::Context,
    slot: &mut CropEditorSlot,
    picker: &PortraitPickerResult,
) {
    // Poll picker — when raw bytes arrive, open the crop popup automatically.
    if let Ok(mut guard) = picker.0.lock() {
        if let Some(raw_bytes) = guard.take() {
            slot.editor = PortraitCropEditor::from_raw_bytes(ctx, &raw_bytes);
            if slot.editor.is_some() {
                slot.open = true;
            }
        }
    }

    // Only show the popup when we have an image to edit.
    let Some(editor) = slot.editor.as_mut() else {
        return;
    };
    if !slot.open {
        return;
    }

    let popup_w = 340.0;
    let mut still_open = true;

    egui::Window::new("Portrait")
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .order(egui::Order::Foreground)
        .title_bar(false)
        .collapsible(false)
        .resizable(false)
        .frame(
            egui::Frame::new()
                .fill(MAIN_COLOR)
                .corner_radius(12.0)
                .stroke(egui::Stroke::new(1.0, STROKE_COLOR))
                .inner_margin(egui::Margin::same(16)),
        )
        .min_width(popup_w)
        .max_width(popup_w)
        .show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.label(
                    egui::RichText::new("Portrait")
                        .size(18.0)
                        .color(TEXT_COLOR)
                        .strong(),
                );
                ui.add_space(10.0);

                // Crop preview.
                let preview_w = (popup_w - 32.0).max(150.0);
                let preview_h = (preview_w / 1.4).max(100.0);
                let preview_w = preview_h * 1.4;

                let (preview_rect, resp) = ui.allocate_exact_size(
                    egui::vec2(preview_w, preview_h),
                    egui::Sense::click_and_drag(),
                );

                if resp.dragged() {
                    editor.pan += resp.drag_delta();
                }

                let scroll = ui.input(|i| i.smooth_scroll_delta.y);
                if scroll != 0.0 {
                    editor.zoom = (editor.zoom + scroll * 0.005).clamp(0.2, 5.0);
                }

                // Clamp pan so the ellipse stays within the image.
                let base_scale = (preview_w / editor.img_w).max(preview_h / editor.img_h);
                let (rx, ry) = portrait_ellipse(preview_rect);
                let max_pan_x = (editor.img_w * base_scale * editor.zoom / 2.0 - rx).max(0.0);
                let max_pan_y = (editor.img_h * base_scale * editor.zoom / 2.0 - ry).max(0.0);
                editor.pan.x = editor.pan.x.clamp(-max_pan_x, max_pan_x);
                editor.pan.y = editor.pan.y.clamp(-max_pan_y, max_pan_y);

                let painter = ui.painter_at(preview_rect);
                paint_crop_preview(
                    &painter,
                    editor.preview_texture.id(),
                    preview_rect,
                    editor.img_w,
                    editor.img_h,
                    editor.zoom,
                    editor.pan,
                );

                ui.add_space(8.0);

                // Zoom slider.
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("Zoom:").size(13.0).color(TEXT_COLOR));
                    ui.add(
                        egui::Slider::new(&mut editor.zoom, 0.2..=5.0)
                            .show_value(false)
                            .trailing_fill(true),
                    );
                });

                ui.add_space(12.0);

                // Ok / Cancel buttons.
                ui.horizontal(|ui| {
                    let cancel = egui::Button::new(
                        egui::RichText::new("Cancel").size(14.0).color(TEXT_COLOR),
                    )
                    .corner_radius(6.0)
                    .stroke(egui::Stroke::new(1.0, STROKE_COLOR))
                    .fill(MAIN_COLOR)
                    .min_size(egui::vec2(80.0, 28.0));
                    if ui.add(cancel).clicked() {
                        still_open = false;
                    }

                    ui.add_space(8.0);

                    let ok_btn =
                        egui::Button::new(egui::RichText::new("Ok").size(14.0).color(TEXT_COLOR))
                            .corner_radius(6.0)
                            .stroke(egui::Stroke::new(1.0, STROKE_COLOR))
                            .fill(MAIN_COLOR)
                            .min_size(egui::vec2(80.0, 28.0));
                    if ui.add(ok_btn).clicked() {
                        let enc_rect = egui::Rect::from_min_size(
                            egui::Pos2::ZERO,
                            egui::vec2(preview_w, preview_h),
                        );
                        if let Some(png) = crop_and_encode(
                            &editor.full_image,
                            editor.img_w,
                            editor.img_h,
                            enc_rect,
                            editor.zoom,
                            editor.pan,
                        ) {
                            slot.result = Some(png);
                            still_open = false;
                        }
                    }
                });

                ui.add_space(4.0);
            });
        });

    if !still_open {
        slot.open = false;
        slot.editor = None;
    }
}

/// Paints the zoomed/panned image with a bright portrait-shaped ellipse
/// and dimmed surroundings.
fn paint_crop_preview(
    painter: &egui::Painter,
    tex_id: egui::TextureId,
    rect: egui::Rect,
    img_w: f32,
    img_h: f32,
    zoom: f32,
    pan: egui::Vec2,
) {
    let preview_w = rect.width();
    let preview_h = rect.height();
    let base_scale = (preview_w / img_w).max(preview_h / img_h);
    let sx = img_w * base_scale * zoom;
    let sy = img_h * base_scale * zoom;
    let center = rect.center();

    let pos_to_uv = |pos: egui::Pos2| -> egui::Pos2 {
        egui::pos2(
            (pos.x - center.x - pan.x) / sx + 0.5,
            (pos.y - center.y - pan.y) / sy + 0.5,
        )
    };

    // 0. Dark background (visible when zoomed out below 1.0).
    painter.rect_filled(rect, 0.0, egui::Color32::from_gray(20));

    // 1. Full-rect image at reduced brightness (dimmed area).
    let dim = egui::Color32::from_rgba_unmultiplied(80, 80, 80, 255);
    {
        let mut mesh = egui::Mesh::with_texture(tex_id);
        let corners = [
            rect.left_top(),
            rect.right_top(),
            rect.right_bottom(),
            rect.left_bottom(),
        ];
        for c in &corners {
            mesh.vertices.push(egui::epaint::Vertex {
                pos: *c,
                uv: pos_to_uv(*c),
                color: dim,
            });
        }
        mesh.indices.extend_from_slice(&[0, 1, 2, 0, 2, 3]);
        painter.add(egui::Shape::mesh(mesh));
    }

    // 2. Bright ellipse matching the portrait widget shape.
    let (rx, ry) = portrait_ellipse(rect);
    {
        const SEGMENTS: usize = 64;
        let mut mesh = egui::Mesh::with_texture(tex_id);
        mesh.vertices.push(egui::epaint::Vertex {
            pos: center,
            uv: pos_to_uv(center),
            color: egui::Color32::WHITE,
        });
        for i in 0..=SEGMENTS {
            let angle = std::f32::consts::TAU * (i as f32) / (SEGMENTS as f32);
            let pos = egui::pos2(center.x + rx * angle.cos(), center.y + ry * angle.sin());
            mesh.vertices.push(egui::epaint::Vertex {
                pos,
                uv: pos_to_uv(pos),
                color: egui::Color32::WHITE,
            });
        }
        for i in 0..SEGMENTS {
            mesh.indices.push(0);
            mesh.indices.push((i + 1) as u32);
            mesh.indices.push((i + 2) as u32);
        }
        painter.add(egui::Shape::mesh(mesh));
    }

    // 3. Ellipse border stroke.
    {
        let mut points = Vec::with_capacity(65);
        for i in 0..=64 {
            let angle = std::f32::consts::TAU * (i as f32) / 64.0;
            points.push(egui::pos2(
                center.x + rx * angle.cos(),
                center.y + ry * angle.sin(),
            ));
        }
        painter.add(egui::Shape::line(
            points,
            egui::Stroke::new(2.0, egui::Color32::from_white_alpha(200)),
        ));
    }
}

// ---------------------------------------------------------------------------
// Systems
// ---------------------------------------------------------------------------

/// Converts `PendingPortraitData` into a `PortraitTexture` component on the active character.
pub fn apply_portrait_data(
    mut commands: Commands,
    pending: Option<Res<PendingPortraitData>>,
    mut contexts: EguiContexts,
    query: Query<(Entity, &CharacterId), With<ActiveCharacter>>,
) {
    let Some(pending) = pending else { return };
    let Ok(ctx) = contexts.ctx_mut() else { return };

    if let Some(texture) = png_to_texture(ctx, "character_portrait", &pending.png_data) {
        if let Some((entity, _)) = query.iter().find(|(_, id)| id.0 == pending.id) {
            commands.entity(entity).insert(PortraitTexture(texture));
        }
    }
    commands.remove_resource::<PendingPortraitData>();
}

// ---------------------------------------------------------------------------
// Plugin
// ---------------------------------------------------------------------------

pub struct PortraitPlugin;

impl Plugin for PortraitPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PortraitPickerResult>()
            .init_resource::<PendingCreationPortrait>()
            .init_resource::<CropEditorSlot>()
            .add_systems(
                Update,
                apply_portrait_data.run_if(in_state(crate::state::AppScreen::CharacterSheet)),
            );
    }
}
