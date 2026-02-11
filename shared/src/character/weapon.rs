use std::collections::BTreeMap;

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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Display, Serialize, Deserialize)]
pub enum WeaponKind {
    #[strum(to_string = "{0}")]
    Range(RangeKind),
    #[strum(to_string = "{0}")]
    Melee(MeleeKind),
    Shield,
    BardInstrument,
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
    #[serde(default)]
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

    /// Load weapons from a JSON file.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn load_from_file(path: &std::path::Path) -> Result<Self, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(path)?;
        Ok(Self::load_from_str(&content)?)
    }

    /// Get a weapon by name.
    pub fn get(&self, name: &str) -> Option<&Weapon> {
        self.weapons.get(name)
    }
}
