use crate::app::App;

use crate::app::utils::{calculate_uv_rect, convert_size};

use egui::{
    Align, Color32, Frame, Image, Layout, Pos2, Rect, RichText, Sense, Ui, UiBuilder, Vec2,
};
use egui_material_icons::icons;

impl App {
    pub fn render_img(&mut self, ui: &mut Ui) {
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

    pub fn render_context_menu(&mut self, ui: &mut Ui) {
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
            self.image_state.copy_uri_to_clipboard();
            self.app_state
                .notify(String::from("Image was copied to clipboard"));
        }

        let copy_path_button = ui.button(format!(
            "{} {}",
            icons::ICON_FILE_COPY,
            "Copy path [Ctrl + Shift + C]"
        ));
        if copy_path_button.clicked() {
            ui.close();
            self.image_state.copy_path_to_clipboard();
            self.app_state
                .notify(String::from("Path was copied to clipboard"));
        }

        ui.separator();

        let info_button = ui.button(match self.app_state.show_info {
            true => format!("{} {}", icons::ICON_TAG, "Hide info [I]"),
            false => format!("{} {}", icons::ICON_TAG, "Show info [I]"),
        });
        if info_button.clicked() {
            self.app_state.toggle_show_info();
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
            self.app_state.toggle_maintain_aspect_ratio();
            self.app_state.notify(format!(
                "Maintain Aspect Ratio: {}",
                self.app_state.maintain_aspect_ratio
            ));
        }

        ui.separator();

        let flip_h_button = ui.button(format!(
            "{} {}",
            icons::ICON_SWAP_HORIZ,
            "Flip horizontal [H]"
        ));
        if flip_h_button.clicked() {
            self.image_state.flip_horizontal();
            self.app_state
                .notify(format!("Flip H: {}", self.image_state.uv_rect.min.x == 1.0));
        }

        let flip_v_button = ui.button(format!("{} {}", icons::ICON_SWAP_VERT, "Flip vertical [H]"));
        if flip_v_button.clicked() {
            self.image_state.flip_vertical();
            self.app_state
                .notify(format!("Flip V: {}", self.image_state.uv_rect.min.y == 1.0));
        }

        ui.separator();

        let rotate_90_button = ui.button(format!(
            "{} {}",
            icons::ICON_ROTATE_RIGHT,
            "Rotate (90 deg) [R]"
        ));
        if rotate_90_button.clicked() {
            self.image_state.rotate_image();
            self.app_state.notify(format!(
                "Rotation: {} deg",
                self.image_state.rotation as u16 * 90
            ));
        }

        let rotate_180_button = ui.button(format!(
            "{} {}",
            icons::ICON_ROTATE_RIGHT,
            "Rotate (180 deg) [R]"
        ));
        if rotate_180_button.clicked() {
            self.image_state.rotate_image();
            self.image_state.rotate_image();
            self.app_state.notify(format!(
                "Rotation: {} deg",
                self.image_state.rotation as u16 * 90
            ));
        }

        let rotate_270_button = ui.button(format!(
            "{} {}",
            icons::ICON_ROTATE_RIGHT,
            "Rotate (270 deg) [R]"
        ));
        if rotate_270_button.clicked() {
            self.image_state.rotate_image();
            self.image_state.rotate_image();
            self.image_state.rotate_image();
            self.app_state.notify(format!(
                "Rotation: {} deg",
                self.image_state.rotation as u16 * 90
            ));
        }

        ui.separator();

        let reset_offset_button = ui.button(format!("{} {}", icons::ICON_UNDO, "Reset offset [C]"));
        if reset_offset_button.clicked() {
            self.image_state.reset_offset();
            self.app_state
                .notify(String::from("Position Offset: (0.0, 0.0)"));
        }

        let reset_zoom_button = ui.button(format!("{} {}", icons::ICON_UNDO, "Reset zoom [C]"));
        if reset_zoom_button.clicked() {
            self.image_state.reset_zoom();
            self.app_state.notify(String::from("Zoom Factor: 1.0"));
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

    pub fn render_info(&mut self, ui: &mut Ui) {
        let info_rect = Rect::from_min_max(
            Pos2::new(0.0, self.app_state.window_size.y - 100.0),
            Pos2::new(self.app_state.window_size.x, self.app_state.window_size.y),
        );

        let info_text = RichText::new(format!(
            "File Name: {}\nFile Path: {}\nFile Size: {}\nImage Format: {}\nImage Resolution: {}x{}",
            self.image_state.info.name,
            self.image_state.info.path.display(),
            convert_size(self.image_state.info.size as f64),
            self.image_state.info.format,
            self.image_state.info.resolution.unwrap().0,
            self.image_state.info.resolution.unwrap().1
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
