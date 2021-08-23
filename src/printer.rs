pub use self::printer_profile::{PrinterProfile, PrinterConnectionData, PrinterProfileBuilder};
pub use self::printer_model::PrinterModel;

mod printer_profile;
mod printer_model;

use crate::{
    Instruction,
    PrintData,
    EscposImage,
    Error,
    command::{Command, Font},
    Formatter
};

extern crate codepage_437;
extern crate log;

use log::{warn};
use rusb::{UsbContext, Context, DeviceHandle, TransferType, Direction};
use codepage_437::{IntoCp437, CP437_CONTROL};

/// Keeps the actual living connection to the device
enum PrinterConnection {
    Usb {
        /// Bulk write endpoint
        endpoint: u8,
        /// Device handle
        dh: DeviceHandle<Context>,
        /// Time to wait before giving up writing to the bulk endpoint
        timeout: std::time::Duration
    },
    #[allow(dead_code)]
    Network,
    Terminal
}

/// Main escpos-rs structure
///
/// The printer represents the thermal printer connected to the computer.
/// ```rust,no_run
/// use escpos_rs::{Printer, PrinterModel};
///
/// let printer = match Printer::new(PrinterModel::TMT20.usb_profile()) {
///     Ok(maybe_printer) => match maybe_printer {
///         Some(printer) => printer,
///         None => panic!("No printer was found :(")
///     },
///     Err(e) => panic!("Error: {}", e)
/// };
/// // Now we have a printer
/// ```
pub struct Printer {
    printer_profile: PrinterProfile,
    /// Actual connection to the printer
    printer_connection: PrinterConnection,
    /// Current font and width for printing text
    font_and_width: (Font, u8),
    /// The auxiliary formatter to print nicely
    formatter: Formatter,
    /// If words should be splitted or not
    space_split: bool
}

impl Printer {
    /// Creates a new printer
    /// 
    /// Creates the printer with the given details, from the printer details provided, and in the given USB context.
    pub fn new(printer_profile: PrinterProfile) -> Result<Option<Printer>, Error> {
        // Font and width, at least one required.
        let font_and_width = if let Some(width) = printer_profile.columns_per_font.get(&Font::FontA) {
            (Font::FontA, *width)
        } else {
            return Err(Error::NoFontFound);
        };
        let formatter = Formatter::new(font_and_width.1);
        // Quick check for the profile containing at least one font
        match printer_profile.printer_connection_data {
            PrinterConnectionData::Usb{vendor_id, product_id, endpoint, timeout} => {
                let context = Context::new().map_err(Error::RusbError)?;
        
                let devices = context.devices().map_err(Error::RusbError)?;
                for device in devices.iter() {
                    let s = device.device_descriptor().map_err(Error::RusbError)?;
                    if s.vendor_id() == vendor_id && s.product_id() == product_id {
                        // Before opening the device, we must find the bulk endpoint
                        let config_descriptor = device.active_config_descriptor().map_err(Error::RusbError)?;
                        let actual_endpoint = if let Some(endpoint) = endpoint {
                            endpoint
                        } else {
                            let mut detected_endpoint: Option<u8> = None;
                            // Horrible to have 3 nested for, but so be it
                            for interface in config_descriptor.interfaces() {
                                for descriptor in interface.descriptors() {
                                    for endpoint in descriptor.endpoint_descriptors() {
                                        if let (TransferType::Bulk, Direction::Out) = (endpoint.transfer_type(), endpoint.direction()) {
                                            detected_endpoint = Some(endpoint.number());   
                                        }
                                    }
                                }
                            }
            
                            if let Some(detected_endpoint) = detected_endpoint {
                                detected_endpoint
                            } else {
                                return Err(Error::NoBulkEndpoint);
                            }
                        };
        
                        // Now we continue opening the device
        
                        match device.open() {
                            Ok(mut dh) => {
                                if let Ok(active) = dh.kernel_driver_active(0) {
                                    if active {
                                        // The kernel is active, we have to detach it
                                        match dh.detach_kernel_driver(0) {
                                            Ok(_) => (),
                                            Err(e) => return Err(Error::RusbError(e))
                                        };
                                    }
                                } else {
                                    warn!("Could not find out if kernel driver is active, might encounter a problem soon.");
                                };
                                // Now we claim the interface
                                match dh.claim_interface(0) {
                                    Ok(_) => (),
                                    Err(e) => return Err(Error::RusbError(e))
                                }
                                return Ok(Some(Printer {
                                    printer_connection: PrinterConnection::Usb {
                                        endpoint: actual_endpoint,
                                        dh,
                                        timeout
                                    },
                                    printer_profile,
                                    font_and_width,
                                    formatter,
                                    space_split: false
                                }));
                            },
                            Err(e) => return Err(Error::RusbError(e))
                        };
                    }
                }
                // No printer was found with such vid and pid
                Ok(None)
            },
            PrinterConnectionData::Network{..} => panic!("Unsupported!"),
            PrinterConnectionData::Terminal => Ok(Some(Printer{
                printer_connection: PrinterConnection::Terminal,
                printer_profile,
                font_and_width,
                formatter,
                space_split: false
            }))
        }
    }

    /// Guesses the printer, and connects to it (not meant for production)
    ///
    /// Might help to find which printer you have if you have only one connected. The function will try to connect to a printer, based on the common ones recognized by this library.
    pub fn with_context_feeling_lucky() -> Result<Option<Printer>, Error> {
        // Match to force update then a new model gets added, just as a reminder
        /*****
        IF YOU ARE READING THIS, AND YOU GOT AN  ERROR BECAUSE A PRINTER WAS MISSING,
        UPDATE THE FOR FOLLOWING THE MATCH TO TRY ALL PRINTERS
        *****/
        match PrinterModel::TMT20 {
            PrinterModel::TMT20 => (),
            PrinterModel::ZKTeco => ()
        }
        // Keep up to date! All printers should appear here for the function to work
        for printer_model in vec![PrinterModel::TMT20, PrinterModel::ZKTeco] {
            let printer_profile = printer_model.usb_profile();
            let candidate = Printer::new(printer_profile)?;
            if candidate.is_some() {
                return Ok(candidate)
            }
        }
        // No printer was found
        Ok(None)
    }

    /// Print an instruction
    ///
    /// You can pass optional printer data to the printer to fill in the dynamic parts of the instruction.
    pub fn instruction(&self, instruction: &Instruction, print_data: Option<&PrintData>) -> Result<(), Error> {
        let content = instruction.to_vec(&self.printer_profile, print_data)?;
        self.raw(&content)
    }
    
    /// Print some text.
    ///
    /// By default, lines will break when the text exceeds the current font's width. If you want to break lines with whitespaces, according to the width, you can use the [set_space_split](Printer::set_space_split) function.
    pub fn print<T: Into<String>>(&self, content: T) -> Result<(), Error> {
        let content = if self.space_split {
            self.formatter.space_split(content.into())
        } else {
            content.into()
        };
        match self.printer_connection {
            PrinterConnection::Usb{..} => {
                let feed = content.into_cp437(&CP437_CONTROL).map_err(|e| Error::CP437Error(e.into_string()))?;
                self.raw(&feed)
            },
            PrinterConnection::Network => panic!("Unimplemented!"),
            PrinterConnection::Terminal => {
                print!("{}", content);
                Ok(())
            }
        }
    }

    /// Print some text, with a newline at the end.
    ///
    /// By default, lines will break when the text exceeds the current font's width. If you want to break lines with whitespaces, according to the width, you can use the [set_space_split](Printer::set_space_split) function.
    pub fn println<T: Into<String>>(&self, content: T) -> Result<(), Error> {
        let feed = content.into() + "\n";
        self.print(&feed)
    }

    /// Sets the current printing font.
    ///
    /// The function will return an error if the specified font does not exist in the printer profile.
    pub fn set_font(&mut self, font: Font) -> Result<(), Error> {
        if let Some(width) = self.printer_profile.columns_per_font.get(&font) {
            self.font_and_width = (font, *width);
            Ok(())
        } else {
            Err(Error::UnsupportedFont)
        }
    }

    /// Enables or disables space splitting for long text printing.
    ///
    /// By default, the printer writes text in a single stream to the printer (which splits it wherever the maximum width is reached). To split by whitespaces, you can call this function with `true` as argument.
    pub fn set_space_split(&mut self, state: bool) {
        self.space_split = state;
    }

    /// Jumps _n_ number of lines (to leave whitespaces). Basically `n * '\n'` passed to `print`
    pub fn jump(&self, n: u8) -> Result<(), Error> {
        let feed = vec![b'\n', n];
        self.raw(&feed)
    }

    /// Cuts the paper, in case the instruction is supported by the printer
    pub fn cut(&self) -> Result<(), Error> {
        self.raw(&Command::Cut.as_bytes())
    }

    /// Prints a table with two columns.
    ///
    /// For more details, check [Formatter](crate::Formatter)'s [duo_table](crate::Formatter::duo_table).
    pub fn duo_table<A: Into<String>, B: Into<String>, C: IntoIterator<Item = (D, E)>, D: Into<String>, E: Into<String>>(&self, headers: (A, B), rows: C) -> Result<(), Error> {
        let content = self.formatter.duo_table(headers, rows);
        match &self.printer_connection {
            PrinterConnection::Terminal => {
                println!("{}", content);
                Ok(())
            },
            _other => {
                self.raw(&content)
            }
        }
    }

    /// Prints a table with three columns.
    ///
    /// For more details, check [Formatter](crate::Formatter)'s [trio_table](crate::Formatter::trio_table).
    pub fn trio_table<A: Into<String>, B: Into<String>, C: Into<String>, D: IntoIterator<Item = (E, F, G)>, E: Into<String>, F: Into<String>, G: Into<String>>(&self, headers: (A, B, C), rows: D) -> Result<(), Error> {
        let content = self.formatter.trio_table(headers, rows);
        match &self.printer_connection {
            PrinterConnection::Terminal => {
                println!("{}", content);
                Ok(())
            },
            _other => {
                self.raw(&content)
            }
        }
    }

    pub fn image(&self, escpos_image: EscposImage) -> Result<(), Error> {
        self.raw(&escpos_image.feed(self.printer_profile.width))
    }

    /// Sends raw information to the printer
    ///
    /// As simple as it sounds
    /// ```rust,no_run
    /// use escpos_rs::{Printer,PrinterProfile};
    /// let printer_profile = PrinterProfile::usb_builder(0x0001, 0x0001).build();
    /// let printer = Printer::new(printer_profile).unwrap().unwrap();
    /// printer.raw(&[0x01, 0x02])?;
    /// # Ok::<(), escpos_rs::Error>(())
    /// ```
    pub fn raw<A: AsRef<[u8]>>(&self, bytes: A) -> Result<(), Error> {
        match &self.printer_connection {
            PrinterConnection::Usb{endpoint, dh, timeout} => {
                dh.write_bulk(
                    *endpoint,
                    bytes.as_ref(),
                    *timeout
                ).map_err(Error::RusbError)?;
                Ok(())
            },
            _other => panic!("Unimplemented")
        }
    }
}