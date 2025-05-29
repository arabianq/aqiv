mod misc;

use egui::{
    Align, CentralPanel, Color32, Context, Frame, Image, Key, Layout, Pos2, Rect, RichText, Sense,
    Ui, UiBuilder, Vec2,
};
use egui_notify::Toasts;
use misc::{
    ImageInfo, calculate_initial_window_size, calculate_uv_rect, convert_size, get_image_info,
};
use std::path::PathBuf;
use std::time::Duration;

struct App {
    window_size: Vec2,

    background_color: Color32,

    image_info: ImageInfo,

    image_uri: String,

    maintain_aspect_ratio: bool,
    show_info: bool,

    image_rotation: u8,

    uv_rect: Rect,

    zoom_factor: f32,
    zoom_step: f32,

    offset: Vec2,

    dragging: bool,

    toasts: Toasts,
    notifications_duration: Option<Duration>,
}

impl eframe::App for App {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        CentralPanel::default()
            .frame(Frame::new().fill(self.background_color))
            .show(ctx, |ui| {
                self.window_size = ui.available_size();

                self.handle_input(ui, ctx); // Handle all input, including keybindings
                self.render_img(ui); // Render image

                if self.show_info {
                    self.render_info(ui);
                }

                self.toasts.show(ctx); // Show all notifications
            });
    }
}

impl App {
    pub fn new(_cc: &eframe::CreationContext<'_>, img_path: PathBuf, img_info: ImageInfo) -> Self {
        Self {
            window_size: Vec2::ZERO,

            background_color: Color32::from_hex("#1B1B1B").unwrap_or(Color32::BLACK),

            image_info: img_info,

            image_uri: format!("file://{}", img_path.display()),

            maintain_aspect_ratio: true,
            show_info: false,

            image_rotation: 0,

            uv_rect: Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),

            zoom_factor: 1.0,
            zoom_step: 0.1,

            offset: Vec2::ZERO,

            dragging: false,

            toasts: Toasts::default(),
            notifications_duration: Option::from(Duration::from_millis(500)),
        }
    }

    fn notify(&mut self, message: String) {
        self.toasts
            .basic(message)
            .duration(self.notifications_duration);
    }

    fn handle_input(&mut self, ui: &mut Ui, ctx: &Context) {
        ctx.input(|i| {
            // Exit on Escape
            if i.key_pressed(Key::Escape) {
                std::process::exit(0);
            }

            // Maintain Aspect Ratio: true -> false or false -> true
            if i.key_pressed(Key::D) {
                self.maintain_aspect_ratio = !self.maintain_aspect_ratio;
                self.notify(format!(
                    "Maintain Aspect Ratio: {}",
                    self.maintain_aspect_ratio
                ));
            }

            // Show info: true -> false or false -> true
            if i.key_pressed(Key::I) {
                self.show_info = !self.show_info;
                self.notify(format!("Show info: {}", self.show_info));
            }

            // Flip H
            if i.key_pressed(Key::H) {
                if self.uv_rect.min.x == 0.0 {
                    self.uv_rect.min.x = 1.0;
                    self.uv_rect.max.x = 0.0;
                } else {
                    self.uv_rect.min.x = 0.0;
                    self.uv_rect.max.x = 1.0;
                }
                self.notify(format!("Flip H: {}", self.uv_rect.min.x == 1.0));
            }

            //Flip V
            if i.key_pressed(Key::V) {
                if self.uv_rect.min.y == 0.0 {
                    self.uv_rect.min.y = 1.0;
                    self.uv_rect.max.y = 0.0;
                } else {
                    self.uv_rect.min.y = 0.0;
                    self.uv_rect.max.y = 1.0;
                }
                self.notify(format!("Flip V: {}", self.uv_rect.min.y == 1.0));
            }

            // Rotate image
            if i.key_pressed(Key::R) {
                self.image_rotation += 1;
                if self.image_rotation == 4 {
                    self.image_rotation = 0;
                }
                self.notify(format!("Rotation: {} deg", self.image_rotation as u16 * 90));
            }

            // Reset image position
            if i.key_pressed(Key::C) {
                self.offset = Vec2::ZERO;
                self.notify(String::from("Position Offset: (0.0, 0.0)"));
            }

            // Reset zoom
            if i.key_pressed(Key::X) {
                self.zoom_factor = 1.0;
                self.notify(String::from("Zoom Factor: 1.0"));
            }

            // Mouse wheel zoom
            let scroll = i.raw_scroll_delta.y;
            if scroll != 0.0 {
                let old_zoom = self.zoom_factor;
                let new_zoom =
                    (old_zoom * (1.0 + scroll.signum() * self.zoom_step)).clamp(0.1, 10.0);

                if let Some(mouse_pos) = i.pointer.interact_pos() {
                    let window_center = (ui.available_size() / 2.0).to_pos2();
                    let delta = mouse_pos - window_center;
                    self.offset += delta * (1.0 / new_zoom - 1.0 / old_zoom);
                }

                self.zoom_factor = new_zoom;
            }

            // Zoom in using W
            if i.key_pressed(Key::W) && i.raw_scroll_delta.y == 0.0 {
                self.zoom_factor += self.zoom_step * self.zoom_factor;
            }

            // Zoom out using S
            if i.key_pressed(Key::S) && i.raw_scroll_delta.y == 0.0 {
                self.zoom_factor -= self.zoom_step * self.zoom_factor;
            }

            // Clamp zoom factor
            self.zoom_factor = self.zoom_factor.clamp(0.1, 10.0);

            // Is mouse left down
            self.dragging = i.pointer.primary_down();
        });
    }

    fn render_img(&mut self, ui: &mut Ui) {
        let mut img_rect =
            calculate_uv_rect(self.window_size.to_pos2(), self.zoom_factor, self.offset);
        let mut img_size = img_rect.size();

        if [1u8, 3u8].contains(&self.image_rotation) {
            img_size = Vec2::new(img_size.y, img_size.x);
            img_rect = Rect::from_center_size(img_rect.center(), img_size);
        }

        let img = Image::new(&self.image_uri)
            .maintain_aspect_ratio(self.maintain_aspect_ratio)
            .fit_to_exact_size(img_size)
            .uv(self.uv_rect)
            .rotate(
                self.image_rotation as f32 * std::f32::consts::PI / 2.0,
                Vec2::splat(0.5),
            );

        // Creating full screen area to handle dragging
        let full_area_rect = Rect::from_min_size(
            Pos2::ZERO,
            Vec2::new(self.window_size.x, self.window_size.y),
        );
        let full_area_response = ui.allocate_rect(full_area_rect, Sense::drag());

        // Handle dragging
        if full_area_response.dragged() && self.dragging {
            let delta = full_area_response.drag_motion();
            self.offset += delta / self.zoom_factor;
            self.offset.x = self.offset.x.clamp(-500.0, 500.0);
            self.offset.y = self.offset.y.clamp(-500.0, 500.0);
        }

        // Show image
        ui.put(img_rect, img);
    }

    fn render_info(&mut self, ui: &mut Ui) {
        let info_rect = Rect::from_min_max(
            Pos2::new(0.0, self.window_size.y - 100.0),
            Pos2::new(self.window_size.x, self.window_size.y),
        );

        let info_text = RichText::new(format!(
            "Name: {}\nFormat: {}\nFile Size: {}\nResolution: {}\nPath: {}",
            self.image_info.name,
            self.image_info.format,
            convert_size(self.image_info.size as f64),
            format!(
                "{}x{}",
                self.image_info.resolution.0, self.image_info.resolution.1
            ),
            self.image_info.path.display()
        ))
        .color(Color32::WHITE);

        ui.allocate_new_ui(UiBuilder::new().max_rect(info_rect), |ui| {
            Frame::new()
                .fill(self.background_color)
                .multiply_with_opacity(0.95)
                .corner_radius(15.0)
                .inner_margin(10)
                .outer_margin(5)
                .show(ui, |ui| {
                    let layout = Layout::left_to_right(Align::Min);
                    ui.with_layout(layout, |ui| {
                        ui.label(info_text);
                    });
                });
        });
    }
}

pub fn run(img_path: PathBuf) -> Result<(), eframe::Error> {
    let initial_window_size = calculate_initial_window_size(&img_path);
    let img_info = get_image_info(&img_path);

    let mut options = eframe::NativeOptions {
        ..Default::default()
    };
    options.vsync = true;
    options.hardware_acceleration = eframe::HardwareAcceleration::Preferred;
    options.centered = true;
    options.viewport.app_id = Option::from("aqiv".to_string());
    options.viewport.inner_size = Option::from(initial_window_size);
    options.viewport.min_inner_size = Option::from(Vec2::new(200.0, 200.0));

    eframe::run_native(
        format!(
            "Quick Image Viewer - {}",
            img_path.file_name().unwrap().display()
        )
        .as_str(),
        options,
        Box::new(|cc| {
            cc.egui_ctx.options_mut(|options| {
                options.zoom_with_keyboard = true;
                options.reduce_texture_memory = true;
            });

            egui_extras::install_image_loaders(&cc.egui_ctx);
            Ok(Box::new(App::new(cc, img_path, img_info)))
        }),
    )
}
