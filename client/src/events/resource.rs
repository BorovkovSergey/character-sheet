use bevy::prelude::*;

/// Fired when the player clicks a progress-bar cell to change a resource.
#[derive(Message)]
pub enum ResourceChanged {
    Hp(u32),
    Mp(u32),
    Ap(u32),
}
