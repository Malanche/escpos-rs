//! Library for controlling esc/pos printers with rust
//!
//! Not ready for production (yet, but soon!). For printing, a libusb [Context](https://docs.rs/libusb/0.3.0/libusb/struct.Context.html) is required.
//!
//! ```rust,no_run
//! use escpos_rs::{Printer, PrinterProfile};
//! use libusb::{Context};
//!
//! // We create a usb contest for the printer
//! let context = Context::new().unwrap();
//! let printer_profile = PrinterProfile::builder(0x0001, 0x0001).build();
//! // We pass it to the printer
//! let printer = match Printer::with_context(&context, printer_profile) {
//!     Ok(maybe_printer) => match maybe_printer {
//!         Some(printer) => printer,
//!         None => panic!("No printer was found :(")
//!     },
//!     Err(e) => panic!("Error: {}", e)
//! };
//! // We print simple text
//! match printer.println("Hello, world!") {
//!     Ok(_) => (),
//!     Err(e) => println!("Error: {}", e)
//! }
//! ```
//! 
//! The context must be alive at least for the same time the printer will stay in scope. See the [Printer](crate::Printer) structure to see the rest of the implemented functions for interacting with the thermal printer (raw printing, images, etc.).
//!
//! ## Printer Details
//!
//! In order to print, some data about the printer must be known. The [PrinterProfile](crate::PrinterProfile) structure fulfills this purpose.
//!
//! The strict minimum information needed to print, are the vendor id, the product id. Both vendor and product id should be found in the maker's website, or sometimes they get printed in test prints (which usually occur if you hold the feed button on the printer).
//!
//! If you are running linux, then one way to get these values is by executing the `lsusb` command.
//!
//! ### Instructions
//!
//! Because of the usual applications for thermal printers, the [Instruction](crate::Instruction) structure has been implemented, which allows you to define a sort of __template__, that you can use to print multiple documents with __certain__ data customized for each print.
//!
//! ```rust,no_run
//! use escpos_rs::{
//!     Printer, PrintData, PrinterProfile,
//!     Instruction, Justification, command::Font
//! };
//! use libusb::{Context};
//! 
//! // We create a usb contest for the printer
//! let context = Context::new().unwrap();
//! // Printer details...
//! let printer_profile = PrinterProfile::builder(0x0001, 0x0001)
//!     .with_font_width(Font::FontA, 32)
//!     .build();
//! // We pass it to the printer
//! let printer = match Printer::with_context(&context, printer_profile) {
//!     Ok(maybe_printer) => match maybe_printer {
//!         Some(printer) => printer,
//!         None => panic!("No printer was found :(")
//!     },
//!     Err(e) => panic!("Error: {}", e)
//! };
//! // We create a simple instruction with a single substitution
//! let instruction = Instruction::text(
//!     "Hello, %name%!",
//!     Font::FontA,
//!     Justification::Center,
//!     /// Words that will be replaced in this specific instruction
//!     Some(vec!["%name%".into()].into_iter().collect())
//! );
//! // We create custom information for the instruction
//! let print_data_1 = PrintData::builder()
//!     .replacement("%name%", "Carlos")
//!     .build();
//! // And a second set...
//! let print_data_2 = PrintData::builder()
//!     .replacement("%name%", "John")
//!     .build();
//! // We send the instruction to the printer, along with the custom data
//! // for this particular print
//! match printer.instruction(&instruction, Some(&print_data_1)) {
//!     Ok(_) => (), // "Hello, Carlos!" should've been printed.
//!     Err(e) => println!("Error: {}", e)
//! }
//! // Now we print the second data
//! match printer.instruction(&instruction, Some(&print_data_2)) {
//!     Ok(_) => (), // "Hello, John!" should've been printed.
//!     Err(e) => println!("Error: {}", e)
//! }
//! ```
//!
//! This structure implements both Serialize, and Deserialize from [serde](https://docs.rs/serde), so it is possible to store these instructions to recover them from memory. You can serialize to a json, as pictures are encoded to base64 first to be utf-8 compatible.

pub use printer::{Printer, PrinterProfile, PrinterProfileBuilder, PrinterModel, PrinterConnectionData};
pub use instruction::{Instruction, Justification, PrintData, PrintDataBuilder, EscposImage};
pub use error::{Error};

/// Contains raw esc/pos commands
pub mod command;

mod printer;
mod instruction;
mod error;