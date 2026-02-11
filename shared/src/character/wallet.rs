use serde::{Deserialize, Serialize};

/// Character's currency, stored as a single total value in base (copper) units.
///
/// Conversion rates: 1 gold = 1000 copper, 1 silver = 10 copper.
/// Display breakdown: gold = total / 1000, silver = (total % 1000) / 10, copper = total % 10.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Wallet(u64);

impl Wallet {
    /// Creates a wallet from individual currency amounts.
    pub fn new(gold: u32, silver: u32, copper: u32) -> Self {
        Self(gold as u64 * 1000 + silver as u64 * 10 + copper as u64)
    }

    pub fn gold(&self) -> u32 {
        (self.0 / 1000) as u32
    }

    pub fn silver(&self) -> u32 {
        ((self.0 % 1000) / 10) as u32
    }

    pub fn copper(&self) -> u32 {
        (self.0 % 10) as u32
    }

    /// Returns the total value in base (copper) units.
    pub fn total(&self) -> u64 {
        self.0
    }

    /// Adds a signed delta in base currency units, clamping at zero.
    pub fn add(&mut self, delta: i64) {
        self.0 = (self.0 as i64 + delta).max(0) as u64;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wallet_parts() {
        let w = Wallet::new(1, 23, 4);
        assert_eq!(w.total(), 1234);
        assert_eq!(w.gold(), 1);
        assert_eq!(w.silver(), 23);
        assert_eq!(w.copper(), 4);
    }

    #[test]
    fn test_wallet_add_clamps_at_zero() {
        let mut w = Wallet::new(0, 0, 5);
        w.add(-100);
        assert_eq!(w.total(), 0);
    }

    #[test]
    fn test_wallet_default_is_zero() {
        let w = Wallet::default();
        assert_eq!(w.total(), 0);
    }
}
