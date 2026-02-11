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
pub use weapon::{MeleeKind, RangeKind, Weapon, WeaponGrip, WeaponKind, WeaponRegistry};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Character {
    pub id: Uuid,
    pub name: String,
    pub race: Race,
    pub class: Class,
    pub level: u32,
    pub experience: u32,
    pub hp: Resource,
    pub mana: Resource,
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
    /// Total currency stored as a single value.
    /// Gold = value / 1000, Silver = (value % 1000) / 10, Copper = value % 10.
    #[serde(default)]
    pub wallet: u64,
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
            hp: Resource::new(20),
            mana: Resource::new(10),
            action_points: Resource::new(3),
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
            wallet: 0,
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

    /// Recalculates active effects from all sources (race, traits, equipment, etc.).
    pub fn recalculate_effects(
        &mut self,
        trait_registry: &TraitRegistry,
        equipment_registry: &EquipmentRegistry,
    ) {
        self.active_effects.clear();
        self.active_effects.extend(self.race.get_effects());
        self.active_effects.extend(self.race.size().get_effects());
        for trait_name in &self.traits {
            if let Some(character_trait) = trait_registry.get(trait_name) {
                self.active_effects
                    .extend(character_trait.effects.iter().cloned());
            }
        }
        for names in self.equipped_equipment.values() {
            for equipment_name in names {
                if let Some(equipment) = equipment_registry.get(equipment_name) {
                    self.active_effects
                        .extend(equipment.effects.iter().cloned());
                }
            }
        }
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
