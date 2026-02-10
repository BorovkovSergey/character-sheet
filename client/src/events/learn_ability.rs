use bevy::prelude::*;

/// Fired when the player learns an ability from the learn screen.
#[derive(Message)]
pub struct LearnAbility(pub String);
