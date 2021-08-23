use escpos_rs::{Printer, PrinterProfile, command::Font};

fn main() {
    let printer_profile = PrinterProfile::terminal_builder().with_font_width(Font::FontA, 20).build();
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
    
    println!("Table with two columns:");
    match printer.duo_table(("Product", "Price"), vec![
        ("Milk", "5.00"),
        ("Cereal", "10.00")
    ]) {
        Ok(_) => (),
        Err(e) => println!("Error: {}", e)
    }

    println!("Table with three columns:");
    match printer.trio_table(("Product", "Price", "Qty."), vec![
        ("Milk", "5.00", "3"),
        ("Cereal", "10.00", "1")
    ]) {
        Ok(_) => (),
        Err(e) => println!("Error: {}", e)
    }
}