use bevy::prelude::*;

/// Fired when the player adds experience through the portrait context menu.
#[derive(Message)]
pub struct ExperienceChanged(pub u32);
