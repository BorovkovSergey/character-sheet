use std::io::Cursor;
use std::sync::{Arc, Mutex};

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use uuid::Uuid;

use crate::components::{ActiveCharacter, CharacterId, PortraitTexture};
use crate::network::PendingClientMessages;

/// Maximum portrait size in bytes (512KB).
/// Portraits are resized to 256x256 PNG on the client, typically <100KB.
const MAX_PORTRAIT_SIZE: usize = 512 * 1024;

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

/// Opens the file dialog and stores the result in `PortraitPickerResult`.
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

/// Decodes raw image bytes, resizes to fit within 256x256, and re-encodes as PNG.
pub fn process_raw_image(raw_bytes: &[u8]) -> Option<Vec<u8>> {
    if raw_bytes.len() > MAX_PORTRAIT_SIZE * 10 {
        warn!(
            "Uploaded image is too large ({} bytes), skipping",
            raw_bytes.len()
        );
        return None;
    }
    let img = image::load_from_memory(raw_bytes)
        .inspect_err(|e| warn!("Failed to decode image: {}", e))
        .ok()?;
    let resized = img.resize(256, 256, image::imageops::FilterType::Lanczos3);
    let mut png_bytes = Vec::new();
    resized
        .write_to(&mut Cursor::new(&mut png_bytes), image::ImageFormat::Png)
        .inspect_err(|e| warn!("Failed to encode PNG: {}", e))
        .ok()?;
    Some(png_bytes)
}

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

/// System: converts `PendingPortraitData` into a `PortraitTexture` component on the active character.
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

/// System: when a portrait is picked on the character sheet (not creation),
/// processes it, uploads to server, and updates the entity texture.
pub fn process_sheet_portrait_upload(
    picker_result: Res<PortraitPickerResult>,
    mut contexts: EguiContexts,
    mut pending_messages: ResMut<PendingClientMessages>,
    mut commands: Commands,
    query: Query<(Entity, &CharacterId), With<ActiveCharacter>>,
) {
    let mut guard = match picker_result.0.lock() {
        Ok(g) => g,
        Err(e) => {
            warn!("Portrait picker mutex poisoned: {}", e);
            return;
        }
    };
    let Some(raw_bytes) = guard.take() else {
        return;
    };

    let Some(png_bytes) = process_raw_image(&raw_bytes) else {
        return;
    };

    let Ok(ctx) = contexts.ctx_mut() else { return };

    if let Some(texture) = png_to_texture(ctx, "character_portrait", &png_bytes) {
        if let Some((entity, char_id)) = query.iter().next() {
            commands.entity(entity).insert(PortraitTexture(texture));
            pending_messages
                .0
                .push(shared::ClientMessage::UploadPortrait {
                    id: char_id.0,
                    png_data: png_bytes,
                });
        }
    }
}

pub struct PortraitPlugin;

impl Plugin for PortraitPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PortraitPickerResult>()
            .init_resource::<PendingCreationPortrait>()
            .add_systems(
                Update,
                (apply_portrait_data, process_sheet_portrait_upload)
                    .run_if(in_state(crate::state::AppScreen::CharacterSheet)),
            );
    }
}
