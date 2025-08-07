mod config;
mod gui;
mod image_utils;
mod input;
mod state;
mod utils;

use config::AppConfig;
use image_utils::{ImageInfo, ImageState, get_image_info};
use state::AppState;
use utils::{calculate_initial_window_size, pathbuf_as_uri};

use eframe::CreationContext;
use egui::{CentralPanel, ColorImage, Context, Frame, Pos2, Rect, Vec2, load::SizedTexture};
use egui_notify::Toasts;

use rayon::prelude::*;
use rfd::FileDialog;
use std::path::PathBuf;
use std::time::Duration;

const SUPPORTED_EXTENSIONS: [&str; 42] = [
    "avif", "bmp", "dds", "ff", "gif", "hdr", "ico", "jpeg", "jpg", "exr", "png", "pnm", "qoi",
    "svg", "tga", "tiff", "webp", "heif", "jxl", "mrw", "arw", "srf", "sr2", "mef", "orf", "srw",
    "erf", "kdc", "dcs", "rw2", "raf", "dcr", "dng", "pef", "crw", "iiq", "3fr", "nrw", "nef",
    "mos", "cr2", "ari",
];
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

                self.handle_input(ui, ctx);

                if !self.image_state.info.path.exists() {
                    if !self.open_image() {
                        std::process::exit(0);
                    }
                }

                if !self.image_state.color_image.is_none() {
                    self.image_state.texture_handle = Some(ctx.load_texture(
                        &self.image_state.uri,
                        self.image_state.color_image.take().unwrap(),
                        Default::default(),
                    ));

                    self.image_state.sized_texture = Some(SizedTexture::from_handle(
                        &self.image_state.texture_handle.as_mut().unwrap(),
                    ));
                    self.image_state.color_image = None;
                }

                self.render_img(ui);

                if let Some(uri) = &self.image_state.uri_to_forget {
                    ctx.forget_image(uri);
                }

                if self.app_state.show_info {
                    self.render_info(ui);
                }

                self.app_state.toasts.show(ctx); // Show all notifications
            });
    }
}

impl App {
    pub fn new(_cc: &CreationContext<'_>, img_info: ImageInfo, color_image: ColorImage) -> Self {
        let cfg = AppConfig::default();

        let image_state = ImageState {
            info: img_info.clone(),

            uri: pathbuf_as_uri(&img_info.path),
            uri_to_forget: None,

            rotation: 0,
            zoom_factor: 1.0,

            uv_rect: Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
            offset: Vec2::ZERO,

            color_image: Some(color_image),
            texture_handle: None,
            sized_texture: None,
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

    fn open_image(&mut self) -> bool {
        let file = FileDialog::new()
            .set_directory(dirs::home_dir().unwrap_or_default())
            .add_filter("image", &SUPPORTED_EXTENSIONS)
            .pick_file();

        if let Some(file) = file {
            self.image_state.load_new_image(&file).ok();
            return true;
        }

        false
    }

    fn next_image(&mut self, step: i128) -> Result<(), Box<dyn std::error::Error>> {
        let current_dir = self.image_state.info.path.parent().unwrap();
        let mut img_files: Vec<PathBuf> = std::fs::read_dir(current_dir)?
            .filter_map(|entry| {
                entry.ok().and_then(|e| {
                    let path = e.path();
                    let ext = path
                        .extension()
                        .and_then(|e| e.to_str())
                        .map(|e| e.to_lowercase())
                        .unwrap_or_default();

                    if SUPPORTED_EXTENSIONS.contains(&ext.as_str()) {
                        Some(path)
                    } else {
                        None
                    }
                })
            })
            .collect();

        img_files.sort_by(|a, b| {
            let a_name = a.file_name().unwrap_or_default();
            let b_name = b.file_name().unwrap_or_default();

            match (a_name.to_str(), b_name.to_str()) {
                (Some(a_str), Some(b_str)) => a_str.cmp(b_str),
                _ => a_name.cmp(b_name), // Compare as OsStr, may cause some issues =P
            }
        });

        let current_file_index = img_files
            .par_iter()
            .position_any(|f| f == &self.image_state.info.path)
            .unwrap() as i128;
        let max_index = img_files.len() as i128 - 1;

        let mut new_file_index = current_file_index + step;
        let mut broken_indexes: Vec<i128> = Vec::new();
        loop {
            if broken_indexes.contains(&new_file_index) {
                self.app_state
                    .notify(String::from("Couldn't open any image in this directory"))
            }

            new_file_index = {
                if new_file_index == -1 {
                    max_index
                } else if new_file_index > max_index {
                    0
                } else {
                    new_file_index
                }
            };

            let new_file_pathbuf = img_files.get(new_file_index as usize).unwrap();
            if self.image_state.load_new_image(new_file_pathbuf)? {
                return Ok(());
            }

            self.app_state.notify(String::from(format!(
                "Couldn't open {}",
                new_file_pathbuf.to_str().unwrap()
            )));

            broken_indexes.push(new_file_index);
            new_file_index += step;
        }
    }
}

pub fn run(img_path: Option<PathBuf>) -> Result<(), Box<dyn std::error::Error>> {
    let initial_window_size: Vec2;
    let img_info: ImageInfo;
    let color_image: ColorImage;

    if let Some(img_path) = img_path {
        (img_info, color_image) = get_image_info(&img_path)?;
        initial_window_size = calculate_initial_window_size(&img_info);
    } else {
        initial_window_size = Vec2::new(600.0, 600.0);
        img_info = ImageInfo::default();
        color_image = ColorImage::default();
    }

    let options = eframe::NativeOptions {
        vsync: true,
        centered: true,
        hardware_acceleration: eframe::HardwareAcceleration::Preferred,

        viewport: egui::ViewportBuilder::default()
            .with_app_id("ru.arabianq.aqiv")
            .with_inner_size(initial_window_size)
            .with_min_inner_size(Vec2::new(200.0, 200.0)),

        ..Default::default()
    };

    match eframe::run_native(
        format!("Quick Image Viewer - {}", img_info.name).as_str(),
        options,
        Box::new(|cc| {
            cc.egui_ctx.options_mut(|options| {
                options.zoom_with_keyboard = true;
                options.reduce_texture_memory = true;
            });

            egui_material_icons::initialize(&cc.egui_ctx);
            egui_extras::install_image_loaders(&cc.egui_ctx);
            Ok(Box::new(App::new(cc, img_info, color_image)))
        }),
    ) {
        Ok(_) => Ok(()),
        Err(e) => Err(e.into()),
    }
}
