use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{Character, Equipment, Item, Weapon};

/// Messages sent from client to server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClientMessage {
    /// Request the full character list
    RequestCharacterList,

    /// Create a new character with the given name
    CreateCharacter { name: String },

    /// Delete a character by ID
    DeleteCharacter { id: Uuid },

    /// Update a character
    UpdateCharacter { character: Character },

    /// Register a new weapon definition
    CreateWeapon { weapon: Weapon },

    /// Register a new equipment definition
    CreateEquipment { equipment: Equipment },

    /// Register a new item definition
    CreateItem { item: Item },
}

/// Messages sent from server to client
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServerMessage {
    /// Full list of all characters
    CharacterList { characters: Vec<Character> },

    /// A new character was created
    CharacterCreated { character: Character },

    /// A character was updated
    CharacterUpdated { character: Character },

    /// A character was deleted
    CharacterDeleted { id: Uuid },

    /// An error occurred
    Error { message: String },
}
