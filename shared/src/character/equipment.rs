use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use strum::{Display, EnumIter};

use super::effect::Effect;

/// Equipment slot on a character's body.
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
pub enum EquipmentSlot {
    Head,
    Pants,
    Gloves,
    Armor,
    Suit,
    Ring,
    Necklace,
    Cloak,
    Any,
}

/// A piece of equipment that can be worn by a character.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Equipment {
    pub name: String,
    pub description: String,
    pub slot: EquipmentSlot,
    pub effects: Vec<Effect>,
}

/// Registry of all equipment, keyed by name.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct EquipmentRegistry {
    pub equipment: BTreeMap<String, Equipment>,
}

impl EquipmentRegistry {
    /// Load equipment from a JSON string (array of Equipment objects).
    pub fn load_from_str(json: &str) -> Result<Self, serde_json::Error> {
        let list: Vec<Equipment> = serde_json::from_str(json)?;
        let equipment = list.into_iter().map(|e| (e.name.clone(), e)).collect();
        Ok(Self { equipment })
    }

    /// Load equipment from a JSON file.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn load_from_file(path: &std::path::Path) -> Result<Self, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(path)?;
        Ok(Self::load_from_str(&content)?)
    }

    /// Get an equipment piece by name.
    pub fn get(&self, name: &str) -> Option<&Equipment> {
        self.equipment.get(name)
    }
}
