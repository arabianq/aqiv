use crate::app::App;

use egui::{Context, Key, Ui};

impl App {
    pub fn handle_input(&mut self, ui: &mut Ui, ctx: &Context) {
        let mut ui_scale_factor = ctx.zoom_factor();

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
                self.app_state.toggle_maintain_aspect_ratio();
                self.app_state.notify(format!(
                    "Maintain Aspect Ratio: {}",
                    self.app_state.maintain_aspect_ratio
                ));
            }

            // Show Info on I
            if i.key_pressed(Key::I) {
                self.app_state.toggle_show_info();
            }

            // Horizontal Flip on H
            if i.key_pressed(Key::H) {
                self.image_state.flip_horizontal();
                self.app_state
                    .notify(format!("Flip H: {}", self.image_state.uv_rect.min.x == 1.0));
            }

            // Vertical Flip on V
            if i.key_pressed(Key::V) {
                self.image_state.flip_vertical();
                self.app_state
                    .notify(format!("Flip V: {}", self.image_state.uv_rect.min.y == 1.0));
            }

            // Image rotation on R
            if i.key_pressed(Key::R) {
                self.image_state.rotate_image();
                self.app_state.notify(format!(
                    "Rotation: {} deg",
                    self.image_state.rotation as u16 * 90
                ));
            }

            // Reset offset on C
            if i.key_pressed(Key::C) {
                self.image_state.reset_offset();
                self.app_state
                    .notify(String::from("Position Offset: (0.0, 0.0)"));
            }

            // Reset zoom on X
            if i.key_pressed(Key::X) {
                self.image_state.reset_zoom();
                self.app_state.notify(String::from("Zoom Factor: 1.0"));
            }

            // Increase UI scale on Ctrl+Plus
            if i.modifiers.ctrl && i.key_pressed(Key::Plus) {
                ui_scale_factor += 0.1;
            }

            // Decrease UI scale on Ctrl+Minus
            if i.modifiers.ctrl && i.key_pressed(Key::Minus) {
                ui_scale_factor -= 0.1;
            }

            if i.key_pressed(Key::ArrowRight) {
                self.next_image(1).ok();
            } else if i.key_pressed(Key::ArrowLeft) {
                self.next_image(-1).ok();
            }

            if i.events.contains(&egui::Event::Copy) {
                match i.modifiers.shift {
                    true => {
                        self.image_state.copy_path_to_clipboard();
                        self.app_state
                            .notify(String::from("Path was copied to clipboard"));
                    }
                    false => {
                        self.image_state.copy_uri_to_clipboard();
                        self.app_state
                            .notify(String::from("Image was copied to clipboard"));
                    }
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

        ctx.set_zoom_factor(ui_scale_factor);
    }
}
