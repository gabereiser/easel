use easel_core::{core::image::Image, core::image::ImageFormat, core::color_space::{ColorSpace, ColorSpaceConverter, ColorSpaceConverterImpl}, StrokeEvent, StylusDriver, GraphEngine, Context, Renderer};

#[test]
fn test_image_creation() {
    let img = Image::new(1920, 1080);
    assert_eq!(img.width, 1920);
    assert_eq!(img.height, 1080);
    assert_eq!(img.data.len(), 1920 * 1080 * 3);
}

#[test]
fn test_color_space_conversion() {
    let img = Image::new(100, 100);
    let converter = ColorSpaceConverterImpl;
    let converted = img.convert(&converter, ColorSpace::Linear);
    assert_eq!(converted.width, 100);
    assert_eq!(converted.height, 100);
}

#[test]
fn test_color_space_converter_impl() {
    let converter = ColorSpaceConverterImpl;
    let gamut = converter.get_gamut_info();
    assert!(!gamut.is_hdr);
    let result = converter.convert([0.5, 0.25, 0.125], easel_core::core::color_space::ColorSpace::SRGB, easel_core::core::color_space::ColorSpace::Linear);
    assert!(result[0] > 0.0);
}

#[test]
fn test_image_save_png() {
    let tmp = std::env::temp_dir().join("test_image.png");
    let path = tmp.to_str().unwrap();
    let img = Image::new(4, 4);
    let result = img.save(path, ImageFormat::PNG);
    assert!(result.is_ok(), "save PNG failed: {:?}", result.err());
    assert!(tmp.exists(), "saved file should exist");
    let _ = std::fs::remove_file(&tmp);
}

#[test]
fn test_image_save_jpeg() {
    let tmp = std::env::temp_dir().join("test_image.jpeg");
    let path = tmp.to_str().unwrap();
    let img = Image::new(4, 4);
    let result = img.save(path, ImageFormat::JPEG);
    assert!(result.is_ok(), "save JPEG failed: {:?}", result.err());
    assert!(tmp.exists(), "saved file should exist");
    let _ = std::fs::remove_file(&tmp);
}

#[test]
fn test_stroke_event_creation() {
    let event = StrokeEvent {
        timestamp: 1,
        x: 50.0,
        y: 75.0,
        pressure: 0.9,
    };
    assert_eq!(event.timestamp, 1);
    assert_eq!(event.x, 50.0);
}

#[test]
fn test_graph_engine_new() {
    let engine = GraphEngine::new();
    let context = Context::new((1920, 1080));
    let result = engine.render(&context);
    assert!(result.is_ok());
    let img = result.unwrap();
    assert_eq!(img.width, 1920);
    assert_eq!(img.height, 1080);
}

#[tokio::test]
async fn test_renderer_new() {
    let renderer = Renderer::new().await;
    assert!(renderer.is_ok());
}

#[tokio::test]
async fn test_stylus_driver() {
    let driver = StylusDriver::new(10);
    let event = StrokeEvent {
        timestamp: 0,
        x: 10.0,
        y: 20.0,
        pressure: 0.5,
    };
    driver.push_event(event).await;
    tokio::time::sleep(std::time::Duration::from_millis(10)).await;
}

#[tokio::test]
async fn test_phase_one_e2e_pipeline() {
    let mut image = Image::new(1920, 1080);
    let driver = StylusDriver::new(5);
    let event = StrokeEvent {
        timestamp: 1,
        x: 100.0,
        y: 200.0,
        pressure: 0.8,
    };
    driver.push_event(event).await;

    let renderer = Renderer::new().await.unwrap();
    renderer.execute_render(&mut image).unwrap();
    renderer.read_back_texture().unwrap();

    let converter = ColorSpaceConverterImpl;
    let final_image = image.convert(&converter, ColorSpace::SRGB);
    let tmp = std::env::temp_dir().join("test_phase1_output.png");
    let save_result = final_image.save(tmp.to_str().unwrap(), ImageFormat::PNG);
    assert!(save_result.is_ok(), "save PNG failed: {:?}", save_result.err());
    let _ = std::fs::remove_file(&tmp);
}
