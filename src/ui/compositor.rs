use crate::core::image::Image;
use crate::ui::canvas_area::CanvasArea;
use crate::ui::drawer::Drawer;
use crate::ui::palette::Palette;
use crate::ui::theme::Theme;
use crate::ui::title_bar::TitleBar;
use crate::ui::toolbar::Toolbar;
use crate::ui::widget::{DrawFrame, DrawRect, DrawText, Pos, RectF, Widget};
use euclid::{Point2D, Size2D};
use std::sync::{Arc, Mutex};
use winit::event::WindowEvent;

pub struct UiCompositor {
    pub theme: Theme,
    pub title_bar: TitleBar,
    pub toolbar: Toolbar,
    pub palette: Palette,
    pub canvas: CanvasArea,
    pub drawer: Drawer,
    pub size: (f32, f32),

    // Draw buffers
    rects: Vec<DrawRect>,
    texts: Vec<DrawText>,

    // Event state
    cursor_pos: Pos,

    // Brush info displayed in toolbar area
    pub brush_info: String,
}

impl UiCompositor {
    pub fn new(width: f32, height: f32) -> Self {
        Self {
            theme: Theme::dark(),
            title_bar: TitleBar::new(1, "Easel — untitled"),
            toolbar: Toolbar::new(2, vec![]),
            palette: Palette::new(3),
            canvas: CanvasArea::new(4),
            drawer: Drawer::new(5, "Brushes"),
            size: (width, height),
            rects: Vec::new(),
            texts: Vec::new(),
            cursor_pos: Pos::zero(),
            brush_info: String::new(),
        }
    }

    pub fn resize(&mut self, width: f32, height: f32) {
        self.size = (width, height);
    }

    pub fn layout(&mut self) {
        let size: Size2D<_, euclid::UnknownUnit> = Size2D::new(self.size.0, self.size.1);
        let mut y = 0.0;

        // Title bar
        let tb_bounds = RectF::new(Point2D::new(0.0, 0.0), Size2D::new(size.width, self.theme.title_bar_height));
        self.title_bar.layout(tb_bounds, &self.theme);
        y += self.theme.title_bar_height;

        // Toolbar
        let tl_bounds = RectF::new(Point2D::new(0.0, y), Size2D::new(size.width, self.theme.toolbar_height));
        self.toolbar.layout(tl_bounds, &self.theme);
        y += self.theme.toolbar_height;

        // Palette (left)
        let pl_bounds = RectF::new(Point2D::new(0.0, y), Size2D::new(self.theme.palette_width, size.height - y));
        self.palette.layout(pl_bounds, &self.theme);

        // Drawer (right)
        let dr_bounds = RectF::new(
            Point2D::new(size.width - self.theme.drawer_width, y),
            Size2D::new(self.theme.drawer_width, size.height - y),
        );
        self.drawer.layout(dr_bounds, &self.theme);

        // Canvas (center, between palette and drawer)
        let cv_bounds = RectF::new(
            Point2D::new(self.theme.palette_width, y),
            Size2D::new(
                size.width - self.theme.palette_width - self.theme.drawer_width,
                size.height - y,
            ),
        );
        self.canvas.layout(cv_bounds, &self.theme);
    }

    pub fn render(&mut self, _wgpu_device: &wgpu::Device, _wgpu_queue: &wgpu::Queue) {
        self.rects.clear();
        self.texts.clear();

        // Draw background
        self.rects.push(DrawRect {
            rect: RectF::new(Point2D::zero(), Size2D::new(self.size.0, self.size.1)),
            color: self.theme.background,
            corner_radius: 0.0,
        });

        let mut frame = DrawFrame {
            rects: &mut self.rects,
            texts: &mut self.texts,
            clip: RectF::new(Point2D::zero(), Size2D::new(self.size.0, self.size.1)),
        };

        self.title_bar.render(&mut frame);
        self.toolbar.render(&mut frame);
        self.palette.render(&mut frame);
        self.canvas.render(&mut frame);
        self.drawer.render(&mut frame);

        // Brush info overlay in toolbar area
        if !self.brush_info.is_empty() {
            self.texts.push(DrawText {
                rect: RectF::new(
                    Point2D::new(self.theme.palette_width + self.theme.padding, self.theme.title_bar_height),
                    Size2D::new(400.0, self.theme.toolbar_height),
                ),
                text: self.brush_info.clone(),
                color: self.theme.text_primary,
                font_size: 12.0,
            });
        }
    }

    pub fn handle_event(&mut self, event: &WindowEvent) -> bool {
        // Track cursor position for MouseInput events
        if let WindowEvent::CursorMoved { position, .. } = event {
            self.cursor_pos = Pos::new(position.x as f32, position.y as f32);
        }
        // Dispatch to widgets (frontmost first)
        if self.toolbar.handle_event(event, self.cursor_pos) { return true; }
        if self.title_bar.handle_event(event, self.cursor_pos) { return true; }
        if self.drawer.handle_event(event, self.cursor_pos) { return true; }
        if self.palette.handle_event(event, self.cursor_pos) { return true; }
        if self.canvas.handle_event(event, self.cursor_pos) { return true; }
        false
    }

    pub fn draw_data(&self) -> (&[DrawRect], &[DrawText]) {
        (&self.rects, &self.texts)
    }

    pub fn canvas_viewport(&self) -> (u32, u32) {
        self.canvas.viewport()
    }

    pub fn canvas_rect(&self) -> RectF {
        self.canvas.rect()
    }

    pub fn canvas_image_arc(&self) -> Arc<Mutex<Image>> {
        self.canvas.image.clone()
    }

    pub fn set_canvas_image_arc(&mut self, img: Arc<Mutex<Image>>) {
        self.canvas.set_image_arc(img);
    }

    pub fn canvas_origin(&self) -> (f32, f32) {
        (self.canvas.rect().origin.x, self.canvas.rect().origin.y)
    }

    /// Reads and clears the palette's clicked color (set by swatch click).
    pub fn take_clicked_color(&mut self) -> Option<[f32; 3]> {
        self.palette.clicked_color.take()
    }
}
