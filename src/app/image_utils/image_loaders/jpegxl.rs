use image::DynamicImage;
use jpegxl_rs::image::ToDynamic;

use std::error::Error;

pub fn load_image_jpegxl(buf: &Vec<u8>) -> Result<DynamicImage, Box<dyn Error>> {
    let decoder = jpegxl_rs::decoder_builder().build()?;
    let image = decoder.decode_to_image(buf)?.unwrap();
    let rgba_image = image.to_rgba8();

    Ok(DynamicImage::ImageRgba8(rgba_image))
}
