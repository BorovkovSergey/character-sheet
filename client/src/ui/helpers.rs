use bevy::prelude::*;
use shared::character::OnLvlUp;
use shared::Effect;

pub fn format_effect(effect: &Effect) -> String {
    match effect {
        Effect::Resist(r, v) => format!("{r} Resist {v:+}"),
        Effect::Skill(name, v) => format!("{name} {v:+}"),
        Effect::Protection(p, v) => format!("{p} Protection {v:+}"),
        Effect::Initiative(v) => format!("Initiative {v:+}"),
        Effect::Characteristic(c, v) => format!("{c} {v:+}"),
        Effect::ActionPoints(v) => format!("Action Points {v:+}"),
        Effect::Armor(v) => format!("Armor {v:+}"),
        Effect::Mana {
            dependent,
            increase_per_point,
        } => format!("Mana {increase_per_point:+}/point of {dependent}"),
        Effect::OnLvlUp(OnLvlUp::AddSkillPoints(v)) => {
            format!("{v:+} Skill Points per level")
        }
        Effect::OnLvlUp(OnLvlUp::AddAbilityPoints(v)) => {
            format!("{v:+} Ability Points per level")
        }
        Effect::OnLvlUp(OnLvlUp::AddCharacteristicPoints(v)) => {
            format!("{v:+} Characteristic Points per level")
        }
    }
}

pub(super) fn check_trait_requirement(
    stats: &shared::Characteristics,
    condition: Option<&shared::TraitCondition>,
) -> bool {
    match condition {
        Some(shared::TraitCondition::CharacteristicsRequired {
            characteristic,
            lvl,
        }) => stats.get_level(*characteristic) >= *lvl,
        None => true,
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub(super) fn save_to_json_file<T: serde::Serialize>(path: &str, items: Vec<&T>) {
    match serde_json::to_string_pretty(&items) {
        Ok(json) => {
            if let Err(e) = std::fs::write(path, json) {
                warn!("Failed to save {path}: {e}");
            }
        }
        Err(e) => warn!("Failed to serialize to {path}: {e}"),
    }
}

#[cfg(target_arch = "wasm32")]
pub(super) fn save_to_json_file<T>(_path: &str, _items: Vec<&T>) {}
