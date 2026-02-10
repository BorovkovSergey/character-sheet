use bevy::prelude::*;

/// Fired once per level gained when the character levels up.
#[derive(Message)]
pub struct LevelUp;
