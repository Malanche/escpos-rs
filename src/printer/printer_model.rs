use super::{PrinterProfile};
use crate::{PrinterConnectionData, command::Font};

/// Printers known to this library
///
/// Probably needs updates. If you know one that is not in the list, send them to the author through email to be considered in future updates.
pub enum PrinterModel {
    /// ZKTeco mini printer
    ZKTeco,
    /// Epson most used printer
    TMT20,
    /// Epson most used printer
    TMT88VI
}

impl PrinterModel {
    /// Get the vendor, product id and endpoint of the current model
    pub fn vp_id(&self) -> (u16, u16, Option<u8>) {
        match self {
            PrinterModel::ZKTeco => (0x6868, 0x0200, Some(0x02)),
            PrinterModel::TMT20 => (0x04b8, 0x0e15, Some(0x01)),
            PrinterModel::TMT88VI => (0x04b8, 0x0202, Some(0x01))
        }
    }

    /// Obtain the details to connect to a printer model through usb
    pub fn usb_profile(&self) -> PrinterProfile {
        let (vendor_id, product_id, endpoint) = self.vp_id();
        match self {
            PrinterModel::ZKTeco => {
                PrinterProfile {
                    printer_connection_data: PrinterConnectionData::Usb {
                        vendor_id,
                        product_id,
                        endpoint,
                        timeout: std::time::Duration::from_secs(2)
                    },
                    columns_per_font: vec![(Font::FontA, 32), (Font::FontB, 42)].into_iter().collect(),
                    width: 384
                }
            },
            PrinterModel::TMT20 => {
                PrinterProfile {
                    printer_connection_data: PrinterConnectionData::Usb {
                        vendor_id,
                        product_id,
                        endpoint,
                        timeout: std::time::Duration::from_secs(2)
                    },
                    columns_per_font: vec![(Font::FontA, 48)].into_iter().collect(),
                    width: 576
                }
            },
            PrinterModel::TMT88VI => {
                PrinterProfile {
                    printer_connection_data: PrinterConnectionData::Usb {
                        vendor_id,
                        product_id,
                        endpoint,
                        timeout: std::time::Duration::from_secs(2)
                    },
                    columns_per_font: vec![(Font::FontA, 42), (Font::FontB, 56)].into_iter().collect(),
                    width: 576
                }
            }
        }
    }
}