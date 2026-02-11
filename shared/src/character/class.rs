use serde::{Deserialize, Serialize};
use strum::Display;

/// Character class
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default, Display, Serialize, Deserialize,
)]
pub enum Class {
    #[default]
    Bard,
}
