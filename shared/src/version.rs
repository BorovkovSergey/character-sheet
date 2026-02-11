use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::character::{Character, Class, Race};

/// Unix timestamp in seconds (UTC).
pub type Timestamp = i64;

/// A single versioned snapshot of a character.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CharacterVersion {
    /// Monotonically increasing version number within a character file (1-based).
    pub version: u32,
    /// Unix timestamp (seconds, UTC) when this version was saved.
    pub saved_at: Timestamp,
    /// The full character data at this point in time.
    pub character: Character,
}

/// The on-disk format for a single character's file.
/// Stored as JSON in `data/characters/{sanitized_name}_{uuid_prefix}.json`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterFile {
    /// The character's UUID (same across all versions).
    pub id: Uuid,
    /// Versions ordered oldest-first. The last element is the latest version.
    pub versions: Vec<CharacterVersion>,
}

/// Lightweight summary sent to clients for the character list screen.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterSummary {
    pub id: Uuid,
    pub name: String,
    pub race: Race,
    pub class: Class,
    pub level: u32,
    pub version_count: u32,
    /// Timestamp of the most recent version.
    pub last_updated: Timestamp,
}

/// Lightweight info about a single version, sent when the client asks for
/// the version list of a specific character.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionSummary {
    pub version: u32,
    pub saved_at: Timestamp,
    pub level: u32,
}
