use egui::{ColorImage, Pos2, Rect, Vec2};
use image::{GenericImageView, ImageFormat, ImageReader};
use std::fmt::Write;
use std::fs::read_to_string;
use std::path::{PathBuf, absolute};

pub struct ImageInfo {
    pub path: PathBuf,

    pub name: String,
    pub format: String,

    pub size: u64,
    pub resolution: Option<(u32, u32)>,
}

impl Default for ImageInfo {
    fn default() -> Self {
        ImageInfo {
            path: PathBuf::new(),

            name: String::new(),
            format: String::from("Unknown"),

            size: 0,
            resolution: None,
        }
    }
}

impl Clone for ImageInfo {
    fn clone(&self) -> Self {
        ImageInfo {
            path: self.path.clone(),

            name: self.name.clone(),
            format: self.format.clone(),

            size: self.size,
            resolution: self.resolution,
        }
    }
}

pub fn get_image(
    img_path: &PathBuf,
) -> Result<(ImageInfo, ColorImage), Box<dyn std::error::Error>> {
    let img_path = absolute(img_path)?;
    let extension = img_path.extension().unwrap_or_default();

    if extension == "svg" {
        let svg_data = read_to_string(&img_path)?;
        let usvg_tree = usvg::Tree::from_str(&svg_data, &usvg::Options::default())?;

        let og_size = usvg_tree.size().to_int_size();
        let og_width = og_size.width();
        let og_height = og_size.height();

        let scaled_width = og_width * 4;
        let scaled_height = og_height * 4;

        let mut pixmap = resvg::tiny_skia::Pixmap::new(scaled_width, scaled_height).unwrap();
        let transform = resvg::tiny_skia::Transform::from_scale(4.0, 4.0);
        resvg::render(&usvg_tree, transform, &mut pixmap.as_mut());

        let image_bytes = pixmap.data().iter().as_slice().to_vec();

        let color_image = ColorImage::from_rgba_unmultiplied(
            [scaled_width as usize, scaled_height as usize],
            &image_bytes,
        );

        Ok((
            ImageInfo {
                path: img_path.clone(),
                name: img_path
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string(),
                format: "Svg".to_string(),
                size: image_bytes.len() as u64,
                resolution: Some((og_width, og_height)),
            },
            color_image,
        ))
    } else {
        let reader = ImageReader::open(&img_path)?;
        let image_format = reader.format().unwrap_or(ImageFormat::Png);
        let image = reader.decode()?;
        let image_resolution = image.dimensions();
        let image_bytes = image.as_bytes().to_vec();

        let color_image = ColorImage::from_rgb(
            [image_resolution.0 as usize, image_resolution.1 as usize],
            &image_bytes,
        );

        Ok((
            ImageInfo {
                path: img_path.clone(),
                name: img_path
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string(),
                format: format!("{:?}", image_format),
                size: image_bytes.len() as u64,
                resolution: Some(image_resolution),
            },
            color_image,
        ))
    }
}

pub fn calculate_uv_rect(window_size: Pos2, zoom_factor: f32, offset: Vec2) -> Rect {
    let window_center = window_size / 2.0;

    let zoomed_width = window_size.x * zoom_factor;
    let zoomed_height = window_size.y * zoom_factor;

    let img_center_x =
        window_center.x - (window_center.x - window_size.x / 2.0 - offset.x) * zoom_factor;
    let img_center_y =
        window_center.y - (window_center.y - window_size.y / 2.0 - offset.y) * zoom_factor;

    let x_min = img_center_x - zoomed_width / 2.0;
    let x_max = img_center_x + zoomed_width / 2.0;
    let y_min = img_center_y - zoomed_height / 2.0;
    let y_max = img_center_y + zoomed_height / 2.0;

    Rect::from_min_max(Pos2::new(x_min, y_min), Pos2::new(x_max, y_max))
}

pub fn calculate_initial_window_size(img_info: &ImageInfo) -> Vec2 {
    let screen_size = screen_size::get_primary_screen_size().unwrap_or_default();
    let screen_size_vec = Vec2::new(screen_size.0 as f32, screen_size.1 as f32);

    let img_size = img_info.resolution;

    if let Some((img_width, img_height)) = img_size {
        let mut initial_window_size = Vec2::new(img_width as f32, img_height as f32);

        if initial_window_size.x > screen_size_vec.x {
            initial_window_size.y =
                initial_window_size.y * (screen_size_vec.x / initial_window_size.x);
            initial_window_size.x = screen_size_vec.x;
        }
        if initial_window_size.y > screen_size_vec.y {
            initial_window_size.x =
                initial_window_size.x * (screen_size_vec.y / initial_window_size.y);
            initial_window_size.y = screen_size_vec.y;
        }

        initial_window_size
    } else {
        Vec2::new(screen_size.0 as f32, screen_size.1 as f32)
    }
}

pub fn convert_size(size_bytes: f64) -> String {
    if size_bytes <= 0.0 {
        return "-".to_string();
    }

    const UNITS: [&str; 4] = ["B", "KB", "MB", "GB"];

    let i = if size_bytes < 1.0 {
        0
    } else {
        size_bytes.log(1024.0).floor() as usize
    };

    let i = i.min(UNITS.len().saturating_sub(1));

    let p = 1024_f64.powf(i as f64);
    let s = (size_bytes / p).round();

    let mut buffer = String::with_capacity(10);
    write!(&mut buffer, "{:.2} {}", s / 100.0, UNITS[i]).unwrap();

    buffer
}

pub fn pathbuf_as_uri(path_buf: &PathBuf) -> String {
    let path_str = path_buf.to_str().unwrap_or_default();
    format!("file://{}", path_str)
}
