use serde::{Deserialize, Serialize};
use strum::{Display, EnumIter};

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

/// Effect with magnitude
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Effect {
    Resist(Resist, u32),
    Protection(Protection, u32),
}

/// Trait for getting effects
pub trait GetEffects {
    fn get_effects(&self) -> Vec<Effect>;
}
