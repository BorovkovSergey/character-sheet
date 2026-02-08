mod characteristic;
mod class;
mod effect;
mod race;
mod resource;
mod skill;

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub use characteristic::{
    Characteristic, CharacteristicKind, CharacteristicKindMarker, CharacteristicTrait,
    Characteristics, Charisma, Dexterity, Endurance, Intellect, Magic, Perception, Strength,
    Willpower,
};
pub use class::Class;
pub use effect::{Effect, GetEffects, Resist};
pub use race::Race;
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
    }

    /// Aggregates resist values from active effects, summing magnitudes per resist type.
    pub fn get_resists(&self) -> BTreeMap<Resist, u32> {
        let mut resists = BTreeMap::new();
        for effect in &self.active_effects {
            match effect {
                Effect::Resist(resist, magnitude) => {
                    let entry = resists.entry(*resist).or_insert(0u32);
                    *entry = entry.saturating_add(*magnitude);
                }
            }
        }
        resists
    }
}
