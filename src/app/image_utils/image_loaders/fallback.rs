use image::DynamicImage;

use std::error::Error;

pub fn load_image_fallback(buf: &[u8]) -> Result<DynamicImage, Box<dyn Error>> {
    let rgba_image = image::load_from_memory(buf)?.to_rgba8();
    Ok(DynamicImage::ImageRgba8(rgba_image))
}
