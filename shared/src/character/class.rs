use serde::{Deserialize, Serialize};
use strum::Display;

/// Character class
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Display, Serialize, Deserialize)]
pub enum Class {
    Bard,
}

impl Default for Class {
    fn default() -> Self {
        Self::Bard
    }
}
