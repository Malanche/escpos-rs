use crate::{
    Instruction,
    PrintData,
    EscposImage,
    Error,
    command::Command
};

extern crate libusb;
extern crate codepage_437;
extern crate log;

use log::{warn};
use libusb::{Context, DeviceHandle};
use codepage_437::{IntoCp437, CP437_CONTROL};
use super::{PrinterProfile, PrinterModel};

/// Main escpos-rs structure
///
/// The printer represents the thermal printer connected to the computer.
/// ```rust
/// use escpos_rs::{Printer};
/// use libusb::{Context};
///
/// fn main() {
///     // We create a usb contest for the printer
///     let context = Context::new().unwrap();
///     // We pass it to the printer
///     let printer = match Printer::with_context(&context, PrinterModel::TMT20.profile()) {
///         Ok(maybe_printer) => match maybe_printer {
///             Some(printer) => printer,
///             None => panic!("No printer was found :(")
///         },
///         Err(e) => panic!("Error: {}", e)
///     };
///     // Now we have a printer
/// }
/// ```
pub struct Printer<'a> {
    printer_profile: PrinterProfile,
    /// Bulk write endpoint
    endpoint: u8,
    dh: DeviceHandle<'a>
}

impl<'a> Printer<'a> {
    /// Creates a new printer
    /// 
    /// Creates the printer with the given details, from the printer details provided, and in the given USB context.
    pub fn with_context(context: &'a Context, printer_profile: PrinterProfile) -> Result<Option<Printer<'a>>, Error> {
        let (vendor_id, product_id) = (printer_profile.vendor_id, printer_profile.product_id);
        let devices = context.devices().map_err(|e| Error::LibusbError(e))?;
        for device in devices.iter() {
            let s = device.device_descriptor().map_err(|e| Error::LibusbError(e))?;
            if s.vendor_id() == vendor_id && s.product_id() == product_id {
                // Before opening the device, we must find the bulk endpoint
                let config_descriptor = device.active_config_descriptor().map_err(|e| Error::LibusbError(e))?;
                let endpoint = if let Some(endpoint) = printer_profile.endpoint {
                    endpoint
                } else {
                    let mut detected_endpoint: Option<u8> = None;
                    // Horrible to have 3 nested for, but so be it
                    for interface in config_descriptor.interfaces() {
                        for descriptor in interface.descriptors() {
                            for endpoint in descriptor.endpoint_descriptors() {
                                match (endpoint.transfer_type(), endpoint.direction()) {
                                    (libusb::TransferType::Bulk, libusb::Direction::Out) => if detected_endpoint.is_none() {
                                        detected_endpoint = Some(endpoint.number());
                                    },
                                    _ => ()
                                };
                            }
                        }
                    }
    
                    if let Some(endpoint) = detected_endpoint {
                        endpoint
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
                                    Err(e) => return Err(Error::LibusbError(e))
                                };
                            }
                        } else {
                            warn!("Could not find out if kernel driver is active, might encounter a problem soon.");
                        };
                        // Now we claim the interface
                        match dh.claim_interface(0) {
                            Ok(_) => (),
                            Err(e) => return Err(Error::LibusbError(e))
                        }
                        return Ok(Some(Printer {
                            printer_profile,
                            endpoint,
                            dh
                        }));
                    },
                    Err(e) => return Err(Error::LibusbError(e))
                };
            }
        }
        // No printer was found with such vid and pid
        Ok(None)
    }

    /// Guesses the printer, and connects to it (not meant for production)
    ///
    /// Might help to find which printer you have if you have only one connected. The function will try to connect to a printer, based on the common ones recognized by this library.
    pub fn with_context_feeling_lucky(context: &'a Context) -> Result<Option<Printer<'a>>, Error> {
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
            let printer_profile = printer_model.profile();
            let candidate = Printer::with_context(context, printer_profile)?;
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
        let content = instruction.to_vec(&self.printer_profile, print_data).unwrap();
        self.raw(&content)
    }
    
    /// Print some text. By default, there is line skipping with spaces as newlines (when the text does not fit). At the end, no newline is added.
    pub fn print<T: AsRef<str>>(&self, content: T) -> Result<(), Error> {
        let feed = String::from(content.as_ref()).into_cp437(&CP437_CONTROL).map_err(|e| Error::CP437Error(e.into_string()))?;
        self.raw(&feed)
    }

    /// Print some text.
    ///
    /// By default, there is line skipping with spaces as newlines (when the text does not fit, assuming a width for at least one font was provided, else the text will split exacty where the line is full). At the end, no newline is added.
    pub fn println<T: AsRef<str>>(&self, content: T) -> Result<(), Error> {
        let feed = String::from(content.as_ref()) + "\n";
        self.print(&feed)
    }

    /// Jumps _n_ number of lines (to leave whitespaces). Basically `n * '\n'` passed to `print`
    pub fn jump(&self, n: u8) -> Result<(), Error> {
        let feed = vec!['\n' as u8, n];
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
            for _ in 0..num_spaces {
                feed.push(' ' as u8);
            }
            feed.append(&mut pair.1.as_bytes().to_vec());
            feed.push('\n' as u8);
        }
        self.raw(&feed)
    }

    pub fn image(&self, escpos_image: EscposImage) -> Result<(), Error> {
        self.raw(&escpos_image.feed(self.printer_profile.width))
    }

    /// Sends raw information to the printer
    ///
    /// As simple as it sounds
    /// ```rust
    /// let bytes = vec![0x01, 0x02];
    /// printer.raw(bytes)
    /// ```
    pub fn raw<A: AsRef<[u8]>>(&self, bytes: A) -> Result<(), Error> {
        self.dh.write_bulk(
            self.endpoint,
            bytes.as_ref(),
            self.printer_profile.timeout
        ).map_err(|e| Error::LibusbError(e))?;
        Ok(())
    }
}