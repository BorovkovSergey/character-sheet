mod characteristic;
mod class;
mod effect;
mod race;
mod resource;
mod skill;

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator;
use uuid::Uuid;

pub use characteristic::{
    Characteristic, CharacteristicKind, CharacteristicKindMarker, CharacteristicTrait,
    Characteristics, Charisma, Dexterity, Endurance, Intellect, Magic, Perception, Strength,
    Willpower,
};
pub use class::Class;
pub use effect::{Effect, GetEffects, Protection, Resist};
pub use race::{Race, Size};
pub use resource::Resource;
pub use skill::{CharacterSkill, Skill, SkillRegistry};

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
    pub skills: Vec<CharacterSkill>,
    #[serde(skip)]
    pub active_effects: Vec<Effect>,
}

impl Character {
    pub fn new(name: String) -> Self {
        let mut character = Self {
            id: Uuid::new_v4(),
            name,
            race: Race::default(),
            class: Class::default(),
            level: 1,
            experience: 0,
            hp: Resource::new(20),
            mana: Resource::new(10),
            action_points: Resource::new(3),
            stats: Characteristics::default(),
            characteristic_points: 0,
            skill_points: 0,
            skills: Vec::new(),
            active_effects: Vec::new(),
        };
        character.recalculate_effects();
        character
    }

    /// Recalculates active effects from all sources (race, class, items, etc.).
    pub fn recalculate_effects(&mut self) {
        self.active_effects.clear();
        self.active_effects.extend(self.race.get_effects());
        self.active_effects.extend(self.race.size().get_effects());
    }

    /// Aggregates resist values from active effects, summing magnitudes per resist type.
    pub fn get_resists(&self) -> BTreeMap<Resist, u32> {
        let mut resists: BTreeMap<Resist, u32> = Resist::iter().map(|r| (r, 0)).collect();
        for effect in &self.active_effects {
            if let Effect::Resist(resist, magnitude) = effect {
                let entry = resists.entry(*resist).or_insert(0u32);
                *entry = entry.saturating_add(*magnitude);
            }
        }
        resists
    }

    /// Aggregates protection values from active effects, summing magnitudes per protection type.
    pub fn get_protections(&self) -> BTreeMap<Protection, u32> {
        let mut protections: BTreeMap<Protection, u32> =
            Protection::iter().map(|p| (p, 0)).collect();
        for effect in &self.active_effects {
            if let Effect::Protection(protection, magnitude) = effect {
                let entry = protections.entry(*protection).or_insert(0u32);
                *entry = entry.saturating_add(*magnitude);
            }
        }
        protections
    }
}
