use bevy::prelude::*;
use shared::Character;

/// Connection status to the server
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum ConnectionStatus {
    #[default]
    Disconnected,
    Connecting,
    Connected,
    Error(String),
}

/// Main application state resource
#[derive(Resource, Default)]
pub struct AppState {
    pub characters: Vec<Character>,
    pub connection_status: ConnectionStatus,
    pub new_character_name: String,
}

impl AppState {
    pub fn set_characters(&mut self, characters: Vec<Character>) {
        self.characters = characters;
    }

    pub fn add_character(&mut self, character: Character) {
        // Avoid duplicates
        if !self.characters.iter().any(|c| c.id == character.id) {
            self.characters.push(character);
        }
    }

    pub fn remove_character(&mut self, id: uuid::Uuid) {
        self.characters.retain(|c| c.id != id);
    }

    pub fn update_character(&mut self, character: Character) {
        if let Some(existing) = self.characters.iter_mut().find(|c| c.id == character.id) {
            *existing = character;
        }
    }
}
