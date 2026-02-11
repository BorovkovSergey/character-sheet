use bevy::prelude::*;

#[derive(States, Default, Debug, Clone, PartialEq, Eq, Hash)]
pub enum AppScreen {
    #[default]
    CharacterSelect,
    VersionSelect,
    CharacterSheet,
}
