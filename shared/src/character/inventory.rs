use serde::{Deserialize, Serialize};

/// An item stored in the character's inventory.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum InventoryItem {
    Weapon(String),
    Equipment(String),
    Item(String),
}
