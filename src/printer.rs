pub use self::usb_printer::Printer;
pub use self::printer_profile::{PrinterProfile, PrinterProfileBuilder};
pub use self::printer_model::PrinterModel;

mod usb_printer;
mod printer_profile;
mod printer_model;