use serde::de::DeserializeOwned;
use shared::{
    Character, CharacterFile, CharacterSummary, CharacterVersion, Equipment, EquipmentRegistry,
    Item, ItemRegistry, Named, TraitRegistry, VersionSummary, Weapon, WeaponRegistry,
};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{error, info, warn};
use uuid::Uuid;

/// Legacy format for migration from single-file storage.
#[derive(Debug, serde::Serialize, serde::Deserialize, Default)]
struct LegacyStorageData {
    characters: Vec<Character>,
}

/// In-memory index entry for a character.
struct CharacterIndex {
    file_path: PathBuf,
    summary: CharacterSummary,
}

#[derive(Clone)]
pub struct CharacterStore {
    characters: Arc<RwLock<BTreeMap<Uuid, CharacterIndex>>>,
    trait_registry: Arc<TraitRegistry>,
    weapon_registry: Arc<WeaponRegistry>,
    equipment_registry: Arc<EquipmentRegistry>,
    #[allow(dead_code)]
    item_registry: Arc<ItemRegistry>,
    characters_dir: PathBuf,
    data_dir: PathBuf,
}

fn current_timestamp() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_else(|_| {
            warn!("System clock is before Unix epoch, using timestamp 0");
            std::time::Duration::from_secs(0)
        })
        .as_secs() as i64
}

fn character_filename(name: &str, id: Uuid) -> String {
    let sanitized: String = name
        .to_lowercase()
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
        .collect();
    // Ensure filename is not empty or only underscores
    let sanitized = if sanitized.trim_matches('_').is_empty() {
        "character".to_string()
    } else {
        sanitized
    };
    let short_id = &id.to_string()[..8];
    format!("{sanitized}_{short_id}.json")
}

fn summary_from_file(file: &CharacterFile) -> Option<CharacterSummary> {
    let latest = file.versions.last()?;
    Some(CharacterSummary {
        id: file.id,
        name: latest.character.name.clone(),
        race: latest.character.race,
        class: latest.character.class,
        level: latest.character.level,
        version_count: file.versions.len() as u32,
        last_updated: latest.saved_at,
    })
}

async fn write_character_file(path: &Path, file: &CharacterFile) {
    let json = match serde_json::to_string_pretty(file) {
        Ok(json) => json,
        Err(e) => {
            error!("Failed to serialize character file: {}", e);
            return;
        }
    };
    let tmp = path.with_extension("json.tmp");
    if let Err(e) = tokio::fs::write(&tmp, &json).await {
        error!("Failed to write temp file {:?}: {}", tmp, e);
        return;
    }
    if let Err(e) = tokio::fs::rename(&tmp, path).await {
        error!("Failed to rename {:?} to {:?}: {}", tmp, path, e);
    }
}

impl CharacterStore {
    pub async fn new(data_dir: &str) -> Self {
        let data_dir_path = PathBuf::from(data_dir);
        let characters_dir = data_dir_path.join("characters");

        // Ensure directories exist
        if let Err(e) = tokio::fs::create_dir_all(&characters_dir).await {
            error!(
                "Failed to create characters directory {:?}: {}",
                characters_dir, e
            );
        }

        let traits_path = data_dir_path.join("traits.json");
        let trait_registry = Arc::new(TraitRegistry::load_from_file(&traits_path).unwrap_or_else(
            |e| {
                warn!("Failed to load traits from {:?}: {}", traits_path, e);
                TraitRegistry::default()
            },
        ));

        let weapons_path = data_dir_path.join("weapons.json");
        let weapon_registry = Arc::new(
            WeaponRegistry::load_from_file(&weapons_path).unwrap_or_else(|e| {
                warn!("Failed to load weapons from {:?}: {}", weapons_path, e);
                WeaponRegistry::default()
            }),
        );

        let equipment_path = data_dir_path.join("equipment.json");
        let equipment_registry = Arc::new(
            EquipmentRegistry::load_from_file(&equipment_path).unwrap_or_else(|e| {
                warn!("Failed to load equipment from {:?}: {}", equipment_path, e);
                EquipmentRegistry::default()
            }),
        );

        let items_path = data_dir_path.join("items.json");
        let item_registry = Arc::new(ItemRegistry::load_from_file(&items_path).unwrap_or_else(
            |e| {
                warn!("Failed to load items from {:?}: {}", items_path, e);
                ItemRegistry::default()
            },
        ));

        // Migrate legacy characters.json if it exists
        let legacy_path = data_dir_path.join("characters.json");
        if legacy_path.exists() {
            Self::migrate_legacy(&legacy_path, &characters_dir).await;
        }

        // Scan characters directory and build index
        let index = Self::build_index(&characters_dir).await;

        Self {
            characters: Arc::new(RwLock::new(index)),
            trait_registry,
            weapon_registry,
            equipment_registry,
            item_registry,
            characters_dir,
            data_dir: data_dir_path,
        }
    }

    async fn migrate_legacy(legacy_path: &Path, characters_dir: &Path) {
        let content = match tokio::fs::read_to_string(legacy_path).await {
            Ok(c) => c,
            Err(e) => {
                warn!("Failed to read legacy characters file: {}", e);
                return;
            }
        };
        let data: LegacyStorageData = match serde_json::from_str(&content) {
            Ok(d) => d,
            Err(e) => {
                warn!("Failed to parse legacy characters file: {}", e);
                return;
            }
        };

        let now = current_timestamp();
        let count = data.characters.len();

        for character in data.characters {
            let file = CharacterFile {
                id: character.id,
                versions: vec![CharacterVersion {
                    version: 1,
                    saved_at: now,
                    character,
                }],
            };
            let name = file
                .versions
                .first()
                .map(|v| v.character.name.as_str())
                .unwrap_or("unnamed");
            let filename = character_filename(name, file.id);
            let path = characters_dir.join(&filename);
            write_character_file(&path, &file).await;
        }

        // Rename legacy file so it is not re-migrated
        let backup = legacy_path.with_extension("json.migrated");
        if let Err(e) = tokio::fs::rename(legacy_path, &backup).await {
            warn!("Failed to rename legacy file to backup: {}", e);
        } else {
            info!(
                "Migrated {} characters from legacy file, backup at {:?}",
                count, backup
            );
        }
    }

    async fn build_index(characters_dir: &Path) -> BTreeMap<Uuid, CharacterIndex> {
        let mut index = BTreeMap::new();
        let mut entries = match tokio::fs::read_dir(characters_dir).await {
            Ok(entries) => entries,
            Err(e) => {
                warn!("Failed to read characters directory: {}", e);
                return index;
            }
        };

        while let Ok(Some(entry)) = entries.next_entry().await {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("json") {
                continue;
            }
            let content = match tokio::fs::read_to_string(&path).await {
                Ok(c) => c,
                Err(e) => {
                    warn!("Failed to read character file {:?}: {}", path, e);
                    continue;
                }
            };
            let file: CharacterFile = match serde_json::from_str(&content) {
                Ok(f) => f,
                Err(e) => {
                    warn!("Failed to parse character file {:?}: {}", path, e);
                    continue;
                }
            };
            if let Some(summary) = summary_from_file(&file) {
                index.insert(
                    file.id,
                    CharacterIndex {
                        file_path: path,
                        summary,
                    },
                );
            }
        }

        info!("Loaded {} character(s) from index", index.len());
        index
    }

    async fn read_character_file(&self, id: Uuid) -> Option<(PathBuf, CharacterFile)> {
        let path = {
            let index = self.characters.read().await;
            index.get(&id)?.file_path.clone()
        };
        let content = tokio::fs::read_to_string(&path).await.ok()?;
        let file: CharacterFile = serde_json::from_str(&content).ok()?;
        Some((path, file))
    }

    pub async fn get_all_summaries(&self) -> Vec<CharacterSummary> {
        let index = self.characters.read().await;
        index.values().map(|ci| ci.summary.clone()).collect()
    }

    pub async fn get_version_list(&self, id: Uuid) -> Option<Vec<VersionSummary>> {
        let (_, file) = self.read_character_file(id).await?;
        Some(
            file.versions
                .iter()
                .map(|v| VersionSummary {
                    version: v.version,
                    saved_at: v.saved_at,
                    level: v.character.level,
                })
                .collect(),
        )
    }

    pub async fn get_character_version(
        &self,
        id: Uuid,
        version: Option<u32>,
    ) -> Option<CharacterVersion> {
        let (_, file) = self.read_character_file(id).await?;
        match version {
            Some(v) => file.versions.into_iter().find(|cv| cv.version == v),
            None => file.versions.into_iter().last(),
        }
    }

    pub async fn create(&self, name: String) -> CharacterSummary {
        let mut character = Character::new(name);
        character.recalculate_effects(
            &self.trait_registry,
            &self.weapon_registry,
            &self.equipment_registry,
        );

        let now = current_timestamp();
        let file = CharacterFile {
            id: character.id,
            versions: vec![CharacterVersion {
                version: 1,
                saved_at: now,
                character: character.clone(),
            }],
        };

        let filename = character_filename(&character.name, character.id);
        let file_path = self.characters_dir.join(&filename);
        write_character_file(&file_path, &file).await;

        let summary = CharacterSummary {
            id: character.id,
            name: character.name,
            race: character.race,
            class: character.class,
            level: character.level,
            version_count: 1,
            last_updated: now,
        };

        {
            let mut index = self.characters.write().await;
            index.insert(
                summary.id,
                CharacterIndex {
                    file_path,
                    summary: summary.clone(),
                },
            );
        }

        summary
    }

    pub async fn delete_version(&self, id: Uuid, version: u32) -> Option<bool> {
        let (path, mut file) = self.read_character_file(id).await?;

        // Prevent deleting the last remaining version
        if file.versions.len() <= 1 {
            warn!("Cannot delete the last version of character {}", id);
            return Some(false);
        }

        let before = file.versions.len();
        file.versions.retain(|v| v.version != version);
        if file.versions.len() == before {
            return Some(false);
        }

        {
            write_character_file(&path, &file).await;
            if let Some(summary) = summary_from_file(&file) {
                let mut index = self.characters.write().await;
                if let Some(ci) = index.get_mut(&id) {
                    ci.summary = summary;
                }
            }
        }

        Some(true)
    }

    pub async fn delete(&self, id: Uuid) -> bool {
        let path = {
            let mut index = self.characters.write().await;
            match index.remove(&id) {
                Some(ci) => ci.file_path,
                None => return false,
            }
        };
        if let Err(e) = tokio::fs::remove_file(&path).await {
            warn!("Failed to remove character file {:?}: {}", path, e);
        }
        true
    }

    pub async fn update(&self, mut character: Character) -> Option<CharacterSummary> {
        character.recalculate_effects(
            &self.trait_registry,
            &self.weapon_registry,
            &self.equipment_registry,
        );

        let (path, mut file) = self.read_character_file(character.id).await?;

        // Skip saving if nothing changed since the last version.
        // Compare with active_effects cleared because it is #[serde(skip)]
        // and will always be empty in the deserialized stored version.
        if let Some(latest) = file.versions.last() {
            let mut incoming = character.clone();
            incoming.active_effects.clear();
            if latest.character == incoming {
                info!("No changes for character {}, skipping save", character.id);
                return summary_from_file(&file);
            }
        }

        let now = current_timestamp();
        let new_version_num = file.versions.last().map(|v| v.version + 1).unwrap_or(1);
        file.versions.push(CharacterVersion {
            version: new_version_num,
            saved_at: now,
            character: character.clone(),
        });

        write_character_file(&path, &file).await;

        let summary = CharacterSummary {
            id: character.id,
            name: character.name,
            race: character.race,
            class: character.class,
            level: character.level,
            version_count: file.versions.len() as u32,
            last_updated: now,
        };

        {
            let mut index = self.characters.write().await;
            if let Some(ci) = index.get_mut(&character.id) {
                ci.summary = summary.clone();
            }
        }

        Some(summary)
    }

    /// Upserts a named item into a JSON array file: reads existing items,
    /// removes the one with the same name (if any), appends the new item, and writes back.
    async fn upsert_named_item<T>(&self, filename: &str, item: T) -> Result<(), String>
    where
        T: Named + serde::Serialize + DeserializeOwned,
    {
        let path = self.data_dir.join(filename);
        let mut items: Vec<T> = match tokio::fs::read_to_string(&path).await {
            Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
            Err(_) => Vec::new(),
        };
        let name = item.name().to_string();
        items.retain(|existing| existing.name() != name);
        items.push(item);
        let json = serde_json::to_string_pretty(&items).map_err(|e| e.to_string())?;
        tokio::fs::write(&path, json)
            .await
            .map_err(|e| e.to_string())
    }

    pub async fn save_weapon(&self, weapon: Weapon) -> Result<(), String> {
        self.upsert_named_item("weapons.json", weapon).await
    }

    pub async fn save_equipment(&self, equipment: Equipment) -> Result<(), String> {
        self.upsert_named_item("equipment.json", equipment).await
    }

    pub async fn save_item(&self, item: Item) -> Result<(), String> {
        self.upsert_named_item("items.json", item).await
    }
}
