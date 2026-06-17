use crate::ui::theme::Theme;
use crate::ui::widget::{DrawFrame, DrawRect, DrawText, Pos, RectF, Widget};
use euclid::{Point2D, Size2D};

pub struct Drawer {
    pub id: u64,
    pub title: String,
    pub open: bool,
    rect: RectF,

    // Brush control values
    pub opacity: f32,
    pub flow: f32,
    pub hardness: f32,

    // Internal state
    dragging: Option<usize>,
    slider_rects: Vec<RectF>,
    slider_labels: [&'static str; 3],
}

impl Drawer {
    pub fn new(id: u64, title: &str) -> Self {
        Self {
            id,
            title: title.to_string(),
            open: true,
            rect: RectF::zero(),
            opacity: 1.0,
            flow: 1.0,
            hardness: 0.8,
            dragging: None,
            slider_rects: Vec::new(),
            slider_labels: ["Opacity", "Flow", "Hardness"],
        }
    }
}

impl Widget for Drawer {
    fn id(&self) -> u64 { self.id }

    fn layout(&mut self, bounds: RectF, theme: &Theme) -> RectF {
        let width = if self.open { theme.drawer_width } else { 0.0 };
        self.rect = RectF::new(bounds.origin, Size2D::new(width, bounds.size.height));
        self.slider_rects.clear();
        let x = self.rect.origin.x + 12.0;
        let track_w = (self.rect.size.width - 24.0).max(20.0);
        let mut y = self.rect.origin.y + 28.0;
        for _ in 0..3 {
            self.slider_rects.push(RectF::new(
                Point2D::new(x, y),
                Size2D::new(track_w, 10.0),
            ));
            y += 44.0;
        }
        self.rect
    }

    fn handle_event(&mut self, event: &winit::event::WindowEvent, pos: Pos) -> bool {
        use winit::event::ElementState;
        use winit::event::MouseButton;
        use winit::event::WindowEvent;

        match event {
            WindowEvent::MouseInput { state: ElementState::Pressed, button: MouseButton::Left, .. } => {
                for (i, track) in self.slider_rects.iter().enumerate() {
                    if track.contains(pos) {
                        self.dragging = Some(i);
                        let rel_x = (pos.x - track.origin.x).clamp(0.0, track.size.width);
                        let val = rel_x / track.size.width;
                        match i {
                            0 => self.opacity = val,
                            1 => self.flow = val,
                            2 => self.hardness = val,
                            _ => {}
                        }
                        return true;
                    }
                }
                false
            }
            WindowEvent::MouseInput { state: ElementState::Released, button: MouseButton::Left, .. } => {
                self.dragging = None;
                false
            }
            WindowEvent::CursorMoved { .. } => {
                if let Some(i) = self.dragging {
                    if let Some(track) = self.slider_rects.get(i) {
                        let rel_x = (pos.x - track.origin.x).clamp(0.0, track.size.width);
                        let val = rel_x / track.size.width;
                        match i {
                            0 => self.opacity = val,
                            1 => self.flow = val,
                            2 => self.hardness = val,
                            _ => {}
                        }
                    }
                    return true;
                }
                false
            }
            _ => false,
        }
    }

    fn render(&self, frame: &mut DrawFrame) {
        if !self.open { return; }

        // Background
        frame.rects.push(DrawRect {
            rect: self.rect,
            color: [0.12, 0.125, 0.14, 1.0],
            corner_radius: 0.0,
        });
        // Left border
        frame.rects.push(DrawRect {
            rect: RectF::new(
                Point2D::new(self.rect.origin.x, self.rect.origin.y),
                Size2D::new(1.0, self.rect.size.height),
            ),
            color: [0.16, 0.18, 0.22, 1.0],
            corner_radius: 0.0,
        });

        let x = self.rect.origin.x + 12.0;

        // Section header
        frame.texts.push(DrawText {
            rect: RectF::new(
                Point2D::new(x, self.rect.origin.y + 6.0),
                Size2D::new(self.rect.size.width - 24.0, 16.0),
            ),
            text: self.title.clone(),
            color: [1.0, 1.0, 1.0, 0.7],
            font_size: 11.0,
        });

        let values = [self.opacity, self.flow, self.hardness];

        for i in 0..3 {
            let track = &self.slider_rects[i];
            let label_y = track.origin.y - 16.0;

            // Label
            frame.texts.push(DrawText {
                rect: RectF::new(
                    Point2D::new(x, label_y),
                    Size2D::new(self.rect.size.width - 24.0, 14.0),
                ),
                text: self.slider_labels[i].to_string(),
                color: [1.0, 1.0, 1.0, 0.6],
                font_size: 11.0,
            });

            // Track background
            frame.rects.push(DrawRect {
                rect: *track,
                color: [0.08, 0.09, 0.11, 1.0],
                corner_radius: 3.0,
            });

            // Track fill
            if values[i] > 0.0 {
                let fill_w = track.size.width * values[i];
                frame.rects.push(DrawRect {
                    rect: RectF::new(track.origin, Size2D::new(fill_w, track.size.height)),
                    color: [0.3, 0.5, 0.8, 1.0],
                    corner_radius: 3.0,
                });
            }

            // Value text (right-aligned)
            frame.texts.push(DrawText {
                rect: RectF::new(
                    Point2D::new(x + track.size.width - 50.0, label_y),
                    Size2D::new(50.0, 14.0),
                ),
                text: format!("{:.0}%", (values[i] * 100.0).round()),
                color: [1.0, 1.0, 1.0, 0.8],
                font_size: 11.0,
            });
        }
    }
}
