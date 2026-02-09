use serde::{Deserialize, Serialize};
use strum::{Display, EnumIter};

use super::characteristic::CharacteristicKind;

/// Resistance types
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Display,
    EnumIter,
    Serialize,
    Deserialize,
)]
pub enum Resist {
    Fire,
    Ice,
    Lightning,
    Poison,
    Spirit,
    Dark,
}

/// Protection types
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Display,
    EnumIter,
    Serialize,
    Deserialize,
)]
pub enum Protection {
    Melee,
    Range,
    Magic,
    Body,
    Mind,
}

/// Effect triggered on level up
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OnLvlUp {
    AddSkillPoints,
}

/// Effect with magnitude
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Effect {
    Resist(Resist, u32),
    Skill(String, i32),
    Protection(Protection, u32),
    Initiative(i32),
    Characteristic(CharacteristicKind, i32),
    ActionPoints(i32),
    Armor(i32),
    /// Increases mana by an amount dependent on characteristic level
    Mana {
        dependent: CharacteristicKind,
        increase_per_point: i32,
    },
    OnLvlUp(OnLvlUp),
}

/// Trait for getting effects
pub trait GetEffects {
    fn get_effects(&self) -> Vec<Effect>;
}
