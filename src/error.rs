#[derive(Debug)]
pub enum Error {
    /// Error related to libusb
    LibusbError(libusb::Error),
    /// For text printing, the replaced sequence could not be found
    CP437Error(String),
    /// Error regarding image treatment
    ImageError(image::ImageError),
    NoReplacementFound(String),
    PrinterError(String),
    WrongMarkdown,
    NoTables,
    NoTableFound(String),
    NoWidth,
    NoQrContent(String),
    NoQrContents,
    Encoding
}

impl std::fmt::Display for Error {
    fn fmt(&self, formatter: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        let content = match self {
            Error::LibusbError(e) => format!("Libusb error: {}", e),
            Error::CP437Error(detail) => format!("CP437 error: {}", detail),
            Error::ImageError(e) => format!("Image error: {}", e),
            Error::NoReplacementFound(replacement) => format!("Could not find replacement for tag {{{}}}", replacement),
            Error::PrinterError(detail) => format!("An error occured while printing, {}", detail),
            Error::WrongMarkdown => format!("Incorrect markdown structure"),
            Error::NoTables => format!("Not a single table was found in the PrintData structure"),
            Error::NoTableFound(table) => format!("No table was found for id {{{}}}", table),
            Error::NoWidth => format!("No width was found for the selected font"),
            Error::NoQrContent(name) => format!("Could not find qr code content for \"{}\"", name),
            Error::NoQrContents => format!("Could not find qr contents"),
            Error::Encoding => format!("An unsupported utf-8 character was found when passing to cp437")
        };
        write!(formatter, "{}", content)
    }
}

impl std::error::Error for Error{}