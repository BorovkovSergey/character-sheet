use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use ui_widgets::styles::UiStyle;

#[derive(Resource)]
pub(super) struct UiIcons {
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
    pub resist_fire: egui::TextureHandle,
    pub resist_ice: egui::TextureHandle,
    pub resist_lightning: egui::TextureHandle,
    pub resist_poison: egui::TextureHandle,
    pub resist_spirit: egui::TextureHandle,
    pub resist_dark: egui::TextureHandle,
    pub protection_melee: egui::TextureHandle,
    pub protection_range: egui::TextureHandle,
    pub protection_magic: egui::TextureHandle,
    pub protection_body: egui::TextureHandle,
    pub protection_mind: egui::TextureHandle,
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
        resist_fire: load_png_texture(
            ctx,
            "resist_fire",
            include_bytes!("../../assets/resist_fire.png"),
        ),
        resist_ice: load_png_texture(
            ctx,
            "resist_ice",
            include_bytes!("../../assets/resist_ice.png"),
        ),
        resist_lightning: load_png_texture(
            ctx,
            "resist_lightning",
            include_bytes!("../../assets/resist_lightning.png"),
        ),
        resist_poison: load_png_texture(
            ctx,
            "resist_poison",
            include_bytes!("../../assets/resist_poison.png"),
        ),
        resist_spirit: load_png_texture(
            ctx,
            "resist_spirit",
            include_bytes!("../../assets/resist_spirit.png"),
        ),
        resist_dark: load_png_texture(
            ctx,
            "resist_dark",
            include_bytes!("../../assets/resist_dark.png"),
        ),
        protection_melee: load_png_texture(
            ctx,
            "protection_melee",
            include_bytes!("../../assets/protection_melee.png"),
        ),
        protection_range: load_png_texture(
            ctx,
            "protection_range",
            include_bytes!("../../assets/protection_range.png"),
        ),
        protection_magic: load_png_texture(
            ctx,
            "protection_magic",
            include_bytes!("../../assets/protection_magic.png"),
        ),
        protection_body: load_png_texture(
            ctx,
            "protection_body",
            include_bytes!("../../assets/protection_body.png"),
        ),
        protection_mind: load_png_texture(
            ctx,
            "protection_mind",
            include_bytes!("../../assets/protection_mind.png"),
        ),
    });
    Ok(())
}
