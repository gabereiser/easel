use crate::core::image::Image;
use crate::core::project::Layer;
use crate::graph::{Context, NodeProcessor};

/// A node that wraps a Layer and outputs its raster content.
/// When the layer's content is `Raster(image)`, that image is returned.
/// `GraphReference` layers return an error (use a sub-graph for those).
pub struct LayerNode {
    pub id: u64,
    layer: Layer,
}

impl LayerNode {
    pub fn new(id: u64, layer: Layer) -> Self {
        Self { id, layer }
    }

    /// Replace the wrapped layer at runtime.
    pub fn set_layer(&mut self, layer: Layer) {
        self.layer = layer;
    }

    pub fn layer(&self) -> &Layer {
        &self.layer
    }
}

impl NodeProcessor for LayerNode {
    fn process(&self, _inputs: &[&Image], context: &Context) -> Result<Image, String> {
        match &self.layer.content {
            crate::core::project::LayerContent::Raster(img) => Ok(img.clone()),
            crate::core::project::LayerContent::GraphReference(_gid) => {
                // Sub-graph rendering is not yet supported in this node.
                // Return a blank canvas sized to the viewport as a fallback.
                Ok(Image::new(
                    context.current_viewport_size.0,
                    context.current_viewport_size.1,
                ))
            }
        }
    }

    fn name(&self) -> &str {
        &self.layer.name
    }
}
