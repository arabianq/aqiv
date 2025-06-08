mod config;
mod misc;

use config::AppConfig;
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

struct ImageState {
    info: ImageInfo,

    uri: String,

    rotation: u8,
    zoom_factor: f32,

    uv_rect: Rect,
    offset: Vec2,
}

struct AppState {
    window_size: Vec2,
    background_color: Color32,

    maintain_aspect_ratio: bool,
    show_info: bool,
    dragging: bool,

    toasts: Toasts,
    notification_duration: Option<Duration>,
}

struct App {
    app_state: AppState,
    image_state: ImageState,
}

impl eframe::App for App {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        CentralPanel::default()
            .frame(Frame::new().fill(self.app_state.background_color))
            .show(ctx, |ui| {
                self.app_state.window_size = ui.available_size();

                self.handle_input(ui, ctx); // Handle all input, including keybindings
                self.render_img(ui); // Render image

                if self.app_state.show_info {
                    self.render_info(ui);
                }

                self.app_state.toasts.show(ctx); // Show all notifications
            });
    }
}

impl App {
    pub fn new(_cc: &eframe::CreationContext<'_>, img_path: PathBuf, img_info: ImageInfo) -> Self {
        let cfg = AppConfig::default();

        let image_state = ImageState {
            info: img_info,

            uri: format!("file://{}", img_path.display()),

            rotation: 0,
            zoom_factor: 1.0,

            uv_rect: Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
            offset: Vec2::ZERO,
        };

        let app_state = AppState {
            window_size: Vec2::ZERO,
            background_color: cfg.background_color,

            maintain_aspect_ratio: cfg.maintain_aspect_ratio,
            show_info: cfg.show_info,
            dragging: false,

            toasts: Toasts::default(),
            notification_duration: Option::from(Duration::from_millis(
                cfg.notification_duration_millis,
            )),
        };

        Self {
            app_state,
            image_state,
        }
    }

    fn notify(&mut self, message: String) {
        self.app_state
            .toasts
            .basic(message)
            .duration(self.app_state.notification_duration);
    }

    fn toggle_maintain_aspect_ratio(&mut self) {
        self.app_state.maintain_aspect_ratio = !self.app_state.maintain_aspect_ratio;
        self.notify(format!(
            "Maintain Aspect Ratio: {}",
            self.app_state.maintain_aspect_ratio
        ));
    }

    fn toggle_show_info(&mut self) {
        self.app_state.show_info = !self.app_state.show_info;
        self.notify(format!("Show info: {}", self.app_state.show_info));
    }

    fn flip_horizontal(&mut self) {
        if self.image_state.uv_rect.min.x == 0.0 {
            self.image_state.uv_rect.min.x = 1.0;
            self.image_state.uv_rect.max.x = 0.0;
        } else {
            self.image_state.uv_rect.min.x = 0.0;
            self.image_state.uv_rect.max.x = 1.0;
        }
        self.notify(format!("Flip H: {}", self.image_state.uv_rect.min.x == 1.0));
    }

    fn flip_vertical(&mut self) {
        if self.image_state.uv_rect.min.y == 0.0 {
            self.image_state.uv_rect.min.y = 1.0;
            self.image_state.uv_rect.max.y = 0.0;
        } else {
            self.image_state.uv_rect.min.y = 0.0;
            self.image_state.uv_rect.max.y = 1.0;
        }
        self.notify(format!("Flip V: {}", self.image_state.uv_rect.min.y == 1.0));
    }

    fn rotate_image(&mut self) {
        self.image_state.rotation += 1;
        if self.image_state.rotation == 4 {
            self.image_state.rotation = 0;
        }
        self.notify(format!(
            "Rotation: {} deg",
            self.image_state.rotation as u16 * 90
        ));
    }

    fn reset_offset(&mut self) {
        self.image_state.offset = Vec2::ZERO;
        self.notify(String::from("Position Offset: (0.0, 0.0)"));
    }

    fn reset_zoom(&mut self) {
        self.image_state.zoom_factor = 1.0;
        self.notify(String::from("Zoom Factor: 1.0"));
    }

    fn handle_input(&mut self, ui: &mut Ui, ctx: &Context) {
        ctx.input(|i| {
            // Exit on Escape
            if i.key_pressed(Key::Escape) {
                std::process::exit(0);
            }

            // Maintain Aspect Ratio on D
            if i.key_pressed(Key::D) {
                self.toggle_maintain_aspect_ratio();
            }

            // Show Info on I
            if i.key_pressed(Key::I) {
                self.toggle_show_info();
            }

            // Horizontal Flip on H
            if i.key_pressed(Key::H) {
                self.flip_horizontal();
            }

            // Vertical Flip on V
            if i.key_pressed(Key::V) {
                self.flip_vertical();
            }

            // Image rotation on R
            if i.key_pressed(Key::R) {
                self.rotate_image();
            }

            // Reset offset on C
            if i.key_pressed(Key::C) {
                self.reset_offset();
            }

            // Reset zoom on X
            if i.key_pressed(Key::X) {
                self.reset_zoom();
            }

            // Zoom handler
            let scroll = i.raw_scroll_delta.y;
            if scroll != 0.0 {
                let old_zoom = self.image_state.zoom_factor;
                let new_zoom = (old_zoom * (1.0 + scroll.signum() * 0.1)).clamp(0.1, 10.0);

                if let Some(mouse_pos) = i.pointer.interact_pos() {
                    let window_center = (ui.available_size() / 2.0).to_pos2();
                    let delta = mouse_pos - window_center;
                    self.image_state.offset += delta * (1.0 / new_zoom - 1.0 / old_zoom);
                }

                self.image_state.zoom_factor = new_zoom;
            }

            // Zoom in on W
            if i.key_pressed(Key::W) && i.raw_scroll_delta.y == 0.0 {
                self.image_state.zoom_factor += 0.1 * self.image_state.zoom_factor;
            }

            // Zoom out on S
            if i.key_pressed(Key::S) && i.raw_scroll_delta.y == 0.0 {
                self.image_state.zoom_factor -= 0.1 * self.image_state.zoom_factor;
            }

            self.image_state.zoom_factor = self.image_state.zoom_factor.clamp(0.1, 10.0);

            self.app_state.dragging = i.pointer.primary_down();
        });
    }

    fn render_img(&mut self, ui: &mut Ui) {
        let mut img_rect = calculate_uv_rect(
            self.app_state.window_size.to_pos2(),
            self.image_state.zoom_factor,
            self.image_state.offset,
        );
        let mut img_size = img_rect.size();

        if [1u8, 3u8].contains(&self.image_state.rotation) {
            img_size = Vec2::new(img_size.y, img_size.x);
            img_rect = Rect::from_center_size(img_rect.center(), img_size);
        }

        let img = Image::new(&self.image_state.uri)
            .maintain_aspect_ratio(self.app_state.maintain_aspect_ratio)
            .fit_to_exact_size(img_size)
            .uv(self.image_state.uv_rect)
            .rotate(
                self.image_state.rotation as f32 * std::f32::consts::PI / 2.0,
                Vec2::splat(0.5),
            );

        // Creating full screen area to handle dragging
        let full_area_rect = Rect::from_min_size(
            Pos2::ZERO,
            Vec2::new(self.app_state.window_size.x, self.app_state.window_size.y),
        );
        let full_area_response = ui.allocate_rect(full_area_rect, Sense::drag());

        // Handle dragging
        if full_area_response.dragged() && self.app_state.dragging {
            let delta = full_area_response.drag_motion();
            self.image_state.offset += delta / self.image_state.zoom_factor;
            self.image_state.offset.x = self.image_state.offset.x.clamp(-500.0, 500.0);
            self.image_state.offset.y = self.image_state.offset.y.clamp(-500.0, 500.0);
        }

        // Show image
        ui.put(img_rect, img);
    }

    fn render_info(&mut self, ui: &mut Ui) {
        let info_rect = Rect::from_min_max(
            Pos2::new(0.0, self.app_state.window_size.y - 100.0),
            Pos2::new(self.app_state.window_size.x, self.app_state.window_size.y),
        );

        let info_text = RichText::new(format!(
            "Name: {}\nFormat: {}\nFile Size: {}\nResolution: {}\nPath: {}",
            self.image_state.info.name,
            self.image_state.info.format,
            convert_size(self.image_state.info.size as f64),
            format!(
                "{}x{}",
                self.image_state.info.resolution.0, self.image_state.info.resolution.1
            ),
            self.image_state.info.path.display()
        ))
        .color(Color32::WHITE);

        ui.allocate_new_ui(UiBuilder::new().max_rect(info_rect), |ui| {
            Frame::new()
                .fill(self.app_state.background_color)
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

    let options = eframe::NativeOptions {
        vsync: true,
        centered: true,
        hardware_acceleration: eframe::HardwareAcceleration::Preferred,
        
        viewport: egui::ViewportBuilder::default()
            .with_app_id("aqiv")
            .with_inner_size(initial_window_size)
            .with_min_inner_size(Vec2::new(200.0, 200.0)),
        
        ..Default::default()
    };

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
