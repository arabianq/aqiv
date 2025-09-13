use egui::Color32;

pub struct AppConfig {
    pub background_color: Color32,

    pub maintain_aspect_ratio: bool,
    pub show_info: bool,

    pub notification_duration_millis: u64,
    pub default_ui_scale: f32,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            background_color: Color32::from_hex("#1B1B1B").unwrap(),

            maintain_aspect_ratio: true,
            show_info: false,

            notification_duration_millis: 500,
            default_ui_scale: 1.25,
        }
    }
}
