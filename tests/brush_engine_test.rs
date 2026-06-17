use easel_core::core::image::Image;
use easel_core::input::event::StrokeEvent;
use easel_core::core::brush_engine::{
    BrushEngine, BrushConfig,
    BrushNozzle, NozzleCtx,
    EllipseNozzle, StarNozzle, DiamondNozzle, TextureNozzle,
};

#[tokio::test]
async fn test_brush_engine_application() {
    let engine = BrushEngine::new(10.0, 1.0, 1.0, 1.0);
    let config = BrushConfig::new("Standard", 10.0, 1.0, 1.0, 1.0);
    let mut canvas = Image::new(256, 256);

    let events = vec![StrokeEvent {
        timestamp: 1000,
        x: 128.0,
        y: 128.0,
        pressure: 0.5,
    }];

    engine.apply_brush(&config, &events, &mut canvas);
}

#[tokio::test]
async fn test_brush_custom_color() {
    let mut engine = BrushEngine::new(10.0, 1.0, 1.0, 1.0);
    engine.color = [0.0, 200.0, 100.0];
    let mut canvas = Image::new(32, 32);

    let events = vec![StrokeEvent { timestamp: 1, x: 16.0, y: 16.0, pressure: 1.0 }];
    engine.apply(&events, &mut canvas);

    let pixel = canvas.get_pixel(16, 16).unwrap();
    assert!((pixel[1] - 200.0).abs() < 55.0, "green channel should be near 200, got {}", pixel[1]);
    assert!((pixel[2] - 100.0).abs() < 55.0, "blue channel should be near 100, got {}", pixel[2]);
}

#[tokio::test]
async fn test_brush_texture_nozzle() {
    let mut tex = Image::new(2, 2);
    tex.set_pixel(0, 0, [255.0, 255.0, 255.0]);

    let mut engine = BrushEngine::new(4.0, 1.0, 1.0, 1.0);
    engine.nozzle = Box::new(TextureNozzle { image: tex });
    let mut canvas = Image::new(16, 16);

    let events = vec![StrokeEvent { timestamp: 1, x: 8.0, y: 8.0, pressure: 1.0 }];
    engine.apply(&events, &mut canvas);

    let pixel = canvas.get_pixel(8, 8).unwrap();
    assert!(pixel[0] > 0.0, "center should have paint from texture");
}

#[test]
fn test_circle_nozzle_paints_center() {
    let engine = BrushEngine::new(20.0, 1.0, 1.0, 1.0);
    let mut canvas = Image::new(64, 64);

    let events = vec![StrokeEvent { timestamp: 1, x: 32.0, y: 32.0, pressure: 1.0 }];
    engine.apply(&events, &mut canvas);

    let pixel = canvas.get_pixel(32, 32).unwrap();
    assert!(pixel[0] > 0.0, "circle nozzle should paint center");
}

#[test]
fn test_ellipse_nozzle_paints_center() {
    let mut engine = BrushEngine::new(20.0, 1.0, 1.0, 1.0);
    engine.nozzle = Box::new(EllipseNozzle { ratio: 2.0 });
    let mut canvas = Image::new(64, 64);

    let events = vec![StrokeEvent { timestamp: 1, x: 32.0, y: 32.0, pressure: 1.0 }];
    engine.apply(&events, &mut canvas);

    let pixel = canvas.get_pixel(32, 32).unwrap();
    assert!(pixel[0] > 0.0, "ellipse nozzle should paint center");
}

#[test]
fn test_star_nozzle_paints_center() {
    let mut engine = BrushEngine::new(20.0, 1.0, 1.0, 1.0);
    engine.nozzle = Box::new(StarNozzle { points: 5, inner_ratio: 0.5, rotation_deg: 0.0 });
    let mut canvas = Image::new(64, 64);

    let events = vec![StrokeEvent { timestamp: 1, x: 32.0, y: 32.0, pressure: 1.0 }];
    engine.apply(&events, &mut canvas);

    let pixel = canvas.get_pixel(32, 32).unwrap();
    assert!(pixel[0] > 0.0, "star nozzle should paint center");
}

#[test]
fn test_diamond_nozzle_paints_center() {
    let mut engine = BrushEngine::new(20.0, 1.0, 1.0, 1.0);
    engine.nozzle = Box::new(DiamondNozzle);
    let mut canvas = Image::new(64, 64);

    let events = vec![StrokeEvent { timestamp: 1, x: 32.0, y: 32.0, pressure: 1.0 }];
    engine.apply(&events, &mut canvas);

    let pixel = canvas.get_pixel(32, 32).unwrap();
    assert!(pixel[0] > 0.0, "diamond nozzle should paint center");
}

#[test]
fn test_custom_nozzle_via_trait() {
    struct CheckerNozzle;
    impl BrushNozzle for CheckerNozzle {
        fn sample(&self, dx: f32, dy: f32, radius: f32, _hardness: f32, _ctx: &NozzleCtx) -> f32 {
            let cell = ((dx / radius * 2.0).floor() as i32 + (dy / radius * 2.0).floor() as i32) % 2;
            if cell == 0 { 1.0 } else { 0.0 }
        }
    }

    let mut engine = BrushEngine::new(32.0, 1.0, 1.0, 1.0);
    engine.nozzle = Box::new(CheckerNozzle);
    let mut canvas = Image::new(64, 64);

    let events = vec![StrokeEvent { timestamp: 1, x: 32.0, y: 32.0, pressure: 1.0 }];
    engine.apply(&events, &mut canvas);

    let pixel = canvas.get_pixel(32, 32).unwrap();
    assert!(pixel[0] > 0.0, "custom nozzle should paint");
}