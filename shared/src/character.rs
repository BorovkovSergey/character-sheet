use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A consumable resource with current and maximum values
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct Resource {
    pub current: i32,
    pub max: i32,
}

impl Resource {
    pub fn new(max: i32) -> Self {
        Self { current: max, max }
    }

    /// Spend resource, returns true if successful
    pub fn spend(&mut self, amount: i32) -> bool {
        if self.current >= amount {
            self.current -= amount;
            true
        } else {
            false
        }
    }

    /// Restore resource up to max
    pub fn restore(&mut self, amount: i32) {
        self.current = (self.current + amount).min(self.max);
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Character {
    pub id: Uuid,
    pub name: String,
    pub hp: Resource,
    pub mana: Resource,
    pub action_points: Resource,
}

impl Character {
    pub fn new(name: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            hp: Resource::new(20),
            mana: Resource::new(10),
            action_points: Resource::new(3),
        }
    }
}
