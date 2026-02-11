use std::collections::BTreeMap;

use bevy::prelude::*;
use shared::character::OnLvlUp;
use shared::{
    Character, CharacterSkill, Characteristics, Class, Effect, EquipmentSlot, GetEffects,
    InventoryItem, Protection, Race, Resist,
};
use strum::IntoEnumIterator;
use uuid::Uuid;

/// Marker for the currently active (selected) character entity.
#[derive(Component)]
pub struct ActiveCharacter;

#[derive(Component)]
#[allow(dead_code)]
pub struct CharacterId(pub Uuid);

#[derive(Component)]
pub struct CharacterName(pub String);

#[derive(Component)]
pub struct CharacterRace(pub Race);

#[derive(Component)]
pub struct CharacterClass(pub Class);

#[derive(Component)]
pub struct Level(pub u32);

#[derive(Component)]
pub struct Experience(pub u32);

#[derive(Component)]
pub struct Hp {
    pub current: u32,
    pub max: u32,
}

#[derive(Component)]
pub struct Mana {
    pub current: u32,
    pub max: u32,
}

#[derive(Component)]
pub struct ActionPoints {
    pub current: u32,
    pub max: u32,
}

#[derive(Component)]
pub struct CharacterStats(pub Characteristics);

#[derive(Component)]
pub struct CharacteristicPoints(pub u32);

#[derive(Component)]
pub struct SkillPoints(pub u32);

#[derive(Component)]
pub struct AbilityPoints(pub u32);

#[derive(Component)]
pub struct TraitPoints(pub u32);

#[derive(Component)]
#[allow(dead_code)]
pub struct CharacterSkillList(pub Vec<CharacterSkill>);

#[derive(Component)]
pub struct CharacterTraitNames(pub Vec<String>);

#[derive(Component)]
pub struct CharacterAbilityNames(pub Vec<String>);

#[derive(Component)]
pub struct Wallet(pub u64);

impl Wallet {
    pub fn gold(&self) -> u32 {
        (self.0 / 1000) as u32
    }

    pub fn silver(&self) -> u32 {
        ((self.0 % 1000) / 10) as u32
    }

    pub fn copper(&self) -> u32 {
        (self.0 % 10) as u32
    }
}

#[derive(Component)]
#[allow(dead_code)]
pub struct CharacterWeaponNames(pub Vec<String>);

#[derive(Component)]
pub struct CharacterEquipment(pub BTreeMap<EquipmentSlot, Vec<String>>);

#[derive(Component)]
#[allow(dead_code)]
pub struct Inventory(pub Vec<InventoryItem>);

#[derive(Component)]
pub struct ActiveEffects(pub Vec<Effect>);

impl ActiveEffects {
    pub fn get_resists(&self) -> BTreeMap<Resist, u32> {
        let mut result: BTreeMap<Resist, u32> = Resist::iter().map(|k| (k, 0)).collect();
        for effect in &self.0 {
            if let Effect::Resist(r, m) = effect {
                *result.entry(*r).or_insert(0) += m;
            }
        }
        result
    }

    pub fn get_protections(&self) -> BTreeMap<Protection, u32> {
        let mut result: BTreeMap<Protection, u32> = Protection::iter().map(|k| (k, 0)).collect();
        for effect in &self.0 {
            if let Effect::Protection(p, m) = effect {
                *result.entry(*p).or_insert(0) += m;
            }
        }
        result
    }

    pub fn armor(&self) -> i32 {
        self.0
            .iter()
            .filter_map(|e| match e {
                Effect::Armor(v) => Some(*v),
                _ => None,
            })
            .sum()
    }

    pub fn initiative_bonus(&self) -> i32 {
        self.0
            .iter()
            .filter_map(|e| match e {
                Effect::Initiative(v) => Some(*v),
                _ => None,
            })
            .sum()
    }
}

/// Spawns a new entity representing the active character with all ECS components.
pub fn spawn_character(commands: &mut Commands, character: &Character) -> Entity {
    commands
        .spawn((
            ActiveCharacter,
            CharacterId(character.id),
            CharacterName(character.name.clone()),
            CharacterRace(character.race),
            CharacterClass(character.class),
            Level(character.level),
            Experience(character.experience),
            {
                let max = character.stats.endurance.level * 3 + 3;
                Hp {
                    current: max.saturating_sub(character.hp_spent),
                    max,
                }
            },
            {
                let max = character.stats.willpower.level * 3 + 3;
                Mana {
                    current: max.saturating_sub(character.mana_spent),
                    max,
                }
            },
            ActionPoints {
                current: character.action_points.current,
                max: character.action_points.max,
            },
        ))
        .insert((
            CharacterStats(character.stats),
            CharacteristicPoints(character.characteristic_points),
            SkillPoints(character.skill_points),
            AbilityPoints(character.ability_points),
            TraitPoints(character.trait_points),
            CharacterSkillList(character.skills.clone()),
            CharacterTraitNames(character.traits.clone()),
            CharacterAbilityNames(character.abilities.clone()),
            CharacterWeaponNames(character.equipped_weapons.clone()),
            CharacterEquipment(character.equipped_equipment.clone()),
            Inventory(character.inventory.clone()),
            Wallet(character.wallet),
            ActiveEffects(character.active_effects.clone()),
        ))
        .id()
}

/// Recalculates active effects from race, traits, equipment, and base level-up bonuses.
pub fn recalculate_effects(
    mut query: Query<
        (
            &CharacterRace,
            &CharacterTraitNames,
            &CharacterWeaponNames,
            &CharacterEquipment,
            &CharacterStats,
            &mut ActiveEffects,
            &mut Hp,
            &mut Mana,
        ),
        Or<(
            Changed<CharacterRace>,
            Changed<CharacterTraitNames>,
            Changed<CharacterWeaponNames>,
            Changed<CharacterEquipment>,
            Changed<CharacterStats>,
        )>,
    >,
    trait_registry: Res<crate::network::ClientTraitRegistry>,
    weapon_registry: Res<crate::network::ClientWeaponRegistry>,
    equipment_registry: Res<crate::network::ClientEquipmentRegistry>,
) {
    for (race, traits, weapons, equipment, stats, mut effects, mut hp, mut mana) in &mut query {
        effects.0.clear();

        let s = &stats.0;
        let default_effects = vec![
            Effect::OnLvlUp(OnLvlUp::AddAbilityPoints(1)),
            Effect::OnLvlUp(OnLvlUp::AddSkillPoints(3 + s.intellect.level as i32)),
            Effect::OnLvlUp(OnLvlUp::AddCharacteristicPoints(2)),
            Effect::Protection(Protection::Melee, 10 + s.dexterity.level),
            Effect::Protection(Protection::Magic, 10 + s.magic.level),
            Effect::Protection(Protection::Body, 10 + s.endurance.level),
            Effect::Protection(Protection::Mind, 10 + s.willpower.level),
            // ProtectionRange is taking from the size
        ];
        effects.0.extend(default_effects);
        effects.0.extend(race.0.get_effects());
        effects.0.extend(race.0.size().get_effects());
        for trait_name in &traits.0 {
            if let Some(ct) = trait_registry.0.get(trait_name) {
                effects.0.extend(ct.effects.iter().cloned());
            }
        }
        for weapon_name in &weapons.0 {
            if let Some(w) = weapon_registry.0.get(weapon_name) {
                effects.0.extend(w.effects.iter().cloned());
            }
        }
        for names in equipment.0.values() {
            for equipment_name in names {
                if let Some(eq) = equipment_registry.0.get(equipment_name) {
                    effects.0.extend(eq.effects.iter().cloned());
                }
            }
        }

        // Recompute max HP and Mana from stats
        let new_max_hp = s.endurance.level * 3 + 3;
        if hp.max != new_max_hp {
            let spent = hp.max.saturating_sub(hp.current);
            hp.max = new_max_hp;
            hp.current = new_max_hp.saturating_sub(spent);
        }

        let new_max_mana = s.willpower.level * 3 + 3;
        if mana.max != new_max_mana {
            let spent = mana.max.saturating_sub(mana.current);
            mana.max = new_max_mana;
            mana.current = new_max_mana.saturating_sub(spent);
        }
    }
}

/// Despawns the active character entity when leaving the character sheet screen.
pub fn despawn_active_character(
    mut commands: Commands,
    query: Query<Entity, With<ActiveCharacter>>,
) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}
