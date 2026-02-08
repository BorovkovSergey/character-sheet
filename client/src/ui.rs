use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPrimaryContextPass};
use ui_widgets::colors::MAIN_COLOR;
use ui_widgets::composites::{
    Abilities, Characteristics, EquippedGear, IdentityBar, Inventory, Points, Portrait, Skills,
    Stats, StatusBar, Traits, Wallet,
};

use crate::character_select::AppScreen;

struct UiIcons {
    heart: egui::TextureHandle,
}

fn load_png_texture(ctx: &egui::Context, name: &str, png_bytes: &[u8]) -> egui::TextureHandle {
    let img = image::load_from_memory(png_bytes).expect("failed to decode PNG");
    let rgba = img.to_rgba8();
    let size = [rgba.width() as usize, rgba.height() as usize];
    let color_image = egui::ColorImage::from_rgba_unmultiplied(size, &rgba);
    ctx.load_texture(name, color_image, egui::TextureOptions::default())
}

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(EguiPrimaryContextPass, render_ui);
    }
}

fn render_ui(
    mut contexts: EguiContexts,
    mut icons: Local<Option<UiIcons>>,
    app_screen: Res<AppScreen>,
) -> Result {
    let character = match &*app_screen {
        AppScreen::CharacterSelect => return Ok(()),
        AppScreen::CharacterSheet(character) => character,
    };

    let ctx = contexts.ctx_mut()?;

    let icons = icons.get_or_insert_with(|| UiIcons {
        heart: load_png_texture(ctx, "heart", include_bytes!("../assets/heart.png")),
    });
    let heart_icon = icons.heart.id();

    let name = &character.name;
    let race = character.race.to_string();
    let class = character.class.to_string();

    egui::CentralPanel::default()
        .frame(egui::Frame::NONE.fill(MAIN_COLOR))
        .show(ctx, |ui| {
            let total_w = ui.available_width();
            let total_h = ui.available_height();

            let margin = total_w * 0.02;
            let gap = total_w * 0.01;
            let col1_w = total_w * 0.24;
            let col2_w = total_w * 0.46;
            let col3_w = total_w * 0.24;

            let top_margin = margin / 2.0;
            let col_h = total_h - top_margin;

            ui.spacing_mut().item_spacing = egui::vec2(0.0, 0.0);

            ui.add_space(top_margin);

            ui.horizontal(|ui| {
                ui.add_space(margin);

                let gap_between = col_h * 0.03 / 4.0;

                let portrait_h = col_h * 0.30;
                let identity_h = col_h * 0.11;
                let status1_h = col_h * 0.16;
                let stats_h = col_h * 0.20;
                let status2_h = col_h * 0.20;

                ui.vertical(|ui| {
                    ui.spacing_mut().item_spacing = egui::vec2(0.0, 0.0);

                    ui.add_sized([col1_w, portrait_h], Portrait::new());
                    ui.add_space(gap_between);
                    ui.add_sized([col1_w, identity_h], IdentityBar::new(name, &race, &class));
                    ui.add_space(gap_between);
                    ui.add_sized([col1_w, status1_h], StatusBar::new());
                    ui.add_space(gap_between);
                    let resists = character
                        .get_resists()
                        .into_iter()
                        .map(|(r, v)| (r.to_string(), v))
                        .collect();
                    let protections = character
                        .get_protections()
                        .into_iter()
                        .map(|(p, v)| (p.to_string(), v))
                        .collect();
                    ui.add_sized(
                        [col1_w, stats_h],
                        Stats::new(heart_icon, resists, protections),
                    );
                    ui.add_space(gap_between);
                    ui.add_sized([col1_w, status2_h], StatusBar::new());
                });

                ui.add_space(gap);

                let characteristics_h = col_h * 0.14;
                let points_h = col_h * 0.05;
                let skills_h = col_h * 0.24;
                let traits_h = col_h * 0.14;
                let abilities_h = col_h * 0.40;

                ui.vertical(|ui| {
                    ui.spacing_mut().item_spacing = egui::vec2(0.0, 0.0);

                    ui.add_sized([col2_w, characteristics_h], Characteristics::new());
                    ui.add_space(gap_between);
                    ui.add_sized([col2_w, points_h], Points::new());
                    ui.add_space(gap_between);
                    ui.add_sized([col2_w, skills_h], Skills::new());
                    ui.add_space(gap_between);
                    ui.add_sized([col2_w, traits_h], Traits::new());
                    ui.add_space(gap_between);
                    ui.add_sized([col2_w, abilities_h], Abilities::new());
                });

                ui.add_space(gap);

                let gap_between_col3 = col_h * 0.03 / 2.0;

                let equipped_gear_h = col_h * 0.41;
                let wallet_h = col_h * 0.08;
                let inventory_h = col_h * 0.48;

                ui.vertical(|ui| {
                    ui.spacing_mut().item_spacing = egui::vec2(0.0, 0.0);

                    ui.add_sized([col3_w, equipped_gear_h], EquippedGear::new());
                    ui.add_space(gap_between_col3);
                    ui.add_sized([col3_w, wallet_h], Wallet::new());
                    ui.add_space(gap_between_col3);
                    ui.add_sized([col3_w, inventory_h], Inventory::new());
                });
            });
        });

    Ok(())
}
