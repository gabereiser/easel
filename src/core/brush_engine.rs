use crate::core::image::Image;
use crate::core::pixol::{PixolBuffer, SurfaceSample};
use crate::input::event::StrokeEvent;

/// Context passed to every brush operation — gives the nozzle access to
/// surface data, global stroke state, and per-pixel information.
pub struct NozzleCtx<'a> {
    /// Surface data under the current pixel (depth, normal, angle).
    /// `None` when operating on a flat 2D canvas with no pixol buffer.
    pub surface: Option<&'a SurfaceSample>,
    /// Brush pressure (0.0–1.0) for the current stroke point.
    pub pressure: f32,
}

/// A pluggable brush nozzle that defines the alpha profile of the brush tip.
/// Implement this trait to create custom brush shapes.
///
/// The `ctx` parameter provides access to surface data so nozzles can
/// respond to depth/angle/cavity — the foundation for ZBrush-style
/// surface-aware brushes.
pub trait BrushNozzle: Send + Sync {
    /// Returns an alpha value in [0.0, 1.0] at the given offset from the
    /// brush center. `dx`, `dy` are pixel offsets, `radius` is the effective
    /// brush radius in pixels, `hardness` controls edge falloff for nozzles
    /// that support it (0.0 = soft, 1.0 = hard), and `ctx` carries surface
    /// and stroke context.
    fn sample(&self, dx: f32, dy: f32, radius: f32, hardness: f32, ctx: &NozzleCtx) -> f32;
}

/// Standard circular nozzle with hardness-controlled edge falloff.
pub struct CircleNozzle;

impl BrushNozzle for CircleNozzle {
    fn sample(&self, dx: f32, dy: f32, radius: f32, hardness: f32, _ctx: &NozzleCtx) -> f32 {
        let dist2 = dx * dx + dy * dy;
        let r2 = radius * radius;
        if dist2 > r2 {
            return 0.0;
        }
        let hardness_factor = 1.0 - (dist2 / r2).sqrt() * (1.0 - hardness);
        hardness_factor.max(0.0)
    }
}

/// Elliptical nozzle — stretch the circle along one axis for oval/cigar shapes.
/// `ratio` controls the aspect ratio (1.0 = circle, < 1.0 = wider, > 1.0 = taller).
pub struct EllipseNozzle {
    pub ratio: f32,
}

impl BrushNozzle for EllipseNozzle {
    fn sample(&self, dx: f32, dy: f32, radius: f32, hardness: f32, _ctx: &NozzleCtx) -> f32 {
        let r = radius.max(0.5);
        let r_x = if self.ratio >= 1.0 { r } else { r * self.ratio.max(0.01) };
        let r_y = if self.ratio <= 1.0 { r } else { r / self.ratio.max(0.01) };
        let dist2 = (dx * dx) / (r_x * r_x) + (dy * dy) / (r_y * r_y);
        if dist2 > 1.0 {
            return 0.0;
        }
        let hardness_factor = 1.0 - dist2.sqrt() * (1.0 - hardness);
        hardness_factor.max(0.0)
    }
}

/// Star-shaped nozzle with configurable points, inner radius ratio, and rotation.
pub struct StarNozzle {
    pub points: u32,
    pub inner_ratio: f32,
    pub rotation_deg: f32,
}

impl StarNozzle {
    fn star_sdf(&self, dx: f32, dy: f32, radius: f32) -> f32 {
        let r = radius.max(0.5);
        let angle = (-dy).atan2(dx) + self.rotation_deg.to_radians();
        let sector = std::f32::consts::PI / self.points as f32;
        let a = (angle % (2.0 * sector) + 2.0 * std::f32::consts::PI) % (2.0 * sector);
        let t = a / sector;
        let r1 = r;
        let r2 = r * self.inner_ratio;
        let r_eff = r2 + (r1 - r2) * (1.0 - (t - 1.0).abs());
        let dist = (dx * dx + dy * dy).sqrt();
        dist - r_eff
    }
}

impl BrushNozzle for StarNozzle {
    fn sample(&self, dx: f32, dy: f32, radius: f32, hardness: f32, _ctx: &NozzleCtx) -> f32 {
        let sdf = self.star_sdf(dx, dy, radius);
        if sdf > 0.0 {
            return 0.0;
        }
        let dist = sdf.abs();
        let r = radius.max(0.5);
        let falloff = 1.0 - (dist / r) * (1.0 - hardness);
        falloff.max(0.0)
    }
}

/// Diamond-shaped nozzle.
pub struct DiamondNozzle;

impl BrushNozzle for DiamondNozzle {
    fn sample(&self, dx: f32, dy: f32, radius: f32, hardness: f32, _ctx: &NozzleCtx) -> f32 {
        let r = radius.max(0.5);
        let dist = dx.abs() + dy.abs();
        let half_diag = r * 2.0f32.sqrt() / 2.0;
        if dist > half_diag {
            return 0.0;
        }
        let t = dist / half_diag;
        let falloff = 1.0 - t * (1.0 - hardness);
        falloff.max(0.0)
    }
}

/// Texture-based nozzle using a grayscale image as the alpha mask.
pub struct TextureNozzle {
    pub image: Image,
}

impl BrushNozzle for TextureNozzle {
    fn sample(&self, dx: f32, dy: f32, radius: f32, _hardness: f32, _ctx: &NozzleCtx) -> f32 {
        if self.image.width == 0 || self.image.height == 0 {
            return 1.0;
        }
        let u = ((dx / radius) * 0.5 + 0.5).clamp(0.0, 1.0);
        let v = ((dy / radius) * 0.5 + 0.5).clamp(0.0, 1.0);
        let tx = ((self.image.width as f32 - 1.0) * u) as u32;
        let ty = ((self.image.height as f32 - 1.0) * v) as u32;
        self.image.get_pixel(tx, ty).map_or(0.0, |p| p[0] / 255.0)
    }
}

/// A surface-aware nozzle that masks the brush by surface angle.
/// Pixels facing away from the camera (high angle) receive less paint.
pub struct AngleNozzle {
    pub inner: Box<dyn BrushNozzle>,
    /// Angle threshold in radians; pixels at or below this angle get full opacity,
    /// pixels above fade to zero.
    pub max_angle: f32,
    /// How sharply the mask falls off (1.0 = linear, higher = sharper).
    pub falloff_power: f32,
}

impl BrushNozzle for AngleNozzle {
    fn sample(&self, dx: f32, dy: f32, radius: f32, hardness: f32, ctx: &NozzleCtx) -> f32 {
        let base = self.inner.sample(dx, dy, radius, hardness, ctx);
        if base <= 0.0 {
            return 0.0;
        }
        let angle_mask = match ctx.surface {
            Some(surface) => {
                let t = (surface.angle / self.max_angle).clamp(0.0, 1.0);
                1.0 - t.powf(self.falloff_power)
            }
            None => 1.0,
        };
        base * angle_mask
    }
}

/// Per-pixel surface deformation.
/// After the nozzle determines where paint lands, deformers modify the
/// surface depth/normal — this is the ZBrush push/pull/smooth/pinch mechanism.
pub trait BrushDeformer: Send + Sync {
    /// Given the brush hit at `(dx, dy)` with the computed `alpha` and
    /// the current surface state, return the depth delta to apply.
    /// Positive = push out, negative = indent.
    fn deform(
        &self,
        dx: f32,
        dy: f32,
        radius: f32,
        alpha: f32,
        surface: &SurfaceSample,
    ) -> f32;
}

/// Push/pull deformer — the classic ZBrush displacement.
pub struct PushPullDeformer {
    /// Strength of the deformation. Positive = push, negative = pull.
    pub strength: f32,
    /// If true, the deformation accumulates (additive). If false, it
    /// targets an absolute depth value.
    pub additive: bool,
}

impl BrushDeformer for PushPullDeformer {
    fn deform(
        &self,
        dx: f32,
        dy: f32,
        radius: f32,
        alpha: f32,
        _surface: &SurfaceSample,
    ) -> f32 {
        let r = radius.max(0.5);
        let dist = (dx * dx + dy * dy).sqrt();
        let falloff = 1.0 - (dist / r).clamp(0.0, 1.0);
        falloff * alpha * self.strength
    }
}

/// Smooth deformer — averages depth with neighbors (laplacian smooth).
pub struct SmoothDeformer {
    pub strength: f32,
    pub iterations: u32,
}

impl SmoothDeformer {
    fn smooth_at(&self, buf: &PixolBuffer, x: u32, y: u32) -> f32 {
        let w = buf.width;
        let h = buf.height;
        if x == 0 || y == 0 || x >= w - 1 || y >= h - 1 {
            return buf.depth[(y * w + x) as usize];
        }
        let idx = (y * w + x) as usize;
        let mut sum = 0.0;
        let mut count = 0;
        for dy in -1..=1i32 {
            for dx in -1..=1i32 {
                let nx = (x as i32 + dx) as u32;
                let ny = (y as i32 + dy) as u32;
                if nx < w && ny < h {
                    sum += buf.depth[(ny * w + nx) as usize];
                    count += 1;
                }
            }
        }
        if count > 0 { sum / count as f32 } else { buf.depth[idx] }
    }
}

impl BrushDeformer for SmoothDeformer {
    fn deform(
        &self,
        _dx: f32,
        _dy: f32,
        _radius: f32,
        alpha: f32,
        _surface: &SurfaceSample,
    ) -> f32 {
        // The smooth deformer requires access to the whole buffer, so it
        // operates at the pipeline level rather than per-pixel.
        // This value is unused in the per-pixel path.
        alpha * self.strength
    }
}

/// A configurable brush pipeline that chains modifiers, a nozzle, and deformers.
///
/// ZBrush brushes follow a DSP-like chain:
///   input → stroke modifiers → surface readback → nozzle → deformer → writeback
///
/// `BrushPipeline` lets you compose these stages independently.
pub struct BrushPipeline {
    /// The nozzle that defines the tip shape.
    pub nozzle: Box<dyn BrushNozzle>,
    /// Deformers applied per pixel after the nozzle alpha is computed.
    /// Run in order; each receives the surface state from the previous.
    pub deformers: Vec<Box<dyn BrushDeformer>>,
}

impl BrushPipeline {
    pub fn new(nozzle: Box<dyn BrushNozzle>) -> Self {
        Self { nozzle, deformers: Vec::new() }
    }

    pub fn with_deformer(mut self, deformer: Box<dyn BrushDeformer>) -> Self {
        self.deformers.push(deformer);
        self
    }

    /// Evaluate the pipeline at a single pixel offset.
    /// Returns (alpha, depth_delta) — the alpha for color blending and
    /// the depth displacement for the pixol buffer.
    pub fn evaluate(
        &self,
        dx: f32,
        dy: f32,
        radius: f32,
        hardness: f32,
        ctx: &NozzleCtx,
    ) -> (f32, f32) {
        let alpha = self.nozzle.sample(dx, dy, radius, hardness, ctx);
        if alpha <= 0.0 {
            return (0.0, 0.0);
        }
        let mut depth_delta = 0.0;
        let default_surface = SurfaceSample::default();
        let surface = ctx.surface.unwrap_or(&default_surface);
        for deformer in &self.deformers {
            depth_delta += deformer.deform(dx, dy, radius, alpha, surface);
        }
        (alpha, depth_delta)
    }
}

#[derive(Debug, Clone)]
pub struct BrushConfig {
    pub name: String,
    pub brush_size: f32,
    pub opacity: f32,
    pub flow: f32,
    pub hardness: f32,
    pub color: [f32; 3],
}

impl BrushConfig {
    pub fn new(name: &str, brush_size: f32, opacity: f32, flow: f32, hardness: f32) -> Self {
        Self {
            name: name.to_string(),
            brush_size,
            opacity,
            flow,
            hardness,
            color: [255.0, 0.0, 0.0],
        }
    }
}

impl Default for BrushConfig {
    fn default() -> Self {
        Self::new("Standard", 10.0, 1.0, 1.0, 1.0)
    }
}

pub struct BrushEngine {
    pub brush_size: f32,
    pub opacity: f32,
    pub flow: f32,
    pub hardness: f32,
    pub nozzle: Box<dyn BrushNozzle>,
    pub pipeline: Option<BrushPipeline>,
    pub color: [f32; 3],
}

impl BrushEngine {
    pub fn new(brush_size: f32, opacity: f32, flow: f32, hardness: f32) -> Self {
        Self {
            brush_size,
            opacity,
            flow,
            hardness,
            nozzle: Box::new(CircleNozzle),
            pipeline: None,
            color: [255.0, 0.0, 0.0],
        }
    }

    pub fn with_config(config: &BrushConfig) -> Self {
        Self {
            brush_size: config.brush_size,
            opacity: config.opacity,
            flow: config.flow,
            hardness: config.hardness,
            nozzle: Box::new(CircleNozzle),
            pipeline: None,
            color: config.color,
        }
    }

    pub fn with_pipeline(pipeline: BrushPipeline, config: &BrushConfig) -> Self {
        Self {
            brush_size: config.brush_size,
            opacity: config.opacity,
            flow: config.flow,
            hardness: config.hardness,
            nozzle: Box::new(CircleNozzle),
            pipeline: Some(pipeline),
            color: config.color,
        }
    }

    fn render_at(
        &self,
        canvas: &mut Image,
        mut pixol_buffer: Option<&mut PixolBuffer>,
        cx: f32,
        cy: f32,
        radius: f32,
        alpha: f32,
        pressure: f32,
    ) {
        let r = radius.max(0.5);
        let min_y = ((cy - r).max(0.0) as usize).min(canvas.height as usize - 1);
        let max_y = ((cy + r).max(0.0) as usize).min(canvas.height as usize - 1);
        let min_x = ((cx - r).max(0.0) as usize).min(canvas.width as usize - 1);
        let max_x = ((cx + r).max(0.0) as usize).min(canvas.width as usize - 1);

        for py in min_y..=max_y {
            for px in min_x..=max_x {
                let dx = px as f32 - cx;
                let dy = py as f32 - cy;

                let surface_sample = pixol_buffer.as_ref().and_then(|pb| pb.surface_at(px as u32, py as u32));
                let ctx = NozzleCtx { surface: surface_sample.as_ref(), pressure };

                if let Some(ref pipeline) = self.pipeline {
                    let (a, depth_delta) = pipeline.evaluate(dx, dy, r, self.hardness, &ctx);
                    if a <= 0.0 {
                        continue;
                    }

                    // Color blend
                    let blend = (alpha * a).clamp(0.0, 1.0);
                    let idx = (py * canvas.width as usize + px) * 3;
                    canvas.data[idx] += (self.color[0] - canvas.data[idx]) * blend;
                    canvas.data[idx + 1] += (self.color[1] - canvas.data[idx + 1]) * blend;
                    canvas.data[idx + 2] += (self.color[2] - canvas.data[idx + 2]) * blend;

                    // Surface deformation
                    if let Some(ref mut pb) = pixol_buffer {
                        if depth_delta != 0.0 {
                            pb.displace_depth(px as u32, py as u32, depth_delta);
                            pb.recompute_normal(px as u32, py as u32);
                        }
                    }
                } else {
                    // Legacy fallback: nozzle only, no pipeline
                    let falloff = self.nozzle.sample(dx, dy, r, self.hardness, &ctx);
                    if falloff <= 0.0 {
                        continue;
                    }
                    let a = (alpha * falloff).clamp(0.0, 1.0);
                    let idx = (py * canvas.width as usize + px) * 3;
                    canvas.data[idx] += (self.color[0] - canvas.data[idx]) * a;
                    canvas.data[idx + 1] += (self.color[1] - canvas.data[idx + 1]) * a;
                    canvas.data[idx + 2] += (self.color[2] - canvas.data[idx + 2]) * a;
                }
            }
        }
    }

    /// Applies strokes to a canvas using this engine's parameters.
    /// Optionally accepts a `PixolBuffer` for surface-aware painting.
    pub fn apply_to(
        &self,
        events: &[StrokeEvent],
        canvas: &mut Image,
        mut pixol_buffer: Option<&mut PixolBuffer>,
    ) {
        for event in events {
            let radius = (self.brush_size * event.pressure) / 2.0;
            let alpha = self.opacity * self.flow;
            self.render_at(canvas, pixol_buffer.as_deref_mut(), event.x, event.y, radius, alpha, event.pressure);
        }
    }

    /// Legacy: applies strokes to a 2D canvas with no surface data.
    pub fn apply(&self, events: &[StrokeEvent], canvas: &mut Image) {
        self.apply_to(events, canvas, None);
    }

    pub fn apply_brush(&self, config: &BrushConfig, events: &[StrokeEvent], canvas: &mut Image) {
        let engine = Self::with_config(config);
        for event in events {
            let radius = (config.brush_size * event.pressure) / 2.0;
            let alpha = config.opacity * config.flow;
            let ctx = NozzleCtx { surface: None, pressure: event.pressure };
            let falloff = engine.nozzle.sample(0.0, 0.0, radius, engine.hardness, &ctx);
            let a = (alpha * falloff).clamp(0.0, 1.0);
            let px = event.x as usize;
            let py = event.y as usize;
            if px < canvas.width as usize && py < canvas.height as usize {
                let idx = (py * canvas.width as usize + px) * 3;
                canvas.data[idx] += (config.color[0] - canvas.data[idx]) * a;
                canvas.data[idx + 1] += (config.color[1] - canvas.data[idx + 1]) * a;
                canvas.data[idx + 2] += (config.color[2] - canvas.data[idx + 2]) * a;
            }
        }
    }

    pub fn process_stroke(&self, events: &[StrokeEvent], canvas: &Image) -> Image {
        let mut result = canvas.clone();
        self.apply(events, &mut result);
        result
    }
}
