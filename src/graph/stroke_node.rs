use crate::core::image::Image;
use crate::core::brush_engine::BrushEngine;
use crate::input::event::StrokeEvent;
use crate::graph::{Context, NodeProcessor};

/// Renders stroke events onto a canvas using a BrushEngine.
/// Stroke events are delivered via `Context.pending_strokes` during
/// `GraphEngine::render()`. The internal `events` buffer is kept for
/// direct programmatic usage (e.g. tests that bypass the event buffer).
pub struct StrokeNode {
    pub id: u64,
    pub name: String,
    pub events: Vec<StrokeEvent>,
    brush_engine: BrushEngine,
}

impl StrokeNode {
    pub fn new(id: u64, name: String) -> Self {
        Self {
            id,
            name,
            events: Vec::new(),
            brush_engine: BrushEngine::new(10.0, 1.0, 1.0, 1.0),
        }
    }

    pub fn add_event(&mut self, event: StrokeEvent) {
        self.events.push(event);
    }

    pub fn clear_events(&mut self) {
        self.events.clear();
    }
}

impl NodeProcessor for StrokeNode {
    fn process(&self, _inputs: &[&Image], context: &Context) -> Result<Image, String> {
        let (w, h) = (context.current_viewport_size.0, context.current_viewport_size.1);
        let mut result = Image::new(w, h);

        let all_events: Vec<&StrokeEvent> = context.pending_strokes.iter().chain(self.events.iter()).collect();
        if !all_events.is_empty() {
            let borrowed: Vec<StrokeEvent> = all_events.into_iter().cloned().collect();
            self.brush_engine.apply(&borrowed, &mut result);
        }

        Ok(result)
    }

    fn name(&self) -> &str {
        &self.name
    }
}