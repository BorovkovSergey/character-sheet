use serde::{Deserialize, Serialize};
use strum::{Display, EnumIter};

use super::effect::{Effect, GetEffects, Protection, Resist};

/// Character race
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Display, EnumIter, Serialize, Deserialize)]
pub enum Race {
    #[default]
    #[strum(serialize = "Half-Elf")]
    HalfElf,
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
            Race::HalfElf => Size::Medium,
        }
    }

    pub fn base_action_points(&self) -> u32 {
        match self {
            Race::HalfElf => 5,
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
            Race::HalfElf => vec![
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
    fn test_half_elf_effects() {
        let race = Race::HalfElf;
        let effects = race.get_effects();
        assert_eq!(effects.len(), 2);
        assert!(effects.contains(&Effect::Resist(Resist::Lightning, 1)));
        assert!(effects.contains(&Effect::Resist(Resist::Spirit, 1)));
    }
}
