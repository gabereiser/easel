use crate::core::image::Image;
use crate::input::event::StrokeEvent;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub mod stroke_node;
pub mod source_node;
pub mod adjustment_node;
pub mod painting_node;
pub mod layer_node;
pub mod composite_node;

#[derive(Debug, Clone)]
pub struct Pin {
    pub name: String,
    pub data_type: String,
}

#[derive(Debug, Clone)]
pub struct Node {
    pub id: u64,
    pub name: String,
    pub inputs: Vec<Pin>,
    pub outputs: Vec<Pin>,
}

/// Per-node execution context passed to every NodeProcessor::process call.
/// Contains viewport info and any pending stroke events for this node.
pub struct Context {
    pub current_viewport_size: (u32, u32),
    pub pending_strokes: Vec<StrokeEvent>,
}

impl Context {
    pub fn new(viewport_size: (u32, u32)) -> Self {
        Self {
            current_viewport_size: viewport_size,
            pending_strokes: Vec::new(),
        }
    }
}

pub trait NodeProcessor: Send + Sync {
    fn process(&self, inputs: &[&Image], context: &Context) -> Result<Image, String>;
    fn name(&self) -> &str;
}

/// Shared event buffer used to pipe stylus events into the graph engine.
pub type EventBuffer = Arc<Mutex<HashMap<u64, Vec<StrokeEvent>>>>;

pub struct GraphEngine {
    nodes: HashMap<u64, Node>,
    processors: HashMap<u64, Box<dyn NodeProcessor>>,
    edges: Vec<(u64, usize, u64, usize)>,
    /// Holds pending stroke events keyed by target node ID.
    /// Populated externally by InputBridge, consumed during render().
    event_buffer: EventBuffer,
}

impl Default for GraphEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl GraphEngine {
    pub fn new() -> Self {
        GraphEngine {
            nodes: HashMap::new(),
            processors: HashMap::new(),
            edges: Vec::new(),
            event_buffer: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Return a clone of the internal event buffer so external components
    /// (e.g. InputBridge) can push stroke events into the graph.
    pub fn event_buffer(&self) -> EventBuffer {
        self.event_buffer.clone()
    }

    /// Push a stroke event targeting a specific node.
    pub fn push_stroke_event(&self, node_id: u64, event: StrokeEvent) {
        let mut buf = self.event_buffer.lock().unwrap();
        buf.entry(node_id).or_default().push(event);
    }

    /// Drain and return all pending stroke events for a given node.
    pub fn take_stroke_events(&self, node_id: u64) -> Vec<StrokeEvent> {
        let mut buf = self.event_buffer.lock().unwrap();
        buf.remove(&node_id).unwrap_or_default()
    }

    pub fn add_node(&mut self, id: u64, name: String) {
        self.nodes.insert(id, Node {
            id,
            name,
            inputs: Vec::new(),
            outputs: Vec::new(),
        });
    }

    pub fn add_processor(&mut self, id: u64, processor: Box<dyn NodeProcessor>) {
        self.processors.insert(id, processor);
    }

    pub fn add_edge(&mut self, from_node: u64, from_output: usize, to_node: u64, to_input: usize) {
        if let Some(target_node) = self.nodes.get_mut(&to_node) {
            while target_node.inputs.len() <= to_input {
                target_node.inputs.push(Pin {
                    name: format!("input_{}", target_node.inputs.len()),
                    data_type: "Image".to_string(), 
                });
            }
        }

        if let Some(source_node) = self.nodes.get_mut(&from_node) {
            while source_node.outputs.len() <= from_output {
                source_node.outputs.push(Pin {
                    name: format!("output_{}", source_node.outputs.len()),
                    data_type: "Image".to_string(), 
                });
            }
        }

        self.edges.push((from_node, from_output, to_node, to_input));
    }

    pub fn resolve_order(&self) -> Result<Vec<u64>, String> {
        let mut order = Vec::new();
        let mut in_degree: HashMap<u64, usize> = HashMap::new();
        let mut adj: HashMap<u64, Vec<u64>> = HashMap::new();

        for &node_id in self.nodes.keys() {
            in_degree.insert(node_id, 0);
        }

        for &(from_node, _, to_node, _) in &self.edges {
            adj.entry(from_node).or_default().push(to_node);
            *in_degree.entry(to_node).or_insert(0) += 1;
        }

        let mut stack: Vec<u64> = in_degree
            .iter()
            .filter(|&(_, &deg)| deg == 0)
            .map(|(&id, _)| id)
            .collect();

        while let Some(u) = stack.pop() {
            order.push(u);
            if let Some(neighbors) = adj.get(&u) {
                for &v in neighbors {
                    let degree = in_degree.get_mut(&v).unwrap();
                    *degree -= 1;
                    if *degree == 0 {
                        stack.push(v);
                    }
                }
            }
        }

        if order.len() != self.nodes.len() {
            return Err("Cycle detected in graph or unreachable nodes".to_string());
        }

        Ok(order)
    }

    pub fn render(&self, context: &Context) -> Result<Image, String> {
        println!("Starting graph resolution...");
        
        let order = self.resolve_order()?;
        let mut image_cache: HashMap<u64, Image> = HashMap::new();
        
        for &node_id in &order {
            let node = self.nodes.get(&node_id)
                .ok_or_else(|| format!("Node {} not found", node_id))?;
            let processor = self.processors.get(&node.id)
                .ok_or_else(|| format!("No processor registered for node {}", node.id))?;
        
            let mut input_images: Vec<&Image> = Vec::new();
            for (from_node, _f_out, to_node, t_in) in &self.edges {
                if *to_node == node_id && *t_in < node.inputs.len() {
                    match image_cache.get(from_node) {
                        Some(img) => input_images.push(img),
                        None => return Err(format!("Missing dependency for node {} from node {}", node_id, from_node)),
                    }
                }
            }

            // Build per-node context with pending stroke events
            let node_context = Context {
                current_viewport_size: context.current_viewport_size,
                pending_strokes: self.take_stroke_events(node_id),
            };

            println!("Processing Node: {}...", processor.name());
            let result = processor.process(&input_images, &node_context)?;
            image_cache.insert(node.id, result);
        }
        
        // Return the output of the last node, or a blank image for empty graphs
        match order.last() {
            Some(last_node_id) => {
                image_cache.remove(last_node_id)
                    .ok_or_else(|| format!("No output produced for node {}", last_node_id))
            }
            None => Ok(Image::new(
                context.current_viewport_size.0,
                context.current_viewport_size.1,
            )),
        }
    }

    /// Number of nodes in the graph.
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }
}
