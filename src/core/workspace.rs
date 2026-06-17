use crate::core::project::Layer;
use std::collections::HashMap;

pub struct Workspace {
    pub layers: HashMap<u64, Layer>,
}

impl Default for Workspace {
    fn default() -> Self {
        Self::new()
    }
}

impl Workspace {
    pub fn new() -> Self {
        Self {
            layers: HashMap::new(),
        }
    }

    pub fn add_layer(&mut self, layer: Layer) -> u64 {
        let id = (self.layers.len() as u64) + 1; // Simple ID generation for now
        // In a real system, we would use a more robust ID generator
        // Note: This might overwrite IDs if layers are removed from the middle.
        self.layers.insert(id, layer);
        id
    }

    pub fn get_layer(&self, id: u64) -> Option<&Layer> {
        self.layers.get(&id)
    }
}