use std::collections::BTreeMap;

use bevy::prelude::*;
use shared::character::OnLvlUp;
use shared::{
    Character, CharacterSkill, CharacteristicKind, Characteristics, Class, Effect, EquipmentSlot,
    InventoryItem, Protection, Race, Resist, Wallet as SharedWallet,
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

#[derive(Component, Deref)]
pub struct CharacterRace(pub Race);

#[derive(Component, Deref)]
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

#[derive(Component, Deref, DerefMut)]
pub struct CharacterStats(pub Characteristics);

#[derive(Component)]
pub struct CharacteristicPoints(pub u32);

#[derive(Component)]
pub struct SkillPoints(pub u32);

#[derive(Component)]
pub struct AbilityPoints(pub u32);

#[derive(Component)]
pub struct TraitPoints(pub u32);

#[derive(Component, Deref, DerefMut)]
#[allow(dead_code)]
pub struct CharacterSkillList(pub Vec<CharacterSkill>);

#[derive(Component, Deref, DerefMut)]
pub struct CharacterTraitNames(pub Vec<String>);

#[derive(Component, Deref, DerefMut)]
pub struct CharacterAbilityNames(pub Vec<String>);

#[derive(Component, Deref, DerefMut)]
pub struct Wallet(pub SharedWallet);

#[derive(Component, Deref, DerefMut)]
#[allow(dead_code)]
pub struct CharacterWeaponNames(pub Vec<String>);

#[derive(Component, Deref, DerefMut)]
pub struct CharacterEquipment(pub BTreeMap<EquipmentSlot, Vec<String>>);

#[derive(Component, Deref, DerefMut)]
#[allow(dead_code)]
pub struct Inventory(pub Vec<InventoryItem>);

#[derive(Component, Deref, DerefMut)]
pub struct ActiveEffects(pub Vec<Effect>);

impl ActiveEffects {
    pub fn get_resists(&self) -> BTreeMap<Resist, u32> {
        let mut result: BTreeMap<Resist, u32> = Resist::iter().map(|k| (k, 0)).collect();
        for effect in self.iter() {
            if let Effect::Resist(r, m) = effect {
                *result.entry(*r).or_insert(0) += m;
            }
        }
        result
    }

    pub fn get_protections(&self) -> BTreeMap<Protection, u32> {
        let mut result: BTreeMap<Protection, u32> = Protection::iter().map(|k| (k, 0)).collect();
        for effect in self.iter() {
            if let Effect::Protection(p, m) = effect {
                *result.entry(*p).or_insert(0) += m;
            }
        }
        result
    }

    pub fn armor(&self) -> i32 {
        self.iter()
            .filter_map(|e| match e {
                Effect::Armor(v) => Some(*v),
                _ => None,
            })
            .sum()
    }

    pub fn initiative_bonus(&self) -> i32 {
        self.iter()
            .filter_map(|e| match e {
                Effect::Initiative(v) => Some(*v),
                _ => None,
            })
            .sum()
    }

    pub fn characteristic_bonuses(&self) -> BTreeMap<CharacteristicKind, i32> {
        let mut result = BTreeMap::new();
        for effect in self.iter() {
            if let Effect::Characteristic(kind, v) = effect {
                *result.entry(*kind).or_insert(0) += v;
            }
        }
        result
    }

    /// Returns the effective characteristic level (base + bonus from effects), minimum 0.
    pub fn effective_level(&self, stats: &Characteristics, kind: CharacteristicKind) -> u32 {
        let base = stats.get_level(kind);
        let bonus: i32 = self
            .0
            .iter()
            .filter_map(|e| match e {
                Effect::Characteristic(k, v) if *k == kind => Some(*v),
                _ => None,
            })
            .sum();
        (base as i32 + bonus).max(0) as u32
    }

    pub fn skill_bonus(&self, skill_name: &str) -> i32 {
        self.iter()
            .filter_map(|e| match e {
                Effect::Skill(name, v) if name == skill_name => Some(*v),
                _ => None,
            })
            .sum()
    }

    pub fn action_points_bonus(&self) -> i32 {
        self.iter()
            .filter_map(|e| match e {
                Effect::ActionPoints(v) => Some(*v),
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
/// Applies all effect types: Characteristic bonuses flow into derived stats (protections, HP, mana),
/// ActionPoints bonus adjusts AP max, and Mana effects add to max mana.
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
            &mut ActionPoints,
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
    for (race, traits, weapons, equipment, stats, mut effects, mut hp, mut mana, mut ap) in
        &mut query
    {
        let s = &**stats;

        // Step 1: Collect source effects and store them so effective_level() works
        **effects = shared::collect_source_effects(
            **race,
            traits,
            weapons,
            equipment,
            &trait_registry,
            &weapon_registry,
            &equipment_registry,
        );

        // Step 2: Compute effective characteristic levels (base + bonuses from source effects)
        let eff_dexterity = effects.effective_level(s, CharacteristicKind::Dexterity);
        let eff_endurance = effects.effective_level(s, CharacteristicKind::Endurance);
        let eff_magic = effects.effective_level(s, CharacteristicKind::Magic);
        let eff_willpower = effects.effective_level(s, CharacteristicKind::Willpower);
        let eff_intellect = effects.effective_level(s, CharacteristicKind::Intellect);

        // Step 3: Prepend default effects (computed from effective levels)
        let mut combined = vec![
            Effect::OnLvlUp(OnLvlUp::AddAbilityPoints(1)),
            Effect::OnLvlUp(OnLvlUp::AddSkillPoints(3 + eff_intellect as i32)),
            Effect::OnLvlUp(OnLvlUp::AddCharacteristicPoints(2)),
            Effect::Protection(Protection::Melee, 10 + eff_dexterity),
            Effect::Protection(Protection::Magic, 10 + eff_magic),
            Effect::Protection(Protection::Body, 10 + eff_endurance),
            Effect::Protection(Protection::Mind, 10 + eff_willpower),
            // ProtectionRange comes from race size
        ];
        combined.append(&mut effects);
        **effects = combined;

        // Step 4: Recompute HP max using effective endurance
        let new_max_hp = eff_endurance * 3 + 3;
        if hp.max != new_max_hp {
            let spent = hp.max.saturating_sub(hp.current);
            hp.max = new_max_hp;
            hp.current = new_max_hp.saturating_sub(spent);
        }

        // Step 5: Recompute Mana max using effective willpower + Mana effect bonuses
        let mut new_max_mana = eff_willpower * 3 + 3;
        for effect in effects.iter() {
            if let Effect::Mana {
                dependent,
                increase_per_point,
            } = effect
            {
                let dep_level = effects.effective_level(s, *dependent) as i32;
                new_max_mana = (new_max_mana as i32 + dep_level * increase_per_point).max(0) as u32;
            }
        }
        if mana.max != new_max_mana {
            let spent = mana.max.saturating_sub(mana.current);
            mana.max = new_max_mana;
            mana.current = new_max_mana.saturating_sub(spent);
        }

        // Step 8: Recompute AP max = race base + action points bonus
        let ap_bonus: i32 = effects.action_points_bonus();
        let new_max_ap = (race.base_action_points() as i32 + ap_bonus).max(0) as u32;
        if ap.max != new_max_ap {
            let spent = ap.max.saturating_sub(ap.current);
            ap.max = new_max_ap;
            ap.current = new_max_ap.saturating_sub(spent);
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
