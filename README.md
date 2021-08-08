# escpos-rs: A Rust crate for thermal printers

Escpos-rs builds a bit on top of `escpospp`, which aims to bring relatively easy communication to thermal printers that understand the ESC/POS protocol. Here is an example of a simple print with `escpos-rs`

```rust
use escpos_rs::{Printer, PrinterDetails};
use libusb::{Context};

fn main() {
    // We create a usb contest for the printer
    let context = Context::new().unwrap();
    // We create the printer details
    let mut printer_details = PrinterDetails::builder(0x0001, 0x0001).build();
    // We pass it to the printer
    let printer = match Printer::with_context(&context, printer_details) {
        Ok(maybe_printer) => match maybe_printer {
            Some(printer) => printer,
            None => panic!("No printer was found :(")
        },
        Err(e) => panic!("Error: {}", e)
    };
    // We print simple text
    match printer.println("Hello, world!") {
        Ok(_) => (),
        Err(e) => println!("Error: {}", e)
    }
}
```

## The Instruction structure

The Instruction structure has as primary goal the construction of a __template__, which can be used to print multiple documents with dynamic data.

```rust
use escpos_rs::{Printer, PrintData, PrinterDetails, Instruction, Justification, command::Font};
use libusb::{Context};

fn main() {
    // We create a usb contest for the printer
    let context = Context::new().unwrap();
    // Printer details...
    let printer_details = PrinterDetails::builder(0x0001, 0x0001)
        .with_font_width(Font::FontA, 32)
        .build();
    // We pass it to the printer
    let printer = match Printer::with_context(&context, printer_details) {
        Ok(maybe_printer) => match maybe_printer {
            Some(printer) => printer,
            None => panic!("No printer was found :(")
        },
        Err(e) => panic!("Error: {}", e)
    };
    // We create a simple instruction with a single substitution
    let instruction = Instruction::text(
        "Hello, %name%!",
        Font::FontA,
        Justification::Center,
        /// Words that will be replaced in this specific instruction
        Some(vec!["%name%".into()].into_iter().collect())
    );
    // We create custom information for the instruction
    let print_data_1 = PrintData::builder()
        .replacement("%name%", "Carlos")
        .build();
    // And a second set...
    let print_data_2 = PrintData::builder()
        .replacement("%name%", "John")
        .build();
    // We send the instruction to the printer, along with the custom data
    // for this particular print
    match printer.instruction(&instruction, &print_data_1) {
        Ok(_) => (), // "Hello, Carlos!" should've been printed.
        Err(e) => println!("Error: {}", e)
    }
    // Now we print the second data
    match printer.instruction(&instruction, &print_data_2) {
        Ok(_) => (), // "Hello, John!" should've been printed.
        Err(e) => println!("Error: {}", e)
    }
}
```

Instructions can be added up to form a complex instruction.

# About building this library on Windows

Lib usb is needed for the compilation. Go to https://github.com/libusb/libusb/releases and download the compiled binaries, and put them in your include and bin folders for mingw. You will also need a pkg config file.

* Execute the command `pkg-config.exe --variable pc_path pkg-config` to know where `pkg-config` looks up `pc` files

* Add, in any of those routes, the file `libusb-1.0.pc` with the following content

```pc
prefix=c:/mingw-w64/x86_64-8.1.0-posix-seh-rt_v6-rev0/mingw64
exec_prefix=${prefix}
libdir=${prefix}/lib
includedir=${prefix}/include

Name: libusb-1.0
Description: C API for USB device access from Linux, Mac OS X, Windows, OpenBSD/NetBSD and Solaris userspace
Version: 1.0.23
Libs: -L${libdir} -lusb-1.0
Libs.private: -ludev -pthread
Cflags: -I${includedir}/libusb-1.0
```

**Note**: The version must match your libusb version, and the prefix must also match your main include and lib folders for MinGW.

The following steps are based on [this](https://stackoverflow.com/questions/1710922/how-to-install-pkg-config-in-windows) stackoverflow post.

We assume your mingw installation has its binaries in `C:\MinGW\bin`

* Go to [gnome](http://ftp.gnome.org/pub/gnome/binaries/win32/dependencies/), and download the `pkg-config_0.26-1_win32.zip` package
* Extract the file `bin/pkg-config.exe` to `C:\MinGW\bin`
* Download the file [gettext-runtime_0.18.1.1-2_win32.zip](http://ftp.gnome.org/pub/gnome/binaries/win32/dependencies/gettext-runtime_0.18.1.1-2_win32.zip)
* Extract the file bin/intl.dll to `C:\MinGW\bin`
go to http://ftp.gnome.org/pub/gnome/binaries/win32/glib/2.28
* Download the file `glib_2.28.8-1_win32.zip` from [here](http://ftp.gnome.org/pub/gnome/binaries/win32/glib/2.28) (gnome's website, again).
* Extract the file `bin/libglib-2.0-0.dll` to `C:\MinGW\bin`

# Using the library on Windows

I've only been able to use this library when WinUSB driver is in use for the chosen printer. You can use a tool like [Zadig](https://zadig.akeo.ie/) to change your printer's driver. Just bear in mind that this driver change might make the printer invisible to other tools ;).