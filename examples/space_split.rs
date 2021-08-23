use escpos_rs::{Printer, PrinterProfile, command::Font};

fn main() {
    let printer_profile = PrinterProfile::terminal_builder().with_font_width(Font::FontA, 32).build();
    // We pass it to the printer
    let mut printer = match Printer::new(printer_profile) {
        Ok(maybe_printer) => match maybe_printer {
            Some(printer) => printer,
            None => panic!("No printer was found :(")
        },
        Err(e) => panic!("Error: {}", e)
    };
    // We set word splitting
    printer.set_space_split(true);

    // We print simple text
    match printer.println("Really long sentence that should be splitted into three components, yay!") {
        Ok(_) => (),
        Err(e) => println!("Error: {}", e)
    }
}