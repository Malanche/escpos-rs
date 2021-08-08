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

/*
//Cuts the paper after \x96 vertical spaces
 CUT = {'\x1d', '\x56', '\x41', '\x96'};
//Equivalent to ESC @
std::string RESET = {'\x1d', '\x40'};
//Equivalent to ESC M n for font type A
std::string FONT_A = {'\x1b', '\x4d', '\x00'};
//Equivalent to ESC M n for font type B
std::string FONT_B = {'\x1b', '\x4d', '\x01'};
// Equivalent to ESC - n for underline text
std::string UNDERLINE_OFF = {'\x1b', '\x2d', '\x00'};
std::string UNDERLINE_1_DOT = {'\x1b', '\x2d', '\x01'};
std::string UNDERLINE_2_DOT = {'\x1b', '\x2d', '\x02'};
// Equivalent to ESC
std::string BOLD_ON = {'\x1b', '\x45', '\x01'};
std::string BOLD_OFF = {'\x1b', '\x45', '\x00'};
// Equivalent to ESC * m=0
std::string BITMAP = {'\x1b', '\x2a'};
// Change line size
std::string NO_LINE = {'\x1b', '\x33', (unsigned char) 0};
// Reset line Size
std::string RESET_LINE = {'\x1b', '\x32'};
*/