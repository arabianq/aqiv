mod image_loaders;

pub use image_info::ImageInfo;
pub use image_state::ImageState;

use crate::app::utils::pathbuf_as_uri;
use image_loaders::*;

use clipboard_rs::{Clipboard, ClipboardContext};
use wl_clipboard_rs::copy::{
    MimeType as ClipboardMimeType, Options as ClipboardOptions, Source as ClipboardSource,
};

use egui::{ColorImage, Rect, TextureHandle, Vec2, load::SizedTexture};
use image::{DynamicImage, GenericImageView};
use rayon::prelude::*;

use std::error::Error;
use std::path::{PathBuf, absolute};

pub mod image_info {
    use super::*;

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
}

pub mod image_state {
    use super::*;

    pub struct ImageState {
        pub info: ImageInfo,

        pub uri: String,
        pub uri_to_forget: Option<String>,

        pub rotation: u8,
        pub zoom_factor: f32,

        pub uv_rect: Rect,
        pub offset: Vec2,

        pub color_image: Option<ColorImage>,
        pub texture_handle: Option<TextureHandle>,
        pub sized_texture: Option<SizedTexture>,
    }

    impl ImageState {
        pub fn load_new_image(&mut self, path: &PathBuf) -> Result<bool, Box<dyn Error>> {
            let (new_img_info, new_color_image) = get_image_info(&path)?;

            if new_img_info.resolution.is_none() {
                return Ok(false);
            }

            self.uri_to_forget = Some(self.uri.clone());

            self.info = new_img_info;
            self.uri = pathbuf_as_uri(&self.info.path);
            self.zoom_factor = 1.0;
            self.rotation = 0;
            self.color_image = Some(new_color_image);

            Ok(true)
        }

        pub fn flip_horizontal(&mut self) {
            if self.uv_rect.min.x == 0.0 {
                self.uv_rect.min.x = 1.0;
                self.uv_rect.max.x = 0.0;
            } else {
                self.uv_rect.min.x = 0.0;
                self.uv_rect.max.x = 1.0;
            }
        }
        pub fn flip_vertical(&mut self) {
            if self.uv_rect.min.y == 0.0 {
                self.uv_rect.min.y = 1.0;
                self.uv_rect.max.y = 0.0;
            } else {
                self.uv_rect.min.y = 0.0;
                self.uv_rect.max.y = 1.0;
            }
        }

        pub fn rotate_image(&mut self) {
            self.rotation += 1;
            if self.rotation == 4 {
                self.rotation = 0;
            }
        }

        pub fn reset_offset(&mut self) {
            self.offset = Vec2::ZERO;
        }

        pub fn reset_zoom(&mut self) {
            self.zoom_factor = 1.0;
        }

        pub fn copy_path_to_clipboard(&mut self) {
            let clipboard_ctx = ClipboardContext::new().unwrap();
            clipboard_ctx
                .set_text(self.info.path.to_string_lossy().to_string())
                .unwrap();

            // Clipboard-rs does not support wayland, so I have to use wl-clipboard-rs in addition to it
            // BTW I don't know how will it work in xorg session =P
            {
                let opts = ClipboardOptions::new();
                opts.copy(
                    ClipboardSource::Bytes(self.info.path.to_string_lossy().as_bytes().into()),
                    ClipboardMimeType::Specific(String::from("text/plain;charset=utf-8")),
                )
                .ok();
            }
        }

        pub fn copy_uri_to_clipboard(&mut self) {
            let clipboard_ctx = ClipboardContext::new().unwrap();

            clipboard_ctx.set_files(vec![self.uri.to_string()]).ok();

            // For wayland
            {
                let opts = ClipboardOptions::new();
                opts.copy(
                    ClipboardSource::Bytes(self.uri.as_bytes().into()),
                    ClipboardMimeType::Specific(String::from("text/uri-list")),
                )
                .ok();
            }
        }
    }
}

pub fn get_image_info(img_path: &PathBuf) -> Result<(ImageInfo, ColorImage), Box<dyn Error>> {
    let img_path = absolute(img_path)?;
    let extension = img_path
        .extension()
        .and_then(|s| s.to_str())
        .map(|s| s.to_lowercase());

    let mut loaders: Vec<(&str, fn(&Vec<u8>) -> Result<DynamicImage, Box<dyn Error>>)> = vec![
        ("default", load_image_default),
        ("svg", load_image_svg),
        ("heif", load_image_heif),
        ("jxl", load_image_jpegxl),
    ];
    let formats = common_macros::hash_map! {
        "default" => "Unknown",
        "svg" => "SVG",
        "heif" => "HEIF",
        "jxl" => "JPEG XL",
    };

    // Determine default loader based on extension
    if let Some(ref e) = extension {
        if let Some(pos) = loaders.par_iter().position_any(|(ext, _)| ext == e) {
            let loader = loaders.remove(pos);
            loaders.insert(0, loader);
        }
    }

    let buf = std::fs::read(&img_path)?;

    let mut image: Option<DynamicImage> = None;
    let mut image_format: Option<String> = None;

    // Try every loader until it works
    for &(ext, loader) in &loaders {
        match loader(&buf) {
            Ok(img) => image = Some(img),
            Err(_err) => (),
        }

        if image.is_some() {
            image_format = match ext {
                "default" => {
                    let reader = image::ImageReader::open(&img_path)?.with_guessed_format()?;
                    let format = reader.format().unwrap();
                    Some(format!("{:?}", format).to_uppercase())
                }
                _ => Some(formats[ext].to_string()),
            };
            break;
        }
    }

    if let Some(img) = &image {
        let image_resolution = img.dimensions();
        let image_bytes = img.as_bytes();

        let color_image = ColorImage::from_rgba_unmultiplied(
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
                format: image_format.unwrap(),
                size: std::fs::metadata(&img_path)?.len(),
                resolution: Some(image_resolution),
            },
            color_image,
        ))
    } else {
        Err("No loaders available".into())
    }
}
