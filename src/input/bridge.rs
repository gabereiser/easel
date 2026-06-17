use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;
use crate::graph::EventBuffer;
use crate::input::event::StrokeEvent;

/// Bridges stylus input events into the graph engine's event buffer.
/// Subscribes to a StylusDriver and forwards events to registered target nodes.
pub struct InputBridge {
    target_nodes: Arc<Mutex<Vec<u64>>>,
    event_buffer: EventBuffer,
}

impl InputBridge {
    pub fn new(event_buffer: EventBuffer) -> Self {
        Self {
            target_nodes: Arc::new(Mutex::new(Vec::new())),
            event_buffer,
        }
    }

    /// Register a node ID as a target for stroke events.
    pub fn register_target(&self, node_id: u64) {
        self.target_nodes.lock().unwrap().push(node_id);
    }

    /// Forward events from the given mpsc receiver to all registered target nodes.
    /// Runs until the channel is closed. Spawn this as a background task.
    pub async fn run(&self, mut rx: mpsc::UnboundedReceiver<StrokeEvent>) {
        println!("InputBridge running...");
        loop {
            match rx.recv().await {
                Some(event) => {
                    let targets = self.target_nodes.lock().unwrap().clone();
                    for node_id in &targets {
                        let mut buf = self.event_buffer.lock().unwrap();
                        buf.entry(*node_id).or_default().push(event.clone());
                    }
                }
                None => {
                    println!("InputBridge: input channel closed, exiting.");
                    break;
                }
            }
        }
    }
}