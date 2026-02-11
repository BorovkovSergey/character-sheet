use bevy::prelude::*;

/// Fired when the player learns a trait from the learn screen.
#[derive(Message)]
pub struct LearnTrait(pub String);
