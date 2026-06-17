use crate::ui::theme::Theme;
use euclid::{Point2D, Rect, SideOffsets2D, Size2D, Vector2D, UnknownUnit};
use winit::event::WindowEvent;

pub type Pos = Point2D<f32, UnknownUnit>;
pub type Size = Size2D<f32, UnknownUnit>;
pub type RectF = Rect<f32, UnknownUnit>;

/// Base trait for all UI widgets.
pub trait Widget {
    /// The widget's unique ID for event routing.
    fn id(&self) -> u64;

    /// Lay out this widget within the given bounds.
    /// Returns the actual layout rect.
    fn layout(&mut self, bounds: RectF, theme: &Theme) -> RectF;

    /// Handle a window event.
    /// Returns true if the event was consumed.
    fn handle_event(&mut self, event: &WindowEvent, pos: Pos) -> bool;

    /// Queue draw commands.
    fn render(&self, frame: &mut DrawFrame);
}

/// Resources passed to widgets during rendering.
pub struct DrawFrame<'a> {
    pub rects: &'a mut Vec<DrawRect>,
    pub texts: &'a mut Vec<DrawText>,
    pub clip: RectF,
}

#[derive(Clone)]
pub struct DrawRect {
    pub rect: RectF,
    pub color: [f32; 4],
    pub corner_radius: f32,
}

#[derive(Clone)]
pub struct DrawText {
    pub rect: RectF,
    pub text: String,
    pub color: [f32; 4],
    pub font_size: f32,
}

/// A container that holds child widgets and lays them out in a direction.
pub enum Direction {
    Horizontal,
    Vertical,
}

pub struct Container {
    pub id: u64,
    pub children: Vec<Box<dyn Widget>>,
    pub direction: Direction,
    pub padding: f32,
    pub spacing: f32,
    rect: RectF,
}

impl Container {
    pub fn new(id: u64, direction: Direction) -> Self {
        Self {
            id,
            children: Vec::new(),
            direction,
            padding: 0.0,
            spacing: 0.0,
            rect: RectF::zero(),
        }
    }

    pub fn push(&mut self, child: Box<dyn Widget>) {
        self.children.push(child);
    }
}

impl Widget for Container {
    fn id(&self) -> u64 { self.id }

    fn layout(&mut self, bounds: RectF, theme: &Theme) -> RectF {
        let pad = self.padding;
        let inner = bounds.inner_rect(SideOffsets2D::new(pad, pad, pad, pad));
        let mut cursor = inner.origin;
        let (dx, dy) = match self.direction {
            Direction::Horizontal => (1.0, 0.0),
            Direction::Vertical => (0.0, 1.0),
        };

        let mut child_rect = RectF::zero();
        for child in &mut self.children {
            let child_bounds = RectF::new(
                Point2D::new(cursor.x, cursor.y),
                Size2D::new(
                    if dx > 0.0 { inner.size.width - (cursor.x - inner.origin.x) } else { inner.size.width },
                    if dy > 0.0 { inner.size.height - (cursor.y - inner.origin.y) } else { inner.size.height },
                ),
            );
            let laid = child.layout(child_bounds, theme);
            cursor += Vector2D::new(
                laid.size.width * dx + self.spacing * dx,
                laid.size.height * dy + self.spacing * dy,
            );
            if child_rect.is_empty() {
                child_rect = laid;
            } else {
                child_rect = child_rect.union(&laid);
            }
        }

        self.rect = RectF::new(bounds.origin, Size2D::new(bounds.size.width, child_rect.size.height.min(bounds.size.height)));
        self.rect
    }

    fn handle_event(&mut self, event: &WindowEvent, pos: Pos) -> bool {
        for child in &mut self.children {
            if child.handle_event(event, pos) {
                return true;
            }
        }
        false
    }

    fn render(&self, frame: &mut DrawFrame) {
        for child in &self.children {
            child.render(frame);
        }
    }
}
