use serde::{Deserialize, Serialize};

/// Character class
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Class {
    Bard,
}

impl Default for Class {
    fn default() -> Self {
        Self::Bard
    }
}
