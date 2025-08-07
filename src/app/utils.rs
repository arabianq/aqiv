use crate::app::image_utils::ImageInfo;

use egui::{Pos2, Rect, Vec2};

use std::fmt::Write;
use std::path::PathBuf;

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
    let s = size_bytes / p;

    let mut buffer = String::with_capacity(10);
    write!(&mut buffer, "{:.2} {}", s, UNITS[i]).unwrap();

    buffer
}

pub fn pathbuf_as_uri(path_buf: &PathBuf) -> String {
    let path_str = path_buf.to_str().unwrap_or_default();
    format!("file://{}", path_str)
}
