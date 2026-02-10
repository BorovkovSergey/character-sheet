use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPrimaryContextPass};
use ui_widgets::colors::MAIN_COLOR;
use ui_widgets::composites::{
    Abilities, AbilityEntry, Characteristics, EquippedGear, IdentityBar, Inventory, Points,
    Portrait, SkillEntry, Skills, Stats, StatusBar, StatusBarResponse, TraitEntry, Traits,
    Wallet as WalletWidget, WalletResponse, Weapon, WeaponSlot,
};
use ui_widgets::molecules::{CellAction, InventoryTooltip};
use ui_widgets::styles::UiStyle;

use crate::components::{
    ActionPoints, ActiveCharacter, ActiveEffects, CharacterAbilityNames, CharacterClass,
    CharacterEquipment, CharacterName, CharacterRace, CharacterSkillList, CharacterStats,
    CharacterTraitNames, CharacterWeaponNames, CharacteristicPoints, Experience, Hp,
    AbilityPoints, Inventory as InventoryComponent, Level, Mana, SkillPoints, Wallet,
};
use crate::events::{ExperienceChanged, InventoryChanged, LevelUp, ResourceChanged, WalletChanged};
use crate::state::AppScreen;
use shared::character::OnLvlUp;
use shared::{AbilityCheck, AbilityType, Effect, EquipmentSlot, InventoryItem};

#[derive(Resource)]
struct UiIcons {
    heart: egui::TextureHandle,
    avatar_border_1: egui::TextureHandle,
    avatar_border_2: egui::TextureHandle,
    avatar_placeholder: egui::TextureHandle,
    wallet_gold: egui::TextureHandle,
    wallet_silver: egui::TextureHandle,
    wallet_copper: egui::TextureHandle,
    ability_placeholder: egui::TextureHandle,
    weapon_placeholder: egui::TextureHandle,
    inventory_placeholder: egui::TextureHandle,
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
            .add_message::<InventoryChanged>()
            .add_message::<ExperienceChanged>()
            .add_message::<LevelUp>()
            .add_systems(
                EguiPrimaryContextPass,
                (
                    init_icons.run_if(not(resource_exists::<UiIcons>)),
                    render_ui.run_if(in_state(AppScreen::CharacterSheet)),
                ),
            )
            .add_systems(
                Update,
                (
                    apply_resource_changes,
                    apply_wallet_changes,
                    apply_inventory_changes,
                    apply_experience_changes,
                    apply_level_up,
                ),
            );
    }
}

fn init_icons(mut contexts: EguiContexts, mut commands: Commands) -> Result {
    let ctx = contexts.ctx_mut()?;
    UiStyle::apply_global_style(ctx);
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
        ability_placeholder: load_png_texture(
            ctx,
            "ability_placeholder",
            include_bytes!("../assets/ph_ability.png"),
        ),
        weapon_placeholder: load_png_texture(
            ctx,
            "weapon_placeholder",
            include_bytes!("../assets/ph_weapon.png"),
        ),
        inventory_placeholder: load_png_texture(
            ctx,
            "inventory_placeholder",
            include_bytes!("../assets/ph_inventory.png"),
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
    ability_names: &'static CharacterAbilityNames,
    char_pts: &'static CharacteristicPoints,
    skill_pts: &'static SkillPoints,
    ability_pts: &'static AbilityPoints,
    skills: &'static CharacterSkillList,
    wallet: &'static Wallet,
    weapon_names: &'static CharacterWeaponNames,
    equipment: &'static CharacterEquipment,
    inventory: &'static InventoryComponent,
    effects: &'static ActiveEffects,
}

fn render_ui(
    mut contexts: EguiContexts,
    icons: Option<Res<UiIcons>>,
    character_query: Query<CharacterQueryData, With<ActiveCharacter>>,
    trait_registry: Res<crate::network::ClientTraitRegistry>,
    skill_registry: Res<crate::network::ClientSkillRegistry>,
    ability_registry: Res<crate::network::ClientAbilityRegistry>,
    weapon_registry: Res<crate::network::ClientWeaponRegistry>,
    equipment_registry: Res<crate::network::ClientEquipmentRegistry>,
    item_registry: Res<crate::network::ClientItemRegistry>,
    mut events: MessageWriter<ResourceChanged>,
    mut wallet_events: MessageWriter<WalletChanged>,
    mut inventory_events: MessageWriter<InventoryChanged>,
    mut exp_events: MessageWriter<ExperienceChanged>,
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
            let bottom_margin = top_margin;
            let col_h = total_h - top_margin - bottom_margin;

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
                    &weapon_registry,
                    &mut events,
                    &mut inventory_events,
                    &mut exp_events,
                );
                ui.add_space(gap);
                render_center_column(
                    ui,
                    total_w * COL2_WIDTH,
                    col_h,
                    &icons,
                    &character,
                    &trait_registry,
                    &skill_registry,
                    &ability_registry,
                    &mut events,
                );
                ui.add_space(gap);
                render_right_column(
                    ui,
                    total_w * COL3_WIDTH,
                    col_h,
                    &icons,
                    &character,
                    &weapon_registry,
                    &equipment_registry,
                    &item_registry,
                    &mut wallet_events,
                    &mut inventory_events,
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
    weapon_registry: &crate::network::ClientWeaponRegistry,
    events: &mut MessageWriter<ResourceChanged>,
    inventory_events: &mut MessageWriter<InventoryChanged>,
    exp_events: &mut MessageWriter<ExperienceChanged>,
) {
    let gap = height * 0.03 / 4.0;
    let initiative =
        character.stats.0.perception.level as i32 + character.effects.initiative_bonus();

    ui.vertical(|ui| {
        ui.spacing_mut().item_spacing = egui::vec2(0.0, 0.0);

        let portrait_size = egui::vec2(width, height * 0.30);
        let (portrait_rect, _) = ui.allocate_exact_size(portrait_size, egui::Sense::hover());
        let mut portrait_ui = ui.new_child(
            egui::UiBuilder::new()
                .max_rect(portrait_rect)
                .layout(egui::Layout::top_down(egui::Align::Min)),
        );
        if let Some(exp) = Portrait::new(
            icons.avatar_border_1.id(),
            icons.avatar_border_2.id(),
            icons.avatar_placeholder.id(),
            character.level.0,
            character.exp.0,
        )
        .show(&mut portrait_ui)
        {
            exp_events.write(ExperienceChanged(exp));
        }
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

        let weapon_slots: Vec<WeaponSlot> = character
            .weapon_names
            .0
            .iter()
            .filter_map(|name| {
                weapon_registry.0.get(name).map(|w| WeaponSlot {
                    name: w.name.clone(),
                    kind: w.kind.to_string(),
                    attack: format!("{:+}", w.attack),
                    damage: w.damage.clone(),
                    range: w.range.to_string(),
                })
            })
            .collect();

        let weapon_size = egui::vec2(width, height * 0.20);
        let (weapon_rect, _) = ui.allocate_exact_size(weapon_size, egui::Sense::hover());
        let mut weapon_ui = ui.new_child(
            egui::UiBuilder::new()
                .max_rect(weapon_rect)
                .layout(egui::Layout::top_down(egui::Align::Min)),
        );
        if let Some(i) =
            Weapon::new(icons.weapon_placeholder.id(), weapon_slots).show(&mut weapon_ui)
        {
            inventory_events.write(InventoryChanged::UnequipWeapon(i));
        }
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

/// Applies experience change messages to the active character's ECS components.
fn apply_experience_changes(
    mut query: Query<(&mut Experience, &mut Level), With<ActiveCharacter>>,
    mut reader: MessageReader<ExperienceChanged>,
    mut level_up: MessageWriter<LevelUp>,
) {
    let Ok((mut exp, mut level)) = query.single_mut() else {
        return;
    };
    for event in reader.read() {
        exp.0 += event.0;
        // Level up: subtract (next_level * 10) XP each time the threshold is met.
        loop {
            let needed = (level.0 + 1) * 10;
            if exp.0 < needed {
                break;
            }
            exp.0 -= needed;
            level.0 += 1;
            level_up.write(LevelUp);
        }
    }
}

/// Applies all OnLvlUp effects from active effects on level up.
fn apply_level_up(
    mut query: Query<
        (
            &ActiveEffects,
            &mut AbilityPoints,
            &mut SkillPoints,
            &mut CharacteristicPoints,
        ),
        With<ActiveCharacter>,
    >,
    mut reader: MessageReader<LevelUp>,
) {
    let Ok((effects, mut ability_pts, mut skill_pts, mut char_pts)) = query.single_mut() else {
        return;
    };
    for _ in reader.read() {
        for effect in &effects.0 {
            if let Effect::OnLvlUp(on_lvl_up) = effect {
                match on_lvl_up {
                    OnLvlUp::AddAbilityPoints(v) => {
                        ability_pts.0 = (ability_pts.0 as i32 + v).max(0) as u32;
                    }
                    OnLvlUp::AddSkillPoints(v) => {
                        skill_pts.0 = (skill_pts.0 as i32 + v).max(0) as u32;
                    }
                    OnLvlUp::AddCharacteristicPoints(v) => {
                        char_pts.0 = (char_pts.0 as i32 + v).max(0) as u32;
                    }
                }
            }
        }
    }
}

fn render_center_column(
    ui: &mut egui::Ui,
    width: f32,
    height: f32,
    icons: &UiIcons,
    character: &CharacterQueryDataItem,
    trait_registry: &crate::network::ClientTraitRegistry,
    skill_registry: &crate::network::ClientSkillRegistry,
    ability_registry: &crate::network::ClientAbilityRegistry,
    events: &mut MessageWriter<ResourceChanged>,
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
        let ability_entries: Vec<AbilityEntry> = character
            .ability_names
            .0
            .iter()
            .filter_map(|name| {
                let ability = ability_registry
                    .0
                    .get_class_abilities(&character.class.0)
                    .and_then(|ca| ca.innate.get(name).or_else(|| ca.acquire.get(name)));
                ability.map(|a| AbilityEntry {
                    name: name.clone(),
                    description: a.description.clone(),
                    image: icons.ability_placeholder.id(),
                    mp_cost: a.requirements.as_ref().and_then(|r| r.mp),
                    ap_cost: a.requirements.as_ref().and_then(|r| r.action_points),
                    self_only: a.self_only,
                    range: a.requirements.as_ref().and_then(|r| r.range),
                    ability_type: format_ability_type(a.ability_type),
                    check: a
                        .check
                        .as_ref()
                        .map(format_ability_check)
                        .unwrap_or_default(),
                    enemy_check: a
                        .enemy_check
                        .as_ref()
                        .map(|e| e.to_string())
                        .unwrap_or_default(),
                })
            })
            .collect();
        let abilities_size = egui::vec2(width, height * 0.40);
        let (abilities_rect, _) = ui.allocate_exact_size(abilities_size, egui::Sense::hover());
        let mut abilities_ui = ui.new_child(egui::UiBuilder::new().max_rect(abilities_rect));
        if let Some(new_mp) =
            Abilities::new(ability_entries, character.mana.current).show(&mut abilities_ui)
        {
            events.write(ResourceChanged::Mp(new_mp));
        }
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
        Effect::OnLvlUp(OnLvlUp::AddAbilityPoints(v)) => {
            format!("{v:+} Ability Points per level")
        }
        Effect::OnLvlUp(OnLvlUp::AddCharacteristicPoints(v)) => {
            format!("{v:+} Characteristic Points per level")
        }
    }
}

fn format_ability_type(t: AbilityType) -> String {
    match t {
        AbilityType::Stance => "Stance",
        AbilityType::Attack => "Attack",
        AbilityType::Debuff => "Debuff",
        AbilityType::Peaceful => "Peaceful",
        AbilityType::Passive => "Passive",
        AbilityType::Touch => "Touch",
    }
    .to_string()
}

fn format_ability_check(check: &AbilityCheck) -> String {
    check.to_string()
}

fn render_right_column(
    ui: &mut egui::Ui,
    width: f32,
    height: f32,
    icons: &UiIcons,
    character: &CharacterQueryDataItem,
    weapon_registry: &crate::network::ClientWeaponRegistry,
    equipment_registry: &crate::network::ClientEquipmentRegistry,
    item_registry: &crate::network::ClientItemRegistry,
    wallet_events: &mut MessageWriter<WalletChanged>,
    inventory_events: &mut MessageWriter<InventoryChanged>,
) {
    let gap = height * 0.03 / 2.0;
    let wallet = character.wallet;

    let inventory_items: Vec<Option<InventoryTooltip>> = character
        .inventory
        .0
        .iter()
        .map(|inv_item| match inv_item {
            shared::InventoryItem::Weapon(name) => {
                weapon_registry
                    .0
                    .get(name)
                    .map(|w| InventoryTooltip::Weapon {
                        name: w.name.clone(),
                        kind: w.kind.to_string(),
                        attack: format!("{:+}", w.attack),
                        damage: w.damage.clone(),
                        range: w.range.to_string(),
                    })
            }
            shared::InventoryItem::Equipment(name) => {
                equipment_registry
                    .0
                    .get(name)
                    .map(|e| InventoryTooltip::Equipment {
                        name: e.name.clone(),
                        slot: e.slot.to_string(),
                        description: e.description.clone(),
                        armor: e.armor,
                        effects: e.effects.iter().map(format_effect).collect(),
                    })
            }
            shared::InventoryItem::Item(name) => {
                item_registry.0.get(name).map(|i| InventoryTooltip::Item {
                    name: i.name.clone(),
                    description: i.description.clone(),
                })
            }
        })
        .collect();

    let equipped_items: Vec<Option<InventoryTooltip>> = character
        .equipment
        .0
        .values()
        .flat_map(|names| names.iter())
        .map(|name| {
            equipment_registry
                .0
                .get(name)
                .map(|e| InventoryTooltip::Equipment {
                    name: e.name.clone(),
                    slot: e.slot.to_string(),
                    description: e.description.clone(),
                    armor: e.armor,
                    effects: e.effects.iter().map(format_effect).collect(),
                })
        })
        .collect();

    ui.vertical(|ui| {
        ui.spacing_mut().item_spacing = egui::vec2(0.0, 0.0);

        let equipped_size = egui::vec2(width, height * 0.41);
        let (equipped_rect, _) = ui.allocate_exact_size(equipped_size, egui::Sense::hover());
        let mut equipped_ui = ui.new_child(
            egui::UiBuilder::new()
                .max_rect(equipped_rect)
                .layout(egui::Layout::top_down(egui::Align::Min)),
        );
        if let Some(i) = EquippedGear::new(icons.inventory_placeholder.id())
            .items(equipped_items)
            .show(&mut equipped_ui)
        {
            inventory_events.write(InventoryChanged::UnequipGear(i));
        }

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

        let inventory_size = egui::vec2(width, height * 0.48);
        let (inventory_rect, _) = ui.allocate_exact_size(inventory_size, egui::Sense::hover());
        let mut inventory_ui = ui.new_child(
            egui::UiBuilder::new()
                .max_rect(inventory_rect)
                .layout(egui::Layout::top_down(egui::Align::Min)),
        );
        match Inventory::new(icons.inventory_placeholder.id())
            .items(inventory_items)
            .show(&mut inventory_ui)
        {
            Some(CellAction::Primary(i)) => {
                inventory_events.write(InventoryChanged::Equip(i));
            }
            Some(CellAction::Remove(i)) => {
                inventory_events.write(InventoryChanged::Remove(i));
            }
            None => {}
        }
    });
}

fn send_wallet_events(events: &mut MessageWriter<WalletChanged>, result: WalletResponse) {
    for delta in [result.gold, result.silver, result.copper]
        .into_iter()
        .flatten()
    {
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

const MAX_EQUIPPED_WEAPONS: usize = 3;

fn apply_inventory_changes(
    mut query: Query<
        (
            &mut InventoryComponent,
            &mut CharacterEquipment,
            &mut CharacterWeaponNames,
        ),
        With<ActiveCharacter>,
    >,
    mut reader: MessageReader<InventoryChanged>,
    equipment_registry: Res<crate::network::ClientEquipmentRegistry>,
) {
    let Ok((mut inventory, mut equipment, mut weapons)) = query.single_mut() else {
        return;
    };
    for event in reader.read() {
        match event {
            InventoryChanged::Equip(idx) => {
                let idx = *idx;
                if idx >= inventory.0.len() {
                    continue;
                }
                match &inventory.0[idx] {
                    InventoryItem::Equipment(name) => {
                        let name = name.clone();
                        if let Some(eq) = equipment_registry.0.get(&name) {
                            let slot = eq.slot;
                            if slot != EquipmentSlot::Ring {
                                if let Some(existing) = equipment.0.get(&slot) {
                                    for old_name in existing.clone() {
                                        inventory.0.push(InventoryItem::Equipment(old_name));
                                    }
                                }
                                equipment.0.insert(slot, vec![name]);
                            } else {
                                equipment.0.entry(slot).or_default().push(name);
                            }
                            inventory.0.remove(idx);
                        }
                    }
                    InventoryItem::Weapon(name) => {
                        if weapons.0.len() >= MAX_EQUIPPED_WEAPONS {
                            continue;
                        }
                        let name = name.clone();
                        weapons.0.push(name);
                        inventory.0.remove(idx);
                    }
                    InventoryItem::Item(_) => {}
                }
            }
            InventoryChanged::Remove(idx) => {
                let idx = *idx;
                if idx < inventory.0.len() {
                    inventory.0.remove(idx);
                }
            }
            InventoryChanged::UnequipGear(idx) => {
                let idx = *idx;
                let mut current = 0;
                let mut target: Option<(EquipmentSlot, usize)> = None;
                for (slot, names) in equipment.0.iter() {
                    if idx < current + names.len() {
                        target = Some((*slot, idx - current));
                        break;
                    }
                    current += names.len();
                }
                if let Some((slot, inner_idx)) = target {
                    let name = equipment.0.get_mut(&slot).unwrap().remove(inner_idx);
                    if equipment.0.get(&slot).is_some_and(|v| v.is_empty()) {
                        equipment.0.remove(&slot);
                    }
                    inventory.0.push(InventoryItem::Equipment(name));
                }
            }
            InventoryChanged::UnequipWeapon(idx) => {
                let idx = *idx;
                if idx < weapons.0.len() {
                    let name = weapons.0.remove(idx);
                    inventory.0.push(InventoryItem::Weapon(name));
                }
            }
        }
    }
}
