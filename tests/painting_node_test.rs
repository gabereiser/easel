use easel_core::{
    GraphEngine, Context, PaintingNode,
    input::event::StrokeEvent,
};

#[tokio::test]
async fn test_painting_node_applies_strokes() {
    let mut engine = GraphEngine::new();
    engine.add_node(1, "Painter".to_string());
    let node = PaintingNode::new(1, "Painter", (64, 64));
    engine.add_processor(1, Box::new(node));

    let event = StrokeEvent {
        timestamp: 1,
        x: 10.0,
        y: 20.0,
        pressure: 0.8,
    };
    engine.push_stroke_event(1, event);

    let context = Context::new((64, 64));
    let result = engine.render(&context);
    assert!(result.is_ok(), "render failed: {:?}", result.err());

    let output = result.unwrap();
    let pixel = output.get_pixel(10, 20).unwrap();
    assert!(
        pixel[0] > 0.0,
        "expected non-zero red at stroke position, got {:?}",
        pixel
    );
}

#[tokio::test]
async fn test_painting_node_no_strokes_returns_blank() {
    let mut engine = GraphEngine::new();
    engine.add_node(1, "Painter".to_string());
    let node = PaintingNode::new(1, "Painter", (16, 16));
    engine.add_processor(1, Box::new(node));

    let context = Context::new((16, 16));
    let result = engine.render(&context);
    assert!(result.is_ok());

    let output = result.unwrap();
    for val in &output.data {
        assert_eq!(*val, 0.0, "expected blank canvas without strokes");
    }
}

#[tokio::test]
async fn test_painting_node_accumulates_strokes() {
    let mut engine = GraphEngine::new();
    engine.add_node(1, "Painter".to_string());
    let node = PaintingNode::new(1, "Painter", (32, 32));
    engine.add_processor(1, Box::new(node));

    engine.push_stroke_event(1, StrokeEvent {
        timestamp: 1, x: 5.0, y: 5.0, pressure: 0.5,
    });
    let context = Context::new((32, 32));
    let _ = engine.render(&context).unwrap();

    engine.push_stroke_event(1, StrokeEvent {
        timestamp: 2, x: 10.0, y: 10.0, pressure: 0.5,
    });
    let output = engine.render(&context).unwrap();

    assert!(output.get_pixel(5, 5).unwrap()[0] > 0.0, "first stroke missing");
    assert!(output.get_pixel(10, 10).unwrap()[0] > 0.0, "second stroke missing");
}
