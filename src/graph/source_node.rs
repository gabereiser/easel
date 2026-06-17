use crate::core::image::Image;
use crate::graph::{Context, NodeProcessor};

pub struct SourceNode {
    pub id: u64,
    pub pattern: SourcePattern,
}

pub enum SourcePattern {
    Checkerboard { tile_size: u32 },
    Solid { r: f32, g: f32, b: f32 },
    Gradient { angle_deg: f32 },
}

impl SourceNode {
    pub fn new(id: u64) -> Self {
        Self { id, pattern: SourcePattern::Checkerboard { tile_size: 32 } }
    }
}

impl NodeProcessor for SourceNode {
    fn process(&self, _inputs: &[&Image], context: &Context) -> Result<Image, String> {
        let (w, h) = (context.current_viewport_size.0, context.current_viewport_size.1);
        let mut img = Image::new(w, h);

        match self.pattern {
            SourcePattern::Checkerboard { tile_size } => {
                let ts = tile_size.max(1);
                for y in 0..h {
                    for x in 0..w {
                        let cell_x = x / ts;
                        let cell_y = y / ts;
                        let is_light = (cell_x + cell_y) % 2 == 0;
                        let val = if is_light { 200.0 } else { 55.0 };
                        let idx = (y * w + x) as usize * 3;
                        img.data[idx] = val;
                        img.data[idx + 1] = val;
                        img.data[idx + 2] = val;
                    }
                }
            }
            SourcePattern::Solid { r, g, b } => {
                for pixel in img.data.chunks_exact_mut(3) {
                    pixel[0] = r.clamp(0.0, 255.0);
                    pixel[1] = g.clamp(0.0, 255.0);
                    pixel[2] = b.clamp(0.0, 255.0);
                }
            }
            SourcePattern::Gradient { angle_deg } => {
                let rad = angle_deg.to_radians();
                let (sin_a, cos_a) = rad.sin_cos();
                let cx = w as f32 / 2.0;
                let cy = h as f32 / 2.0;
                let max_dist = (cx * cx + cy * cy).sqrt();
                for y in 0..h {
                    for x in 0..w {
                        let dx = x as f32 - cx;
                        let dy = y as f32 - cy;
                        let proj = dx * cos_a + dy * sin_a;
                        let t = ((proj / max_dist) + 1.0) * 0.5;
                        let val = (t * 255.0).clamp(0.0, 255.0);
                        let idx = (y * w + x) as usize * 3;
                        img.data[idx] = val;
                        img.data[idx + 1] = val;
                        img.data[idx + 2] = val;
                    }
                }
            }
        }

        Ok(img)
    }

    fn name(&self) -> &str {
        "SourceNode"
    }
}