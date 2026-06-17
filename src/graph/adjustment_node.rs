use crate::core::image::Image;
use crate::graph::{Context, NodeProcessor};

/// Supported adjustment operations.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AdjustmentKind {
    BrightnessContrast,
}

pub struct AdjustmentNode {
    pub id: u64,
    pub brightness: f32,
    pub contrast: f32,
    pub kind: AdjustmentKind,
}

impl AdjustmentNode {
    pub fn new(id: u64) -> Self {
        Self { id, brightness: 0.0, contrast: 1.0, kind: AdjustmentKind::BrightnessContrast }
    }
}

impl AdjustmentNode {
    /// Compute the adjustment for a single pixel value.
    fn adjust_channel(val: f32, brightness: f32, contrast: f32) -> f32 {
        let centered = val - 128.0;
        let contrasted = centered * contrast + 128.0;
        (contrasted + brightness * 255.0).clamp(0.0, 255.0)
    }
}

impl NodeProcessor for AdjustmentNode {
    fn process(&self, inputs: &[&Image], _context: &Context) -> Result<Image, String> {
        let img = inputs.first().ok_or_else(|| "Adjustment node requires at least one input".to_string())?;
        let mut result = (*img).clone();

        // Optional mask at inputs[1]: first channel controls blend factor (0–255).
        let mask = inputs.get(1).copied();

        for (i, pixel) in result.data.chunks_exact_mut(3).enumerate() {
            let orig = [pixel[0], pixel[1], pixel[2]];

            let adj = match self.kind {
                AdjustmentKind::BrightnessContrast => [
                    Self::adjust_channel(orig[0], self.brightness, self.contrast),
                    Self::adjust_channel(orig[1], self.brightness, self.contrast),
                    Self::adjust_channel(orig[2], self.brightness, self.contrast),
                ],
            };

            // Blend with mask
            if let Some(m) = mask {
                let mask_val = m.data.get(i * 3).copied().unwrap_or(255.0).clamp(0.0, 255.0) / 255.0;
                pixel[0] = orig[0] * (1.0 - mask_val) + adj[0] * mask_val;
                pixel[1] = orig[1] * (1.0 - mask_val) + adj[1] * mask_val;
                pixel[2] = orig[2] * (1.0 - mask_val) + adj[2] * mask_val;
            } else {
                pixel[0] = adj[0];
                pixel[1] = adj[1];
                pixel[2] = adj[2];
            }
        }

        Ok(result)
    }

    fn name(&self) -> &str {
        "AdjustmentNode"
    }
}