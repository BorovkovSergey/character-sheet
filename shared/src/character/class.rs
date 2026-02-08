use std::fmt;

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

impl fmt::Display for Class {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Class::Bard => write!(f, "Bard"),
        }
    }
}
