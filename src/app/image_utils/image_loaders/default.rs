use image::DynamicImage;
use magick_rust::MagickWand;

use std::error::Error;

pub fn load_image_default(buf: &Vec<u8>) -> Result<DynamicImage, Box<dyn Error>> {
    let wand = MagickWand::new();
    wand.read_image_blob(buf)?;

    wand.auto_orient();

    let blob_png = wand.write_image_blob("PNG")?;
    let dynamic_image = image::load_from_memory(&blob_png)?;
    let rgba_image = dynamic_image.to_rgba8();

    Ok(DynamicImage::ImageRgba8(rgba_image))
}
