use std::collections::BTreeMap;
use std::fmt;

use serde::{Deserialize, Serialize};
use strum::{Display, EnumIter};

use super::effect::Effect;

/// Ranged weapon subtypes.
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
pub enum RangeKind {
    Bow,
    Firearm,
    Crossbow,
}

/// Melee weapon subtypes.
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
pub enum MeleeKind {
    Slashing,
    Crushing,
    Piercing,
    Polearm,
    Chopping,
}

/// Top-level weapon type: either ranged or melee, each with a subtype.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WeaponKind {
    Range(RangeKind),
    Melee(MeleeKind),
}

impl fmt::Display for WeaponKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WeaponKind::Range(k) => write!(f, "{k}"),
            WeaponKind::Melee(k) => write!(f, "{k}"),
        }
    }
}

/// How the weapon is held.
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
pub enum WeaponGrip {
    OneHanded,
    TwoHanded,
    HandAndAHalf,
}

/// A weapon that can be equipped by a character.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Weapon {
    pub name: String,
    /// Free-form damage description (e.g. "2d6+3").
    pub damage: String,
    /// Attack bonus.
    pub attack: i32,
    pub kind: WeaponKind,
    pub grip: WeaponGrip,
    /// Attack range in squares.
    pub range: u32,
    pub effects: Vec<Effect>,
    /// Optional extra condition or note (free-form text).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub condition: Option<String>,
}

/// Registry of all weapons, keyed by name.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct WeaponRegistry {
    pub weapons: BTreeMap<String, Weapon>,
}

impl WeaponRegistry {
    /// Load weapons from a JSON string (array of Weapon objects).
    pub fn load_from_str(json: &str) -> Result<Self, serde_json::Error> {
        let list: Vec<Weapon> = serde_json::from_str(json)?;
        let weapons = list.into_iter().map(|w| (w.name.clone(), w)).collect();
        Ok(Self { weapons })
    }

    /// Get a weapon by name.
    pub fn get(&self, name: &str) -> Option<&Weapon> {
        self.weapons.get(name)
    }
}
