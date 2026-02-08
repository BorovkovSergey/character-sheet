use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPrimaryContextPass};
use shared::Character;
use ui_widgets::colors::MAIN_COLOR;
use ui_widgets::composites::{
    Abilities, Characteristics, EquippedGear, IdentityBar, Inventory, Points, Portrait, Skills,
    Stats, StatusBar, Traits, Wallet,
};

use crate::character_select::AppScreen;

#[derive(Resource)]
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
        app.add_systems(
            EguiPrimaryContextPass,
            (
                init_icons.run_if(not(resource_exists::<UiIcons>)),
                render_ui,
            ),
        );
    }
}

fn init_icons(mut contexts: EguiContexts, mut commands: Commands) -> Result {
    let ctx = contexts.ctx_mut()?;
    commands.insert_resource(UiIcons {
        heart: load_png_texture(ctx, "heart", include_bytes!("../assets/heart.png")),
    });
    Ok(())
}

const MARGIN: f32 = 0.02;
const COL_GAP: f32 = 0.01;
const COL1_WIDTH: f32 = 0.24;
const COL2_WIDTH: f32 = 0.46;
const COL3_WIDTH: f32 = 0.24;

fn render_ui(
    mut contexts: EguiContexts,
    icons: Option<Res<UiIcons>>,
    app_screen: Res<AppScreen>,
) -> Result {
    let character = match &*app_screen {
        AppScreen::CharacterSelect => return Ok(()),
        AppScreen::CharacterSheet(character) => character,
    };

    let Some(icons) = icons else {
        return Ok(());
    };

    let ctx = contexts.ctx_mut()?;
    let heart_icon = icons.heart.id();

    egui::CentralPanel::default()
        .frame(egui::Frame::NONE.fill(MAIN_COLOR))
        .show(ctx, |ui| {
            let total_w = ui.available_width();
            let total_h = ui.available_height();

            let margin = total_w * MARGIN;
            let gap = total_w * COL_GAP;
            let top_margin = margin / 2.0;
            let col_h = total_h - top_margin;

            ui.spacing_mut().item_spacing = egui::vec2(0.0, 0.0);
            ui.add_space(top_margin);

            ui.horizontal(|ui| {
                ui.add_space(margin);
                render_left_column(ui, total_w * COL1_WIDTH, col_h, character, heart_icon);
                ui.add_space(gap);
                render_center_column(ui, total_w * COL2_WIDTH, col_h, character);
                ui.add_space(gap);
                render_right_column(ui, total_w * COL3_WIDTH, col_h);
            });
        });

    Ok(())
}

fn render_left_column(
    ui: &mut egui::Ui,
    width: f32,
    height: f32,
    character: &Character,
    heart_icon: egui::TextureId,
) {
    let gap = height * 0.03 / 4.0;

    ui.vertical(|ui| {
        ui.spacing_mut().item_spacing = egui::vec2(0.0, 0.0);

        ui.add_sized([width, height * 0.30], Portrait::new());
        ui.add_space(gap);
        ui.add_sized(
            [width, height * 0.11],
            IdentityBar::new(
                &character.name,
                &character.race.to_string(),
                &character.class.to_string(),
            ),
        );
        ui.add_space(gap);
        ui.add_sized([width, height * 0.16], StatusBar::new());
        ui.add_space(gap);

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
            [width, height * 0.20],
            Stats::new(heart_icon, resists, protections),
        );
        ui.add_space(gap);
        ui.add_sized([width, height * 0.20], StatusBar::new());
    });
}

fn render_center_column(ui: &mut egui::Ui, width: f32, height: f32, character: &Character) {
    let gap = height * 0.03 / 4.0;

    ui.vertical(|ui| {
        ui.spacing_mut().item_spacing = egui::vec2(0.0, 0.0);

        let characteristics = [
            ("STR", character.stats.strength.level),
            ("DEX", character.stats.dexterity.level),
            ("END", character.stats.endurance.level),
            ("PER", character.stats.perception.level),
            ("MAG", character.stats.magic.level),
            ("WIL", character.stats.willpower.level),
            ("INT", character.stats.intellect.level),
            ("CHA", character.stats.charisma.level),
        ];
        let char_values = characteristics
            .into_iter()
            .map(|(k, v)| (k.to_string(), v))
            .collect();
        ui.add_sized([width, height * 0.14], Characteristics::new(char_values));
        ui.add_space(gap);
        ui.add_sized([width, height * 0.05], Points::new());
        ui.add_space(gap);
        ui.add_sized([width, height * 0.24], Skills::new());
        ui.add_space(gap);
        ui.add_sized([width, height * 0.14], Traits::new());
        ui.add_space(gap);
        ui.add_sized([width, height * 0.40], Abilities::new());
    });
}

fn render_right_column(ui: &mut egui::Ui, width: f32, height: f32) {
    let gap = height * 0.03 / 2.0;

    ui.vertical(|ui| {
        ui.spacing_mut().item_spacing = egui::vec2(0.0, 0.0);

        ui.add_sized([width, height * 0.41], EquippedGear::new());
        ui.add_space(gap);
        ui.add_sized([width, height * 0.08], Wallet::new());
        ui.add_space(gap);
        ui.add_sized([width, height * 0.48], Inventory::new());
    });
}
