use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPrimaryContextPass};
use ui_widgets::colors::MAIN_COLOR;
use ui_widgets::composites::{
    Abilities, Characteristics, EquippedGear, IdentityBar, Inventory, Points, Portrait, Skills,
    Stats, StatusBar, StatusBarResponse, Traits, Wallet,
};

use crate::components::{
    ActionPoints, ActiveCharacter, ActiveEffects, CharacterClass, CharacterName, CharacterRace,
    CharacterStats, CharacteristicPoints, Experience, Hp, Level, Mana, SkillPoints,
};
use crate::events::ResourceChanged;
use crate::state::AppScreen;

#[derive(Resource)]
struct UiIcons {
    heart: egui::TextureHandle,
    avatar_border_1: egui::TextureHandle,
    avatar_border_2: egui::TextureHandle,
    avatar_placeholder: egui::TextureHandle,
}

fn load_png_texture(ctx: &egui::Context, name: &str, png_bytes: &[u8]) -> egui::TextureHandle {
    let img = image::load_from_memory(png_bytes).expect("failed to decode PNG");
    let rgba = img.to_rgba8();
    let size = [rgba.width() as usize, rgba.height() as usize];
    let color_image = egui::ColorImage::from_rgba_unmultiplied(size, &rgba);
    ctx.load_texture(name, color_image, egui::TextureOptions::LINEAR)
}

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<ResourceChanged>()
            .add_systems(
                EguiPrimaryContextPass,
                (
                    init_icons.run_if(not(resource_exists::<UiIcons>)),
                    render_ui.run_if(in_state(AppScreen::CharacterSheet)),
                ),
            )
            .add_systems(Update, apply_resource_changes);
    }
}

fn init_icons(mut contexts: EguiContexts, mut commands: Commands) -> Result {
    let ctx = contexts.ctx_mut()?;
    commands.insert_resource(UiIcons {
        heart: load_png_texture(ctx, "heart", include_bytes!("../assets/heart.png")),
        avatar_border_1: load_png_texture(
            ctx,
            "avatar_border_1",
            include_bytes!("../assets/avatar_border_1.png"),
        ),
        avatar_border_2: load_png_texture(
            ctx,
            "avatar_border_2",
            include_bytes!("../assets/avatar_border_2.png"),
        ),
        avatar_placeholder: load_png_texture(
            ctx,
            "avatar_placeholder",
            include_bytes!("../assets/avatar_placeholder.png"),
        ),
    });
    Ok(())
}

const MARGIN: f32 = 0.02;
const COL_GAP: f32 = 0.01;
const COL1_WIDTH: f32 = 0.24;
const COL2_WIDTH: f32 = 0.46;
const COL3_WIDTH: f32 = 0.24;

#[derive(bevy::ecs::query::QueryData)]
struct CharacterQueryData {
    name: &'static CharacterName,
    race: &'static CharacterRace,
    class: &'static CharacterClass,
    level: &'static Level,
    exp: &'static Experience,
    hp: &'static Hp,
    mana: &'static Mana,
    ap: &'static ActionPoints,
    stats: &'static CharacterStats,
    char_pts: &'static CharacteristicPoints,
    skill_pts: &'static SkillPoints,
    effects: &'static ActiveEffects,
}

fn render_ui(
    mut contexts: EguiContexts,
    icons: Option<Res<UiIcons>>,
    character_query: Query<CharacterQueryData, With<ActiveCharacter>>,
    mut events: MessageWriter<ResourceChanged>,
) -> Result {
    let Some(icons) = icons else {
        return Ok(());
    };

    let Ok(character) = character_query.single() else {
        return Ok(());
    };

    let ctx = contexts.ctx_mut()?;

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
                render_left_column(ui, total_w * COL1_WIDTH, col_h, &icons, &character, &mut events);
                ui.add_space(gap);
                render_center_column(ui, total_w * COL2_WIDTH, col_h, &character);
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
    icons: &UiIcons,
    character: &CharacterQueryDataItem,
    events: &mut MessageWriter<ResourceChanged>,
) {
    let gap = height * 0.03 / 4.0;
    let initiative = character.stats.0.perception.level as i32 + character.effects.initiative_bonus();

    ui.vertical(|ui| {
        ui.spacing_mut().item_spacing = egui::vec2(0.0, 0.0);

        ui.add_sized(
            [width, height * 0.30],
            Portrait::new(
                icons.avatar_border_1.id(),
                icons.avatar_border_2.id(),
                icons.avatar_placeholder.id(),
                character.level.0,
                character.exp.0,
            ),
        );
        ui.add_space(gap);
        ui.add_sized(
            [width, height * 0.11],
            IdentityBar::new(
                &character.name.0,
                &character.race.0.to_string(),
                &character.class.0.to_string(),
            ),
        );
        ui.add_space(gap);

        send_status_bar_events(ui, width, height * 0.16, character, initiative, events);
        ui.add_space(gap);

        let resists = character
            .effects
            .get_resists()
            .into_iter()
            .map(|(r, v)| (r.to_string(), v))
            .collect();
        let protections = character
            .effects
            .get_protections()
            .into_iter()
            .map(|(p, v)| (p.to_string(), v))
            .collect();
        ui.add_sized(
            [width, height * 0.20],
            Stats::new(icons.heart.id(), resists, protections),
        );
        ui.add_space(gap);

        send_status_bar_events(ui, width, height * 0.20, character, initiative, events);
    });
}

fn send_status_bar_events(
    ui: &mut egui::Ui,
    width: f32,
    height: f32,
    character: &CharacterQueryDataItem,
    initiative: i32,
    events: &mut MessageWriter<ResourceChanged>,
) {
    let status_size = egui::vec2(width, height);
    let (status_rect, _) = ui.allocate_exact_size(status_size, egui::Sense::hover());
    let mut status_ui = ui.new_child(
        egui::UiBuilder::new()
            .max_rect(status_rect)
            .layout(egui::Layout::top_down(egui::Align::Min)),
    );
    let result = StatusBar::new(
        character.hp.current,
        character.hp.max,
        character.mana.current,
        character.mana.max,
        character.ap.current,
        character.ap.max,
        initiative,
    )
    .show(&mut status_ui);

    send_resource_events(events, result);
}

fn send_resource_events(events: &mut MessageWriter<ResourceChanged>, result: StatusBarResponse) {
    if let Some(v) = result.hp {
        events.write(ResourceChanged::Hp(v));
    }
    if let Some(v) = result.mp {
        events.write(ResourceChanged::Mp(v));
    }
    if let Some(v) = result.ap {
        events.write(ResourceChanged::Ap(v));
    }
}

/// Applies resource change messages to the active character's ECS components.
fn apply_resource_changes(
    mut query: Query<(&mut Hp, &mut Mana, &mut ActionPoints), With<ActiveCharacter>>,
    mut reader: MessageReader<ResourceChanged>,
) {
    let Ok((mut hp, mut mana, mut ap)) = query.single_mut() else {
        return;
    };
    for event in reader.read() {
        match event {
            ResourceChanged::Hp(v) => hp.current = (*v).min(hp.max),
            ResourceChanged::Mp(v) => mana.current = (*v).min(mana.max),
            ResourceChanged::Ap(v) => ap.current = (*v).min(ap.max),
        }
    }
}

fn render_center_column(
    ui: &mut egui::Ui,
    width: f32,
    height: f32,
    character: &CharacterQueryDataItem,
) {
    let gap = height * 0.03 / 4.0;
    let stats = &character.stats.0;

    ui.vertical(|ui| {
        ui.spacing_mut().item_spacing = egui::vec2(0.0, 0.0);

        let characteristics = [
            ("STR", stats.strength.level),
            ("DEX", stats.dexterity.level),
            ("END", stats.endurance.level),
            ("PER", stats.perception.level),
            ("MAG", stats.magic.level),
            ("WIL", stats.willpower.level),
            ("INT", stats.intellect.level),
            ("CHA", stats.charisma.level),
        ];
        let char_values = characteristics
            .into_iter()
            .map(|(k, v)| (k.to_string(), v))
            .collect();
        ui.add_sized([width, height * 0.14], Characteristics::new(char_values));
        ui.add_space(gap);
        ui.add_sized(
            [width, height * 0.05],
            Points::new(character.char_pts.0, character.skill_pts.0),
        );
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
