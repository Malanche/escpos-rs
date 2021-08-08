extern crate serde;

use super::{Charset, Font, CodeTable};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum Command {
    /// Cuts the paper after 0x96 vertical spaces
    Cut,
    /// Equivalent to ESC @
    Reset,
    /// Print mode selected to reset the fonts. Equivalent to ESC ! 0
    PrintModeDefault,
    /// Set an international character set, Equivalent to ESC R
    SelectCharset {
        /// Character set to be set
        charset: Charset
    },
    /// Selects a different code table, Equivalent to ESC t
    SelectCodeTable {
        code_table: CodeTable
    },
    /// Sets up a font. Equivalent to ESC M
    SelectFont {
        font: Font
    },
    UnderlineOff,
    Underline1Dot,
    Underline2Dot,
    /// Equivalent to ESC * m = 0
    BoldOn,
    BoldOff,
    /// Equivalent to ESC * m = 0
    Bitmap,
    /// Change line size
    NoLine,
    ResetLine
}

impl Command {
    pub fn as_bytes(&self) -> Vec<u8> {
        match self {
            Command::Cut => vec![0x1d, 0x56, 0x41, 0x96],
            Command::Reset => vec![0x1d, 0x40],
            Command::PrintModeDefault => vec![0x01b, 0x21, 0x00],
            Command::SelectCharset{charset} => {
                let mut res = vec![0x1b, 0x52];
                res.append(&mut charset.as_bytes());
                res
            },
            Command::SelectCodeTable{code_table} => {
                let mut res = vec![0x1b, 0x74];
                res.append(&mut code_table.as_bytes());
                res
            },
            Command::SelectFont{font} => {
                let mut res = vec![0x1b, 0x4d];
                res.append(&mut font.as_bytes());
                res
            },
            Command::UnderlineOff => vec![0x1b, 0x2d, 0x00],
            Command::Underline1Dot => vec![0x1b, 0x2d, 0x01],
            Command::Underline2Dot => vec![0x1b, 0x2d, 0x02],
            Command::BoldOn => vec![0x1b, 0x45, 0x01],
            Command::BoldOff => vec![0x1b, 0x45, 0x00],
            Command::Bitmap => vec![0x1b, 0x2a],
            Command::NoLine => vec![0x1b, 0x33, 0x00],
            Command::ResetLine => vec![0x1b, 0x32]
        }
    }
}