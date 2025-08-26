mod default;
mod heif;
mod jpegxl;
mod svg;

pub use default::load_image_default;
pub use heif::load_image_heif;
pub use jpegxl::load_image_jpegxl;
pub use svg::load_image_svg;
