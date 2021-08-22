use escpos_rs::{Printer, PrinterProfile};
use libusb::{Context};

fn main() {
    // We create a usb contest for the printer
    let context = Context::new().unwrap();
    let printer_profile = PrinterProfile::usb_builder(0x6868, 0x0200).build();
    // We pass it to the printer
    let printer = match Printer::new(&context, printer_profile) {
        Ok(maybe_printer) => match maybe_printer {
            Some(printer) => printer,
            None => panic!("No printer was found :(")
        },
        Err(e) => panic!("Error: {}", e)
    };
    // We print simple text
    match printer.println("Vamos a mandar una oración larguísima a ver que pasa!") {
        Ok(_) => (),
        Err(e) => println!("Error: {}", e)
    }
}