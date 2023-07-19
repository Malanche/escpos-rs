use super::{Justification};
use crate::{Error, command::{Command, ImageMode}};
use image::{DynamicImage, GenericImageView, Pixel};
use serde::{Serialize, Deserialize, ser::Serializer, de::Deserializer};

use base64::{Engine, engine::general_purpose::STANDARD};

use std::collections::{HashMap};
use serde::ser::SerializeTuple;

/// Image adapted to the printer.
///
/// The EscposImage structure keeps the original image, and contains a cache for constructed images for specific printer widths
#[derive(Clone, Debug)]
pub struct EscposImage {
    source: String,
    /// Source image, usefull for scaling
    dynamic_image: DynamicImage,
    /// Cache that holds the picture scaled for specific widths
    pub(crate) cache: HashMap<u16, HashMap<ImageMode, Vec<u8>>>
}

impl EscposImage {
    /// Pub fn creates a new EscposImage from a [DynamicImage](https://docs.rs/image/0.23.14/image/enum.DynamicImage.html)
    ///
    /// The scale parameters goes from 0 to 255, controlling which percentage of the width should the image hold. The justification allows for a bit more specific image alignment.
    pub fn new(mut dynamic_image: DynamicImage, scale: u8, justification: Justification) -> Result<EscposImage, Error> {
        // We extract geometrical data.
        let (im_width, im_height) = dynamic_image.dimensions();
        let aspect_ratio = (im_width as f64)/(im_height as f64);

        // Notice that the width will stay untouched on these steps

        // We compute the scaled width and height, multiplying height by the ratio
        let sc_width = (im_width as f64) * (scale as f64)/255.0;
        // With the aspect ratio, we determine the hight.
        let sc_height = ((sc_width)/aspect_ratio).floor() as u32;
        // We force floor the width, and also cast it as a u32
        let sc_width = sc_width.floor() as u32;

        // We create the new image width
        let mut back = DynamicImage::new_rgba8(im_width, sc_height);

        // We compute the offset for the inner rendering
        let x_offset = match justification {
            Justification::Left => 0,
            Justification::Center => (im_width - sc_width)/2,
            Justification::Right => im_width - sc_width
        };

        // We overlay it in the back image
        image::imageops::overlay(
            &mut back,
            &image::imageops::resize(&dynamic_image, sc_width, sc_height, image::imageops::FilterType::Nearest),
            x_offset, 0 // x and y from the corner
        );

        // We have to create a new cropped image
        dynamic_image = DynamicImage::ImageRgba8(image::imageops::crop(&mut back, 0, 0, im_width, sc_height).to_image());

        let mut encoded = Vec::new();
        // Weird clippy suggestion, the variant acts as a function in the map_err method...
        dynamic_image.write_to(&mut encoded, image::ImageFormat::Png).map_err(Error::ImageError)?;

        let source = STANDARD.encode(&encoded);
        
        Ok(EscposImage {
            source,
            dynamic_image,
            cache: HashMap::new()
        })
    }

    fn build_scaled(&self, image_mode: ImageMode, printer_width: u16) -> Vec<u8> {
        let mut feed = Vec::new();
        feed.extend_from_slice(&Command::NoLine.as_bytes());
        
        let (im_width, im_height) = self.dynamic_image.dimensions();
        // We redefine the aspect ratio
        let aspect_ratio = (im_width as f64)/(im_height as f64);
        
        // Each row will contain the information of 8 rows from the picture
        //const printer_width: usize = 384;
        //const printer_width: usize = 576;
        //let mut printer_rows: Vec<[u8; printer_width]> = Vec::new();
        let mut printer_rows: Vec<Vec<u8>> = Vec::new();

        // El 3.0 es por la baja densidad de impresión vertical (1 byte en lugar de 3)
        let vertical_scale = match image_mode {
            ImageMode::EightDotSingleDensity => 1.0,
            ImageMode::EightDotDoubleDensity => 0.5,
            ImageMode::TwentyfourDotSingleDensity => 3.0,
            ImageMode::TwentyfourDotDoubleDensity => 1.5
        };

        let new_height = ((printer_width as f64) * vertical_scale /(aspect_ratio)).floor() as u32;
        
        let resized_image = image::imageops::resize(&self.dynamic_image, printer_width as u32, new_height, image::imageops::FilterType::Nearest);

        // We will turn the image into a grayscale boolean matrix
        for (y, pixel_row) in resized_image.enumerate_rows() {
            // Here we iterate over each row of the image.
            if y%8 == 0 {
                printer_rows.push(vec![0; printer_width as usize]);
            }
            let row = printer_rows.get_mut((y/8) as usize).unwrap();
            // Here, we iterate horizontally this time
            for (x, y, pixel) in pixel_row {
                let ps = pixel.channels();
                // We get the color as a boolean
                let mut color = if ps.len() == 3 || ps[3] > 64 {
                    let grayscale = 0.2126*(ps[0] as f64) + 0.7152*(ps[1] as f64) + 0.0722*(ps[2] as f64);
                    if grayscale < 78.0 {
                        0x01
                    } else {
                        0x00
                    }
                } else {
                    // It is transparent, so no color
                    0x00
                };
                // We shift the boolean by 7 - y%8 positions in the register
                color <<= 7 - y%8;
                // An or operation preserves the previous pixels in the rows
                row[x as usize] |= color;
            }
        }

        // Finally, we push each row to the feed vector
        match image_mode {
            ImageMode::EightDotSingleDensity | ImageMode::EightDotDoubleDensity => {
                for (_idx, printer_row) in printer_rows.iter().enumerate() {
                    // We first, declare a bitmap mode
                    feed.extend_from_slice(&Command::Bitmap{image_mode: image_mode.clone()}.as_bytes());
                    // The formula on how many pixels we will do, is nL + nH * 256
                    feed.push((printer_width % 256) as u8); // nL
                    feed.push((printer_width / 256) as u8); // nH
                    feed.extend_from_slice(printer_row);
                    feed.push(b'\n'); // Line feed and print
                }
            },
            ImageMode::TwentyfourDotSingleDensity | ImageMode::TwentyfourDotDoubleDensity => {
                for counter in 0..(printer_rows.len() / 3) {
                    let first_row = &printer_rows[3*counter + 0];
                    let second_row = &printer_rows[3*counter + 1];
                    let third_row = &printer_rows[3*counter + 2];

                    let mut buffer = Vec::new();

                    for width_idx in 0..(printer_width as usize) {
                        buffer.push(first_row[width_idx]);
                        buffer.push(second_row[width_idx]);
                        buffer.push(third_row[width_idx]);
                    }
                    // We first, declare a bitmap mode
                    feed.extend_from_slice(&Command::Bitmap{image_mode: image_mode.clone()}.as_bytes());

                    // The formula on how many pixels we will do, is nL + nH * 256
                    feed.push((printer_width % 256) as u8); // nL
                    feed.push((printer_width / 256) as u8); // nH
                    feed.extend_from_slice(&buffer);
                    feed.push(b'\n'); // Line feed and print
                }
            }
        }

        feed.extend_from_slice(&Command::ResetLine.as_bytes());
        feed.extend_from_slice(&Command::Reset.as_bytes());

        feed
    }

    /// Creates a cached image for the specified width
    ///
    /// Useful method to decrease the number of operations done per printing, by skipping the scaling step for a specific printer.
    pub fn cache_for(&mut self, image_mode: ImageMode, width: u16) {
        let cache = self.build_scaled(image_mode.clone(), width);
        let image_modes = self.cache.entry(width).or_insert_with(|| HashMap::new());
        image_modes.insert(image_mode, cache);
    }

    pub fn feed(&self, image_mode: ImageMode, width: u16) -> Vec<u8> {
        if let Some(feed) = self.cache.get(&width).map(|image_modes| image_modes.get(&image_mode)).flatten() {
            feed.clone()
        } else {
            // We have to create the picture... might be costly
            log::warn!("Building an image on the fly in non-mutable mode. Consider caching the width.");
            self.build_scaled(image_mode, width)
        }
    }
}

// Manual implementation of serialization
impl Serialize for EscposImage {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where S: Serializer {
        let mut tup = serializer.serialize_tuple(2)?;
        tup.serialize_element(&self.source)?;
        tup.serialize_element(&self.cache.iter().map(|(width, image_modes)| {
            (*width, image_modes.keys().cloned().collect())
        }).collect::<HashMap<u16, Vec<ImageMode>>>())?;
        tup.end()
    }
}

struct EscposImageVisitor;

impl<'de> serde::de::Visitor<'de> for EscposImageVisitor {
    type Value = EscposImage;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a tuple containing as first element a base64 encoded image, as second a list of cached widths")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error> where A: serde::de::SeqAccess<'de> {
        let value: Option<&[u8]> = seq.next_element()?;
        let value = value.ok_or_else(|| serde::de::Error::custom("first element of tuple missing"))?;
        let content = match STANDARD.decode(value) {
            Ok(v) => v,
            Err(_) => return Err(serde::de::Error::custom("string is not a valid base64 sequence"))
        };
        let dynamic_image = image::load_from_memory(&content).map_err(|_| serde::de::Error::custom("first element of tuple not an image"))?;
        // We will serialize it already
        let mut escpos_image = EscposImage::new(dynamic_image, 255, Justification::Left).map_err(|e| serde::de::Error::custom(format!("failed to create the image, {}", e)))?;

        let cached_widths: HashMap<u16, Vec<ImageMode>> = seq.next_element()?.ok_or_else(|| serde::de::Error::custom("second element of tuple missing"))?;

        for (width, image_modes) in cached_widths {
            for image_mode in image_modes {
                escpos_image.cache_for(image_mode, width);
            }
        }

        Ok(escpos_image)
    }
}

// Manual implementation of deserialization
impl<'de> Deserialize<'de> for EscposImage {
    fn deserialize<D>(deserializer: D) -> Result<EscposImage, D::Error>
    where D: Deserializer<'de> {
        deserializer.deserialize_seq(EscposImageVisitor)
    }
}