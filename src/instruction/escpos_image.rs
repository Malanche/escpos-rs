extern crate serde;
extern crate base64;
extern crate image;

use super::{Justification};
use crate::{Error, command::{Command}};
use image::{DynamicImage, GenericImageView, Pixel};
use serde::{Serialize, Deserialize, ser::Serializer, de::Deserializer};

/// Image adapted to the printer.
///
/// The EscposImage structure keeps the original image, and contains a cache for constructed images for specific printer widths
#[derive(Clone, Debug)]
pub struct EscposImage {
    source: String,
    pub(crate) feed: Vec<u8>
}

impl EscposImage {
    pub fn new(content: Vec<u8>, scale: u8, justification: Justification) -> Result<EscposImage, Error> {
        let mut feed = Vec::new();
        feed.extend_from_slice(&Command::NoLine.as_bytes());

        let img = match image::load_from_memory(&content) {
            Ok(i) => i,
            Err(e) => return Err(Error::ImageError(e))
        };

        // We extract geometrical data
        let (width, height) = img.dimensions();
        let aspect_ratio = (width as f64)/(height as f64);

        // We compute the scaled width and height
        let sc_width = (width as f64) * (scale as f64)/255.0;
        let sc_height = ((sc_width)/aspect_ratio).floor() as u32;
        let sc_width = sc_width.floor() as u32;

        let mut back = DynamicImage::new_rgba8(width, sc_height);

        let x_offset = match justification {
            Justification::Left => 0,
            Justification::Center => (width - sc_width)/2,
            Justification::Right => width - sc_width
        };

        image::imageops::overlay(
            &mut back,
            &image::imageops::resize(&img, sc_width, sc_height, image::imageops::FilterType::Nearest),
            x_offset, 0 // x and y from the corner
        );

        // We have to create a new cropped image
        let cropped = image::imageops::crop(&mut back, 0, 0, width, sc_height);
        
        // We redefine the aspect ratio
        let aspect_ratio = (width as f64)/(sc_height as f64);
        
        // Each row will contain the information of 8 rows from the picture
        //const PRINTER_WIDTH: usize = 384;
        const PRINTER_WIDTH: usize = 576;
        let mut printer_rows: Vec<[u8; PRINTER_WIDTH]> = Vec::new();

        // El *3 es por la baja densidad de impresión horizontal (1 byte en lugar de 3)
        let new_height = ((PRINTER_WIDTH as f64)/(aspect_ratio*3.0)).floor() as u32;
        
        let b = image::imageops::resize(&cropped, PRINTER_WIDTH as u32, new_height, image::imageops::FilterType::Nearest);

        for (y, pixel_row) in b.enumerate_rows() {
            // Here we iterate over each row of the image.
            if y%8 == 0 {
                printer_rows.push([0; PRINTER_WIDTH]);
            }
            let row = printer_rows.get_mut((y/8) as usize).unwrap();
            // Here, we iterate horizontally this time
            for (x, y, pixel) in pixel_row {
                let ps = pixel.channels();
                // We get the color as a boolean
                let mut color = if ps.len() == 3 {
                    let grayscale = 0.2126*(ps[0] as f64) + 0.7152*(ps[1] as f64) + 0.0722*(ps[2] as f64);
                    if grayscale < 78.0 {
                        0x01
                    } else {
                        0x00
                    }
                } else {
                    if ps[3] > 64 {
                        let grayscale = 0.2126*(ps[0] as f64) + 0.7152*(ps[1] as f64) + 0.0722*(ps[2] as f64);
                        if grayscale < 78.0 {
                            0x01
                        } else {
                            0x00
                        }
                    } else {
                        // It is transparent, so no color
                        0x00
                    }
                };
                // We shift the boolean by 7 - y%8 positions in the register
                color = color << (7 - y%8);
                // An or operation preserves the previous pixels in the rows
                row[x as usize] = row[x as usize] | color;
            }
        }

        for (_idx, printer_row) in printer_rows.iter().enumerate() {
            // We first, declare a bitmap mode
            feed.extend_from_slice(&Command::Bitmap.as_bytes());
            // Now, we pass m
            let m = 0x01;
            feed.push(m);
            // The formula on how many pixels we will do, is nL + nH * 256
            feed.push((PRINTER_WIDTH % 256) as u8); // nL
            feed.push((PRINTER_WIDTH / 256) as u8); // nH
            // feed.push(0x80); // nL
            // feed.push(0x01); // nH
            feed.extend_from_slice(printer_row);
            feed.push(b'\n'); // Line feed and print
        }
        feed.extend_from_slice(&Command::ResetLine.as_bytes());
        feed.extend_from_slice(&Command::Reset.as_bytes());

        let source = base64::encode(&feed);

        Ok(EscposImage {
            source,
            feed
        })
    }
}

// Manual implementation of serialization
impl Serialize for EscposImage {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where S: Serializer {
        //Err(e) => Err(serde::ser::Error::custom(format!("could not cast x509 into pem bytes, {}", e)))
        serializer.serialize_str(&self.source)
    }
}

struct EscposImageVisitor;

impl<'de> serde::de::Visitor<'de> for EscposImageVisitor {
    type Value = EscposImage;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a base64 string from any image format except TGA")
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where E: serde::de::Error {
        let feed = match base64::decode(value) {
            Ok(v) => v,
            Err(_) => return Err(serde::de::Error::custom("string is not a valid base64 sequence"))
        };        
        // We will serialize it already
        Ok(EscposImage{source: value.to_string(), feed})
    }
}

// Manual implementation of deserialization
impl<'de> Deserialize<'de> for EscposImage {
    fn deserialize<D>(deserializer: D) -> Result<EscposImage, D::Error>
    where D: Deserializer<'de> {
        deserializer.deserialize_str(EscposImageVisitor)
    }
}