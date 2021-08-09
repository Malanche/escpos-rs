use std::collections::HashMap;
use crate::command::Font;

/// Details required to connect and print
///
/// In order to use the full functionality of the library, some information should be provided regarding the printer. The bare minimum information needed is the product id and the vendor id.
#[derive(Clone, Debug)]
pub struct PrinterProfile {
    /// Vendor id for the printer
    pub (crate) vendor_id: u16,
    /// product id for the printer
    pub (crate) product_id: u16,
    /// Paper width, in characters, for the printer
    pub (crate) columns_per_font: HashMap<Font, u8>,
    /// Total printer width in pixels, for image printing
    pub (crate) width: u16,
    /// Endpoint where the usb data is meant to be written to
    pub (crate) endpoint: Option<u8>,
    /// Timeout for bulk write operations
    pub (crate) timeout: std::time::Duration
}

impl PrinterProfile {
    /// Create custom printing details
    ///
    /// Not recommended to use, as it contains a lot of arguments. See the [builder](PrinterProfile::builder) function instead.
    pub fn new(vendor_id: u16, product_id: u16, columns_per_font: HashMap<Font, u8>, width: u16, endpoint: Option<u8>, timeout: std::time::Duration) -> PrinterProfile {
        PrinterProfile {
            vendor_id,
            product_id,
            columns_per_font,
            width,
            endpoint,
            timeout
        }
    }

    /// Creates a [PrinterProfileBuilder](crate::PrinterProfileBuilder)
    ///
    /// Equivalent to a call to [PrinterProfileBuilder](crate::PrinterProfileBuilder)'s [new](crate::PrinterProfileBuilder::new) function.
    /// ```rust
    /// # use escpos_rs::PrinterProfile;
    /// // Creates a minimum data structure to connect to a printer
    /// let printer_profile = PrinterProfile::builder(0x0001, 0x0001).build();
    /// ```
    pub fn builder(vendor_id: u16, product_id: u16) -> PrinterProfileBuilder {
        PrinterProfileBuilder::new(vendor_id, product_id)
    }
}

/// Helper structure to create [PrinterProfile](crate::PrinterProfile)
///
/// Builder pattern for the [PrinterProfile](crate::PrinterProfile) structure.
pub struct PrinterProfileBuilder {
    vendor_id: u16,
    product_id: u16,
    columns_per_font: HashMap<Font, u8>,
    width: u16,
    endpoint: Option<u8>,
    timeout: std::time::Duration
}

impl PrinterProfileBuilder {
    /// Creates a new [PrinterProfileBuilder](crate::PrinterProfileBuilder)
    ///
    /// ```rust
    /// # use escpos_rs::PrinterProfileBuilder;
    /// // Creates a minimum (probably non-working) data structure to connect to a printer
    /// let printer_profile = PrinterProfileBuilder::new(0x0001, 0x0001).build();
    /// ```
    ///
    /// The data structure will be properly built just with the vendor id and the product id. The [Printer](crate::Printer)'s [with_context](crate::Printer::with_context) method will try to locate a bulk write endpoint, but it might fail to do so. See [with_endpoint](PrinterProfileBuilder::with_endpoint) for manual setup.
    pub fn new(vendor_id: u16, product_id: u16) -> PrinterProfileBuilder {
        PrinterProfileBuilder {
            vendor_id,
            product_id,
            columns_per_font: HashMap::new(),
            width: 384,
            endpoint: None,
            timeout: std::time::Duration::from_secs(2)
        }
    }

    /// Sets the usb endpoint to which the data will be written.
    ///
    /// ```rust
    /// # use escpos_rs::PrinterProfileBuilder;
    /// // Creates the printer details with the endpoint 0x02
    /// let printer_profile = PrinterProfileBuilder::new(0x0001, 0x0001)
    ///     .with_endpoint(0x02)
    ///     .build();
    /// ```
    pub fn with_endpoint(mut self, endpoint: u8) -> PrinterProfileBuilder {
        self.endpoint = Some(endpoint);
        self
    }

    /// Adds a specific pixel width for the printer (required for printing images)
    ///
    /// Defaults to 384, usually for 58mm printers.
    /// ```rust
    /// # use escpos_rs::PrinterProfileBuilder;
    /// let printer_profile = PrinterProfileBuilder::new(0x0001, 0x0001)
    ///     .with_width(384)
    ///     .build();
    /// ```
    pub fn with_width(mut self, width: u16) -> PrinterProfileBuilder {
        self.width = width;
        self
    }

    /// Adds a specific width per font
    ///
    /// This allows the justification, and proper word splitting to work. If you feel insecure about what value to use, the default font (FontA) usually has 32 characters of width for 58mm paper printers, and 48 for 80mm paper. You can also look for the specsheet, or do trial and error.
    /// ```rust
    /// # use escpos_rs::{PrinterProfileBuilder, command::Font};
    /// let printer_profile = PrinterProfileBuilder::new(0x0001, 0x0001)
    ///     .with_font_width(Font::FontA, 32)
    ///     .build();
    /// ```
    pub fn with_font_width(mut self, font: Font, width: u8) -> PrinterProfileBuilder {
        self.columns_per_font.insert(font, width);
        self
    }

    /// Adds a bulk write timeout
    ///
    /// USB devices might fail to write to the bulk endpoint. In such a case, a timeout must be provided to know when to stop waiting for the buffer to flush to the printer. The default value is 2 seconds.
    /// ```rust
    /// # use escpos_rs::PrinterProfileBuilder;
    /// let printer_profile = PrinterProfileBuilder::new(0x0001, 0x0001)
    ///     .with_timeout(std::time::Duration::from_secs(3))
    ///     .build();
    /// ```
    pub fn with_timeout(mut self, timeout: std::time::Duration) -> PrinterProfileBuilder {
        self.timeout = timeout;
        self
    }

    /// Build the `PrinterProfile` that lies beneath the builder
    ///
    /// ```rust
    /// # use escpos_rs::PrinterProfileBuilder;
    /// let printer_profile = PrinterProfileBuilder::new(0x0001, 0x0001).build();
    /// ```
    pub fn build(self) -> PrinterProfile {
        PrinterProfile {
            vendor_id: self.vendor_id,
            product_id: self.product_id,
            columns_per_font: self.columns_per_font,
            width: self.width,
            endpoint: self.endpoint,
            timeout: self.timeout
        }
    }
}