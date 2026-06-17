use easel_core::{EaselProject, core::image::Image, Renderer};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a new Easel project
    let project = EaselProject::new("My Artwork".to_string(), "Artist".to_string());
    project.save("my_artwork.easel")?;
    
    println!("Created and saved Easel project: my_artwork.easel");
    
    // Load the project
    let loaded_project = EaselProject::load("my_artwork.easel")?;
    println!("Loaded project: {} by {}", loaded_project.name, loaded_project.author);
    
    // Run the basic renderer demo
    let renderer = Renderer::new().await?;
    let mut img = Image::new(640, 480);
    renderer.execute_render(&mut img)?;
    renderer.read_back_texture()?;
    
    println!("Executed basic render pipeline successfully");
    Ok(())
}