use shared::{Character, EquipmentRegistry, ItemRegistry, TraitRegistry};
use std::collections::BTreeMap;
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
    characters: Arc<RwLock<BTreeMap<Uuid, Character>>>,
    trait_registry: Arc<TraitRegistry>,
    equipment_registry: Arc<EquipmentRegistry>,
    #[allow(dead_code)]
    item_registry: Arc<ItemRegistry>,
    data_path: PathBuf,
}

impl CharacterStore {
    pub async fn new(data_dir: &str) -> Self {
        let data_path = PathBuf::from(data_dir).join("characters.json");
        let traits_path = PathBuf::from(data_dir).join("traits.json");

        // Ensure data directory exists
        if let Some(parent) = data_path.parent() {
            if let Err(e) = tokio::fs::create_dir_all(parent).await {
                error!("Failed to create data directory {:?}: {}", parent, e);
            }
        }

        let trait_registry = Arc::new(TraitRegistry::load_from_file(&traits_path).unwrap_or_else(
            |e| {
                warn!("Failed to load traits from {:?}: {}", traits_path, e);
                TraitRegistry::default()
            },
        ));

        let equipment_path = PathBuf::from(data_dir).join("equipment.json");
        let equipment_registry = Arc::new(
            EquipmentRegistry::load_from_file(&equipment_path).unwrap_or_else(|e| {
                warn!("Failed to load equipment from {:?}: {}", equipment_path, e);
                EquipmentRegistry::default()
            }),
        );

        let items_path = PathBuf::from(data_dir).join("items.json");
        let item_registry =
            Arc::new(ItemRegistry::load_from_file(&items_path).unwrap_or_else(|e| {
                warn!("Failed to load items from {:?}: {}", items_path, e);
                ItemRegistry::default()
            }));

        let characters =
            Self::load_from_file(&data_path, &trait_registry, &equipment_registry).await;

        Self {
            characters: Arc::new(RwLock::new(characters)),
            trait_registry,
            equipment_registry,
            item_registry,
            data_path,
        }
    }

    async fn load_from_file(
        path: &PathBuf,
        trait_registry: &TraitRegistry,
        equipment_registry: &EquipmentRegistry,
    ) -> BTreeMap<Uuid, Character> {
        match tokio::fs::read_to_string(path).await {
            Ok(content) => match serde_json::from_str::<StorageData>(&content) {
                Ok(data) => data
                    .characters
                    .into_iter()
                    .map(|mut c| {
                        c.recalculate_effects(trait_registry, equipment_registry);
                        (c.id, c)
                    })
                    .collect(),
                Err(e) => {
                    warn!("Failed to parse characters file {:?}: {}", path, e);
                    BTreeMap::new()
                }
            },
            Err(_) => BTreeMap::new(),
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

        let json = match serde_json::to_string_pretty(&data) {
            Ok(json) => json,
            Err(e) => {
                error!("Failed to serialize characters: {}", e);
                return;
            }
        };

        // Write to temp file then rename for atomicity
        let tmp_path = self.data_path.with_extension("json.tmp");
        if let Err(e) = tokio::fs::write(&tmp_path, &json).await {
            error!("Failed to write temp file {:?}: {}", tmp_path, e);
            return;
        }
        if let Err(e) = tokio::fs::rename(&tmp_path, &self.data_path).await {
            error!(
                "Failed to rename {:?} to {:?}: {}",
                tmp_path, self.data_path, e
            );
        }
    }

    pub async fn get_all(&self) -> Vec<Character> {
        let characters = self.characters.read().await;
        characters.values().cloned().collect()
    }

    pub async fn create(&self, name: String) -> Character {
        let mut character = Character::new(name);
        character.recalculate_effects(&self.trait_registry, &self.equipment_registry);
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

    pub async fn update(&self, mut character: Character) -> Option<Character> {
        character.recalculate_effects(&self.trait_registry, &self.equipment_registry);
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
