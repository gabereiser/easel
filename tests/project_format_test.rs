use easel_core::{EaselProject, Layer, LayerContent, Image};

#[test]
fn test_project_new_defaults() {
    let project = EaselProject::new("Test".to_string(), "Tester".to_string());
    assert_eq!(project.name, "Test");
    assert_eq!(project.author, "Tester");
    assert_eq!(project.canvas_width, 1920);
    assert_eq!(project.canvas_height, 1080);
    assert!(project.layers.is_empty());
}

#[test]
fn test_project_add_layer() {
    let mut project = EaselProject::new("Test".to_string(), "Tester".to_string());
    let layer = Layer::new("Background");
    project.add_layer(layer);
    assert_eq!(project.layers.len(), 1);
    assert_eq!(project.layers[0].name, "Background");
}

#[test]
fn test_project_save_load_roundtrip() {
    let tmp = std::env::temp_dir().join("test_project.easel");
    let path = tmp.to_str().unwrap();

    {
        let mut project = EaselProject::new("MyArt".to_string(), "Artist".to_string());
        project.set_canvas_size(256, 256);
        let mut layer = Layer::new("Layer 1");
        // Create a layer with a small image containing non-zero data
        let img = Image::new(256, 256);
        layer.content = LayerContent::Raster(img);
        project.add_layer(layer);
        project.save(path).expect("save should succeed");
    }

    {
        let loaded = EaselProject::load(path).expect("load should succeed");
        assert_eq!(loaded.name, "MyArt");
        assert_eq!(loaded.author, "Artist");
        assert_eq!(loaded.canvas_width, 256);
        assert_eq!(loaded.canvas_height, 256);
        assert_eq!(loaded.layers.len(), 1);
        assert_eq!(loaded.layers[0].name, "Layer 1");
        assert!(matches!(loaded.layers[0].content, LayerContent::Raster(_)));
    }

    let _ = std::fs::remove_file(&tmp);
}

#[test]
fn test_project_save_load_empty() {
    let tmp = std::env::temp_dir().join("test_project_empty.easel");
    let path = tmp.to_str().unwrap();

    {
        let project = EaselProject::new("Empty".to_string(), "Nobody".to_string());
        project.save(path).expect("save should succeed");
    }

    {
        let loaded = EaselProject::load(path).expect("load should succeed");
        assert_eq!(loaded.name, "Empty");
        assert!(loaded.layers.is_empty());
    }

    let _ = std::fs::remove_file(&tmp);
}
