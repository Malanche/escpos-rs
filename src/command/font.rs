extern crate serde;

use serde::{Serialize, Deserialize};

/// Common fonts used in thermal printers
#[derive(Serialize, Deserialize, Clone, Debug, Hash, PartialEq)]
pub enum Font {
    FontA,
    FontB,
    FontC,
    FontD,
    FontE
}

impl Eq for Font{}

impl Font {
    /// Byte representation of each font.
    pub fn as_bytes(&self) -> Vec<u8> {
        match self {
            Font::FontA => vec![0x00],
            Font::FontB => vec![0x01],
            Font::FontC => vec![0x02],
            Font::FontD => vec![0x03],
            Font::FontE => vec![0x04]
        }
    }
}