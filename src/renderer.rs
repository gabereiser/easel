use crate::core::image::Image;
use wgpu;

pub struct Renderer {
    device: wgpu::Device,
    queue: wgpu::Queue,
}

impl Renderer {
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::default());
        let adapter = instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: None,
            force_fallback_adapter: false,
        }).await.ok_or("Failed to find an appropriate adapter")?;
        let (device, queue) = adapter.request_device(&wgpu::DeviceDescriptor {
            label: Some("Easel Renderer Device"),
            required_features: wgpu::Features::empty(),
            required_limits: wgpu::Limits::default(),
        }, None).await?;
        println!("Renderer initialized with wgpu.");
        Ok(Renderer { device, queue })
    }

    pub fn execute_render(&self, image: &mut Image) -> Result<(), Box<dyn std::error::Error>> {
        let w = image.width.max(1);
        let h = image.height.max(1);
        let pixel_count = (w * h) as usize;

        let size = wgpu::Extent3d { width: w, height: h, depth_or_array_layers: 1 };

        let texture = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Easel Render Target"),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });

        let mut u8_data: Vec<u8> = Vec::with_capacity(pixel_count * 4);
        for chunk in image.data.chunks_exact(3) {
            u8_data.push((chunk[0]).clamp(0.0, 255.0) as u8);
            u8_data.push((chunk[1]).clamp(0.0, 255.0) as u8);
            u8_data.push((chunk[2]).clamp(0.0, 255.0) as u8);
            u8_data.push(255);
        }

        let buffer_size = (pixel_count * 4) as u64;
        let staging_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Easel Staging Buffer"),
            size: buffer_size,
            usage: wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::MAP_WRITE,
            mapped_at_creation: true,
        });

        {
            let mut map = staging_buffer.slice(..).get_mapped_range_mut();
            map.copy_from_slice(&u8_data);
        }
        staging_buffer.unmap();

        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Easel Render Encoder"),
        });

        encoder.copy_buffer_to_texture(
            wgpu::ImageCopyBuffer {
                buffer: &staging_buffer,
                layout: wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: Some(4 * w),
                    rows_per_image: Some(h),
                },
            },
            wgpu::ImageCopyTexture {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            size,
        );

        self.queue.submit(Some(encoder.finish()));

        let readback_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Easel Readback Buffer"),
            size: buffer_size,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });

        let mut read_encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Easel Readback Encoder"),
        });

        read_encoder.copy_texture_to_buffer(
            wgpu::ImageCopyTexture {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::ImageCopyBuffer {
                buffer: &readback_buffer,
                layout: wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: Some(4 * w),
                    rows_per_image: Some(h),
                },
            },
            size,
        );

        self.queue.submit(Some(read_encoder.finish()));

        let slice = readback_buffer.slice(..);
        let (tx, rx) = std::sync::mpsc::channel();
        slice.map_async(wgpu::MapMode::Read, move |result| {
            let _ = tx.send(result);
        });
        self.device.poll(wgpu::Maintain::Wait);
        rx.recv()??;

        {
            let data = slice.get_mapped_range();
            let u8_pixels: &[u8] = &data;
            for (i, pixel) in image.data.chunks_exact_mut(3).enumerate() {
                let base = i * 4;
                pixel[0] = u8_pixels[base] as f32;
                pixel[1] = u8_pixels[base + 1] as f32;
                pixel[2] = u8_pixels[base + 2] as f32;
            }
        }
        readback_buffer.unmap();

        println!("Executed GPU render round-trip for {}x{}", w, h);
        Ok(())
    }

    pub fn read_back_texture(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("GPU readback already performed in execute_render.");
        Ok(())
    }
}
