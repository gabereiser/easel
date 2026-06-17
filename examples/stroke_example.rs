use easel_core::{GraphEngine, StrokeNode, Context, Renderer};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a graph engine
    let mut engine = GraphEngine::new();
    
    // Create a stroke node
    let stroke_node = StrokeNode::new(1, "Test Stroke Node".to_string());
    engine.add_node(1, "Stroke Node".to_string());
    engine.add_processor(1, Box::new(stroke_node));
    
    // Process the graph
    let context = Context::new((1920, 1080));
    
    let result = engine.render(&context);
    println!("Render result: {:?}", result.is_ok());
    
    // Also run the basic renderer demo
    let _renderer = Renderer::new().await?;
    println!("Renderer initialized successfully");
    
    Ok(())
}