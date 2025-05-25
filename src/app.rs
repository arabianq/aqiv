use egui::{Image, Key, Pos2, Rect, Sense, Vec2};
use egui_notify::Toasts;
use std::path::PathBuf;
use std::time::Duration;

mod misc;

struct App {
    image_uri: String,
    maintain_aspect_ratio: bool,

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
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default()
            .frame(egui::Frame::new().fill(egui::Color32::BLACK))
            .show(ctx, |ui| {
                self.handle_input(ui, ctx); // Handle all input, including keybindings
                self.render_img(ui); // Render image
                self.toasts.show(ctx); // Show all notifications
            });
    }
}

impl App {
    pub fn new(_cc: &eframe::CreationContext<'_>, img_path: PathBuf) -> Self {
        let image_uri = format!("file://{}", img_path.display());

        Self {
            image_uri,
            maintain_aspect_ratio: false,

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

    fn handle_input(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        ctx.input(|i| {
            // Exit on Escape
            if i.key_pressed(Key::Escape) {
                std::process::exit(0);
            }

            // Maintain Aspect Ratio: true -> false or false -> true
            if i.key_pressed(Key::D) {
                self.maintain_aspect_ratio = !self.maintain_aspect_ratio;
                self.toasts
                    .success(format!(
                        "Maintain Aspect Ratio: {}",
                        self.maintain_aspect_ratio
                    ))
                    .duration(self.notifications_duration);
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
                self.toasts
                    .success(format!("Flip H: {}", self.uv_rect.min.x == 1.0))
                    .duration(self.notifications_duration);
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
                self.toasts
                    .success(format!("Flip V: {}", self.uv_rect.min.y == 1.0))
                    .duration(self.notifications_duration);
            }

            // Rotate image
            if i.key_pressed(Key::R) {
                self.image_rotation += 1;
                if self.image_rotation == 4 {
                    self.image_rotation = 0;
                }
                self.toasts
                    .success(format!("Rotation: {} deg", self.image_rotation as u16 * 90))
                    .duration(self.notifications_duration);
            }

            // Reset image position
            if i.key_pressed(Key::C) {
                self.offset = Vec2::ZERO;
                self.toasts
                    .success("Position Offset: (0.0, 0.0)")
                    .duration(self.notifications_duration);
            }

            // Reset zoom
            if i.key_pressed(Key::X) {
                self.zoom_factor = 1.0;
                self.toasts
                    .success("Zoom Factor: 1.0")
                    .duration(self.notifications_duration);
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

    fn render_img(&mut self, ui: &mut egui::Ui) {
        let window_size = Pos2::new(ui.available_width(), ui.available_height());

        let mut img_rect = misc::calculate_uv_rect(window_size, self.zoom_factor, self.offset);
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
        let full_area_rect =
            Rect::from_min_size(Pos2::ZERO, Vec2::new(window_size.x, window_size.y));
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
}

pub fn run(img_path: PathBuf) -> Result<(), eframe::Error> {
    let initial_window_size = misc::calculate_initial_window_size(&img_path);

    let mut options = eframe::NativeOptions {
        ..Default::default()
    };
    options.vsync = true;
    options.hardware_acceleration = eframe::HardwareAcceleration::Preferred;
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
            egui_extras::install_image_loaders(&cc.egui_ctx);
            Ok(Box::new(App::new(cc, img_path)))
        }),
    )
}
