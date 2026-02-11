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

/// Generic characteristic with a marker type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(bound = "")]
pub struct Characteristic<T> {
    pub level: u32,
    #[serde(skip)]
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

macro_rules! define_characteristic_markers {
    ($($marker:ident => $kind:ident),* $(,)?) => {
        $(
            #[derive(Debug, Clone, Copy, PartialEq, Eq)]
            pub struct $marker;

            impl CharacteristicKindMarker for $marker {
                fn kind() -> CharacteristicKind {
                    CharacteristicKind::$kind
                }
            }
        )*
    };
}

define_characteristic_markers!(
    Strength => Strength,
    Dexterity => Dexterity,
    Endurance => Endurance,
    Perception => Perception,
    Magic => Magic,
    Willpower => Willpower,
    Intellect => Intellect,
    Charisma => Charisma,
);

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

impl Characteristics {
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
