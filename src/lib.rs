//! Library for controlling esc/pos printers with rust
//!
//! For printing, a libusb [Context](https://docs.rs/libusb/0.3.0/libusb/struct.Context.html) is required.
//!
//! ```rust
//! use escpos_rs::{Printer};
//! use libusb::{Context};
//!
//! fn main() {
//!     // We create a usb contest for the printer
//!     let context = Context::new().unwrap();
//!     // We pass it to the printer
//!     let printer = match Printer::with_context(&context, PrinterModel::TMT20.details()) {
//!         Ok(maybe_printer) => match maybe_printer {
//!             Some(printer) => printer,
//!             None => panic!("No printer was found :(")
//!         },
//!         Err(e) => panic!("Error: {}", e)
//!     };
//!     // We print simple text
//!     match printer.print("Hello, world!") {
//!         Ok(_) => (),
//!         Err(e) => println!("Error: {}", e)
//!     }
//! }
//! ```
//! 
//! The context must be alive at least for the same time the printer will stay in scope. See the [Printer](crate::Printer) structure to see the rest of the implemented functions for interacting with the thermal printer (raw printing, images, etc.).
//!
//! ### Instructions
//!
//! Because of the common use of a thermal printer, an Instruction structure has been implemented, which allows you to define a sort of __template__, that you can use to print multiple documents with __certain__ data customized for each print.
//!
//! ```rust
//! use escpos_rs::{Printer, PrintDataBuilder, Instruction};
//! use libusb::{Context};
//!
//! fn main() {
//!     // We create a usb contest for the printer
//!     let context = Context::new().unwrap();
//!     // We pass it to the printer
//!     let printer = match Printer::with_context(&context, PrinterModel::TMT20.details()) {
//!         Ok(maybe_printer) => match maybe_printer {
//!             Some(printer) => printer,
//!             None => panic!("No printer was found :(")
//!         },
//!         Err(e) => panic!("Error: {}", e)
//!     };
//!     // We create a simple instruction with a single substitution
//!     let instruction = Instruction::text("Hello, %name%!");
//!     // We create custom information for the instruction
//!     let print_data = PrintDataBuilder::new()
//!         .replacement("%name%", "Carlos")
//!         .build();
//!     // We send the instruction to the printer, along with the custom data for this particular print
//!     match printer.text(&instruction, &print_data) {
//!         Ok(_) => (), // "Hello, Carlos!" should've been printed.
//!         Err(e) => println!("Error: {}", e)
//!     }
//! }
//! ```

pub use printer::{Printer, PrinterDetails, PrinterModel};
pub use instruction::{Instruction, Justification, PrintData, PrintDataBuilder};
pub use error::{Error};

/// Contains raw esc/pos commands
pub mod command;

mod printer;
mod instruction;
mod error;