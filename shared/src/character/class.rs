use serde::{Deserialize, Serialize};
use strum::{Display, EnumIter};

/// Character class
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Default,
    Display,
    EnumIter,
    Serialize,
    Deserialize,
)]
pub enum Class {
    #[default]
    Bard,
}
