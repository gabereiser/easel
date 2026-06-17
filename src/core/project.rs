use crate::core::image::Image;
use serde::{Deserialize, Serialize};

/// Top-level project container for Easel documents.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EaselProject {
    pub version: String,
    pub name: String,
    pub author: String,
    pub canvas_width: u32,
    pub canvas_height: u32,
    pub layers: Vec<Layer>,
}

impl EaselProject {
    pub fn new(name: String, author: String) -> Self {
        Self {
            version: env!("CARGO_PKG_VERSION").to_string(),
            name,
            author,
            canvas_width: 1920,
            canvas_height: 1080,
            layers: Vec::new(),
        }
    }

    pub fn set_canvas_size(&mut self, width: u32, height: u32) {
        self.canvas_width = width;
        self.canvas_height = height;
    }

    pub fn add_layer(&mut self, layer: Layer) {
        self.layers.push(layer);
    }

    pub fn save(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let file = std::fs::File::create(path)?;
        serde_json::to_writer_pretty(file, self)?;
        Ok(())
    }

    pub fn load(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let file = std::fs::File::open(path)?;
        let project: EaselProject = serde_json::from_reader(file)?;
        Ok(project)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LayerContent {
    /// A standard image buffer (raster data)
    Raster(Image),
    /// A node graph that produces an image result
    GraphReference(u64), // ID of the root node in a GraphEngine
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Layer {
    pub id: u64,
    pub name: String,
    pub content: LayerContent,
    pub visible: bool,
    pub opacity: f32,
    /// Optional engine identifier if this layer references a graph
    pub engine_id: Option<String>,
}

impl Layer {
    pub fn new(name: &str) -> Self {
        Self {
            id: 0,
            name: name.to_string(),
            content: LayerContent::Raster(Image::new(1920, 1080)),
            visible: true,
            opacity: 1.0,
            engine_id: None,
        }
    }

    pub fn as_raster(&self) -> Option<&Image> {
        match &self.content {
            LayerContent::Raster(img) => Some(img),
            _ => None,
        }
    }
}