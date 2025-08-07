use image::{DynamicImage, RgbImage};
use libheif_rs::{HeifContext, LibHeif};

use rayon::prelude::*;
use std::error::Error;

pub fn load_image_heif(buf: &Vec<u8>) -> Result<DynamicImage, Box<dyn Error>> {
    let lib_heif = LibHeif::new();
    let ctx = HeifContext::read_from_bytes(buf)?;
    let handle = ctx.primary_image_handle()?;

    let image = lib_heif.decode(
        &handle,
        libheif_rs::ColorSpace::Rgb(libheif_rs::RgbChroma::Rgb),
        None,
    )?;
    let plane = image.planes().interleaved.unwrap();
    let width = plane.width;
    let height = plane.height;
    let stride = plane.stride;

    let rgb_data: Vec<u8> = (0..height)
        .into_par_iter()
        .flat_map_iter(|y| {
            let row_start = y as usize * stride;
            let row_end = row_start + (width * 3) as usize;
            &plane.data[row_start..row_end]
        })
        .cloned()
        .collect();

    let rgb_image =
        RgbImage::from_raw(width, height, rgb_data).ok_or("Failed to create RgbImage")?;
    let rgba_image = DynamicImage::ImageRgb8(rgb_image).to_rgba8();

    Ok(DynamicImage::ImageRgba8(rgba_image))
}
