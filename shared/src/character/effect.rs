use std::fmt;

use serde::{Deserialize, Serialize};

/// Resistance types
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Resist {
    Fire,
    Ice,
    Lightning,
    Poison,
    Spirit,
    Dark,
}

impl fmt::Display for Resist {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Resist::Fire => write!(f, "Fire"),
            Resist::Ice => write!(f, "Ice"),
            Resist::Lightning => write!(f, "Lightning"),
            Resist::Poison => write!(f, "Poison"),
            Resist::Spirit => write!(f, "Spirit"),
            Resist::Dark => write!(f, "Dark"),
        }
    }
}

/// Effect with magnitude
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Effect {
    Resist(Resist, u32),
}

/// Trait for getting effects
pub trait GetEffects {
    fn get_effects(&self) -> Vec<Effect>;
}
