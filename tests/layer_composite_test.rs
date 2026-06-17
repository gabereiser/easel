use easel_core::{
    core::image::Image,
    core::project::{Layer, LayerContent},
    graph::NodeProcessor,
    GraphEngine, Context, LayerNode, CompositeNode, BlendMode,
};

#[test]
fn test_layer_node_outputs_raster() {
    let mut img = Image::new(10, 10);
    img.set_pixel(3, 4, [100.0, 150.0, 200.0]);

    let layer = Layer {
        id: 1,
        name: "TestLayer".to_string(),
        content: LayerContent::Raster(img.clone()),
        visible: true,
        opacity: 1.0,
        engine_id: None,
    };

    let node = LayerNode::new(1, layer);
    let context = Context::new((10, 10));
    let result = node.process(&[], &context).unwrap();

    let pixel = result.get_pixel(3, 4).unwrap();
    assert_eq!(pixel, [100.0, 150.0, 200.0]);
}

#[test]
fn test_composite_node_passthrough_single_input() {
    let mut engine = GraphEngine::new();
    engine.add_node(1, "Source".to_string());
    engine.add_node(2, "Composite".to_string());

    let mut src_img = Image::new(4, 4);
    src_img.set_pixel(0, 0, [200.0, 0.0, 0.0]);
    let source = easel_core::SourceNode::new(1);
    let composite = CompositeNode::new(2, "PassThrough", BlendMode::Normal);

    engine.add_processor(1, Box::new(source));
    engine.add_processor(2, Box::new(composite));
    engine.add_edge(1, 0, 2, 0);

    let context = Context::new((4, 4));
    // SourceNode always creates blank image, so composite pass-through gives blank
    let result = engine.render(&context).unwrap();
    assert_eq!(result.width, 4);
    assert_eq!(result.height, 4);
}

#[test]
fn test_composite_normal_blend() {
    let mut engine = GraphEngine::new();
    engine.add_node(1, "Bottom".to_string());
    engine.add_node(2, "Top".to_string());
    engine.add_node(3, "Composite".to_string());

    // LayerNode for bottom image (solid 100 green)
    let bottom_img = {
        let mut img = Image::new(2, 2);
        img.data.iter_mut().for_each(|v| *v = 0.0);
        for i in (0..12).step_by(3) {
            img.data[i + 1] = 100.0;
        }
        img
    };
    let custom_bottom = Layer {
        id: 1,
        name: "Bottom".to_string(),
        content: LayerContent::Raster(bottom_img),
        visible: true,
        opacity: 1.0,
        engine_id: None,
    };
    engine.add_processor(1, Box::new(LayerNode::new(1, custom_bottom)));

    // LayerNode for top image (solid 200 red)
    let top_img = {
        let mut img = Image::new(2, 2);
        for i in (0..12).step_by(3) {
            img.data[i] = 200.0;
        }
        img
    };
    let custom_top = Layer {
        id: 2,
        name: "Top".to_string(),
        content: LayerContent::Raster(top_img),
        visible: true,
        opacity: 1.0,
        engine_id: None,
    };
    engine.add_processor(2, Box::new(LayerNode::new(2, custom_top)));

    // Normal blend: top over bottom
    engine.add_processor(3, Box::new(CompositeNode::new(3, "Normal", BlendMode::Normal)));
    engine.add_edge(1, 0, 3, 0);
    engine.add_edge(2, 0, 3, 1);

    let context = Context::new((2, 2));
    let result = engine.render(&context).unwrap();

    // Normal = top wins
    let pixel = result.get_pixel(0, 0).unwrap();
    assert_eq!(pixel[0], 200.0, "normal blend should show top red");
    assert_eq!(pixel[1], 0.0, "normal blend should not show bottom green");
}

#[test]
fn test_composite_multiply_blend() {
    let mut bottom = Image::new(4, 4);
    let mut top = Image::new(4, 4);

    // Bottom: solid 128 gray
    for p in bottom.data.chunks_exact_mut(3) {
        p[0] = 128.0; p[1] = 128.0; p[2] = 128.0;
    }
    // Top: solid 200 gray
    for p in top.data.chunks_exact_mut(3) {
        p[0] = 200.0; p[1] = 200.0; p[2] = 200.0;
    }

    let comp = CompositeNode::new(1, "Multiply", BlendMode::Multiply);
    let context = Context::new((4, 4));

    // Multiply result should be (128 * 200 / 255) ≈ 100.4
    let result = comp.process(&[&bottom, &top], &context).unwrap();
    let pixel = result.get_pixel(0, 0).unwrap();
    let expected = 128.0 * 200.0 / 255.0;
    assert!((pixel[0] - expected).abs() < 1.0, "multiply: expected {:.1}, got {:.1}", expected, pixel[0]);
}

#[test]
fn test_adjustment_node_with_mask() {
    use easel_core::graph::adjustment_node::AdjustmentNode;

    // Source: solid 100 gray
    let mut src = Image::new(2, 2);
    for p in src.data.chunks_exact_mut(3) {
        p[0] = 100.0; p[1] = 100.0; p[2] = 100.0;
    }

    // Mask: white at (0,0), black elsewhere
    let mut mask = Image::new(2, 2);
    mask.set_pixel(0, 0, [255.0, 255.0, 255.0]);

    let node = AdjustmentNode { brightness: 1.0, contrast: 1.0, ..AdjustmentNode::new(1) };
    let context = Context::new((2, 2));

    // Without mask: fully adjusted
    let no_mask = node.process(&[&src], &context).unwrap();
    let p = no_mask.get_pixel(0, 0).unwrap();
    assert!(p[0] > 100.0, "without mask, pixel should be brightened");

    // With mask: (0,0) is white → fully adjusted, (1,0) is black → unchanged
    let with_mask = node.process(&[&src, &mask], &context).unwrap();
    let masked = with_mask.get_pixel(0, 0).unwrap();
    assert!(masked[0] > 100.0, "masked pixel should be adjusted");
    let unmasked = with_mask.get_pixel(1, 0).unwrap();
    assert!((unmasked[0] - 100.0).abs() < 1.0, "unmasked pixel should be original 100, got {}", unmasked[0]);
}
