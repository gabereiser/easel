use crate::core::image::Image;
use crate::core::brush_engine::{BrushEngine, BrushNozzle};
use crate::graph::{Context, NodeProcessor};
use std::sync::{Arc, Mutex};

/// A node that paints stroke events onto a persistent canvas using a BrushEngine.
pub struct PaintingNode {
    pub id: u64,
    pub name: String,
    pub brush_engine: BrushEngine,
    canvas: Arc<Mutex<Image>>,
}

impl PaintingNode {
    pub fn new(id: u64, name: &str, viewport: (u32, u32)) -> Self {
        Self {
            id,
            name: name.to_string(),
            brush_engine: BrushEngine::new(10.0, 1.0, 1.0, 1.0),
            canvas: Arc::new(Mutex::new(Image::new(viewport.0, viewport.1))),
        }
    }

    pub fn canvas(&self) -> Arc<Mutex<Image>> {
        self.canvas.clone()
    }

    /// Replace the brush nozzle (tip shape).
    pub fn set_nozzle(&mut self, nozzle: Box<dyn BrushNozzle>) {
        self.brush_engine.nozzle = nozzle;
    }

    /// Replace brush color (RGB, 0–255 range).
    pub fn set_color(&mut self, rgb: [f32; 3]) {
        self.brush_engine.color = rgb;
    }
}

impl NodeProcessor for PaintingNode {
    fn process(&self, _inputs: &[&Image], context: &Context) -> Result<Image, String> {
        let strokes = &context.pending_strokes;
        if !strokes.is_empty() {
            let mut canvas = self.canvas.lock().unwrap();
            self.brush_engine.apply(strokes, &mut canvas);
        }
        Ok(self.canvas.lock().unwrap().clone())
    }

    fn name(&self) -> &str {
        &self.name
    }
}