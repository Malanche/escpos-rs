extern crate serde;

use serde::{Serialize, Deserialize};

/// Alignment for text printing
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum Justification {
    Left,
    Center,
    Right
}