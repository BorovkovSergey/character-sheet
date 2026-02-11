use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::version::{CharacterSummary, Timestamp, VersionSummary};
use crate::{Character, Equipment, Item, Weapon};

/// Messages sent from client to server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClientMessage {
    /// Request the summary list of all characters
    RequestCharacterList,

    /// Request the version list for a specific character
    RequestVersionList { id: Uuid },

    /// Request a specific version of a character (latest if version is None)
    RequestCharacterVersion { id: Uuid, version: Option<u32> },

    /// Create a new character with the given name
    CreateCharacter { name: String },

    /// Delete a character by ID (all versions)
    DeleteCharacter { id: Uuid },

    /// Update a character (creates a new version)
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
    /// List of character summaries (for character select screen)
    CharacterList { characters: Vec<CharacterSummary> },

    /// Version list for a specific character (for version select screen)
    VersionList {
        id: Uuid,
        versions: Vec<VersionSummary>,
    },

    /// Full character data for a specific version
    CharacterVersion {
        id: Uuid,
        version: u32,
        saved_at: Timestamp,
        character: Box<Character>,
    },

    /// A new character was created
    CharacterCreated { summary: CharacterSummary },

    /// A character was updated (new version created)
    CharacterUpdated { summary: CharacterSummary },

    /// A character was deleted
    CharacterDeleted { id: Uuid },

    /// An error occurred
    Error { message: String },
}
