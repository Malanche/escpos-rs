extern crate serde;

use serde::{Serialize, Deserialize};

/// Possible character sets
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum Charset {
    /// United states of america
    USA,
    /// France
    France,
    /// Germany
    Germany,
    /// United Kingdom
    UK,
    /// 1st possibility for Denmark
    Denmark1,
    /// Sweden
    Sweden,
    /// Italy
    Italy,
    /// 1st possibility for Spain
    Spain1,
    /// Japan
    Japan,
    /// Norway
    Norway,
    /// 2nd possibility for Denmark
    Denmark2,
    /// 2nd possibility for Spain
    Spain2,
    /// Latin America
    LatinAmerica,
    /// Korea
    Korea,
    /// Slovenia or Croatia
    SloveniaCroatia,
    /// China
    China,
    /// Vietnam
    Vietnam,
    /// Arabia
    Arabia
}

impl Charset {
    /// Returns the byte representation of the esc/pos command
    pub fn as_bytes(&self) -> Vec<u8> {
        match self {
            Charset::USA => vec![0x00],
            Charset::France => vec![0x01],
            Charset::Germany => vec![0x02],
            Charset::UK => vec![0x03],
            Charset::Denmark1 => vec![0x04],
            Charset::Sweden => vec![0x05],
            Charset::Italy => vec![0x06],
            Charset::Spain1 => vec![0x07],
            Charset::Japan => vec![0x08],
            Charset::Norway => vec![0x09],
            Charset::Denmark2 => vec![0x0a],
            Charset::Spain2 => vec![0x0b],
            Charset::LatinAmerica => vec![0x0c],
            Charset::Korea => vec![0x0d],
            Charset::SloveniaCroatia => vec![0x0e],
            Charset::China => vec![0x0f],
            Charset::Vietnam => vec![0x10],
            Charset::Arabia => vec![0x11]
        }
    }
}