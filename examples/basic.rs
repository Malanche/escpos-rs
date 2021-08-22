use escpos_rs::{Printer, PrinterProfile};

fn main() {
    let printer_profile = PrinterProfile::usb_builder(0x6868, 0x0200).build();
    // We pass it to the printer
    let printer = match Printer::new(printer_profile) {
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