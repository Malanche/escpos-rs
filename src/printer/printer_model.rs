use super::{PrinterProfile};
use crate::command::Font;

/// Printers known to this library
///
/// Probably needs updates. If you know one that is not in the list, send them to the author through email to be considered in future updates.
pub enum PrinterModel {
    /// ZKTeco mini printer
    ZKTeco,
    /// Epson most used printer
    TMT20
}

impl PrinterModel {
    /// Get the vendor and product id of the current model
    pub fn vp_id(&self) -> (u16, u16) {
        match self {
            PrinterModel::ZKTeco => (0x6868, 0x0200),
            PrinterModel::TMT20 => (0x04b8, 0x0e15)
        }
    }

    /// Obtain the full details of the printer, to make an easy print
    pub fn profile(&self) -> PrinterProfile {
        let (vendor_id, product_id) = self.vp_id();
        match self {
            PrinterModel::ZKTeco => {
                PrinterProfile {
                    vendor_id,
                    product_id,
                    columns_per_font: vec![(Font::FontA, 32)].into_iter().collect(),
                    width: 384,
                    endpoint: Some(0x02),
                    timeout: std::time::Duration::from_secs(2)
                }
            },
            PrinterModel::TMT20 => {
                PrinterProfile {
                    vendor_id,
                    product_id,
                    columns_per_font: vec![(Font::FontA, 48)].into_iter().collect(),
                    width: 576,
                    endpoint: Some(0x01),
                    timeout: std::time::Duration::from_secs(2)
                }
            }
        }
    }
}