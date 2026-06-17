use crate::core::image::Image;
use crate::graph::{Context, NodeProcessor};

/// Standard compositing blend modes.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BlendMode {
    Normal,
    Multiply,
    Screen,
    Overlay,
}

pub struct CompositeNode {
    pub id: u64,
    pub name: String,
    pub mode: BlendMode,
}

impl CompositeNode {
    pub fn new(id: u64, name: &str, mode: BlendMode) -> Self {
        Self { id, name: name.to_string(), mode }
    }
}

impl NodeProcessor for CompositeNode {
    fn process(&self, inputs: &[&Image], _context: &Context) -> Result<Image, String> {
        let bottom = inputs.first().ok_or("Composite node requires at least one input")?;
        if inputs.len() < 2 {
            // Single input — pass through
            return Ok((*bottom).clone());
        }
        let top = &inputs[1];
        let mut result = (*bottom).clone();

        let len = result.data.len().min(top.data.len());
        for i in (0..len).step_by(3) {
            let b = [bottom.data[i], bottom.data[i + 1], bottom.data[i + 2]];
            let t = [top.data[i], top.data[i + 1], top.data[i + 2]];

            let blended = match self.mode {
                BlendMode::Normal => t,
                BlendMode::Multiply => {
                    [b[0] * t[0] / 255.0,
                     b[1] * t[1] / 255.0,
                     b[2] * t[2] / 255.0]
                }
                BlendMode::Screen => {
                    [255.0 - (255.0 - b[0]) * (255.0 - t[0]) / 255.0,
                     255.0 - (255.0 - b[1]) * (255.0 - t[1]) / 255.0,
                     255.0 - (255.0 - b[2]) * (255.0 - t[2]) / 255.0]
                }
                BlendMode::Overlay => {
                    [overlay_channel(b[0], t[0]),
                     overlay_channel(b[1], t[1]),
                     overlay_channel(b[2], t[2])]
                }
            };

            result.data[i] = blended[0].clamp(0.0, 255.0);
            result.data[i + 1] = blended[1].clamp(0.0, 255.0);
            result.data[i + 2] = blended[2].clamp(0.0, 255.0);
        }

        Ok(result)
    }

    fn name(&self) -> &str {
        &self.name
    }
}

fn overlay_channel(bottom: f32, top: f32) -> f32 {
    let bn = bottom / 255.0;
    let tn = top / 255.0;
    let result = if bn < 0.5 {
        2.0 * bn * tn
    } else {
        1.0 - 2.0 * (1.0 - bn) * (1.0 - tn)
    };
    result * 255.0
}
