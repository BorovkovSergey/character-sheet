use serde::{Deserialize, Serialize};

/// A consumable resource with current and maximum values
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct Resource {
    pub current: u32,
    pub max: u32,
}

impl Resource {
    pub fn new(max: u32) -> Self {
        Self { current: max, max }
    }

    /// Spend resource, returns true if successful
    pub fn spend(&mut self, amount: u32) -> bool {
        if self.current >= amount {
            self.current -= amount;
            true
        } else {
            false
        }
    }

    /// Restore resource up to max
    pub fn restore(&mut self, amount: u32) {
        self.current = self.current.saturating_add(amount).min(self.max);
    }

    /// Fully restore to max
    pub fn restore_full(&mut self) {
        self.current = self.max;
    }
}

impl Default for Resource {
    fn default() -> Self {
        Self::new(10)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resource_spend_more_than_available() {
        let mut resource = Resource::new(10);
        resource.current = 5;

        let result = resource.spend(10);

        assert!(!result);
        assert_eq!(resource.current, 5);
    }

    #[test]
    fn test_resource_restore_beyond_max() {
        let mut resource = Resource::new(10);
        resource.current = 5;

        resource.restore(100);

        assert_eq!(resource.current, 10);
    }

    #[test]
    fn test_resource_spend_success() {
        let mut resource = Resource::new(10);

        let result = resource.spend(3);

        assert!(result);
        assert_eq!(resource.current, 7);
    }
}
