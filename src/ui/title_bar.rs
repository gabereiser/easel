use crate::ui::theme::Theme;
use crate::ui::widget::{DrawFrame, DrawRect, DrawText, Pos, RectF, Widget};
use euclid::{Point2D, Size2D};

pub struct TitleBar {
    pub id: u64,
    pub title: String,
    rect: RectF,
    _close_hovered: bool,
    _max_hovered: bool,
    _min_hovered: bool,
}

impl TitleBar {
    pub fn new(id: u64, title: &str) -> Self {
        Self {
            id,
            title: title.to_string(),
            rect: RectF::zero(),
            _close_hovered: false,
            _max_hovered: false,
            _min_hovered: false,
        }
    }
}

impl Widget for TitleBar {
    fn id(&self) -> u64 { self.id }

    fn layout(&mut self, bounds: RectF, theme: &Theme) -> RectF {
        self.rect = RectF::new(bounds.origin, Size2D::new(bounds.size.width, theme.title_bar_height));
        self.rect
    }

    fn handle_event(&mut self, _event: &winit::event::WindowEvent, _pos: Pos) -> bool {
        false
    }

    fn render(&self, frame: &mut DrawFrame) {
        frame.rects.push(DrawRect {
            rect: self.rect,
            color: [0.07, 0.075, 0.09, 1.0],
            corner_radius: 0.0,
        });

        // Title text
        let text_rect = RectF::new(
            Point2D::new(self.rect.origin.x + 12.0, self.rect.origin.y),
            Size2D::new(self.rect.size.width, self.rect.size.height),
        );
        frame.texts.push(DrawText {
            rect: text_rect,
            text: self.title.clone(),
            color: [0.78, 0.79, 0.83, 1.0],
            font_size: 13.0,
        });
    }
}
