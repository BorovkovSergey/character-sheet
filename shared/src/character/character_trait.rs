use std::collections::BTreeMap;

use serde::Deserialize;

use super::characteristic::CharacteristicKind;
use super::effect::Effect;

/// Condition required to learn a trait
#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub enum TraitCondition {
    CharacteristicsRequired {
        characteristic: CharacteristicKind,
        lvl: u32,
    },
}

/// A character trait definition
#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct CharacterTrait {
    pub description: String,
    #[serde(default)]
    pub effects: Vec<Effect>,
    #[serde(default)]
    pub condition: Option<TraitCondition>,
}

/// Registry of all character traits
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TraitRegistry {
    pub traits: BTreeMap<String, CharacterTrait>,
}

impl TraitRegistry {
    /// Load traits from a JSON file
    pub fn load_from_file(path: &std::path::Path) -> Result<Self, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(path)?;
        let traits: BTreeMap<String, CharacterTrait> = serde_json::from_str(&content)?;
        Ok(Self { traits })
    }

    /// Load traits from a JSON string
    pub fn load_from_str(json: &str) -> Result<Self, serde_json::Error> {
        let traits: BTreeMap<String, CharacterTrait> = serde_json::from_str(json)?;
        Ok(Self { traits })
    }

    /// Get a trait by name
    pub fn get(&self, name: &str) -> Option<&CharacterTrait> {
        self.traits.get(name)
    }
}

#[cfg(test)]
mod tests {
    use super::super::effect::{OnLvlUp, Protection, Resist};
    use super::*;

    #[test]
    fn test_load_traits_from_json() {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .join("data/traits.json");
        let registry = TraitRegistry::load_from_file(&path).expect("failed to load traits.json");

        assert!(!registry.traits.is_empty());

        // Trait without condition or effects
        let accuracy = registry.get("Accuracy").expect("Accuracy not found");
        assert!(accuracy.effects.is_empty());
        assert!(accuracy.condition.is_none());

        // Trait with CharacteristicsRequired condition
        let acrobat = registry.get("Acrobat").expect("Acrobat not found");
        assert_eq!(
            acrobat.condition,
            Some(TraitCondition::CharacteristicsRequired {
                characteristic: CharacteristicKind::Dexterity,
                lvl: 4,
            })
        );

        // Protection effect
        let magic_allergy = registry
            .get("Magic Allergy")
            .expect("Magic Allergy not found");
        assert_eq!(
            magic_allergy.effects,
            vec![Effect::Protection(Protection::Magic, 2)]
        );

        // Resist effect
        let pyromancer = registry.get("Pyromancer").expect("Pyromancer not found");
        assert_eq!(pyromancer.effects, vec![Effect::Resist(Resist::Fire, 2)]);

        // ActionPoints effect
        let light_step = registry.get("Light Step").expect("Light Step not found");
        assert_eq!(light_step.effects, vec![Effect::ActionPoints(1)]);

        // Initiative effect
        let restless = registry.get("Restless").expect("Restless not found");
        assert_eq!(restless.effects, vec![Effect::Initiative(2)]);

        // Armor effect
        let thick_skinned = registry
            .get("Thick-Skinned")
            .expect("Thick-Skinned not found");
        assert_eq!(thick_skinned.effects, vec![Effect::Armor(1)]);
        assert_eq!(
            thick_skinned.condition,
            Some(TraitCondition::CharacteristicsRequired {
                characteristic: CharacteristicKind::Endurance,
                lvl: 4,
            })
        );

        // OnLvlUp effect
        let educated = registry.get("Educated").expect("Educated not found");
        assert_eq!(
            educated.effects,
            vec![Effect::OnLvlUp(OnLvlUp::AddSkillPoints(1))]
        );

        // Characteristic + Mana effects
        let strength_of_spirit = registry
            .get("Strength of Spirit")
            .expect("Strength of Spirit not found");
        assert_eq!(
            strength_of_spirit.effects,
            vec![
                Effect::Characteristic(CharacteristicKind::Intellect, 1),
                Effect::Mana {
                    dependent: CharacteristicKind::Willpower,
                    increase_per_point: 1,
                },
            ]
        );

        // Resist Poison
        let poison_expert = registry
            .get("Poison Expert")
            .expect("Poison Expert not found");
        assert_eq!(
            poison_expert.effects,
            vec![Effect::Resist(Resist::Poison, 2)]
        );

        // Willpower condition
        let blessed = registry.get("Blessed").expect("Blessed not found");
        assert_eq!(
            blessed.condition,
            Some(TraitCondition::CharacteristicsRequired {
                characteristic: CharacteristicKind::Willpower,
                lvl: 4,
            })
        );
    }
}
