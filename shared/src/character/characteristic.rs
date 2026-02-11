use std::marker::PhantomData;

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

impl CharacteristicKind {
    pub fn abbrev(self) -> &'static str {
        match self {
            Self::Strength => "STR",
            Self::Dexterity => "DEX",
            Self::Endurance => "END",
            Self::Perception => "PER",
            Self::Magic => "MAG",
            Self::Willpower => "WIL",
            Self::Intellect => "INT",
            Self::Charisma => "CHA",
        }
    }
}

// Marker types for characteristics
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Strength;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Dexterity;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Endurance;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Perception;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Magic;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Willpower;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Intellect;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Charisma;

/// Generic characteristic with a marker type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Characteristic<T> {
    pub level: u32,
    _marker: PhantomData<T>,
}

impl<T> Characteristic<T> {
    pub fn new(level: u32) -> Self {
        Self {
            level,
            _marker: PhantomData,
        }
    }
}

impl<T> Serialize for Characteristic<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("Characteristic", 1)?;
        state.serialize_field("level", &self.level)?;
        state.end()
    }
}

impl<'de, T> Deserialize<'de> for Characteristic<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct CharacteristicData {
            level: u32,
        }

        let data = CharacteristicData::deserialize(deserializer)?;
        Ok(Self::new(data.level))
    }
}

/// Trait for characteristic level-up logic
pub trait CharacteristicTrait {
    /// Returns the kind of this characteristic
    fn kind() -> CharacteristicKind;

    /// Attempt to increase characteristic level.
    /// Returns the number of points spent, or 0 if not enough points available.
    fn up(&mut self, available_points: u32) -> u32;
}

/// Helper trait to get CharacteristicKind from marker type
pub trait CharacteristicKindMarker {
    fn kind() -> CharacteristicKind;
}

impl CharacteristicKindMarker for Strength {
    fn kind() -> CharacteristicKind {
        CharacteristicKind::Strength
    }
}

impl CharacteristicKindMarker for Dexterity {
    fn kind() -> CharacteristicKind {
        CharacteristicKind::Dexterity
    }
}

impl CharacteristicKindMarker for Endurance {
    fn kind() -> CharacteristicKind {
        CharacteristicKind::Endurance
    }
}

impl CharacteristicKindMarker for Perception {
    fn kind() -> CharacteristicKind {
        CharacteristicKind::Perception
    }
}

impl CharacteristicKindMarker for Magic {
    fn kind() -> CharacteristicKind {
        CharacteristicKind::Magic
    }
}

impl CharacteristicKindMarker for Willpower {
    fn kind() -> CharacteristicKind {
        CharacteristicKind::Willpower
    }
}

impl CharacteristicKindMarker for Intellect {
    fn kind() -> CharacteristicKind {
        CharacteristicKind::Intellect
    }
}

impl CharacteristicKindMarker for Charisma {
    fn kind() -> CharacteristicKind {
        CharacteristicKind::Charisma
    }
}

impl<T: CharacteristicKindMarker> CharacteristicTrait for Characteristic<T> {
    fn kind() -> CharacteristicKind {
        T::kind()
    }

    fn up(&mut self, available_points: u32) -> u32 {
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
    pub strength: Characteristic<Strength>,
    pub dexterity: Characteristic<Dexterity>,
    pub endurance: Characteristic<Endurance>,
    pub perception: Characteristic<Perception>,
    pub magic: Characteristic<Magic>,
    pub willpower: Characteristic<Willpower>,
    pub intellect: Characteristic<Intellect>,
    pub charisma: Characteristic<Charisma>,
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
        let mut strength: Characteristic<Strength> = Characteristic::new(5);

        // To go from level 5 to 6, we need 6 points
        let spent = strength.up(10);

        assert_eq!(spent, 6);
        assert_eq!(strength.level, 6);
    }

    #[test]
    fn test_characteristic_up_not_enough_points() {
        let mut strength: Characteristic<Strength> = Characteristic::new(5);

        // To go from level 5 to 6, we need 6 points, but we only have 5
        let spent = strength.up(5);

        assert_eq!(spent, 0);
        assert_eq!(strength.level, 5);
    }

    #[test]
    fn test_characteristic_kind() {
        assert_eq!(
            <Characteristic<Strength> as CharacteristicTrait>::kind(),
            CharacteristicKind::Strength
        );
        assert_eq!(
            <Characteristic<Dexterity> as CharacteristicTrait>::kind(),
            CharacteristicKind::Dexterity
        );
        assert_eq!(
            <Characteristic<Endurance> as CharacteristicTrait>::kind(),
            CharacteristicKind::Endurance
        );
        assert_eq!(
            <Characteristic<Perception> as CharacteristicTrait>::kind(),
            CharacteristicKind::Perception
        );
        assert_eq!(
            <Characteristic<Magic> as CharacteristicTrait>::kind(),
            CharacteristicKind::Magic
        );
        assert_eq!(
            <Characteristic<Willpower> as CharacteristicTrait>::kind(),
            CharacteristicKind::Willpower
        );
        assert_eq!(
            <Characteristic<Intellect> as CharacteristicTrait>::kind(),
            CharacteristicKind::Intellect
        );
        assert_eq!(
            <Characteristic<Charisma> as CharacteristicTrait>::kind(),
            CharacteristicKind::Charisma
        );
    }
}
