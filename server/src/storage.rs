use shared::Character;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{error, warn};
use uuid::Uuid;

#[derive(Debug, serde::Serialize, serde::Deserialize, Default)]
struct StorageData {
    characters: Vec<Character>,
}

#[derive(Clone)]
pub struct CharacterStore {
    characters: Arc<RwLock<HashMap<Uuid, Character>>>,
    data_path: PathBuf,
}

impl CharacterStore {
    pub async fn new(data_dir: &str) -> Self {
        let data_path = PathBuf::from(data_dir).join("characters.json");

        // Ensure data directory exists
        if let Some(parent) = data_path.parent() {
            if let Err(e) = tokio::fs::create_dir_all(parent).await {
                error!("Failed to create data directory {:?}: {}", parent, e);
            }
        }

        let characters = Self::load_from_file(&data_path).await;

        Self {
            characters: Arc::new(RwLock::new(characters)),
            data_path,
        }
    }

    async fn load_from_file(path: &PathBuf) -> HashMap<Uuid, Character> {
        match tokio::fs::read_to_string(path).await {
            Ok(content) => match serde_json::from_str::<StorageData>(&content) {
                Ok(data) => data.characters.into_iter().map(|c| (c.id, c)).collect(),
                Err(e) => {
                    warn!("Failed to parse characters file {:?}: {}", path, e);
                    HashMap::new()
                }
            },
            Err(_) => HashMap::new(),
        }
    }

    async fn save_to_file(&self) {
        // Clone data while holding lock, then release lock before file I/O
        let data = {
            let characters = self.characters.read().await;
            StorageData {
                characters: characters.values().cloned().collect(),
            }
        };

        if let Ok(json) = serde_json::to_string_pretty(&data) {
            if let Err(e) = tokio::fs::write(&self.data_path, json).await {
                error!("Failed to save characters to {:?}: {}", self.data_path, e);
            }
        }
    }

    pub async fn get_all(&self) -> Vec<Character> {
        let characters = self.characters.read().await;
        characters.values().cloned().collect()
    }

    pub async fn create(&self, name: String) -> Character {
        let character = Character::new(name);
        {
            let mut characters = self.characters.write().await;
            characters.insert(character.id, character.clone());
        }
        self.save_to_file().await;
        character
    }

    pub async fn delete(&self, id: Uuid) -> bool {
        let removed = {
            let mut characters = self.characters.write().await;
            characters.remove(&id).is_some()
        };
        if removed {
            self.save_to_file().await;
        }
        removed
    }

    pub async fn update(&self, character: Character) -> Option<Character> {
        let updated = {
            let mut characters = self.characters.write().await;
            if characters.contains_key(&character.id) {
                characters.insert(character.id, character.clone());
                Some(character)
            } else {
                None
            }
        };
        if updated.is_some() {
            self.save_to_file().await;
        }
        updated
    }
}
