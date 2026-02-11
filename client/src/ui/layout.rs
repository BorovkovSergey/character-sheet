use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use ui_widgets::colors::MAIN_COLOR;
use ui_widgets::composites::{
    Abilities, AbilityEntry, AddItemMenu, AddItemSelection, Characteristics, EquippedGear,
    GridAction, IdentityBar, Inventory, Points, Portrait, SkillEntry, Skills, Stats, StatusBar,
    StatusBarResponse, TraitEntry, Traits, Wallet as WalletWidget, WalletResponse, Weapon,
    WeaponSlot,
};
use ui_widgets::molecules::{CellAction, InventoryTooltip};

use crate::components::{
    AbilityPoints, ActionPoints, ActiveCharacter, ActiveEffects, CharacterAbilityNames,
    CharacterClass, CharacterEquipment, CharacterId, CharacterName, CharacterRace,
    CharacterSkillList, CharacterStats, CharacterTraitNames, CharacterWeaponNames,
    CharacteristicPoints, Experience, Hp, Inventory as InventoryComponent, Level, Mana,
    SkillPoints, TraitPoints, Wallet,
};
use crate::events::{
    ExperienceChanged, InventoryChanged, ResourceChanged, UpgradeEvent, WalletChanged,
};

use super::helpers::format_effect;
use super::icons::UiIcons;
use super::params::{Registries, UiEvents, UiModals};

const MARGIN: f32 = 0.02;
const COL_GAP: f32 = 0.01;
const COL1_WIDTH: f32 = 0.24;
const COL2_WIDTH: f32 = 0.46;
const COL3_WIDTH: f32 = 0.24;

#[derive(bevy::ecs::query::QueryData)]
pub(super) struct CharacterQueryData {
    pub id: &'static CharacterId,
    pub name: &'static CharacterName,
    pub race: &'static CharacterRace,
    pub class: &'static CharacterClass,
    pub level: &'static Level,
    pub exp: &'static Experience,
    pub hp: &'static Hp,
    pub mana: &'static Mana,
    pub ap: &'static ActionPoints,
    pub stats: &'static CharacterStats,
    pub trait_names: &'static CharacterTraitNames,
    pub ability_names: &'static CharacterAbilityNames,
    pub char_pts: &'static CharacteristicPoints,
    pub skill_pts: &'static SkillPoints,
    pub ability_pts: &'static AbilityPoints,
    pub trait_pts: &'static TraitPoints,
    pub skills: &'static CharacterSkillList,
    pub wallet: &'static Wallet,
    pub weapon_names: &'static CharacterWeaponNames,
    pub equipment: &'static CharacterEquipment,
    pub inventory: &'static InventoryComponent,
    pub effects: &'static ActiveEffects,
}

pub(super) fn render_ui(
    mut contexts: EguiContexts,
    icons: Option<Res<UiIcons>>,
    character_query: Query<CharacterQueryData, With<ActiveCharacter>>,
    registries: Registries,
    mut ui_events: UiEvents,
    mut modals: UiModals,
    mut pending_messages: ResMut<crate::network::PendingClientMessages>,
    mut next_state: ResMut<NextState<crate::state::AppScreen>>,
) -> Result {
    let Some(icons) = icons else {
        return Ok(());
    };

    let Ok(character) = character_query.single() else {
        return Ok(());
    };

    let ctx = contexts.ctx_mut()?;

    let mut save_clicked = false;
    let mut back_clicked = false;

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
                let left_resp = render_left_column(
                    ui,
                    total_w * COL1_WIDTH,
                    col_h,
                    &icons,
                    &character,
                    &registries,
                    &mut ui_events,
                    &mut modals,
                );
                save_clicked = left_resp.save;
                back_clicked = left_resp.back;
                ui.add_space(gap);
                render_center_column(
                    ui,
                    total_w * COL2_WIDTH,
                    col_h,
                    &icons,
                    &character,
                    &registries,
                    &mut ui_events,
                    modals.edit_mode.0,
                );
                ui.add_space(gap);
                render_right_column(
                    ui,
                    total_w * COL3_WIDTH,
                    col_h,
                    &icons,
                    &character,
                    &registries,
                    &mut ui_events,
                );
            });
        });

    if save_clicked {
        let ch = build_character_from_components(&character);
        pending_messages
            .0
            .push(shared::ClientMessage::UpdateCharacter { character: ch });
    }

    if back_clicked {
        next_state.set(crate::state::AppScreen::CharacterSelect);
    }

    // "Learn Ability" overlay
    if modals.learn_ability.0 {
        super::overlays::render_learn_ability_overlay(
            ctx,
            &character,
            &icons,
            &registries,
            &mut ui_events,
            &mut modals.learn_ability,
        );
    }

    // "Learn Trait" overlay
    if modals.learn_trait.0 {
        let trait_state_id = egui::Id::new("learn_trait_selected");
        let mut trait_selected: Vec<String> =
            ctx.data(|d| d.get_temp(trait_state_id)).unwrap_or_default();

        let trait_result = super::overlays::render_trait_select_overlay(
            ctx,
            character.stats,
            &registries.traits,
            &mut trait_selected,
            &mut modals.learn_trait.0,
            super::overlays::TraitSelectMode::Single {
                known_traits: character.trait_names,
                has_points: character.trait_pts.0 > 0,
            },
            "learn_trait",
        );

        if let super::overlays::TraitSelectResult::Confirmed(name) = trait_result {
            ui_events.learn_trait.write(crate::events::LearnTrait(name));
        }

        ctx.data_mut(|d| d.insert_temp(trait_state_id, trait_selected));
    }

    // "Create Item" overlay
    if modals.create_item.0 {
        let skill_names: Vec<String> = registries
            .skills
            .classes
            .values()
            .flat_map(|skills| skills.keys().cloned())
            .collect();
        crate::create_item::render_create_item_popup(
            ctx,
            &mut modals.create_item,
            &mut ui_events.create_item,
            &format_effect,
            &skill_names,
        );
    }

    Ok(())
}

struct LeftColumnResponse {
    save: bool,
    back: bool,
}

fn render_left_column(
    ui: &mut egui::Ui,
    width: f32,
    height: f32,
    icons: &UiIcons,
    character: &CharacterQueryDataItem,
    registries: &Registries,
    ui_events: &mut UiEvents,
    modals: &mut UiModals,
) -> LeftColumnResponse {
    let gap = height * 0.03 / 4.0;
    let initiative = character.stats.perception.level as i32 + character.effects.initiative_bonus();

    ui.vertical(|ui| {
        ui.spacing_mut().item_spacing = egui::vec2(0.0, 0.0);

        let portrait_size = egui::vec2(width, height * 0.30);
        let (portrait_rect, _) = ui.allocate_exact_size(portrait_size, egui::Sense::hover());
        let mut portrait_ui = ui.new_child(
            egui::UiBuilder::new()
                .max_rect(portrait_rect)
                .layout(egui::Layout::top_down(egui::Align::Min)),
        );
        let add_item_menu = build_add_item_menu(
            &registries.weapons,
            &registries.equipment,
            &registries.items,
        );
        let xp_fraction =
            character.exp.0 as f32 / shared::xp_to_next_level(character.level.0) as f32;
        let portrait_resp = Portrait::new(
            icons.avatar_border_1.id(),
            icons.avatar_border_2.id(),
            icons.avatar_placeholder.id(),
            character.level.0,
            xp_fraction,
            modals.edit_mode.0,
        )
        .shield(icons.shield.id(), character.effects.armor())
        .ability_points(character.ability_pts.0)
        .trait_points(character.trait_pts.0)
        .add_item_menu(add_item_menu)
        .show(&mut portrait_ui);
        let save_clicked = portrait_resp.save;
        let back_clicked = portrait_resp.back;
        if let Some(exp) = portrait_resp.add_exp {
            ui_events.experience.write(ExperienceChanged(exp));
        }
        if portrait_resp.toggle_edit {
            modals.edit_mode.0 = !modals.edit_mode.0;
        }
        if portrait_resp.open_learn_ability {
            modals.learn_ability.0 = true;
        }
        if portrait_resp.open_learn_trait {
            modals.learn_trait.0 = true;
        }
        if portrait_resp.open_create_item {
            modals.create_item.0 = true;
        }
        if let Some(selection) = portrait_resp.add_item {
            let inv_item = match selection {
                AddItemSelection::Item(name) => shared::InventoryItem::Item(name),
                AddItemSelection::Equipment(name) => shared::InventoryItem::Equipment(name),
                AddItemSelection::Weapon(name) => shared::InventoryItem::Weapon(name),
            };
            ui_events
                .inventory
                .write(InventoryChanged::AddExisting(inv_item));
        }
        ui.add_space(gap);
        ui.add_sized(
            [width, height * 0.11],
            IdentityBar::new(
                &character.name.0,
                character.race.to_string(),
                character.class.to_string(),
            ),
        );
        ui.add_space(gap);

        send_status_bar_events(
            ui,
            width,
            height * 0.16,
            character,
            initiative,
            &mut ui_events.resource,
        );
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
            .iter()
            .filter_map(|name| {
                registries.weapons.get(name).map(|w| WeaponSlot {
                    name: w.name.clone(),
                    kind: w.kind.to_string(),
                    attack: format!("{:+}", w.attack),
                    damage: w.damage.clone(),
                    range: w.range.to_string(),
                    condition: w.condition.clone().unwrap_or_default(),
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
            ui_events
                .inventory
                .write(InventoryChanged::UnequipWeapon(i));
        }

        LeftColumnResponse {
            save: save_clicked,
            back: back_clicked,
        }
    })
    .inner
}

fn build_add_item_menu(
    weapon_registry: &shared::WeaponRegistry,
    equipment_registry: &shared::EquipmentRegistry,
    item_registry: &shared::ItemRegistry,
) -> AddItemMenu {
    use std::collections::BTreeMap;

    let items: Vec<InventoryTooltip> = item_registry
        .items
        .values()
        .map(|i| InventoryTooltip::Item {
            name: i.name.clone(),
            description: i.description.clone(),
        })
        .collect();

    let mut equipment: BTreeMap<String, Vec<InventoryTooltip>> = BTreeMap::new();
    for eq in equipment_registry.equipment.values() {
        equipment
            .entry(eq.slot.to_string())
            .or_default()
            .push(InventoryTooltip::Equipment {
                name: eq.name.clone(),
                slot: eq.slot.to_string(),
                description: eq.description.clone(),
                effects: eq.effects.iter().map(format_effect).collect(),
            });
    }

    let mut weapons: BTreeMap<String, Vec<InventoryTooltip>> = BTreeMap::new();
    for w in weapon_registry.weapons.values() {
        weapons
            .entry(w.kind.to_string())
            .or_default()
            .push(InventoryTooltip::Weapon {
                name: w.name.clone(),
                kind: w.kind.to_string(),
                attack: format!("{:+}", w.attack),
                damage: w.damage.clone(),
                range: w.range.to_string(),
                condition: w.condition.clone().unwrap_or_default(),
                effects: w.effects.iter().map(format_effect).collect(),
            });
    }

    AddItemMenu {
        items,
        equipment,
        weapons,
    }
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

fn build_character_from_components(c: &CharacterQueryDataItem) -> shared::Character {
    shared::Character {
        id: c.id.0,
        name: c.name.0.clone(),
        race: c.race.0,
        class: c.class.0,
        level: c.level.0,
        experience: c.exp.0,
        hp_spent: c.hp.max.saturating_sub(c.hp.current),
        mana_spent: c.mana.max.saturating_sub(c.mana.current),
        action_points: shared::Resource {
            current: c.ap.current,
            max: c.ap.max,
        },
        stats: **c.stats,
        characteristic_points: c.char_pts.0,
        skill_points: c.skill_pts.0,
        ability_points: c.ability_pts.0,
        trait_points: c.trait_pts.0,
        skills: c.skills.to_vec(),
        traits: c.trait_names.to_vec(),
        abilities: c.ability_names.to_vec(),
        equipped_weapons: c.weapon_names.to_vec(),
        equipped_equipment: c.equipment.0.clone(),
        inventory: c.inventory.to_vec(),
        wallet: **c.wallet,
        active_effects: Vec::new(),
    }
}

fn render_center_column(
    ui: &mut egui::Ui,
    width: f32,
    height: f32,
    icons: &UiIcons,
    character: &CharacterQueryDataItem,
    registries: &Registries,
    ui_events: &mut UiEvents,
    edit_mode: bool,
) {
    let gap = height * 0.03 / 4.0;
    let stats = character.stats;

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

        let char_size = egui::vec2(width, height * 0.14);
        let (char_rect, _) = ui.allocate_exact_size(char_size, egui::Sense::hover());
        let mut char_ui = ui.new_child(
            egui::UiBuilder::new()
                .max_rect(char_rect)
                .layout(egui::Layout::top_down(egui::Align::Min)),
        );
        if let Some(GridAction::Upgrade(idx)) = Characteristics::new(char_values)
            .edit_mode(edit_mode, character.char_pts.0)
            .show(&mut char_ui)
        {
            ui_events.upgrade.write(UpgradeEvent::Characteristic(idx));
        }
        ui.add_space(gap);
        ui.add_sized(
            [width, height * 0.05],
            Points::new(character.char_pts.0, character.skill_pts.0 as i32),
        );
        ui.add_space(gap);
        let skill_entries: Vec<SkillEntry> = registries
            .skills
            .get_class_skills(character.class)
            .into_iter()
            .flat_map(|skills| skills.iter())
            .map(|(name, skill)| {
                let base_level = character
                    .skills
                    .iter()
                    .find(|s| s.name == *name)
                    .map_or(0, |s| s.level);
                let skill_bonus = character.effects.skill_bonus(name);
                let max_level = character.effects.effective_level(stats, skill.dependency);
                SkillEntry {
                    name: name.clone(),
                    dependency: skill.dependency.to_string(),
                    level: base_level as i32 + skill_bonus,
                    max_level,
                }
            })
            .collect();

        let skill_size = egui::vec2(width, height * 0.24);
        let (skill_rect, _) = ui.allocate_exact_size(skill_size, egui::Sense::hover());
        let mut skill_ui = ui.new_child(
            egui::UiBuilder::new()
                .max_rect(skill_rect)
                .layout(egui::Layout::top_down(egui::Align::Min)),
        );
        if let Some(GridAction::Upgrade(idx)) = Skills::new(skill_entries)
            .edit_mode(edit_mode, character.skill_pts.0)
            .show(&mut skill_ui)
        {
            if let Some(name) = registries
                .skills
                .get_class_skills(character.class)
                .into_iter()
                .flat_map(|skills| skills.keys())
                .nth(idx)
            {
                ui_events.upgrade.write(UpgradeEvent::Skill(name.clone()));
            }
        }
        ui.add_space(gap);
        let trait_entries: Vec<TraitEntry> = character
            .trait_names
            .iter()
            .filter_map(|name| {
                registries.traits.get(name).map(|ct| TraitEntry {
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
            .iter()
            .filter_map(|name| {
                let ability = registries
                    .abilities
                    .get_class_abilities(character.class)
                    .and_then(|ca| ca.innate.get(name).or_else(|| ca.acquire.get(name)));
                ability.map(|a| AbilityEntry {
                    name: name.clone(),
                    description: a.description.clone(),
                    image: icons.ability_placeholder.id(),
                    mp_cost: a.requirements.as_ref().and_then(|r| r.mp),
                    ap_cost: a.requirements.as_ref().and_then(|r| r.action_points),
                    self_only: a.self_only,
                    range: a.requirements.as_ref().and_then(|r| r.range),
                    ability_type: a.ability_type.to_string(),
                    check: a.check.as_ref().map(|c| c.to_string()).unwrap_or_default(),
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
            ui_events.resource.write(ResourceChanged::Mp(new_mp));
        }
    });
}

fn render_right_column(
    ui: &mut egui::Ui,
    width: f32,
    height: f32,
    icons: &UiIcons,
    character: &CharacterQueryDataItem,
    registries: &Registries,
    ui_events: &mut UiEvents,
) {
    let gap = height * 0.03 / 2.0;
    let wallet = character.wallet;

    let inventory_items: Vec<Option<InventoryTooltip>> = character
        .inventory
        .iter()
        .map(|inv_item| match inv_item {
            shared::InventoryItem::Weapon(name) => {
                registries
                    .weapons
                    .get(name)
                    .map(|w| InventoryTooltip::Weapon {
                        name: w.name.clone(),
                        kind: w.kind.to_string(),
                        attack: format!("{:+}", w.attack),
                        damage: w.damage.clone(),
                        range: w.range.to_string(),
                        condition: w.condition.clone().unwrap_or_default(),
                        effects: w.effects.iter().map(format_effect).collect(),
                    })
            }
            shared::InventoryItem::Equipment(name) => {
                registries
                    .equipment
                    .get(name)
                    .map(|e| InventoryTooltip::Equipment {
                        name: e.name.clone(),
                        slot: e.slot.to_string(),
                        description: e.description.clone(),
                        effects: e.effects.iter().map(format_effect).collect(),
                    })
            }
            shared::InventoryItem::Item(name) => {
                registries.items.get(name).map(|i| InventoryTooltip::Item {
                    name: i.name.clone(),
                    description: i.description.clone(),
                })
            }
        })
        .collect();

    let equipped_items: Vec<Option<InventoryTooltip>> = character
        .equipment
        .values()
        .flat_map(|names| names.iter())
        .map(|name| {
            registries
                .equipment
                .get(name)
                .map(|e| InventoryTooltip::Equipment {
                    name: e.name.clone(),
                    slot: e.slot.to_string(),
                    description: e.description.clone(),
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
            ui_events.inventory.write(InventoryChanged::UnequipGear(i));
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
        send_wallet_events(&mut ui_events.wallet, result);

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
                ui_events.inventory.write(InventoryChanged::Equip(i));
            }
            Some(CellAction::Remove(i)) => {
                ui_events.inventory.write(InventoryChanged::Remove(i));
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
