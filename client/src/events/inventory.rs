use bevy::prelude::*;

/// Fired when the player triggers an equip/unequip action from a context menu.
#[derive(Message)]
pub enum InventoryChanged {
    /// Equip the inventory item at the given index.
    Equip(usize),
    /// Remove the inventory item at the given index.
    Remove(usize),
    /// Unequip the equipped equipment at the given index (in the flattened equipped list).
    UnequipGear(usize),
    /// Unequip the weapon at the given slot index.
    UnequipWeapon(usize),
}
