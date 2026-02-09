use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPrimaryContextPass};
use ui_widgets::colors::MAIN_COLOR;
use ui_widgets::composites::{
    Abilities, Characteristics, EquippedGear, IdentityBar, Inventory, Points, Portrait, SkillEntry,
    Skills, Stats, StatusBar, StatusBarResponse, TraitEntry, Traits, Wallet as WalletWidget,
    WalletResponse,
};

use crate::components::{
    ActionPoints, ActiveCharacter, ActiveEffects, CharacterClass, CharacterName, CharacterRace,
    CharacterSkillList, CharacterStats, CharacterTraitNames, CharacteristicPoints, Experience, Hp,
    Level, Mana, SkillPoints, Wallet,
};
use crate::events::{ResourceChanged, WalletChanged};
use crate::state::AppScreen;
use shared::character::OnLvlUp;
use shared::Effect;

#[derive(Resource)]
struct UiIcons {
    heart: egui::TextureHandle,
    avatar_border_1: egui::TextureHandle,
    avatar_border_2: egui::TextureHandle,
    avatar_placeholder: egui::TextureHandle,
    wallet_gold: egui::TextureHandle,
    wallet_silver: egui::TextureHandle,
    wallet_copper: egui::TextureHandle,
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
            .add_message::<WalletChanged>()
            .add_systems(
                EguiPrimaryContextPass,
                (
                    init_icons.run_if(not(resource_exists::<UiIcons>)),
                    render_ui.run_if(in_state(AppScreen::CharacterSheet)),
                ),
            )
            .add_systems(Update, (apply_resource_changes, apply_wallet_changes));
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
        wallet_gold: load_png_texture(
            ctx,
            "wallet_gold",
            include_bytes!("../assets/wallet_gold.png"),
        ),
        wallet_silver: load_png_texture(
            ctx,
            "wallet_silver",
            include_bytes!("../assets/wallet_silver.png"),
        ),
        wallet_copper: load_png_texture(
            ctx,
            "wallet_copper",
            include_bytes!("../assets/wallet_copper.png"),
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
    trait_names: &'static CharacterTraitNames,
    char_pts: &'static CharacteristicPoints,
    skill_pts: &'static SkillPoints,
    skills: &'static CharacterSkillList,
    wallet: &'static Wallet,
    effects: &'static ActiveEffects,
}

fn render_ui(
    mut contexts: EguiContexts,
    icons: Option<Res<UiIcons>>,
    character_query: Query<CharacterQueryData, With<ActiveCharacter>>,
    trait_registry: Res<crate::network::ClientTraitRegistry>,
    skill_registry: Res<crate::network::ClientSkillRegistry>,
    mut events: MessageWriter<ResourceChanged>,
    mut wallet_events: MessageWriter<WalletChanged>,
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
                render_left_column(
                    ui,
                    total_w * COL1_WIDTH,
                    col_h,
                    &icons,
                    &character,
                    &mut events,
                );
                ui.add_space(gap);
                render_center_column(
                    ui,
                    total_w * COL2_WIDTH,
                    col_h,
                    &character,
                    &trait_registry,
                    &skill_registry,
                );
                ui.add_space(gap);
                render_right_column(
                    ui,
                    total_w * COL3_WIDTH,
                    col_h,
                    &icons,
                    &character,
                    &mut wallet_events,
                );
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
    let initiative =
        character.stats.0.perception.level as i32 + character.effects.initiative_bonus();

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
    trait_registry: &crate::network::ClientTraitRegistry,
    skill_registry: &crate::network::ClientSkillRegistry,
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
        let skill_entries: Vec<SkillEntry> = skill_registry
            .0
            .get_class_skills(&character.class.0)
            .into_iter()
            .flat_map(|skills| skills.iter())
            .map(|(name, skill)| {
                let level = character
                    .skills
                    .0
                    .iter()
                    .find(|s| s.name == *name)
                    .map_or(0, |s| s.level);
                SkillEntry {
                    name: name.clone(),
                    dependency: skill.dependency.abbrev().to_string(),
                    level: level as i32,
                }
            })
            .collect();
        ui.add_sized([width, height * 0.24], Skills::new(skill_entries));
        ui.add_space(gap);
        let trait_entries: Vec<TraitEntry> = character
            .trait_names
            .0
            .iter()
            .filter_map(|name| {
                trait_registry.0.get(name).map(|ct| TraitEntry {
                    name: name.clone(),
                    description: ct.description.clone(),
                    effects: ct.effects.iter().map(format_effect).collect(),
                })
            })
            .collect();
        ui.add_sized([width, height * 0.14], Traits::new(trait_entries));
        ui.add_space(gap);
        ui.add_sized([width, height * 0.40], Abilities::new());
    });
}

fn format_effect(effect: &Effect) -> String {
    match effect {
        Effect::Resist(r, v) => format!("{r} Resist +{v}"),
        Effect::Skill(name, v) => format!("{name} {v:+}"),
        Effect::Protection(p, v) => format!("{p} Protection +{v}"),
        Effect::Initiative(v) => format!("Initiative {v:+}"),
        Effect::Characteristic(c, v) => format!("{c:?} {v:+}"),
        Effect::ActionPoints(v) => format!("Action Points {v:+}"),
        Effect::Armor(v) => format!("Armor {v:+}"),
        Effect::Mana {
            dependent,
            increase_per_point,
        } => format!("Mana {increase_per_point:+}/point of {dependent:?}"),
        Effect::OnLvlUp(OnLvlUp::AddSkillPoints(v)) => {
            format!("{v:+} Skill Points per level")
        }
    }
}

fn render_right_column(
    ui: &mut egui::Ui,
    width: f32,
    height: f32,
    icons: &UiIcons,
    character: &CharacterQueryDataItem,
    wallet_events: &mut MessageWriter<WalletChanged>,
) {
    let gap = height * 0.03 / 2.0;
    let wallet = character.wallet;

    ui.vertical(|ui| {
        ui.spacing_mut().item_spacing = egui::vec2(0.0, 0.0);

        ui.add_sized([width, height * 0.41], EquippedGear::new());
        ui.add_space(gap);

        let wallet_size = egui::vec2(width, height * 0.08);
        let (wallet_rect, _) = ui.allocate_exact_size(wallet_size, egui::Sense::hover());
        let mut wallet_ui = ui.new_child(
            egui::UiBuilder::new()
                .max_rect(wallet_rect)
                .layout(egui::Layout::top_down(egui::Align::Min)),
        );
        let result = WalletWidget::new(
            wallet.gold(),
            wallet.silver(),
            wallet.copper(),
            icons.wallet_gold.id(),
            icons.wallet_silver.id(),
            icons.wallet_copper.id(),
        )
        .show(&mut wallet_ui);
        send_wallet_events(wallet_events, result);

        ui.add_space(gap);
        ui.add_sized([width, height * 0.48], Inventory::new());
    });
}

fn send_wallet_events(events: &mut MessageWriter<WalletChanged>, result: WalletResponse) {
    for delta in [result.gold, result.silver, result.copper].into_iter().flatten() {
        events.write(WalletChanged(delta));
    }
}

fn apply_wallet_changes(
    mut query: Query<&mut Wallet, With<ActiveCharacter>>,
    mut reader: MessageReader<WalletChanged>,
) {
    let Ok(mut wallet) = query.single_mut() else {
        return;
    };
    for event in reader.read() {
        let new_val = wallet.0 as i64 + event.0;
        wallet.0 = new_val.max(0) as u64;
    }
}
