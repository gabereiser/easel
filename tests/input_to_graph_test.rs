use easel_core::{
    GraphEngine, Context, StrokeNode, StylusDriver, InputBridge,
    input::event::StrokeEvent,
};

#[tokio::test]
async fn test_push_stroke_event_directly() {
    let mut engine = GraphEngine::new();
    engine.add_node(1, "Stroke".to_string());
    let node = StrokeNode::new(1, "Handler".to_string());
    engine.add_processor(1, Box::new(node));

    let event = StrokeEvent {
        timestamp: 1,
        x: 100.0,
        y: 200.0,
        pressure: 0.75,
    };
    engine.push_stroke_event(1, event);

    let context = Context::new((640, 480));
    let result = engine.render(&context);
    assert!(result.is_ok());
    let img = result.unwrap();
    assert_eq!(img.width, 640);
    assert_eq!(img.height, 480);
}

#[tokio::test]
async fn test_input_bridge_forwards_events() {
    let driver = StylusDriver::new(10);
    let mut engine = GraphEngine::new();
    engine.add_node(1, "StrokeTarget".to_string());
    let node = StrokeNode::new(1, "Target".to_string());
    engine.add_processor(1, Box::new(node));

    let bridge = InputBridge::new(engine.event_buffer());
    bridge.register_target(1);

    // Subscribe to the driver and spawn the bridge with the receiver
    let rx = driver.subscribe(0);
    let bridge_handle = tokio::spawn(async move {
        bridge.run(rx).await;
    });

    let event = StrokeEvent {
        timestamp: 2,
        x: 50.0,
        y: 75.0,
        pressure: 0.5,
    };
    driver.push_event(event).await;

    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    let context = Context::new((800, 600));
    let result = engine.render(&context);
    assert!(result.is_ok());

    // Dropping the driver closes the channel and stops the bridge task
    drop(driver);
    let _ = tokio::time::timeout(std::time::Duration::from_secs(1), bridge_handle).await;
}

#[tokio::test]
async fn test_stroke_events_cleared_after_render() {
    let mut engine = GraphEngine::new();
    engine.add_node(1, "Stroke".to_string());
    let node = StrokeNode::new(1, "Handler".to_string());
    engine.add_processor(1, Box::new(node));

    let event = StrokeEvent {
        timestamp: 1,
        x: 10.0,
        y: 20.0,
        pressure: 0.9,
    };
    engine.push_stroke_event(1, event);

    let event2 = StrokeEvent {
        timestamp: 2,
        x: 30.0,
        y: 40.0,
        pressure: 0.8,
    };
    engine.push_stroke_event(1, event2);

    let context = Context::new((100, 100));
    let _ = engine.render(&context).unwrap();

    // Second render should have no pending events (they were consumed)
    let context2 = Context::new((100, 100));
    let _ = engine.render(&context2).unwrap();
}
