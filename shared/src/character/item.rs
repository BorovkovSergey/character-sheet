use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

/// A generic item that can be carried in inventory.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Item {
    pub name: String,
    pub description: String,
}

/// Registry of all items, keyed by name.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ItemRegistry {
    pub items: BTreeMap<String, Item>,
}

impl ItemRegistry {
    /// Load items from a JSON string (array of Item objects).
    pub fn load_from_str(json: &str) -> Result<Self, serde_json::Error> {
        let list: Vec<Item> = serde_json::from_str(json)?;
        let items = list.into_iter().map(|i| (i.name.clone(), i)).collect();
        Ok(Self { items })
    }

    /// Load items from a JSON file.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn load_from_file(path: &std::path::Path) -> Result<Self, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(path)?;
        Ok(Self::load_from_str(&content)?)
    }

    /// Get an item by name.
    pub fn get(&self, name: &str) -> Option<&Item> {
        self.items.get(name)
    }
}
