use bevy::prelude::*;

/// Fired when the player upgrades a characteristic or skill in edit mode.
#[derive(Message)]
pub enum UpgradeEvent {
    /// Index 0-7 matching the display order: STR, DEX, END, PER, MAG, WIL, INT, CHA.
    Characteristic(usize),
    /// Skill name to upgrade.
    Skill(String),
}
