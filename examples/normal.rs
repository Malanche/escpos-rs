use escpos_rs::{Printer, PrinterProfile, PrinterModel, command::Font};

fn main() {
    let printer_profile = PrinterModel::TMT88VI.usb_profile();
    // We pass it to the printer
    let mut printer = match Printer::new(printer_profile) {
        Ok(maybe_printer) => match maybe_printer {
            Some(printer) => printer,
            None => panic!("No printer was found :(")
        },
        Err(e) => panic!("Error: {}", e)
    };



    printer.set_font(Font::FontB).unwrap();
    //printer.set_space_split(true);
    
    // We print simple text
    match printer.println("Esta va a ser la prueba de lo que ocurre cuando uno manda un texto increiblemente largo") {
        Ok(_) => (),
        Err(e) => println!("Error: {}", e)
    }
}