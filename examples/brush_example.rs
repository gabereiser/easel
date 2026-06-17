use easel_core::{BrushEngine, StrokeEvent, Image};

fn main() {
    // Create a brush engine
    let brush_engine = BrushEngine::new(25.0, 1.0, 0.8, 0.7);
    println!("Created brush engine with size: {}, opacity: {}, flow: {}, hardness: {}", 
            brush_engine.brush_size, brush_engine.opacity, brush_engine.flow, brush_engine.hardness);
    
    // Create a simple image
    let image = Image::new(100, 100);
    println!("Created image with dimensions: {}x{}", image.width, image.height);
    
    // Process some stroke events with the brush engine
    let events = vec![
        StrokeEvent { timestamp: 100, x: 50.0, y: 50.0, pressure: 0.8 },
        StrokeEvent { timestamp: 101, x: 55.0, y: 55.0, pressure: 0.9 },
    ];
    
    let canvas = Image::new(100, 100);
    let result = brush_engine.process_stroke(&events, &canvas);
    println!("Processed stroke with brush engine, result image size: {}x{}", result.width, result.height);
    
    println!("Brush engine example completed successfully!");
}