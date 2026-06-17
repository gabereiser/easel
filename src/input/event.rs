// src/input/event.rs

/// Represents a single recorded input point from the stylus or drawing device.
#[derive(Debug, Clone)]
pub struct StrokeEvent {
    pub timestamp: u64,
    pub x: f32,
    pub y: f32,
    pub pressure: f32, // Pressure applied by the pen (0.0 to 1.0)
}