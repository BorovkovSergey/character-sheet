use bevy::prelude::*;

/// Fired when the player creates a new item from the constructor popup.
#[derive(Message)]
pub enum CreateItem {
    Weapon(shared::Weapon),
    Equipment(shared::Equipment),
    Item(shared::Item),
}
