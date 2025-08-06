mod config;
mod image_loaders;
mod misc;

use clipboard_rs::{Clipboard, ClipboardContext};
use config::AppConfig;
use egui::load::SizedTexture;
use egui::{
    Align, CentralPanel, Color32, ColorImage, Context, Frame, Image, Key, Layout, Pos2, Rect,
    RichText, Sense, TextureHandle, Ui, UiBuilder, Vec2,
};
use egui_material_icons::icons;
use egui_notify::Toasts;
use misc::{
    ImageInfo, calculate_initial_window_size, calculate_uv_rect, convert_size, get_image,
    pathbuf_as_uri,
};
use rfd::FileDialog;
use std::path::PathBuf;
use std::time::Duration;
use wl_clipboard_rs::copy::{
    MimeType as ClipboardMimeType, Options as ClipboardOptions, Source as ClipboardSource,
};

const SUPPORTED_EXTENSIONS: [&str; 17] = [
    "avif", "bmp", "dds", "ff", "gif", "hdr", "ico", "jpeg", "jpg", "exr", "png", "pnm", "qoi",
    "svg", "tga", "tiff", "webp",
];

struct ImageState {
    info: ImageInfo,

    uri: String,
    uri_to_forget: Option<String>,

    rotation: u8,
    zoom_factor: f32,

    uv_rect: Rect,
    offset: Vec2,

    color_image: Option<ColorImage>,
    texture_handle: Option<TextureHandle>,
    sized_texture: Option<SizedTexture>,
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
    pub fn new(
        _cc: &eframe::CreationContext<'_>,
        img_info: ImageInfo,
        color_image: ColorImage,
    ) -> Self {
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

    fn load_new_image(&mut self, path: &PathBuf) -> Result<bool, Box<dyn std::error::Error>> {
        let (new_img_info, new_color_image) = get_image(&path)?;

        if new_img_info.resolution.is_none() {
            return Ok(false);
        }

        self.image_state.uri_to_forget = Some(self.image_state.uri.clone());

        self.image_state.info = new_img_info;
        self.image_state.uri = pathbuf_as_uri(&self.image_state.info.path);
        self.image_state.zoom_factor = 1.0;
        self.image_state.rotation = 0;
        self.image_state.color_image = Some(new_color_image);

        Ok(true)
    }

    fn open_image(&mut self) -> bool {
        let file = FileDialog::new()
            .set_directory(dirs::home_dir().unwrap_or_default())
            .add_filter("image", &SUPPORTED_EXTENSIONS)
            .pick_file();

        if let Some(file) = file {
            self.load_new_image(&file).ok();
            return true;
        }

        false
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

    fn next_image(&mut self, step: i128) -> Result<(), Box<dyn std::error::Error>> {
        let current_dir = self.image_state.info.path.parent().unwrap();
        let all_files = std::fs::read_dir(current_dir)?;
        let mut img_files: Vec<PathBuf> = Vec::new();
        for file in all_files {
            let file = file?;
            let file_path = file.path();
            let extension = file_path.extension().unwrap_or_default();

            if !SUPPORTED_EXTENSIONS.contains(&extension.to_str().unwrap()) {
                continue;
            }

            img_files.push(file_path);
        }

        img_files.sort_by(|a, b| {
            let a_name = a.file_name().unwrap_or_default();
            let b_name = b.file_name().unwrap_or_default();

            match (a_name.to_str(), b_name.to_str()) {
                (Some(a_str), Some(b_str)) => a_str.cmp(b_str),
                _ => a_name.cmp(b_name), // Compare as OsStr, may cause some issues =P
            }
        });

        let current_file_index = img_files
            .iter()
            .position(|f| f == &self.image_state.info.path)
            .unwrap() as i128;
        let max_index = img_files.len() as i128 - 1;

        let mut new_file_index = current_file_index + step;
        let mut broken_indexes: Vec<i128> = Vec::new();
        loop {
            if broken_indexes.contains(&new_file_index) {
                self.notify(String::from("Couldn't open any image in this directory"))
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
            if self.load_new_image(new_file_pathbuf)? {
                return Ok(());
            }

            self.notify(String::from(format!(
                "Couldn't open {}",
                new_file_pathbuf.to_str().unwrap()
            )));

            broken_indexes.push(new_file_index);
            new_file_index += step;
        }
    }

    fn copy_path_to_clipboard(&mut self) {
        let clipboard_ctx = ClipboardContext::new().unwrap();
        clipboard_ctx
            .set_text(self.image_state.info.path.to_string_lossy().to_string())
            .unwrap();

        // Clipboard-rs does not support wayland, so I have to use wl-clipboard-rs in addition to it
        // BTW I don't know how will it work in xorg session =P
        {
            let opts = ClipboardOptions::new();
            opts.copy(
                ClipboardSource::Bytes(
                    self.image_state
                        .info
                        .path
                        .to_string_lossy()
                        .as_bytes()
                        .into(),
                ),
                ClipboardMimeType::Specific(String::from("text/plain;charset=utf-8")),
            )
            .ok();
        }

        self.notify(String::from("Path was copied to clipboard"));
    }

    fn copy_uri_to_clipboard(&mut self) {
        let clipboard_ctx = ClipboardContext::new().unwrap();

        clipboard_ctx
            .set_files(vec![self.image_state.uri.to_string()])
            .ok();

        // For wayland
        {
            let opts = ClipboardOptions::new();
            opts.copy(
                ClipboardSource::Bytes(self.image_state.uri.as_bytes().into()),
                ClipboardMimeType::Specific(String::from("text/uri-list")),
            )
            .ok();
        }

        self.notify(String::from("Image was copied to clipboard"));
    }

    fn handle_input(&mut self, ui: &mut Ui, ctx: &Context) {
        ctx.input(|i| {
            // Exit on Escape
            if i.key_pressed(Key::Escape) {
                std::process::exit(0);
            }

            // Open Image on O
            if i.key_pressed(Key::O) {
                self.open_image();
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

            if i.key_pressed(Key::ArrowRight) {
                self.next_image(1).ok();
            } else if i.key_pressed(Key::ArrowLeft) {
                self.next_image(-1).ok();
            }

            if i.events.contains(&egui::Event::Copy) {
                match i.modifiers.shift {
                    true => self.copy_path_to_clipboard(),
                    false => self.copy_uri_to_clipboard(),
                }
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
        // Creating full screen area to handle dragging
        let full_area_rect = Rect::from_min_size(
            Pos2::ZERO,
            Vec2::new(self.app_state.window_size.x, self.app_state.window_size.y),
        );
        let full_area_response = ui.allocate_rect(full_area_rect, Sense::click_and_drag());

        // Render Context Menu (only visible after right click)
        full_area_response.context_menu(|ui| self.render_context_menu(ui));

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

        let img = Image::from_texture(self.image_state.sized_texture.unwrap())
            .show_loading_spinner(false)
            .alt_text("Failed to load image =(")
            .maintain_aspect_ratio(self.app_state.maintain_aspect_ratio)
            .fit_to_exact_size(img_size)
            .uv(self.image_state.uv_rect)
            .rotate(
                self.image_state.rotation as f32 * std::f32::consts::PI / 2.0,
                Vec2::splat(0.5),
            );

        // Handle dragging
        if full_area_response.dragged() && self.app_state.dragging {
            ui.ctx().set_cursor_icon(egui::CursorIcon::Grabbing);

            let delta = full_area_response.drag_delta();

            self.image_state.offset += delta / self.image_state.zoom_factor;
            self.image_state.offset.x = self.image_state.offset.x.clamp(-500.0, 500.0);
            self.image_state.offset.y = self.image_state.offset.y.clamp(-500.0, 500.0);
        }

        // Show image
        ui.put(img_rect, img);
    }

    fn render_context_menu(&mut self, ui: &mut Ui) {
        ui.set_max_width(170.0);

        let open_button = ui.button(format!("{} {}", icons::ICON_FILE_OPEN, "Open image [O] "));
        if open_button.clicked() {
            ui.close();
            self.open_image();
        }

        let copy_uri_button = ui.button(format!(
            "{} {}",
            icons::ICON_FILE_COPY,
            "Copy image [Ctrl + C]"
        ));
        if copy_uri_button.clicked() {
            ui.close();
            self.copy_uri_to_clipboard();
        }

        let copy_path_button = ui.button(format!(
            "{} {}",
            icons::ICON_FILE_COPY,
            "Copy path [Ctrl + Shift + C]"
        ));
        if copy_path_button.clicked() {
            ui.close();
            self.copy_path_to_clipboard();
        }

        ui.separator();

        let info_button = ui.button(match self.app_state.show_info {
            true => format!("{} {}", icons::ICON_TAG, "Hide info [I]"),
            false => format!("{} {}", icons::ICON_TAG, "Show info [I]"),
        });
        if info_button.clicked() {
            self.toggle_show_info();
        }

        let aspect_ratio_button = ui.button(match self.app_state.maintain_aspect_ratio {
            true => format!(
                "{} {}",
                icons::ICON_ASPECT_RATIO,
                "Stretch aspect ratio [D]"
            ),
            false => format!(
                "{} {}",
                icons::ICON_ASPECT_RATIO,
                "Maintain aspect ratio [D]"
            ),
        });
        if aspect_ratio_button.clicked() {
            self.toggle_maintain_aspect_ratio();
        }

        ui.separator();

        let flip_h_button = ui.button(format!(
            "{} {}",
            icons::ICON_SWAP_HORIZ,
            "Flip horizontal [H]"
        ));
        if flip_h_button.clicked() {
            self.flip_horizontal();
        }

        let flip_v_button = ui.button(format!("{} {}", icons::ICON_SWAP_VERT, "Flip vertical [H]"));
        if flip_v_button.clicked() {
            self.flip_vertical();
        }

        ui.separator();

        let rotate_90_button = ui.button(format!(
            "{} {}",
            icons::ICON_ROTATE_RIGHT,
            "Rotate (90 deg) [R]"
        ));
        if rotate_90_button.clicked() {
            self.rotate_image();
        }

        let rotate_180_button = ui.button(format!(
            "{} {}",
            icons::ICON_ROTATE_RIGHT,
            "Rotate (180 deg) [R]"
        ));
        if rotate_180_button.clicked() {
            self.rotate_image();
            self.rotate_image();
        }

        let rotate_270_button = ui.button(format!(
            "{} {}",
            icons::ICON_ROTATE_RIGHT,
            "Rotate (270 deg) [R]"
        ));
        if rotate_270_button.clicked() {
            self.rotate_image();
            self.rotate_image();
            self.rotate_image();
        }

        ui.separator();

        let reset_offset_button = ui.button(format!("{} {}", icons::ICON_UNDO, "Reset offset [C]"));
        if reset_offset_button.clicked() {
            self.reset_offset();
        }

        let reset_zoom_button = ui.button(format!("{} {}", icons::ICON_UNDO, "Reset zoom [C]"));
        if reset_zoom_button.clicked() {
            self.reset_zoom();
        }

        ui.separator();

        let zoom_in_button = ui.button(format!("{} {}", icons::ICON_ZOOM_IN, "Zoom in [W]"));
        if zoom_in_button.clicked() {
            self.image_state.zoom_factor += 0.1 * self.image_state.zoom_factor;
        }

        let zoom_in_button = ui.button(format!("{} {}", icons::ICON_ZOOM_OUT, "Zoom out [S]"));
        if zoom_in_button.clicked() {
            self.image_state.zoom_factor -= 0.1 * self.image_state.zoom_factor;
        }

        ui.separator();

        let quit_button = ui.button(format!("{} {}", icons::ICON_CLOSE, "Quit [ESC]"));
        if quit_button.clicked() {
            std::process::exit(0);
        }
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
                self.image_state.info.resolution.unwrap().0,
                self.image_state.info.resolution.unwrap().1
            ),
            self.image_state.info.path.display()
        ))
        .color(Color32::WHITE);

        ui.scope_builder(UiBuilder::new().max_rect(info_rect), |ui| {
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

pub fn run(img_path: Option<PathBuf>) -> Result<(), Box<dyn std::error::Error>> {
    let initial_window_size: Vec2;
    let img_info: ImageInfo;
    let color_image: ColorImage;

    if let Some(img_path) = img_path {
        (img_info, color_image) = get_image(&img_path)?;
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
