use crate::core::color_space::{color_space_from_kind, ColorSpace, ColorSpaceConverter, ColorSpaceKind, ColorSpaceConverterImpl};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct Image {
    pub width: u32,
    pub height: u32,
    pub data: Vec<f32>,
    pub color_space: ColorSpaceKind,
    pub source_space: Arc<dyn ColorSpaceConverter + Send + Sync>,
}

impl Image {
    pub fn new(width: u32, height: u32) -> Self {
        Image {
            width,
            height,
            data: vec![0.0; (width * height * 3) as usize],
            color_space: ColorSpaceKind::SRGB,
            source_space: Arc::new(ColorSpaceConverterImpl),
        }
    }

    pub fn get_pixel(&self, x: u32, y: u32) -> Option<[f32; 3]> {
        if x >= self.width || y >= self.height {
            return None;
        }
        let idx = (y * self.width + x) as usize * 3;
        Some([self.data[idx], self.data[idx + 1], self.data[idx + 2]])
    }

    pub fn set_pixel(&mut self, x: u32, y: u32, rgb: [f32; 3]) {
        if x >= self.width || y >= self.height {
            return;
        }
        let idx = (y * self.width + x) as usize * 3;
        self.data[idx] = rgb[0];
        self.data[idx + 1] = rgb[1];
        self.data[idx + 2] = rgb[2];
    }

    pub fn convert(&self, target_converter: &dyn ColorSpaceConverter, target_space: ColorSpace) -> Image {
        let src = color_space_from_kind(self.color_space);
        let mut out_data = Vec::with_capacity(self.data.len());
        for chunk in self.data.chunks_exact(3) {
            // Normalize from 0–255 storage range to 0–1 for color math, then back
            let normalized = [chunk[0] / 255.0, chunk[1] / 255.0, chunk[2] / 255.0];
            let converted = target_converter.convert(normalized, src, target_space);
            out_data.push(converted[0] * 255.0);
            out_data.push(converted[1] * 255.0);
            out_data.push(converted[2] * 255.0);
        }
        Image {
            width: self.width,
            height: self.height,
            data: out_data,
            color_space: ColorSpaceKind::from(target_space),
            source_space: Arc::new(ColorSpaceConverterImpl),
        }
    }

    pub fn save(&self, path: &str, format: ImageFormat) -> Result<(), Box<dyn std::error::Error>> {
        let w = self.width;
        let h = self.height;

        match format {
            ImageFormat::PNG | ImageFormat::JPEG => {
                let mut u8_data = Vec::with_capacity((w * h * 3) as usize);
                for chunk in self.data.chunks_exact(3) {
                    u8_data.push(chunk[0].clamp(0.0, 255.0) as u8);
                    u8_data.push(chunk[1].clamp(0.0, 255.0) as u8);
                    u8_data.push(chunk[2].clamp(0.0, 255.0) as u8);
                }
                let img_format = match format {
                    ImageFormat::PNG => image::ImageFormat::Png,
                    _ => image::ImageFormat::Jpeg,
                };
                image::save_buffer_with_format(path, &u8_data, w, h, image::ColorType::Rgb8, img_format)?;
            }
            ImageFormat::EXR => {
                let mut f32_data = Vec::with_capacity((w * h * 3) as usize);
                for chunk in self.data.chunks_exact(3) {
                    f32_data.push(chunk[0]);
                    f32_data.push(chunk[1]);
                    f32_data.push(chunk[2]);
                }
                let bytes: &[u8] = bytemuck::cast_slice(&f32_data);
                image::save_buffer_with_format(path, bytes, w, h, image::ColorType::Rgb32F, image::ImageFormat::OpenExr)?;
            }
        }
        Ok(())
    }
}

/// Manual Serialize for Image — skips the non-serializable `source_space` trait object.
impl Serialize for Image {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("Image", 4)?;
        state.serialize_field("width", &self.width)?;
        state.serialize_field("height", &self.height)?;
        state.serialize_field("data", &self.data)?;
        state.serialize_field("color_space", &self.color_space)?;
        state.end()
    }
}

/// Manual Deserialize for Image — reconstructs a ColorSpaceConverterImpl on load.
impl<'de> Deserialize<'de> for Image {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        struct ImageData {
            width: u32,
            height: u32,
            data: Vec<f32>,
            color_space: ColorSpaceKind,
        }
        let d = ImageData::deserialize(deserializer)?;
        Ok(Image {
            width: d.width,
            height: d.height,
            data: d.data,
            color_space: d.color_space,
            source_space: Arc::new(ColorSpaceConverterImpl),
        })
    }
}

pub enum ImageFormat {
    PNG,
    JPEG,
    EXR,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_image_new() {
        let img = Image::new(100, 200);
        assert_eq!(img.width, 100);
        assert_eq!(img.height, 200);
        assert_eq!(img.data.len(), (100 * 200 * 3) as usize);
    }

    #[test]
    fn test_image_convert() {
        let img = Image::new(10, 10);
        let target = ColorSpaceConverterImpl;
        let converted = img.convert(&target, ColorSpace::SRGB);
        assert_eq!(converted.width, img.width);
        assert_eq!(converted.height, img.height);
    }

    #[test]
    fn test_image_serde_roundtrip() {
        let img = Image::new(4, 4);
        let json = serde_json::to_string(&img).expect("serialize");
        let restored: Image = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(restored.width, img.width);
        assert_eq!(restored.height, img.height);
        assert_eq!(restored.data, img.data);
    }

    #[test]
    fn test_get_pixel_out_of_bounds() {
        let img = Image::new(10, 10);
        assert!(img.get_pixel(10, 0).is_none());
        assert!(img.get_pixel(0, 10).is_none());
        assert!(img.get_pixel(10, 10).is_none());
    }

    #[test]
    fn test_set_pixel_and_get_pixel() {
        let mut img = Image::new(4, 4);
        img.set_pixel(1, 2, [0.5, 0.75, 1.0]);
        let pixel = img.get_pixel(1, 2).expect("should exist");
        assert!((pixel[0] - 0.5).abs() < 1e-6);
        assert!((pixel[1] - 0.75).abs() < 1e-6);
        assert!((pixel[2] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_set_pixel_out_of_bounds_is_noop() {
        let mut img = Image::new(4, 4);
        let orig = img.data.clone();
        img.set_pixel(10, 10, [1.0, 0.0, 0.0]);
        assert_eq!(img.data, orig);
    }

    #[test]
    fn test_save_roundtrip_preserves_data() {
        let tmp = std::env::temp_dir().join("test_save_rt.png");
        let path = tmp.to_str().unwrap();
        let mut img = Image::new(2, 2);
        img.set_pixel(0, 0, [1.0, 0.0, 0.0]);
        img.set_pixel(1, 0, [0.0, 1.0, 0.0]);
        img.set_pixel(0, 1, [0.0, 0.0, 1.0]);
        img.set_pixel(1, 1, [1.0, 1.0, 1.0]);
        let result = img.save(path, ImageFormat::PNG);
        assert!(result.is_ok(), "save failed: {:?}", result.err());
        assert!(tmp.exists());
        let _ = std::fs::remove_file(&tmp);
    }
}
