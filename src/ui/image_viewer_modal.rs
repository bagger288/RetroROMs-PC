use crate::theme::ThemePreset;
use egui::{
    Align2, Color32, CursorIcon, Rect, RichText, Rounding, Sense, Stroke, UiBuilder, Vec2, Window,
};

#[derive(Clone, Debug)]
pub struct ImageViewerState {
    pub title: String,
    pub image_url: String,
    pub zoom: f32,
    pub pan_offset: Vec2,
}

impl ImageViewerState {
    pub fn new(title: impl Into<String>, image_url: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            image_url: image_url.into(),
            zoom: 1.0,
            pan_offset: Vec2::ZERO,
        }
    }
}

pub fn render_image_viewer_modal(
    ctx: &egui::Context,
    state: &mut Option<ImageViewerState>,
    theme: ThemePreset,
) {
    if state.is_none() {
        return;
    }

    let viewer = state.as_mut().unwrap();
    let mut is_open = true;

    Window::new(format!("🖼 {}", viewer.title))
        .open(&mut is_open)
        .resizable(true)
        .anchor(Align2::CENTER_CENTER, [0.0, 0.0])
        .default_size([700.0, 700.0])
        .min_width(450.0)
        .min_height(450.0)
        .show(ctx, |ui| {
            ui.vertical(|ui| {
                // Top control bar
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Масштаб:").strong());

                    if ui.button("➖").on_hover_text("Уменьшить (или колесико вниз)").clicked() {
                        viewer.zoom = (viewer.zoom * 0.85).clamp(0.2, 10.0);
                    }

                    ui.label(
                        RichText::new(format!("{:.0}%", viewer.zoom * 100.0))
                            .color(theme.primary_color())
                            .strong(),
                    );

                    if ui.button("➕").on_hover_text("Увеличить (или колесико вверх)").clicked() {
                        viewer.zoom = (viewer.zoom * 1.15).clamp(0.2, 10.0);
                    }

                    ui.add_space(8.0);
                    if ui.button("100%").on_hover_text("Исходный размер").clicked() {
                        viewer.zoom = 1.0;
                        viewer.pan_offset = Vec2::ZERO;
                    }

                    if ui.button("Сброс").on_hover_text("Сбросить позицию и зум").clicked() {
                        viewer.zoom = 1.0;
                        viewer.pan_offset = Vec2::ZERO;
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(
                            RichText::new("💡 Колёсико мыши: зум • ЛКМ: перетаскивание • Двойной клик: сброс")
                                .size(11.0)
                                .weak(),
                        );
                    });
                });

                ui.add_space(4.0);
                ui.separator();
                ui.add_space(4.0);

                // Interactive Image Canvas
                let canvas_size = ui.available_size().max(Vec2::new(320.0, 320.0));
                let (canvas_rect, response) =
                    ui.allocate_exact_size(canvas_size, Sense::click_and_drag());

                // 1. Smooth mouse wheel zoom (proportional to scroll delta)
                let scroll_delta = ui.input(|i| i.smooth_scroll_delta.y);
                if response.hovered() && scroll_delta.abs() > 0.01 {
                    let zoom_factor = (scroll_delta * 0.003).exp();
                    viewer.zoom = (viewer.zoom * zoom_factor).clamp(0.2, 10.0);
                }

                // 2. Mouse drag panning
                if response.dragged() {
                    viewer.pan_offset += response.drag_delta();
                }

                // 3. Double click reset
                if response.double_clicked() {
                    viewer.zoom = 1.0;
                    viewer.pan_offset = Vec2::ZERO;
                }

                // Cursor icon
                if response.hovered() {
                    ui.ctx().set_cursor_icon(if response.dragged() {
                        CursorIcon::Grabbing
                    } else {
                        CursorIcon::Grab
                    });
                }

                // 4. Background canvas frame
                ui.painter().rect_filled(
                    canvas_rect,
                    Rounding::same(8.0),
                    Color32::from_rgb(14, 16, 24),
                );
                ui.painter().rect_stroke(
                    canvas_rect,
                    Rounding::same(8.0),
                    Stroke::new(1.0_f32, Color32::from_rgb(38, 44, 62)),
                );

                // 5. Draw image clipped inside canvas
                let clip_rect = canvas_rect.shrink(2.0);
                ui.allocate_new_ui(UiBuilder::new().max_rect(clip_rect), |ui| {
                    ui.set_clip_rect(clip_rect);

                    let center = clip_rect.center() + viewer.pan_offset;
                    let base_side = (clip_rect.width().min(clip_rect.height()) * 0.90).max(120.0);
                    let img_box_size = Vec2::new(base_side * viewer.zoom, base_side * viewer.zoom);
                    let img_rect = Rect::from_center_size(center, img_box_size);

                    ui.allocate_new_ui(UiBuilder::new().max_rect(img_rect), |ui| {
                        ui.centered_and_justified(|ui| {
                            ui.add(
                                egui::Image::new(&viewer.image_url)
                                    .max_size(img_box_size)
                                    .rounding(Rounding::same(6.0)),
                            );
                        });
                    });
                });
            });
        });

    if !is_open {
        *state = None;
    }
}
