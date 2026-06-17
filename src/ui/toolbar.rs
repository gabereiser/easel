use crate::ui::theme::Theme;
use crate::ui::widget::{DrawFrame, DrawRect, Pos, RectF, Widget};
use euclid::{Point2D, Size2D};

pub struct Toolbar {
    pub id: u64,
    pub tools: Vec<String>,
    rect: RectF,
}

impl Toolbar {
    pub fn new(id: u64, tools: Vec<String>) -> Self {
        Self { id, tools, rect: RectF::zero() }
    }
}

impl Widget for Toolbar {
    fn id(&self) -> u64 { self.id }

    fn layout(&mut self, bounds: RectF, theme: &Theme) -> RectF {
        self.rect = RectF::new(bounds.origin, Size2D::new(bounds.size.width, theme.toolbar_height));
        self.rect
    }

    fn handle_event(&mut self, _event: &winit::event::WindowEvent, _pos: Pos) -> bool {
        false
    }

    fn render(&self, frame: &mut DrawFrame) {
        frame.rects.push(DrawRect {
            rect: self.rect,
            color: [0.12, 0.125, 0.14, 1.0],
            corner_radius: 0.0,
        });
        frame.rects.push(DrawRect {
            rect: RectF::new(
                Point2D::new(self.rect.origin.x, self.rect.max_y() - 1.0),
                Size2D::new(self.rect.size.width, 1.0),
            ),
            color: [0.16, 0.18, 0.22, 1.0],
            corner_radius: 0.0,
        });
    }
}
