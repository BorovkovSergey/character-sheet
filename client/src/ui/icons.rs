use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use ui_widgets::styles::UiStyle;

#[derive(Resource)]
pub(super) struct UiIcons {
    pub heart: egui::TextureHandle,
    pub avatar_border_1: egui::TextureHandle,
    pub avatar_border_2: egui::TextureHandle,
    pub avatar_placeholder: egui::TextureHandle,
    pub wallet_gold: egui::TextureHandle,
    pub wallet_silver: egui::TextureHandle,
    pub wallet_copper: egui::TextureHandle,
    pub ability_placeholder: egui::TextureHandle,
    pub weapon_placeholder: egui::TextureHandle,
    pub inventory_placeholder: egui::TextureHandle,
    pub shield: egui::TextureHandle,
}

fn load_png_texture(ctx: &egui::Context, name: &str, png_bytes: &[u8]) -> egui::TextureHandle {
    let img = image::load_from_memory(png_bytes).expect("failed to decode PNG");
    let rgba = img.to_rgba8();
    let size = [rgba.width() as usize, rgba.height() as usize];
    let color_image = egui::ColorImage::from_rgba_unmultiplied(size, &rgba);
    ctx.load_texture(name, color_image, egui::TextureOptions::LINEAR)
}

pub(super) fn init_icons(mut contexts: EguiContexts, mut commands: Commands) -> Result {
    let ctx = contexts.ctx_mut()?;
    UiStyle::apply_global_style(ctx);
    commands.insert_resource(UiIcons {
        heart: load_png_texture(ctx, "heart", include_bytes!("../../assets/heart.png")),
        avatar_border_1: load_png_texture(
            ctx,
            "avatar_border_1",
            include_bytes!("../../assets/avatar_border_1.png"),
        ),
        avatar_border_2: load_png_texture(
            ctx,
            "avatar_border_2",
            include_bytes!("../../assets/avatar_border_2.png"),
        ),
        avatar_placeholder: load_png_texture(
            ctx,
            "avatar_placeholder",
            include_bytes!("../../assets/avatar_placeholder.png"),
        ),
        wallet_gold: load_png_texture(
            ctx,
            "wallet_gold",
            include_bytes!("../../assets/wallet_gold.png"),
        ),
        wallet_silver: load_png_texture(
            ctx,
            "wallet_silver",
            include_bytes!("../../assets/wallet_silver.png"),
        ),
        wallet_copper: load_png_texture(
            ctx,
            "wallet_copper",
            include_bytes!("../../assets/wallet_copper.png"),
        ),
        ability_placeholder: load_png_texture(
            ctx,
            "ability_placeholder",
            include_bytes!("../../assets/ph_ability.png"),
        ),
        weapon_placeholder: load_png_texture(
            ctx,
            "weapon_placeholder",
            include_bytes!("../../assets/ph_weapon.png"),
        ),
        inventory_placeholder: load_png_texture(
            ctx,
            "inventory_placeholder",
            include_bytes!("../../assets/ph_inventory.png"),
        ),
        shield: load_png_texture(ctx, "shield", include_bytes!("../../assets/shield.png")),
    });
    Ok(())
}
