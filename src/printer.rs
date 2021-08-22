pub use self::printer_profile::{PrinterProfile, PrinterConnectionData, PrinterProfileBuilder};
pub use self::printer_model::PrinterModel;

mod printer_profile;
mod printer_model;

use crate::{
    Instruction,
    PrintData,
    EscposImage,
    Error,
    command::{Command, Font}
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
    Network,
    Terminal
}

/// Main escpos-rs structure
///
/// The printer represents the thermal printer connected to the computer.
/// ```rust,no_run
/// use escpos_rs::{Printer, PrinterModel};
/// use libusb::{Context};
///
/// // We create a usb contest for the printer
/// let context = Context::new().unwrap();
/// // We pass it to the printer
/// let printer = match Printer::with_context(&context, PrinterModel::TMT20.profile()) {
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
    font_and_width: (Font, u8)
}

impl Printer {
    /// Creates a new printer
    /// 
    /// Creates the printer with the given details, from the printer details provided, and in the given USB context.
    pub fn new(printer_profile: PrinterProfile) -> Result<Option<Printer>, Error> {
        // Quick check for the profile containing at least one font
        match printer_profile.printer_connection_data {
            PrinterConnectionData::Usb{vendor_id, product_id, endpoint, timeout} => {
                let context = Context::new().map_err(Error::RusbError)?;
                let font_and_width = if let Some(width) = printer_profile.columns_per_font.get(&Font::FontA) {
                    (Font::FontA, *width)
                } else {
                    return Err(Error::NoFontFound);
                };
        
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
                                    font_and_width
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
            PrinterConnectionData::Terminal => panic!("Unimplemented")
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
    /// By default, there is line skipping with spaces as newlines (when the text does not fit). At the end, no newline is added.
    pub fn print<T: AsRef<str>>(&self, content: T) -> Result<(), Error> {
        // First we split it into lines
        /*
        content.as_ref().split("\n").map(|line| {
            // Now, for each line, we split it into words, to create
            let mut current_line = String::new();
            let mut lines = Vec::new();
            for word in line.split_whitespace() {
                let num_chars = word.chars().count();
                if current_line.len() + num_chars + 1 < self.font_and_width.1 {
                    current_line += &format!(" {}", word);
                } else {
                    if num_chars < limit {

                    } else {
                        remaining
                    }
                    // We have a new line
                    current_line += "\n";
                    lines.push(current_line.clone());
                    current_line.clear();
                } else {
                    // Normal word
                }
                /*
                Options:
                1. Current chars + new word < limit (easy peasy)
                2. Current chars + new word >= limit
                  * new_word > limit (not easy peasy)
                  * new_word < limit
                */
            }
        })
        */
        let feed = String::from(content.as_ref()).into_cp437(&CP437_CONTROL).map_err(|e| Error::CP437Error(e.into_string()))?;
        self.raw(&feed)
    }

    /// Print some text, with a newline at the end.
    ///
    /// By default, there is line skipping with spaces as newlines (when the text does not fit, assuming a width for at least one font was provided, else the text will split exacty where the line is full). At the end, no newline is added.
    pub fn println<T: AsRef<str>>(&self, content: T) -> Result<(), Error> {
        let feed = String::from(content.as_ref()) + "\n";
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

    /// Jumps _n_ number of lines (to leave whitespaces). Basically `n * '\n'` passed to `print`
    pub fn jump(&self, n: u8) -> Result<(), Error> {
        let feed = vec![b'\n', n];
        self.raw(&feed)
    }

    /// Cuts the paper, in case the instruction is supported by the printer
    pub fn cut(&self) -> Result<(), Error> {
        self.raw(&Command::Cut.as_bytes())
    }

    /// Prints a table with two columns. The sum of lengths of both strings
    pub fn table_2(&self, rows: Vec<(String, String)>) -> Result<(), Error> {
        let mut feed = Vec::new();
        for pair in rows {
            let len1 = pair.0.len();
            let len2 = pair.1.len();
            let num_spaces = 30 - len1 - len2;
            feed.append(&mut pair.0.as_bytes().to_vec());
            // We add the missing spaces
            feed.resize(feed.len() + num_spaces, b' ');
            feed.append(&mut pair.1.as_bytes().to_vec());
            feed.push(b'\n');
        }
        self.raw(&feed)
    }

    pub fn image(&self, escpos_image: EscposImage) -> Result<(), Error> {
        self.raw(&escpos_image.feed(self.printer_profile.width))
    }

    /// Sends raw information to the printer
    ///
    /// As simple as it sounds
    /// ```rust,no_run
    /// # use libusb::Context;
    /// # use escpos_rs::{Printer,PrinterProfile};
    /// # let context = Context::new().unwrap();
    /// # let printer_profile = PrinterProfile::builder(0x0001, 0x0001).build();
    /// # let printer = Printer::with_context(&context, printer_profile).unwrap().unwrap();
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