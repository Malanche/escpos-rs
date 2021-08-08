extern crate serde;

use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum Justification {
    Left,
    Center,
    Right
}