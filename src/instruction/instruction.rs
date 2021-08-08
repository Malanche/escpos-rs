extern crate serde;
extern crate codepage_437;
extern crate image;
extern crate qrcode;

use qrcode::QrCode;
use codepage_437::{IntoCp437, CP437_CONTROL};
use crate::{
    Error, PrinterProfile,
    command::{Command, Font}
};
use serde::{Serialize, Deserialize};
use super::{Justification, PrintData, EscposImage};
use std::collections::HashSet;

/// Templates for recurrent prints
///
/// The [Instruction](crate::Instruction) structure allows the creation of template prints, which could contain certain data that should change between prints (be it text, tables, or even qr codes).
///
/// It is not adviced to construct the variants of the enum manually, read the available functions to guarantee a predictable outcome.
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(tag = "kind")]
pub enum Instruction {
    /// Compound instruction, composed of multiple instructions that must be executed sequentially
    Compound {
        instructions: Vec<Instruction>
    },
    /// An instruction consisting of a single `esc/pos` command
    Command {
        command: Command
    },
    /// Short for jumping a specified number of lines
    VSpace {
        lines: u8
    },
    /// Raw text
    Text {
        /// Content to be printed
        content: String,
        /// Indicates if markdown translation should be applied
        markdown: bool,
        /// Font to be used with this text
        font: Font,
        /// Justification of the content
        justification: Justification,
        /// Maps a string to be replaced, to a description of the string
        replacements: Option<HashSet<String>>
    },
    /// 2 column table
    DuoTable {
        /// Name of the table. Required for attaching tuples for printing
        name: String,
        /// Header to be displayed on the table
        header: (String, String),
        /// Font used for the table
        font: Font
    },
    /// Table with three columns. Might be to tight for 50mm printers
    TrioTable {
        name: String,
        header: (String, String, String)
    },
    /// Fancy table for really detailed prints
    QuadTable {
        name: String,
        header: (String, String, String)
    },
    /// Contains a static image, that is, does not change with different printing mechanisms
    Image {
        /// Inner image
        image: EscposImage
    },
    /// Prints a QR Code. This field is dynamic
    QRCode {
        /// Name of the QR code, to be searched in the qr code content list
        name: String
    },
    /// Cuts the paper in place. Only for supported printers
    Cut
}

/// Instruction addition
impl std::ops::Add<Instruction> for Instruction {
    type Output = Instruction;
    fn add(self, rhs: Instruction) -> Self::Output {
        match self {
            Instruction::Compound{mut instructions} => {
                match rhs {
                    Instruction::Compound{instructions: mut rhs_instructions} => {
                        instructions.append(&mut rhs_instructions);
                    },
                    rhs => {
                        instructions.push(rhs);
                    }
                }
                Instruction::Compound{instructions}
            },
            lhs => {
                let mut instructions = vec![lhs];
                match rhs {
                    Instruction::Compound{instructions: mut rhs_instructions} => {
                        instructions.append(&mut rhs_instructions);
                    },
                    rhs => {
                        instructions.push(rhs);
                    }
                }
                Instruction::Compound{instructions}
            }
        }
    }
}

/// Mutable addition for instructions
impl std::ops::AddAssign for Instruction {
    fn add_assign(&mut self, other: Self) {
        // Now we deal with this thing
        if !self.is_compound() {
            // It was not a compound element, so we make it such
            *self = Instruction::Compound{instructions: vec![self.clone()]};
        }

        match self {
            Instruction::Compound{instructions} => {
                match other {
                    Instruction::Compound{instructions: mut other_instructions} => {
                        instructions.append(&mut other_instructions);
                    },
                    other => {
                        instructions.push(other);
                    }
                }
            },
            _ => panic!("Impossible error")
        }
    }
}

impl Instruction {
    /// Returns true if the instruction is compund
    pub fn is_compound(&self) -> bool {
        match self {
            Instruction::Compound{..} => true,
            _ => false
        }
    }

    /// Returns true if the instruction is text
    pub fn is_text(&self) -> bool {
        match self {
            Instruction::Text{..} => true,
            _ => false
        }
    }

    /// Sends simple text to the printer.
    ///
    /// Straightfoward text printing. The `replacements` set specifies which contents of the string should be replaced in a per-impresion basis.
    pub fn text<A: Into<String>>(content: A, font: Font, justification: Justification, replacements: Option<HashSet<String>>) -> Instruction {
        Instruction::Text {
            content: content.into(),
            markdown: false,
            font,
            justification,
            replacements
        }
    }

    /// Sends markdown text to the printer
    ///
    /// Allows markdown to be sent to the printer. Not everything is supported, so far the following list works (if the printer supports the corresponding fonts)
    ///  * Bold font, with **
    ///  * Italics, with _
    ///  * Strike
    pub fn markdown(content: String, font: Font, justification: Justification, replacements: Option<HashSet<String>>) -> Instruction {
        Instruction::Text {
            content,
            markdown: true,
            font,
            justification,
            replacements
        }
    }

    /// Attempts to create an image to be printed, from a byte sequence
    ///
    /// * The scale value stands as scale / 255
    ///
    /// For a more precise control of position in the image, it is easier to edit the input image beforehand.
    pub fn image(source: Vec<u8>, scale: u8, justification: Justification) -> Result<Instruction, Error> {
        let content = match image::load_from_memory(&source) {
            Ok(i) => i,
            Err(e) => return Err(Error::ImageError(e))
        };
        let img = EscposImage::new(content, scale, justification)?;
        Ok(Instruction::Image {
            image: img
        })
    }

    /// Creates a new QR code that does not change through different print steps
    pub fn qr_code(content: String) -> Result<Instruction, Error> {
        let code = QrCode::new(content.as_bytes()).unwrap();
        // Render the bits into an image.
        let img = code.render::<image::Rgba<u8>>().build();

        let mut content = Vec::new();
        image::DynamicImage::ImageRgba8(img).write_to(&mut content, image::ImageOutputFormat::Png).unwrap();
        
        Instruction::image(content, 128, Justification::Center)
    }

    /// Creates a dynamic qr code instruction, which requires a string at printing time
    pub fn dynamic_qr_code<A: Into<String>>(name: A) -> Instruction {
        Instruction::QRCode{name: name.into()}
    }

    /// Executes a raw escpos command.
    pub fn command(command: Command) -> Instruction {
        Instruction::Command {
            command
        }
    }

    /// Creates a table with two columns.
    pub fn duo_table(name: String, header: (String, String), font: Font) -> Instruction {
        Instruction::DuoTable {
            name,
            header,
            font
        }
    }

    /// Creates a table with three columns
    pub fn trio_table(name: String, header: (String, String, String)) -> Instruction {
        Instruction::TrioTable {
            name: name,
            header: header
        }
    }

    /// Creates a table with three columns
    pub fn quad_table(name: String, header: (String, String, String)) -> Instruction {
        Instruction::QuadTable {
            name: name,
            header: header
        }
    }

    pub fn cut() -> Instruction {
        Instruction::Cut
    }

    /// Moves the paper a certain amount of vertical spaces
    pub fn vspace(lines: u8) -> Instruction {
        Instruction::VSpace{lines}
    }

    /// Main serialization function
    pub(crate) fn to_vec(&self, printer_profile: &PrinterProfile, print_data: &PrintData) -> Result<Vec<u8>, Error> {
        let mut target = Vec::new();
        match self {
            Instruction::Compound{instructions} => {
                for instruction in instructions {
                    target.append(&mut instruction.to_vec(printer_profile, print_data)?);
                }
            },
            Instruction::Cut => {
                target.extend_from_slice(&Command::Cut.as_bytes());
            },
            Instruction::Command{command} => {
                target.append(&mut command.as_bytes());
            }
            Instruction::VSpace{lines} => {
                target.append(&mut vec![b'\n'; *lines as usize])
            },
            Instruction::Image{image} => {
                target.extend_from_slice(&image.feed(printer_profile.width));
            },
            Instruction::QRCode{name} => {
                if let Some(qr_contents) = &print_data.qr_contents {
                    if let Some(qr_content) = qr_contents.get(name) {
                        target.extend_from_slice(&Instruction::qr_code(qr_content.clone())?.to_vec(printer_profile, print_data)?)
                    } else {
                        return Err(Error::NoQrContent(name.clone()))
                    }
                } else {
                    return Err(Error::NoQrContents)
                }
            },
            // Text serialization for the printer
            Instruction::Text{content, markdown, font, justification, replacements: self_replacements} => {
                // We setup the font, mainly
                target.append(&mut Command::SelectFont{font: font.clone()}.as_bytes());

                // We extract the width for this font
                let width = match printer_profile.columns_per_font.get(&font) {
                    Some(w) => *w,
                    None => return Err(Error::NoWidth)
                };

                let mut replaced_string = content.clone();
                // First of all, we replace all the replacements
                if let Some(self_replacements) = &self_replacements {
                    for key in self_replacements.iter() {
                        if let Some(replacement) = print_data.replacements.get(key) {
                            replaced_string = replaced_string.as_str().replace(key, replacement);
                        } else {
                            return Err(Error::NoReplacementFound(key.clone()))
                        }
                    }
                }

                // Now, we demarkdownize the string
                let demarkdown_string = if *markdown {
                    // We tokenize the string
                    let mut _tmp = String::new();
                    panic!("Not implemented the markdown thingy, is too hard!");
                } else {
                    replaced_string
                };

                // Now, we tokenize by spaces, using the width and justification
                let mut result = Command::Reset.as_bytes();
                // Line to control the text
                let mut line = String::new();
                let tokens = demarkdown_string.split_whitespace();
                let mut width_count = 0;
                
                for token in tokens {
                    if width_count + token.len() + 1 > (width as usize) {
                        // We have to create a new line, this does not fit.
                        width_count = token.len();
                        // Now we actually format the line
                        let mut tmp = match justification {
                            Justification::Left => format!("{}\n", line),
                            Justification::Right => format!("{:>1$}\n", line, width as usize),
                            Justification::Center => format!("{:^1$}\n", line, width as usize)
                        }.into_cp437(&CP437_CONTROL).map_err(|_| Error::Encoding)?;
                        result.append(&mut tmp);

                        // And we start the new line
                        line = token.to_string();
                    } else {
                        width_count += token.len();
                        if line.len() != 0 {
                            width_count += 1;
                            line += " ";
                        }
                        line += token;
                    }
                }

                // Last, we deal with the last line
                if line.len() != 0 {
                    let mut tmp = match justification {
                        Justification::Left => format!("{}\n", line),
                        Justification::Right => format!("{:>1$}\n", line, width as usize),
                        Justification::Center => format!("{:^1$}\n", line, width as usize)
                    }.into_cp437(&CP437_CONTROL).map_err(|_| Error::Encoding)?;
                    result.append(&mut tmp);
                }
                
                target.append(&mut result);
            },
            Instruction::DuoTable{name, header, font} => {
                // We extract the width for this font
                let width = match printer_profile.columns_per_font.get(&font) {
                    Some(w) => *w,
                    None => return Err(Error::NoWidth)
                };
                //First, the headers
                target.extend_from_slice(&format!("{}{:>2$}\n", header.0, header.1, (width as usize) - header.0.len()).into_cp437(&CP437_CONTROL).map_err(|_| Error::Encoding)?);

                // Now, the line too
                target.append(&mut vec![b'-'; width as usize]);
                target.push(b'\n');
                
                // Now we actually look up the table
                if let Some(tables) = &print_data.duo_tables {
                    if let Some(table) = tables.get(name) {
                        for row in table {
                            target.extend_from_slice(&format!("{}{:>2$}\n", row.0, row.1, (width as usize) - row.0.len()).into_cp437(&CP437_CONTROL).map_err(|_| Error::Encoding)?)
                        }
                    } else {
                        return Err(Error::NoTableFound(name.clone()))
                    }
                } else {
                    return Err(Error::NoTables)
                }
            },
            Instruction::TrioTable{name, header} => {
                // First, we will determine the proper alignment for the middle component
                let mut max_left: usize = header.0.len();
                let mut max_middle: usize = header.1.len();
                let mut max_right: usize = header.2.len();
                if let Some(tables) = &print_data.trio_tables {
                    if let Some(table) = tables.get(name) {
                        for row in table {
                            if row.0.len() > max_left {
                                max_left = row.0.len();
                            }
                            if row.1.len() > max_middle {
                                max_middle = row.1.len();
                            }
                            if row.2.len() > max_right {
                                max_right = row.2.len();
                            }
                        }
                    } else {
                        return Err(Error::NoTableFound(name.clone()))
                    }
                } else {
                    return Err(Error::NoTables)
                }

                // We chose a font
                let width = match printer_profile.columns_per_font.get(&Font::FontA) {
                    Some(w) => *w,
                    None => return Err(Error::NoWidth)
                } as usize;

                let (max_left, max_right) = if max_left + max_middle + max_right + 2 <= width {
                    // Todo va excelentemente bien.
                    (max_left, max_right)
                } else {
                    if max_middle + max_right + 2 <= width  && width - max_middle - max_right - 2 > 2 {
                        // I am sorry, Mr. left side.
                        (width - max_middle - max_right - 2, max_right)
                    } else {
                        // Unluckily, we try to go for thirds
                        let third = width / 3;
                        if width % 3 == 0 {
                            (third, third)
                        } else if width % 3 == 1 {
                            (third, third)
                        } else {
                            (third, third)
                        }
                    }
                };

                // We go with the headers
                target.extend_from_slice(
                    &trio_row(header.clone(), width, max_left, max_right)
                .into_cp437(&CP437_CONTROL).map_err(|_| Error::Encoding)?);

                // Now, the line too
                target.append(&mut vec![b'-'; width]);
                target.push(b'\n');
                
                // Now we actually look up the table
                if let Some(tables) = &print_data.trio_tables {
                    if let Some(table) = tables.get(name) {
                        for row in table {
                            target.extend_from_slice(
                                &trio_row(row.clone(), width, max_left, max_right)
                            .into_cp437(&CP437_CONTROL).map_err(|_| Error::Encoding)?);
                        }
                    } else {
                        return Err(Error::NoTableFound(name.clone()))
                    }
                } else {
                    return Err(Error::NoTables)
                }
            },
            Instruction::QuadTable{name, header} => {
                // First, we will determine the proper alignment for the middle component
                let mut max_left: usize = header.0.len();
                let mut max_middle: usize = header.1.len();
                let mut max_right: usize = header.2.len();
                if let Some(tables) = &print_data.quad_tables {
                    if let Some(table) = tables.get(name) {
                        for row in table {
                            if row.1.len() > max_left {
                                max_left = row.1.len();
                            }
                            if row.2.len() > max_middle {
                                max_middle = row.2.len();
                            }
                            if row.3.len() > max_right {
                                max_right = row.3.len();
                            }
                        }
                    } else {
                        return Err(Error::NoTableFound(name.clone()))
                    }
                } else {
                    return Err(Error::NoTables)
                }

                // We chose a font
                let width = match printer_profile.columns_per_font.get(&Font::FontA) {
                    Some(w) => *w,
                    None => return Err(Error::NoWidth)
                } as usize;

                let (max_left, max_right) = if max_left + max_middle + max_right + 2 <= width {
                    // Todo va excelentemente bien.
                    (max_left, max_right)
                } else {
                    if max_middle + max_right + 2 <= width  && width - max_middle - max_right - 2 > 2 {
                        // I am sorry, Mr. left side.
                        (width - max_middle - max_right - 2, max_right)
                    } else {
                        // Unluckily, we try to go for thirds
                        let third = width / 3;
                        if width % 3 == 0 {
                            (third, third)
                        } else if width % 3 == 1 {
                            (third, third)
                        } else {
                            (third, third)
                        }
                    }
                };

                // We go with the headers
                target.extend_from_slice(
                    &trio_row((header.0.clone(), header.1.clone(), header.2.clone()), width, max_left, max_right)
                .into_cp437(&CP437_CONTROL).map_err(|_| Error::Encoding)?);

                // Now, the line too
                target.append(&mut vec![b'-'; width]);
                target.push(b'\n');
                
                // Now we actually look up the table
                if let Some(tables) = &print_data.quad_tables {
                    if let Some(table) = tables.get(name) {
                        for row in table {
                            // First row
                            target.extend_from_slice(&Command::SelectFont{font: Font::FontB}.as_bytes());
                            target.extend_from_slice(&format!("{}\n", row.0).into_cp437(&CP437_CONTROL).map_err(|_| Error::Encoding)?);
                            target.extend_from_slice(&Command::SelectFont{font: Font::FontA}.as_bytes());
                            // Now the three columns
                            target.extend_from_slice(
                                &trio_row((row.1.clone(), row.2.clone(), row.3.clone()), width, max_left, max_right)
                            .into_cp437(&CP437_CONTROL).map_err(|_| Error::Encoding)?);
                        }
                    } else {
                        return Err(Error::NoTableFound(name.clone()))
                    }
                } else {
                    return Err(Error::NoTables)
                }
            }
        }
        Ok(target)
    }
}

// Auxiliar function to obtain three-row formatted string
fn trio_row(mut row: (String, String, String), width: usize, max_left: usize, max_right: usize) -> String {
    if row.0.len() > max_left {
        row.0.replace_range((max_left-2).., "..");
    }
    if row.1.len() > width - max_left - max_right - 2 {
        row.1.replace_range((width - max_left - max_right - 2).., "..");
    }
    if row.2.len() > max_left {
        row.2.replace_range((max_right-2).., "..");
    }
    row.0.truncate(max_left);
    row.2.truncate(max_right);
    row.1.truncate(width - max_left - max_right - 2);

    format!("{:<3$}{:^4$}{:>5$}\n",
        row.0, row.1, row.2, // Words
        max_left, width - max_left - max_right, max_right // Lengths
    )
}