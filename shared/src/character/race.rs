use std::fmt;

use serde::{Deserialize, Serialize};

use super::effect::{Effect, GetEffects, Protection, Resist};

/// Character race
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Race {
    DarkHalfElf,
}

impl Default for Race {
    fn default() -> Self {
        Self::DarkHalfElf
    }
}

impl fmt::Display for Race {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Race::DarkHalfElf => write!(f, "Dark Half-Elf"),
        }
    }
}

/// Character size, determined by race.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Size {
    Small,
    Medium,
    Big,
}

impl Race {
    pub fn size(&self) -> Size {
        match self {
            Race::DarkHalfElf => Size::Medium,
        }
    }

    pub fn base_action_points(&self) -> u32 {
        match self {
            Race::DarkHalfElf => 5,
        }
    }
}

impl GetEffects for Size {
    fn get_effects(&self) -> Vec<Effect> {
        let body = match self {
            Size::Small => 9,
            Size::Medium => 10,
            Size::Big => 11,
        };
        vec![Effect::Protection(Protection::Range, body)]
    }
}

impl GetEffects for Race {
    fn get_effects(&self) -> Vec<Effect> {
        match self {
            Race::DarkHalfElf => vec![
                Effect::Resist(Resist::Lightning, 1),
                Effect::Resist(Resist::Spirit, 1),
            ],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dark_half_elf_effects() {
        let race = Race::DarkHalfElf;
        let effects = race.get_effects();
        assert_eq!(effects.len(), 2);
        assert!(effects.contains(&Effect::Resist(Resist::Lightning, 1)));
        assert!(effects.contains(&Effect::Resist(Resist::Spirit, 1)));
    }
}
