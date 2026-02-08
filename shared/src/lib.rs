pub mod character;
pub mod messages;

pub use character::{
    Character, CharacterSkill, Characteristic, CharacteristicKind, CharacteristicKindMarker,
    CharacteristicTrait, Characteristics, Charisma, Class, Dexterity, Effect, Endurance,
    GetEffects, Intellect, Magic, Perception, Protection, Race, Resist, Resource, Size, Skill,
    SkillRegistry, Strength, Willpower,
};
pub use messages::{ClientMessage, ServerMessage};

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
        let mut decoded: Character = deserialize(&bytes).unwrap();
        decoded.recalculate_effects();
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
        let character = Character::new("Frodo".to_string());
        let msg = ServerMessage::CharacterList {
            characters: vec![character.clone()],
        };
        let bytes = serialize(&msg).unwrap();
        let decoded: ServerMessage = deserialize(&bytes).unwrap();
        match decoded {
            ServerMessage::CharacterList { mut characters } => {
                assert_eq!(characters.len(), 1);
                characters[0].recalculate_effects();
                assert_eq!(characters[0], character);
            }
            _ => panic!("Wrong message type"),
        }
    }
}
