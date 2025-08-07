use egui::{Color32, Vec2};
use egui_notify::Toasts;

use std::time::Duration;

pub struct AppState {
    pub window_size: Vec2,
    pub background_color: Color32,

    pub maintain_aspect_ratio: bool,
    pub show_info: bool,
    pub dragging: bool,

    pub toasts: Toasts,
    pub notification_duration: Option<Duration>,
}

impl AppState {
    pub fn toggle_maintain_aspect_ratio(&mut self) {
        self.maintain_aspect_ratio = !self.maintain_aspect_ratio;
    }

    pub fn toggle_show_info(&mut self) {
        self.show_info = !self.show_info;
        self.notify(format!("Show info: {}", self.show_info));
    }

    pub fn notify(&mut self, message: String) {
        self.toasts
            .basic(message)
            .duration(self.notification_duration);
    }
}
