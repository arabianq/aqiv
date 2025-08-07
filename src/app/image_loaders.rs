use image::{DynamicImage, RgbImage, RgbaImage};
use jpegxl_rs::image::ToDynamic;
use libheif_rs::{HeifContext, LibHeif};
use rayon::prelude::*;
use std::error::Error;

pub fn load_image_default(buf: &Vec<u8>) -> Result<DynamicImage, Box<dyn Error>> {
    let rgba_image = image::load_from_memory(buf)?.to_rgba8();
    Ok(DynamicImage::ImageRgba8(rgba_image))
}

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

pub fn load_image_jpegxl(buf: &Vec<u8>) -> Result<DynamicImage, Box<dyn Error>> {
    let decoder = jpegxl_rs::decoder_builder().build()?;
    let image = decoder.decode_to_image(buf)?.unwrap();
    let rgba_image = image.to_rgba8();

    Ok(DynamicImage::ImageRgba8(rgba_image))
}

pub fn load_image_raw(buf: &Vec<u8>) -> Result<DynamicImage, Box<dyn Error>> {
    // I still don't know does it actually works good or not

    let mut cursor = std::io::Cursor::new(buf);
    let raw_image = rawloader::decode(&mut cursor)?;

    let width = raw_image.width;
    let height = raw_image.height;

    let raw_data = match raw_image.data {
        rawloader::RawImageData::Integer(d) => d,
        rawloader::RawImageData::Float(d) => {
            let scale = (1 << 16) as f32;
            d.par_iter().map(|&f| (f * scale) as u16).collect()
        }
    };

    let mono8: Vec<u8> = (0..width * height)
        .into_par_iter()
        .map(|i| {
            let pix = raw_data[i];
            let x = i % width;
            let y = i / width;

            let c = raw_image.cfa.color_at(y, x);

            let bl = raw_image.blacklevels[c] as f32;
            let wl = raw_image.whitelevels[c] as f32;

            ((pix as f32 - bl) / (wl - bl) * 255.0).clamp(0.0, 255.0) as u8
        })
        .collect();

    let mut rgb8 = vec![0u8; width * height * 3];
    {
        let mut dst = bayer::RasterMut::new(width, height, bayer::RasterDepth::Depth8, &mut rgb8);

        let mut rdr = std::io::Cursor::new(&mono8);
        bayer::run_demosaic(
            &mut rdr,
            bayer::BayerDepth::Depth8,
            match raw_image.cfa.name.as_str() {
                "RGGB" => bayer::CFA::RGGB,
                "GRBG" => bayer::CFA::GRBG,
                "BGGR" => bayer::CFA::BGGR,
                "GBRG" => bayer::CFA::GBRG,
                _ => return Err("Unsupported CFA".into()),
            },
            bayer::Demosaic::NearestNeighbour,
            &mut dst,
        )?;
    }

    let mut rgb_image = RgbImage::from_raw(width as u32, height as u32, rgb8.clone())
        .ok_or("Failed to create RgbImage")?;

    // Calculate white balance
    let wb_coeffs = match raw_image.wb_coeffs {
        coeffs if coeffs.par_iter().all(|&c| !c.is_nan()) => coeffs,
        _ => {
            let (red_sum, green_sum, blue_sum, red_count, green_count, blue_count): (
                f32,
                f32,
                f32,
                usize,
                usize,
                usize,
            ) = mono8
                .par_iter()
                .enumerate()
                .map(|(i, &v)| {
                    let x = i % width;
                    let y = i / width;
                    let c = raw_image.cfa.color_at(y, x);

                    match c {
                        0 => (v as f32, 0.0, 0.0, 1, 0, 0), // red
                        1 => (0.0, v as f32, 0.0, 0, 1, 0), // green
                        2 => (0.0, 0.0, v as f32, 0, 0, 1), // blue
                        _ => (0.0, 0.0, 0.0, 0, 0, 0),      // no color
                    }
                })
                .reduce(
                    || (0.0, 0.0, 0.0, 0, 0, 0),
                    |(
                        red_sum_a,
                        green_sum_a,
                        blue_sum_a,
                        red_count_a,
                        green_count_a,
                        blue_count_a,
                    ),
                     (
                        red_sum_b,
                        green_sum_b,
                        blue_sum_b,
                        red_count_b,
                        green_count_b,
                        blue_count_b,
                    )| {
                        (
                            red_sum_a + red_sum_b,
                            green_sum_a + green_sum_b,
                            blue_sum_a + blue_sum_b,
                            red_count_a + red_count_b,
                            green_count_a + green_count_b,
                            blue_count_a + blue_count_b,
                        )
                    },
                );

            let red_avg = red_sum / red_count as f32;
            let green_avg = green_sum / green_count as f32;
            let blue_avg = blue_sum / blue_count as f32;

            let red_coeff = green_avg / red_avg;
            let green_coeff = 1.0;
            let blue_coeff = green_avg / blue_avg;

            [red_coeff, green_coeff, blue_coeff, green_coeff]
        }
    };

    // Apply white balance
    rgb_image.as_mut().par_chunks_mut(3).for_each(|pixel| {
        pixel[0] = (pixel[0] as f32 * wb_coeffs[0]).clamp(0.0, 255.0) as u8;
        pixel[1] = (pixel[1] as f32 * wb_coeffs[1]).clamp(0.0, 255.0) as u8;
        pixel[2] = (pixel[2] as f32 * wb_coeffs[2]).clamp(0.0, 255.0) as u8;
    });

    let mut dynamic_image = DynamicImage::ImageRgb8(rgb_image);
    dynamic_image = match raw_image.orientation {
        rawloader::Orientation::Rotate90 => dynamic_image.rotate90(),
        rawloader::Orientation::Rotate180 => dynamic_image.rotate180(),
        rawloader::Orientation::Rotate270 => dynamic_image.rotate270(),
        _ => dynamic_image,
    };

    let rgba_image = dynamic_image.to_rgba8();

    Ok(DynamicImage::ImageRgba8(rgba_image))
}
