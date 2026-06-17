use crate::ui::theme::Theme;
use crate::ui::widget::{DrawFrame, DrawRect, Pos, RectF, Widget};
use euclid::{Point2D, Size2D};
use winit::event::{ElementState, MouseButton, WindowEvent};

const SWATCH_SIZE: f32 = 28.0;
const SWATCH_PAD: f32 = 6.0;

pub struct Palette {
    pub id: u64,
    rect: RectF,
    pub swatches: Vec<([f32; 3], RectF)>,
    /// Set by handle_event when a swatch is clicked. Main loop reads and clears this.
    pub clicked_color: Option<[f32; 3]>,
}

impl Palette {
    pub fn new(id: u64) -> Self {
        let colors: [[f32; 3]; 14] = [
            [220.0, 40.0, 40.0],   // red
            [220.0, 130.0, 30.0],  // orange
            [220.0, 200.0, 40.0],  // yellow
            [50.0, 180.0, 60.0],   // green
            [40.0, 160.0, 200.0],  // cyan
            [40.0, 80.0, 220.0],   // blue
            [160.0, 50.0, 200.0],  // purple
            [200.0, 40.0, 120.0],  // magenta
            [220.0, 160.0, 100.0], // skin
            [140.0, 80.0, 40.0],   // brown
            [220.0, 220.0, 220.0], // white
            [160.0, 160.0, 160.0], // light gray
            [80.0, 80.0, 80.0],    // dark gray
            [30.0, 30.0, 30.0],    // black
        ];
        Self {
            id,
            rect: RectF::zero(),
            swatches: colors.iter().map(|c| (*c, RectF::zero())).collect(),
            clicked_color: None,
        }
    }

    fn layout_swatches(&mut self) {
        let x = self.rect.origin.x + SWATCH_PAD;
        let mut y = self.rect.origin.y + SWATCH_PAD;
        for (_, rect) in &mut self.swatches {
            *rect = RectF::new(Point2D::new(x, y), Size2D::new(SWATCH_SIZE, SWATCH_SIZE));
            y += SWATCH_SIZE + SWATCH_PAD;
        }
    }
}

impl Widget for Palette {
    fn id(&self) -> u64 { self.id }

    fn layout(&mut self, bounds: RectF, theme: &Theme) -> RectF {
        self.rect = RectF::new(bounds.origin, Size2D::new(theme.palette_width, bounds.size.height));
        self.layout_swatches();
        self.rect
    }

    fn handle_event(&mut self, event: &WindowEvent, pos: Pos) -> bool {
        if let WindowEvent::MouseInput { state: ElementState::Pressed, button: MouseButton::Left, .. } = event {
            for (color, rect) in &self.swatches {
                if rect.contains(pos) {
                    self.clicked_color = Some(*color);
                    return true;
                }
            }
        }
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
                Point2D::new(self.rect.max_x() - 1.0, self.rect.origin.y),
                Size2D::new(1.0, self.rect.size.height),
            ),
            color: [0.16, 0.18, 0.22, 1.0],
            corner_radius: 0.0,
        });
        // Draw swatches
        for (color, rect) in &self.swatches {
            let c = [color[0] / 255.0, color[1] / 255.0, color[2] / 255.0, 1.0];
            frame.rects.push(DrawRect {
                rect: *rect,
                color: c,
                corner_radius: 4.0,
            });
        }
    }
}
