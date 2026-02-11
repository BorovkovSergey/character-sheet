pub mod character;
pub mod messages;
pub mod version;

pub use character::{
    collect_source_effects, xp_to_next_level, Ability, AbilityCheck, AbilityRegistry,
    AbilityRequirements, AbilityType, AbilityUpgrade, Character, CharacterSkill, CharacterTrait,
    Characteristic, CharacteristicKind, Characteristics, Class, ClassAbilities, Effect, EnemyCheck,
    Equipment, EquipmentRegistry, EquipmentSlot, GetEffects, InventoryItem, Item, ItemRegistry,
    LearnScreenPosition, MeleeKind, Named, Protection, Race, RangeKind, Resist, Resource, Size,
    Skill, SkillRegistry, TraitCondition, TraitRegistry, Wallet, Weapon, WeaponGrip, WeaponKind,
    WeaponRegistry,
};
pub use messages::{ClientMessage, ServerMessage};
pub use version::{CharacterFile, CharacterSummary, CharacterVersion, Timestamp, VersionSummary};

/// Serialize a message to bincode bytes
pub fn serialize<T: serde::Serialize>(msg: &T) -> Result<Vec<u8>, bincode::Error> {
    bincode::serialize(msg)
}

/// Deserialize a message from bincode bytes
pub fn deserialize<'a, T: serde::Deserialize<'a>>(bytes: &'a [u8]) -> Result<T, bincode::Error> {
    bincode::deserialize(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_character_serialization_roundtrip() {
        let character = Character::new("Test Hero".to_string());
        let bytes = serialize(&character).unwrap();
        let decoded: Character = deserialize(&bytes).unwrap();
        assert_eq!(character, decoded);
    }

    #[test]
    fn test_client_message_serialization() {
        let msg = ClientMessage::CreateCharacter {
            name: "Gandalf".to_string(),
        };
        let bytes = serialize(&msg).unwrap();
        let decoded: ClientMessage = deserialize(&bytes).unwrap();
        match decoded {
            ClientMessage::CreateCharacter { name } => assert_eq!(name, "Gandalf"),
            _ => panic!("Wrong message type"),
        }
    }

    #[test]
    fn test_server_message_serialization() {
        let summary = CharacterSummary {
            id: uuid::Uuid::new_v4(),
            name: "Frodo".to_string(),
            race: Race::default(),
            class: Class::default(),
            level: 1,
            version_count: 1,
            last_updated: 0,
        };
        let msg = ServerMessage::CharacterList {
            characters: vec![summary],
        };
        let bytes = serialize(&msg).unwrap();
        let decoded: ServerMessage = deserialize(&bytes).unwrap();
        match decoded {
            ServerMessage::CharacterList { characters } => {
                assert_eq!(characters.len(), 1);
                assert_eq!(characters[0].name, "Frodo");
            }
            _ => panic!("Wrong message type"),
        }
    }
}
