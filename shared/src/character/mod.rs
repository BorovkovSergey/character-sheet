mod characteristic;
mod class;
mod effect;
mod race;
mod resource;
mod skill;

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
}

impl Character {
    pub fn new(name: String) -> Self {
        Self {
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
        }
    }
}
