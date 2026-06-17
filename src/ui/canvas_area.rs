use crate::core::image::Image;
use crate::ui::theme::Theme;
use crate::ui::widget::{DrawFrame, DrawRect, Pos, RectF, Widget};
use std::sync::{Arc, Mutex};
use winit::event::WindowEvent;

/// The main art canvas. Renders the painting via the existing renderer.
/// Shares the pixel buffer with the painting code via `Arc<Mutex<Image>>`.
pub struct CanvasArea {
    pub id: u64,
    rect: RectF,
    pub image: Arc<Mutex<Image>>,
}

impl CanvasArea {
    pub fn new(id: u64) -> Self {
        Self { id, rect: RectF::zero(), image: Arc::new(Mutex::new(Self::make_checkerboard(256, 256))) }
    }

    fn make_checkerboard(w: u32, h: u32) -> Image {
        let mut img = Image::new(w, h);
        for y in 0..h {
            for x in 0..w {
                let cx = x / 32;
                let cy = y / 32;
                let v = if (cx + cy) & 1 == 0 { 200.0 } else { 50.0 };
                img.set_pixel(x, y, [v, v, v]);
            }
        }
        img
    }

    pub fn viewport(&self) -> (u32, u32) {
        (self.rect.size.width as u32, self.rect.size.height as u32)
    }

    pub fn rect(&self) -> RectF {
        self.rect
    }

    /// Replace the shared image buffer.
    pub fn set_image_arc(&mut self, img: Arc<Mutex<Image>>) {
        self.image = img;
    }
}

impl Widget for CanvasArea {
    fn id(&self) -> u64 { self.id }

    fn layout(&mut self, bounds: RectF, _theme: &Theme) -> RectF {
        self.rect = bounds;
        self.rect
    }

    fn handle_event(&mut self, _event: &WindowEvent, _pos: Pos) -> bool {
        false
    }

    fn render(&self, frame: &mut DrawFrame) {
        frame.rects.push(DrawRect {
            rect: self.rect,
            color: [0.14, 0.15, 0.19, 1.0],
            corner_radius: 0.0,
        });
    }
}
