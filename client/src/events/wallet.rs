use bevy::prelude::*;

/// Fired when the player clicks a wallet currency cell.
/// The value is a signed delta in base currency units.
#[derive(Message)]
pub struct WalletChanged(pub i64);
