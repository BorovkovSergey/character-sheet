use serde::{Deserialize, Serialize};

use super::effect::{Effect, GetEffects, Resist};

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
