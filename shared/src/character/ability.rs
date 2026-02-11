use std::collections::BTreeMap;
use std::fmt;

use serde::Deserialize;
use strum::Display;

use super::character_trait::TraitCondition;
use super::characteristic::CharacteristicKind;
use super::class::Class;

/// The type of an ability, determining how it behaves in gameplay.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Display, Deserialize)]
pub enum AbilityType {
    Stance,
    Attack,
    Debuff,
    Peaceful,
    Passive,
    Touch,
}

/// What check the caster must pass when using the ability.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub enum AbilityCheck {
    Skill(String),
    Characteristic(CharacteristicKind),
}

impl fmt::Display for AbilityCheck {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Skill(name) => write!(f, "{name}"),
            Self::Characteristic(kind) => write!(f, "{kind:?}"),
        }
    }
}

/// What defensive check the enemy can make against the ability.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub enum EnemyCheck {
    Protection(CharacteristicKind),
}

impl fmt::Display for EnemyCheck {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Protection(kind) => write!(f, "{kind:?}"),
        }
    }
}

/// Resource costs and range for using an ability.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct AbilityRequirements {
    pub mp: Option<u32>,
    pub hp: Option<u32>,
    pub action_points: Option<u32>,
    pub range: Option<u32>,
}

/// An upgrade to an ability, unlocked when a condition is met.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct AbilityUpgrade {
    pub condition: TraitCondition,
    pub description: String,
}

/// Position on the learn/skill-tree screen (only for Acquire abilities).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub struct LearnScreenPosition {
    pub row: u32,
    pub column: u32,
}

/// A single ability definition.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Ability {
    pub description: String,
    #[serde(default)]
    pub additional: Option<AbilityUpgrade>,
    #[serde(default)]
    pub requirements: Option<AbilityRequirements>,
    #[serde(default)]
    pub check: Option<AbilityCheck>,
    #[serde(default)]
    pub enemy_check: Option<EnemyCheck>,
    #[serde(default)]
    pub self_only: bool,
    #[serde(rename = "type")]
    pub ability_type: AbilityType,
    #[serde(default)]
    pub learn_screen_position: Option<LearnScreenPosition>,
    #[serde(default)]
    pub can_learn_after: Vec<String>,
}

/// Abilities for a single class, split into innate and acquirable.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct ClassAbilities {
    #[serde(rename = "Innate", default)]
    pub innate: BTreeMap<String, Ability>,
    #[serde(rename = "Acquire", default)]
    pub acquire: BTreeMap<String, Ability>,
}

/// Registry of all abilities across all classes.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AbilityRegistry {
    pub classes: BTreeMap<Class, ClassAbilities>,
}

impl AbilityRegistry {
    /// Load from a JSON string.
    pub fn load_from_str(json: &str) -> Result<Self, serde_json::Error> {
        let classes: BTreeMap<Class, ClassAbilities> = serde_json::from_str(json)?;
        Ok(Self { classes })
    }

    /// Load from a JSON file on disk.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn load_from_file(path: &std::path::Path) -> Result<Self, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(path)?;
        let classes: BTreeMap<Class, ClassAbilities> = serde_json::from_str(&content)?;
        Ok(Self { classes })
    }

    /// Get all abilities for a class.
    pub fn get_class_abilities(&self, class: &Class) -> Option<&ClassAbilities> {
        self.classes.get(class)
    }

    /// Get an innate ability by class and name.
    pub fn get_innate(&self, class: &Class, name: &str) -> Option<&Ability> {
        self.classes.get(class)?.innate.get(name)
    }

    /// Get an acquirable ability by class and name.
    pub fn get_acquire(&self, class: &Class, name: &str) -> Option<&Ability> {
        self.classes.get(class)?.acquire.get(name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(not(target_arch = "wasm32"))]
    fn test_load_abilities_from_json() {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .join("data/abilities.json");
        let registry =
            AbilityRegistry::load_from_file(&path).expect("failed to load abilities.json");

        // Registry should have at least the Bard class
        assert!(!registry.classes.is_empty());

        let bard = registry
            .get_class_abilities(&Class::Bard)
            .expect("Bard abilities not found");

        // Verify innate abilities loaded
        assert!(!bard.innate.is_empty());
        assert_eq!(bard.innate.len(), 3);

        // Verify acquire abilities loaded
        assert!(!bard.acquire.is_empty());
        assert_eq!(bard.acquire.len(), 8);

        // ---- Innate: Enchanting song ----
        let enchanting = registry
            .get_innate(&Class::Bard, "Enchanting song")
            .expect("Enchanting song not found");
        assert_eq!(enchanting.ability_type, AbilityType::Stance);
        assert!(enchanting.self_only);
        assert!(enchanting.additional.is_none());
        assert_eq!(
            enchanting.check,
            Some(AbilityCheck::Skill("Art".to_string()))
        );
        assert!(enchanting.enemy_check.is_none());
        let reqs = enchanting.requirements.as_ref().expect("requirements");
        assert_eq!(reqs.mp, Some(2));
        assert_eq!(reqs.hp, None);
        assert_eq!(reqs.action_points, Some(3));
        assert_eq!(reqs.range, None);

        // ---- Innate: False chord (has additional + enemy_check with Intelligence alias) ----
        let false_chord = registry
            .get_innate(&Class::Bard, "False chord")
            .expect("False chord not found");
        assert_eq!(false_chord.ability_type, AbilityType::Attack);
        assert!(!false_chord.self_only);

        let upgrade = false_chord
            .additional
            .as_ref()
            .expect("False chord should have additional");
        assert_eq!(
            upgrade.condition,
            TraitCondition::CharacteristicsRequired {
                characteristic: CharacteristicKind::Charisma,
                lvl: 5,
            }
        );

        // enemy_check uses "Intelligence" which should alias to Intellect
        assert_eq!(
            false_chord.enemy_check,
            Some(EnemyCheck::Protection(CharacteristicKind::Intellect))
        );

        let reqs = false_chord.requirements.as_ref().expect("requirements");
        assert_eq!(reqs.range, Some(5));

        // ---- Innate: From one eye (Characteristic check) ----
        let from_one_eye = registry
            .get_innate(&Class::Bard, "From one eye")
            .expect("From one eye not found");
        assert_eq!(from_one_eye.ability_type, AbilityType::Debuff);
        assert_eq!(
            from_one_eye.check,
            Some(AbilityCheck::Characteristic(CharacteristicKind::Charisma))
        );

        // ---- Acquire: Rebound (learn_screen_position, empty can_learn_after) ----
        let rebound = registry
            .get_acquire(&Class::Bard, "Rebound")
            .expect("Rebound not found");
        assert_eq!(rebound.ability_type, AbilityType::Peaceful);
        assert!(rebound.self_only);
        assert!(rebound.can_learn_after.is_empty());
        let pos = rebound
            .learn_screen_position
            .expect("Rebound should have screen position");
        assert_eq!(pos.row, 0);
        assert_eq!(pos.column, 0);

        // ---- Acquire: Narrator (Passive, can_learn_after populated) ----
        let narrator = registry
            .get_acquire(&Class::Bard, "Narrator")
            .expect("Narrator not found");
        assert_eq!(narrator.ability_type, AbilityType::Passive);
        assert_eq!(narrator.can_learn_after, vec!["Heal word", "Rebound"]);

        // ---- Acquire: Hidden Strike (Agility alias for Dexterity in condition) ----
        let hidden_strike = registry
            .get_acquire(&Class::Bard, "Hidden Strike")
            .expect("Hidden Strike not found");
        let upgrade = hidden_strike
            .additional
            .as_ref()
            .expect("Hidden Strike should have additional");
        assert_eq!(
            upgrade.condition,
            TraitCondition::CharacteristicsRequired {
                characteristic: CharacteristicKind::Dexterity,
                lvl: 4,
            }
        );

        // ---- Acquire: Sabotage (Touch type) ----
        let sabotage = registry
            .get_acquire(&Class::Bard, "Sabotage")
            .expect("Sabotage not found");
        assert_eq!(sabotage.ability_type, AbilityType::Touch);

        // ---- Verify nonexistent ability returns None ----
        assert!(registry.get_innate(&Class::Bard, "Nonexistent").is_none());
        assert!(registry.get_acquire(&Class::Bard, "Nonexistent").is_none());
    }
}
