use egui::{Pos2, Rect, Vec2};
use image::ImageReader;
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

pub fn calculate_initial_window_size(img_path: &PathBuf) -> Vec2 {
    let img_size = ImageReader::open(img_path)
        .unwrap()
        .with_guessed_format()
        .unwrap()
        .into_dimensions()
        .unwrap();

    let screen_size = screen_size::get_primary_screen_size().unwrap();
    let screen_size_vec = Vec2::new(screen_size.0 as f32, screen_size.1 as f32);

    let mut initial_window_size = Vec2::new(img_size.0 as f32, img_size.1 as f32);
    if initial_window_size.x > screen_size_vec.x {
        initial_window_size.y = initial_window_size.y * (screen_size_vec.x / initial_window_size.x);
        initial_window_size.x = screen_size_vec.x;
    }
    if initial_window_size.y > screen_size_vec.y {
        initial_window_size.x = initial_window_size.x * (screen_size_vec.y / initial_window_size.y);
        initial_window_size.y = screen_size_vec.y;
    }

    initial_window_size
}
