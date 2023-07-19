use serde::{Serialize, Deserialize};

/// Specifies de density of the image to be printer
///
/// Not all densities are supported by all printers
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Hash)]
pub enum ImageMode {
    EightDotSingleDensity,
    EightDotDoubleDensity,
    TwentyfourDotSingleDensity,
    TwentyfourDotDoubleDensity
}

impl Eq for ImageMode{}

impl ImageMode {
    /// Returns the byte-array representation of each command
    pub fn as_byte(&self) -> u8 {
        match self {
            ImageMode::EightDotSingleDensity => 0x00,
            ImageMode::EightDotDoubleDensity => 0x01,
            ImageMode::TwentyfourDotSingleDensity => 0x20,
            ImageMode::TwentyfourDotDoubleDensity => 0x21
        }
    }
}