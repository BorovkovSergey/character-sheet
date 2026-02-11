use serde::{Deserialize, Serialize};
use strum::{Display, EnumIter};

/// Enum representing all characteristic types
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Display, EnumIter, Serialize, Deserialize,
)]
pub enum CharacteristicKind {
    #[strum(serialize = "STR")]
    Strength,
    #[strum(serialize = "DEX")]
    Dexterity,
    #[strum(serialize = "END")]
    Endurance,
    #[strum(serialize = "PER")]
    Perception,
    #[strum(serialize = "MAG")]
    Magic,
    #[strum(serialize = "WIL")]
    Willpower,
    #[strum(serialize = "INT")]
    Intellect,
    #[strum(serialize = "CHA")]
    Charisma,
}

/// A single characteristic with a level that can be upgraded.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Characteristic {
    pub level: u32,
}

impl Characteristic {
    pub fn new(level: u32) -> Self {
        Self { level }
    }

    /// Attempt to increase characteristic level.
    /// Returns the number of points spent, or 0 if not enough points available.
    pub fn up(&mut self, available_points: u32) -> u32 {
        let cost = self.level + 1;
        if available_points >= cost {
            self.level += 1;
            cost
        } else {
            0
        }
    }
}

/// Character characteristics (attributes)
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct Characteristics {
    pub strength: Characteristic,
    pub dexterity: Characteristic,
    pub endurance: Characteristic,
    pub perception: Characteristic,
    pub magic: Characteristic,
    pub willpower: Characteristic,
    pub intellect: Characteristic,
    pub charisma: Characteristic,
}

impl Characteristics {
    pub fn new() -> Self {
        Self {
            strength: Characteristic::new(10),
            dexterity: Characteristic::new(10),
            endurance: Characteristic::new(10),
            perception: Characteristic::new(10),
            magic: Characteristic::new(10),
            willpower: Characteristic::new(10),
            intellect: Characteristic::new(10),
            charisma: Characteristic::new(10),
        }
    }

    pub fn get_level(&self, kind: CharacteristicKind) -> u32 {
        match kind {
            CharacteristicKind::Strength => self.strength.level,
            CharacteristicKind::Dexterity => self.dexterity.level,
            CharacteristicKind::Endurance => self.endurance.level,
            CharacteristicKind::Perception => self.perception.level,
            CharacteristicKind::Magic => self.magic.level,
            CharacteristicKind::Willpower => self.willpower.level,
            CharacteristicKind::Intellect => self.intellect.level,
            CharacteristicKind::Charisma => self.charisma.level,
        }
    }
}

impl Default for Characteristics {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_characteristic_up_success() {
        let mut c = Characteristic::new(5);
        let spent = c.up(10);
        assert_eq!(spent, 6);
        assert_eq!(c.level, 6);
    }

    #[test]
    fn test_characteristic_up_not_enough_points() {
        let mut c = Characteristic::new(5);
        let spent = c.up(5);
        assert_eq!(spent, 0);
        assert_eq!(c.level, 5);
    }
}
