use bevy::prelude::*;

use shared::character::OnLvlUp;
use shared::{CharacteristicKind, CharacteristicTrait, Effect, EquipmentSlot, InventoryItem};

use crate::components::{
    AbilityPoints, ActionPoints, ActiveCharacter, ActiveEffects, CharacterAbilityNames,
    CharacterClass, CharacterEquipment, CharacterSkillList, CharacterStats, CharacterTraitNames,
    CharacterWeaponNames, CharacteristicPoints, Experience, Hp, Inventory as InventoryComponent,
    Level, Mana, SkillPoints, TraitPoints, Wallet,
};
use crate::events::{
    CreateItem, ExperienceChanged, InventoryChanged, LearnAbility, LearnTrait, LevelUp,
    ResourceChanged, UpgradeEvent, WalletChanged,
};

use super::helpers::{check_trait_requirement, save_to_json_file};

/// Applies resource change messages to the active character's ECS components.
pub(super) fn apply_resource_changes(
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
pub(super) fn apply_experience_changes(
    mut query: Query<(&mut Experience, &mut Level), With<ActiveCharacter>>,
    mut reader: MessageReader<ExperienceChanged>,
    mut level_up: MessageWriter<LevelUp>,
) {
    let Ok((mut exp, mut level)) = query.single_mut() else {
        return;
    };
    for event in reader.read() {
        exp.0 += event.0;
        loop {
            let needed = shared::xp_to_next_level(level.0);
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
/// Also grants 1 trait point every 3 levels.
pub(super) fn apply_level_up(
    mut query: Query<
        (
            &Level,
            &ActiveEffects,
            &mut AbilityPoints,
            &mut SkillPoints,
            &mut CharacteristicPoints,
            &mut TraitPoints,
        ),
        With<ActiveCharacter>,
    >,
    mut reader: MessageReader<LevelUp>,
) {
    let Ok((level, effects, mut ability_pts, mut skill_pts, mut char_pts, mut trait_pts)) =
        query.single_mut()
    else {
        return;
    };
    let level_ups: u32 = reader.read().count() as u32;
    if level_ups == 0 {
        return;
    }
    for _ in 0..level_ups {
        for effect in effects.iter() {
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
    // Grant 1 trait point for each level divisible by 3 that was crossed.
    let prev_level = level.0 - level_ups;
    trait_pts.0 += level.0 / 3 - prev_level / 3;
}

/// Applies characteristic and skill upgrades from edit mode.
pub(super) fn apply_upgrades(
    mut query: Query<
        (
            &CharacterClass,
            &mut CharacterStats,
            &mut CharacteristicPoints,
            &mut SkillPoints,
            &mut CharacterSkillList,
            &ActiveEffects,
        ),
        With<ActiveCharacter>,
    >,
    mut reader: MessageReader<UpgradeEvent>,
    skill_registry: Res<crate::network::ClientSkillRegistry>,
) {
    let Ok((class, mut stats, mut char_pts, mut skill_pts, mut skills, effects)) =
        query.single_mut()
    else {
        return;
    };

    let char_kinds = [
        CharacteristicKind::Strength,
        CharacteristicKind::Dexterity,
        CharacteristicKind::Endurance,
        CharacteristicKind::Perception,
        CharacteristicKind::Magic,
        CharacteristicKind::Willpower,
        CharacteristicKind::Intellect,
        CharacteristicKind::Charisma,
    ];

    let char_bonuses = effects.characteristic_bonuses();

    for event in reader.read() {
        match event {
            UpgradeEvent::Characteristic(idx) => {
                let s = &mut **stats;
                let cost = match char_kinds.get(*idx) {
                    Some(CharacteristicKind::Strength) => s.strength.up(char_pts.0),
                    Some(CharacteristicKind::Dexterity) => s.dexterity.up(char_pts.0),
                    Some(CharacteristicKind::Endurance) => s.endurance.up(char_pts.0),
                    Some(CharacteristicKind::Perception) => s.perception.up(char_pts.0),
                    Some(CharacteristicKind::Magic) => s.magic.up(char_pts.0),
                    Some(CharacteristicKind::Willpower) => s.willpower.up(char_pts.0),
                    Some(CharacteristicKind::Intellect) => s.intellect.up(char_pts.0),
                    Some(CharacteristicKind::Charisma) => s.charisma.up(char_pts.0),
                    None => 0,
                };
                char_pts.0 -= cost;
            }
            UpgradeEvent::Skill(name) => {
                let max_level = skill_registry
                    .get_skill(class, name)
                    .map(|skill| {
                        let base = stats.get_level(skill.dependency);
                        let bonus = char_bonuses.get(&skill.dependency).copied().unwrap_or(0);
                        (base as i32 + bonus).max(0) as u32
                    })
                    .unwrap_or(0);

                let char_skill = skills.iter_mut().find(|s| s.name == *name);
                if let Some(skill) = char_skill {
                    let cost = skill.up(skill_pts.0, max_level);
                    skill_pts.0 -= cost;
                } else if skill_pts.0 >= 1 && max_level >= 1 {
                    skills.push(shared::CharacterSkill {
                        name: name.clone(),
                        level: 1,
                    });
                    skill_pts.0 -= 1;
                }
            }
        }
    }
}

/// Learns an ability: adds it to the character's ability list and deducts one ability point.
pub(super) fn apply_learn_ability(
    mut query: Query<(&mut CharacterAbilityNames, &mut AbilityPoints), With<ActiveCharacter>>,
    mut reader: MessageReader<LearnAbility>,
) {
    let Ok((mut abilities, mut pts)) = query.single_mut() else {
        return;
    };
    for event in reader.read() {
        if pts.0 > 0 && !abilities.contains(&event.0) {
            abilities.push(event.0.clone());
            pts.0 -= 1;
        }
    }
}

/// Learns a trait: validates conditions, adds it to the character's trait list and deducts one trait point.
pub(super) fn apply_learn_trait(
    mut query: Query<
        (&mut CharacterTraitNames, &mut TraitPoints, &CharacterStats),
        With<ActiveCharacter>,
    >,
    mut reader: MessageReader<LearnTrait>,
    trait_registry: Res<crate::network::ClientTraitRegistry>,
) {
    let Ok((mut traits, mut pts, stats)) = query.single_mut() else {
        return;
    };
    for event in reader.read() {
        if pts.0 == 0 || traits.contains(&event.0) {
            continue;
        }
        let Some(ct) = trait_registry.get(&event.0) else {
            continue;
        };
        if !check_trait_requirement(stats, ct.condition.as_ref()) {
            continue;
        }
        traits.push(event.0.clone());
        pts.0 -= 1;
    }
}

pub(super) fn apply_wallet_changes(
    mut query: Query<&mut Wallet, With<ActiveCharacter>>,
    mut reader: MessageReader<WalletChanged>,
) {
    let Ok(mut wallet) = query.single_mut() else {
        return;
    };
    for event in reader.read() {
        wallet.add(event.0);
    }
}

const MAX_EQUIPPED_WEAPONS: usize = 3;

pub(super) fn apply_inventory_changes(
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
                if idx >= inventory.len() {
                    continue;
                }
                match &inventory[idx] {
                    InventoryItem::Equipment(name) => {
                        let name = name.clone();
                        if let Some(eq) = equipment_registry.get(&name) {
                            let slot = eq.slot;
                            if slot != EquipmentSlot::Ring {
                                if let Some(existing) = equipment.get(&slot) {
                                    for old_name in existing.clone() {
                                        inventory.push(InventoryItem::Equipment(old_name));
                                    }
                                }
                                equipment.insert(slot, vec![name]);
                            } else {
                                equipment.entry(slot).or_default().push(name);
                            }
                            inventory.remove(idx);
                        }
                    }
                    InventoryItem::Weapon(name) => {
                        if weapons.len() >= MAX_EQUIPPED_WEAPONS {
                            continue;
                        }
                        let name = name.clone();
                        weapons.push(name);
                        inventory.remove(idx);
                    }
                    InventoryItem::Item(_) => {}
                }
            }
            InventoryChanged::Remove(idx) => {
                let idx = *idx;
                if idx < inventory.len() {
                    inventory.remove(idx);
                }
            }
            InventoryChanged::UnequipGear(idx) => {
                let idx = *idx;
                let mut current = 0;
                let mut target: Option<(EquipmentSlot, usize)> = None;
                for (slot, names) in equipment.iter() {
                    if idx < current + names.len() {
                        target = Some((*slot, idx - current));
                        break;
                    }
                    current += names.len();
                }
                if let Some((slot, inner_idx)) = target {
                    let name = equipment.get_mut(&slot).unwrap().remove(inner_idx);
                    if equipment.get(&slot).is_some_and(|v| v.is_empty()) {
                        equipment.remove(&slot);
                    }
                    inventory.push(InventoryItem::Equipment(name));
                }
            }
            InventoryChanged::UnequipWeapon(idx) => {
                let idx = *idx;
                if idx < weapons.len() {
                    let name = weapons.remove(idx);
                    inventory.push(InventoryItem::Weapon(name));
                }
            }
            InventoryChanged::AddExisting(item) => {
                inventory.push(item.clone());
            }
        }
    }
}

pub(super) fn apply_create_item(
    mut query: Query<&mut InventoryComponent, With<ActiveCharacter>>,
    mut reader: MessageReader<CreateItem>,
    mut weapon_registry: ResMut<crate::network::ClientWeaponRegistry>,
    mut equipment_registry: ResMut<crate::network::ClientEquipmentRegistry>,
    mut item_registry: ResMut<crate::network::ClientItemRegistry>,
    mut pending_messages: ResMut<crate::network::PendingClientMessages>,
) {
    let Ok(mut inventory) = query.single_mut() else {
        return;
    };
    for event in reader.read() {
        match event {
            CreateItem::Weapon(weapon) => {
                let item_name = weapon.name.clone();
                weapon_registry
                    .weapons
                    .insert(item_name.clone(), weapon.clone());
                inventory.push(InventoryItem::Weapon(item_name));
                save_to_json_file(
                    "data/weapons.json",
                    weapon_registry.weapons.values().collect(),
                );
                pending_messages.push(shared::ClientMessage::CreateWeapon {
                    weapon: weapon.clone(),
                });
            }
            CreateItem::Equipment(eq) => {
                let item_name = eq.name.clone();
                equipment_registry
                    .equipment
                    .insert(item_name.clone(), eq.clone());
                inventory.push(InventoryItem::Equipment(item_name));
                save_to_json_file(
                    "data/equipment.json",
                    equipment_registry.equipment.values().collect(),
                );
                pending_messages.push(shared::ClientMessage::CreateEquipment {
                    equipment: eq.clone(),
                });
            }
            CreateItem::Item(item) => {
                let item_name = item.name.clone();
                item_registry
                    .items
                    .insert(item_name.clone(), item.clone());
                inventory.push(InventoryItem::Item(item_name));
                save_to_json_file("data/items.json", item_registry.items.values().collect());
                pending_messages.push(shared::ClientMessage::CreateItem { item: item.clone() });
            }
        }
    }
}
