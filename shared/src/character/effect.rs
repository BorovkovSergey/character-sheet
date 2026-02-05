use serde::{Deserialize, Serialize};

/// Resistance types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Resist {
    Fire,
    Ice,
    Lightning,
    Poison,
    Spirit,
    Dark,
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
