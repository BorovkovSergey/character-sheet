use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use super::class::Class;
use super::CharacteristicKind;

/// Skill definition - only dependency on characteristic
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct Skill {
    pub dependency: CharacteristicKind,
}

/// Character's skill (reference to definition + level)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CharacterSkill {
    pub name: String,
    pub level: u32,
}

impl CharacterSkill {
    pub fn new(name: String) -> Self {
        Self { name, level: 1 }
    }

    /// Attempt to raise skill level
    /// - available_points: available skill_points of the character
    /// - max_level: level of the dependent characteristic (skill cannot be higher)
    /// Returns the number of points to spend, or 0 if cannot raise
    pub fn up(&mut self, available_points: u32, max_level: u32) -> u32 {
        // Check that we won't exceed the characteristic level
        if self.level >= max_level {
            return 0;
        }

        // Cost: from level N to N+1 requires N+1 points
        let cost = self.level + 1;

        if available_points >= cost {
            self.level += 1;
            cost
        } else {
            0
        }
    }
}

/// Registry of skills by class
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct SkillRegistry {
    #[serde(flatten)]
    pub classes: BTreeMap<Class, BTreeMap<String, Skill>>,
}

impl SkillRegistry {
    /// Load from a JSON string.
    pub fn load_from_str(json: &str) -> Result<Self, serde_json::Error> {
        let classes: BTreeMap<Class, BTreeMap<String, Skill>> = serde_json::from_str(json)?;
        Ok(Self { classes })
    }

    /// Load from file
    #[cfg(not(target_arch = "wasm32"))]
    pub fn load_from_file(path: &std::path::Path) -> Result<Self, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(path)?;
        let classes: BTreeMap<Class, BTreeMap<String, Skill>> = serde_json::from_str(&content)?;
        Ok(Self { classes })
    }

    /// Get skills for a class
    pub fn get_class_skills(&self, class: &Class) -> Option<&BTreeMap<String, Skill>> {
        self.classes.get(class)
    }

    /// Get skill definition for a class
    pub fn get_skill(&self, class: &Class, name: &str) -> Option<&Skill> {
        self.classes.get(class)?.get(name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_skill_registry_load() {
        let json = r#"{
            "Bard": {
                "MeleeAttack": {
                    "dependency": "Dexterity"
                },
                "Eloquence": {
                    "dependency": "Charisma"
                }
            }
        }"#;

        let registry: SkillRegistry = serde_json::from_str(json).expect("Failed to parse JSON");

        // Check that Bard class has skills
        let bard_skills = registry
            .get_class_skills(&Class::Bard)
            .expect("Bard skills not found");
        assert_eq!(bard_skills.len(), 2);

        // Check MeleeAttack skill
        let melee = registry
            .get_skill(&Class::Bard, "MeleeAttack")
            .expect("MeleeAttack not found");
        assert_eq!(melee.dependency, CharacteristicKind::Dexterity);

        // Check Eloquence skill
        let eloquence = registry
            .get_skill(&Class::Bard, "Eloquence")
            .expect("Eloquence not found");
        assert_eq!(eloquence.dependency, CharacteristicKind::Charisma);

        // Check nonexistent skill
        assert!(registry.get_skill(&Class::Bard, "nonexistent").is_none());
    }

    #[test]
    fn test_skill_up_success() {
        let mut skill = CharacterSkill::new("Persuasion".to_string());
        assert_eq!(skill.level, 1);

        // From level 1 to 2 costs 2 points
        let cost = skill.up(10, 10);
        assert_eq!(cost, 2);
        assert_eq!(skill.level, 2);

        // From level 2 to 3 costs 3 points
        let cost = skill.up(10, 10);
        assert_eq!(cost, 3);
        assert_eq!(skill.level, 3);
    }

    #[test]
    fn test_skill_up_not_enough_points() {
        let mut skill = CharacterSkill::new("Stealth".to_string());
        assert_eq!(skill.level, 1);

        // From level 1 to 2 costs 2 points, but we only have 1
        let cost = skill.up(1, 10);
        assert_eq!(cost, 0);
        assert_eq!(skill.level, 1);
    }

    #[test]
    fn test_skill_up_max_level_reached() {
        let mut skill = CharacterSkill::new("Arcana".to_string());
        skill.level = 5;

        // Skill is at level 5, characteristic is also 5 - cannot raise
        let cost = skill.up(100, 5);
        assert_eq!(cost, 0);
        assert_eq!(skill.level, 5);
    }
}
