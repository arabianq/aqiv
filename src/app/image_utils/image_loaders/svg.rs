use image::{DynamicImage, RgbaImage};

use std::error::Error;

pub fn load_image_svg(buf: &Vec<u8>) -> Result<DynamicImage, Box<dyn Error>> {
    let usvg_tree = usvg::Tree::from_data(buf, &usvg::Options::default())?;

    let og_size = usvg_tree.size().to_int_size();
    let width = og_size.width();
    let height = og_size.height();

    let mut pixmap = resvg::tiny_skia::Pixmap::new(width, height).unwrap();
    let transform = resvg::tiny_skia::Transform::default();
    resvg::render(&usvg_tree, transform, &mut pixmap.as_mut());

    let image_data = pixmap.data();
    let rgba_image = RgbaImage::from_raw(width, height, image_data.to_vec())
        .ok_or("Failed to create RgbaImage")?;

    Ok(DynamicImage::ImageRgba8(rgba_image))
}
