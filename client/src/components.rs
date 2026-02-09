use std::collections::BTreeMap;

use bevy::prelude::*;
use shared::{
    Character, CharacterSkill, Characteristics, Class, Effect, GetEffects, Protection, Race, Resist,
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
#[allow(dead_code)]
pub struct CharacterSkillList(pub Vec<CharacterSkill>);

#[derive(Component)]
pub struct CharacterTraitNames(pub Vec<String>);

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
            Hp {
                current: character.hp.current,
                max: character.hp.max,
            },
            Mana {
                current: character.mana.current,
                max: character.mana.max,
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
            CharacterSkillList(character.skills.clone()),
            CharacterTraitNames(character.traits.clone()),
            ActiveEffects(character.active_effects.clone()),
        ))
        .id()
}

/// Recalculates active effects from race and traits whenever they change.
pub fn recalculate_effects(
    mut query: Query<
        (&CharacterRace, &CharacterTraitNames, &mut ActiveEffects),
        Or<(Changed<CharacterRace>, Changed<CharacterTraitNames>)>,
    >,
    trait_registry: Res<crate::network::ClientTraitRegistry>,
) {
    for (race, traits, mut effects) in &mut query {
        effects.0.clear();
        effects.0.extend(race.0.get_effects());
        effects.0.extend(race.0.size().get_effects());
        for trait_name in &traits.0 {
            if let Some(ct) = trait_registry.0.get(trait_name) {
                effects.0.extend(ct.effects.iter().cloned());
            }
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
