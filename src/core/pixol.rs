/// Per-pixel surface data for 3D-aware brush operations.
/// Models ZBrush's pixol concept: each pixel stores color + depth + normal + material ID.
#[derive(Debug, Clone, Copy)]
pub struct SurfaceSample {
    /// Depth/height value (0.0 = base, positive = raised, negative = indented).
    pub depth: f32,
    /// Surface normal in view space (nx, ny, nz). (0, 0, 1) = facing camera.
    pub normal: [f32; 3],
    /// Material or layer index.
    pub material_id: u32,
    /// Angle between surface normal and view direction (radians). 0 = facing camera.
    pub angle: f32,
}

impl Default for SurfaceSample {
    fn default() -> Self {
        Self {
            depth: 0.0,
            normal: [0.0, 0.0, 1.0],
            material_id: 0,
            angle: 0.0,
        }
    }
}

/// A 2D buffer where each pixel carries both color and surface data.
/// This is the foundation for ZBrush-style brushes that deform and paint simultaneously.
#[derive(Debug, Clone)]
pub struct PixolBuffer {
    pub width: u32,
    pub height: u32,
    pub color: Vec<f32>,           // RGB interleaved, 3 floats per pixel
    pub depth: Vec<f32>,           // 1 float per pixel
    pub normal: Vec<[f32; 3]>,     // 3 floats per pixel
    pub material_id: Vec<u32>,     // 1 u32 per pixel
}

impl PixolBuffer {
    pub fn new(width: u32, height: u32) -> Self {
        let len = (width * height) as usize;
        Self {
            width,
            height,
            color: vec![0.0; len * 3],
            depth: vec![0.0; len],
            normal: vec![[0.0, 0.0, 1.0]; len],
            material_id: vec![0; len],
        }
    }

    pub fn surface_at(&self, x: u32, y: u32) -> Option<SurfaceSample> {
        if x >= self.width || y >= self.height {
            return None;
        }
        let idx = (y * self.width + x) as usize;
        Some(SurfaceSample {
            depth: self.depth[idx],
            normal: self.normal[idx],
            material_id: self.material_id[idx],
            angle: self.normal[idx][2].acos(), // angle from Z-axis
        })
    }

    pub fn set_surface(&mut self, x: u32, y: u32, surface: &SurfaceSample) {
        if x >= self.width || y >= self.height {
            return;
        }
        let idx = (y * self.width + x) as usize;
        self.depth[idx] = surface.depth;
        self.normal[idx] = surface.normal;
        self.material_id[idx] = surface.material_id;
    }

    pub fn set_color(&mut self, x: u32, y: u32, rgb: [f32; 3]) {
        if x >= self.width || y >= self.height {
            return;
        }
        let idx = (y * self.width + x) as usize * 3;
        self.color[idx] = rgb[0];
        self.color[idx + 1] = rgb[1];
        self.color[idx + 2] = rgb[2];
    }

    pub fn get_color(&self, x: u32, y: u32) -> Option<[f32; 3]> {
        if x >= self.width || y >= self.height {
            return None;
        }
        let idx = (y * self.width + x) as usize * 3;
        Some([self.color[idx], self.color[idx + 1], self.color[idx + 2]])
    }

    /// Sample surface data at fractional coordinates via nearest-neighbor.
    pub fn sample_surface(&self, fx: f32, fy: f32) -> SurfaceSample {
        let x = (fx.round() as u32).min(self.width.saturating_sub(1));
        let y = (fy.round() as u32).min(self.height.saturating_sub(1));
        self.surface_at(x, y).unwrap_or_default()
    }

    /// Push/pull the depth at a pixel by an amount (positive = push out, negative = indent).
    pub fn displace_depth(&mut self, x: u32, y: u32, delta: f32) {
        if x >= self.width || y >= self.height {
            return;
        }
        let idx = (y * self.width + x) as usize;
        self.depth[idx] += delta;
    }

    /// Recompute normal at a pixel from neighbor depths (central differences).
    pub fn recompute_normal(&mut self, x: u32, y: u32) {
        if x == 0 || y == 0 || x >= self.width - 1 || y >= self.height - 1 {
            return;
        }
        let idx = (y * self.width + x) as usize;
        let l = self.depth[(y * self.width + x - 1) as usize];
        let r = self.depth[(y * self.width + x + 1) as usize];
        let d = self.depth[((y - 1) * self.width + x) as usize];
        let u = self.depth[((y + 1) * self.width + x) as usize];

        let nx = l - r;
        let ny = d - u;
        let nz = 2.0; // strength factor
        let len = (nx * nx + ny * ny + nz * nz).sqrt();
        if len > 0.0 {
            self.normal[idx] = [nx / len, ny / len, nz / len];
        }
    }
}
