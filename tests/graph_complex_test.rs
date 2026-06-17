use easel_core::{GraphEngine, Context};
use easel_core::graph::{source_node::SourceNode, adjustment_node::AdjustmentNode, stroke_node::StrokeNode};

#[tokio::test]
async fn test_complex_graph() {
    let mut engine = GraphEngine::new();
    
    // 1. Source Node (ID: 1) -> Provides the base image
    engine.add_node(1, "Source".to_string());
    engine.add_processor(1, Box::new(SourceNode::new(1)));

    // 2. Adjustment Node (ID: 2) -> Takes input from Source Node's output pin 0 to its own input pin 0
    engine.add_node(2, "Adjust".to_string());
    engine.add_processor(2, Box::new(AdjustmentNode::new(2)));
    engine.add_edge(1, 0, 2, 0);

    // 3. Stroke Node (ID: 3) -> Handles stroke events independently
    engine.add_node(3, "Stroke".to_string());
    let mut s_node = StrokeNode::new(3, "StrokeHandler".to_string());
    s_node.add_event(easel_core::input::event::StrokeEvent {
        timestamp: 100,
        x: 10.0,
        y: 20.0,
        pressure: 0.5,
    });
    engine.add_processor(3, Box::new(s_node));

    let context = Context::new((1920, 1080));

    // Test render for the image path (Source -> Adjustment)
    let result = engine.render(&context).unwrap();
    assert_eq!(result.width, 1920);
    assert_eq!(result.height, 1080);
}