mod ability;
mod character_trait;
mod characteristic;
mod class;
mod effect;
mod equipment;
mod inventory;
mod item;
mod race;
mod resource;
mod skill;
mod wallet;
mod weapon;

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator;
use uuid::Uuid;

pub use ability::{
    Ability, AbilityCheck, AbilityRegistry, AbilityRequirements, AbilityType, AbilityUpgrade,
    ClassAbilities, EnemyCheck, LearnScreenPosition,
};
pub use character_trait::{CharacterTrait, TraitCondition, TraitRegistry};
pub use characteristic::{
    Characteristic, CharacteristicKind, CharacteristicKindMarker, CharacteristicTrait,
    Characteristics, Charisma, Dexterity, Endurance, Intellect, Magic, Perception, Strength,
    Willpower,
};
pub use class::Class;
pub use effect::{Effect, GetEffects, OnLvlUp, Protection, Resist};
pub use equipment::{Equipment, EquipmentRegistry, EquipmentSlot};
pub use inventory::InventoryItem;
pub use item::{Item, ItemRegistry};
pub use race::{Race, Size};
pub use resource::Resource;
pub use skill::{CharacterSkill, Skill, SkillRegistry};
pub use wallet::Wallet;
pub use weapon::{MeleeKind, RangeKind, Weapon, WeaponGrip, WeaponKind, WeaponRegistry};

/// Trait for types that have a name field.
pub trait Named {
    fn name(&self) -> &str;
}

impl Named for Weapon {
    fn name(&self) -> &str {
        &self.name
    }
}

impl Named for Equipment {
    fn name(&self) -> &str {
        &self.name
    }
}

impl Named for Item {
    fn name(&self) -> &str {
        &self.name
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Character {
    pub id: Uuid,
    pub name: String,
    pub race: Race,
    pub class: Class,
    pub level: u32,
    pub experience: u32,
    /// Damage taken (max HP is computed from Endurance * 3 + 3).
    #[serde(default)]
    pub hp_spent: u32,
    /// Mana spent (max Mana is computed from Willpower * 3 + 3).
    #[serde(default)]
    pub mana_spent: u32,
    pub action_points: Resource,
    pub stats: Characteristics,
    pub characteristic_points: u32,
    pub skill_points: u32,
    #[serde(default)]
    pub ability_points: u32,
    #[serde(default)]
    pub trait_points: u32,
    pub skills: Vec<CharacterSkill>,
    #[serde(default)]
    pub traits: Vec<String>,
    #[serde(default)]
    pub abilities: Vec<String>,
    #[serde(default)]
    pub equipped_weapons: Vec<String>,
    #[serde(default)]
    pub equipped_equipment: BTreeMap<EquipmentSlot, Vec<String>>,
    #[serde(default)]
    pub inventory: Vec<InventoryItem>,
    /// Character's currency purse.
    #[serde(default)]
    pub wallet: Wallet,
    #[serde(skip)]
    pub active_effects: Vec<Effect>,
}

impl Character {
    pub fn new(name: String) -> Self {
        let character = Self {
            id: Uuid::new_v4(),
            name,
            race: Race::default(),
            class: Class::default(),
            level: 0,
            experience: 0,
            hp_spent: 0,
            mana_spent: 0,
            action_points: Resource::new(Race::default().base_action_points()),
            stats: Characteristics::default(),
            characteristic_points: 0,
            skill_points: 0,
            ability_points: 0,
            trait_points: 0,
            skills: Vec::new(),
            traits: Vec::new(),
            abilities: Vec::new(),
            equipped_weapons: Vec::new(),
            equipped_equipment: BTreeMap::new(),
            inventory: Vec::new(),
            wallet: Wallet::default(),
            active_effects: Vec::new(),
        };
        // Effects will be calculated after traits are assigned
        character
    }

    /// Equips an item into the appropriate slot.
    /// Ring slot allows multiple items; all other slots replace the previous item.
    pub fn equip(&mut self, slot: EquipmentSlot, name: String) {
        let items = self.equipped_equipment.entry(slot).or_default();
        if slot == EquipmentSlot::Ring {
            items.push(name);
        } else {
            *items = vec![name];
        }
    }

    /// Removes an item by name from the given slot.
    pub fn unequip(&mut self, slot: EquipmentSlot, name: &str) {
        if let Some(items) = self.equipped_equipment.get_mut(&slot) {
            items.retain(|n| n != name);
            if items.is_empty() {
                self.equipped_equipment.remove(&slot);
            }
        }
    }

    /// Recalculates active effects from all sources (race, traits, weapons, equipment).
    pub fn recalculate_effects(
        &mut self,
        trait_registry: &TraitRegistry,
        weapon_registry: &WeaponRegistry,
        equipment_registry: &EquipmentRegistry,
    ) {
        self.active_effects = collect_source_effects(
            self.race,
            &self.traits,
            &self.equipped_weapons,
            &self.equipped_equipment,
            trait_registry,
            weapon_registry,
            equipment_registry,
        );
    }

    /// Aggregates effect values of a specific kind, summing magnitudes per key.
    fn aggregate<K>(&self, extract: impl Fn(&Effect) -> Option<(K, u32)>) -> BTreeMap<K, u32>
    where
        K: Ord + Copy + IntoEnumIterator,
    {
        let mut result: BTreeMap<K, u32> = K::iter().map(|k| (k, 0)).collect();
        for effect in &self.active_effects {
            if let Some((key, magnitude)) = extract(effect) {
                let entry = result.entry(key).or_insert(0);
                *entry = entry.saturating_add(magnitude);
            }
        }
        result
    }

    /// Aggregates resist values from active effects, summing magnitudes per resist type.
    pub fn get_resists(&self) -> BTreeMap<Resist, u32> {
        self.aggregate(|e| match e {
            Effect::Resist(r, m) => Some((*r, *m)),
            _ => None,
        })
    }

    /// Aggregates protection values from active effects, summing magnitudes per protection type.
    pub fn get_protections(&self) -> BTreeMap<Protection, u32> {
        self.aggregate(|e| match e {
            Effect::Protection(p, m) => Some((*p, *m)),
            _ => None,
        })
    }

    /// Calculates initiative as Perception level + sum of Initiative effects.
    pub fn get_initiative(&self) -> i32 {
        let perception = self.stats.perception.level as i32;
        let bonus: i32 = self
            .active_effects
            .iter()
            .filter_map(|e| match e {
                Effect::Initiative(v) => Some(*v),
                _ => None,
            })
            .sum();
        perception + bonus
    }
}

/// Collects effects from all sources: race, size, traits, weapons, equipment.
pub fn collect_source_effects(
    race: Race,
    trait_names: &[String],
    weapon_names: &[String],
    equipped_equipment: &BTreeMap<EquipmentSlot, Vec<String>>,
    trait_registry: &TraitRegistry,
    weapon_registry: &WeaponRegistry,
    equipment_registry: &EquipmentRegistry,
) -> Vec<Effect> {
    let mut effects = Vec::new();
    effects.extend(race.get_effects());
    effects.extend(race.size().get_effects());
    for name in trait_names {
        if let Some(ct) = trait_registry.get(name) {
            effects.extend(ct.effects.iter().cloned());
        }
    }
    for name in weapon_names {
        if let Some(w) = weapon_registry.get(name) {
            effects.extend(w.effects.iter().cloned());
        }
    }
    for names in equipped_equipment.values() {
        for name in names {
            if let Some(eq) = equipment_registry.get(name) {
                effects.extend(eq.effects.iter().cloned());
            }
        }
    }
    effects
}

/// XP required to advance from `level` to `level + 1`.
pub fn xp_to_next_level(level: u32) -> u32 {
    (level + 1) * 10
}
