#![allow(deprecated)]

use easel_core::ui::widget::{DrawRect, RectF};
use easel_core::ui::UiCompositor;
use glyphon::{
    Attrs, Buffer, Color as GlyphonColor, Family, FontSystem, Metrics, Resolution, Shaping,
    SwashCache, TextArea, TextAtlas, TextBounds, TextRenderer,
};
use wgpu::util::DeviceExt;
use winit::dpi::LogicalSize;
use winit::event::{Event, MouseScrollDelta, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::Window;
use std::sync::{Arc, Mutex};
use easel_core::BrushEngine;
use easel_core::StrokeEvent;
use easel_core::Image;
use winit::event::ElementState;
use winit::event::MouseButton;
use winit::keyboard;

fn main() {
    let event_loop = EventLoop::new().unwrap();
    let window = Arc::new(
        event_loop
            .create_window(
                Window::default_attributes()
                    .with_title("Easel")
                    .with_inner_size(LogicalSize::new(1600, 1000)),
            )
            .unwrap(),
    );
    let window_size = window.inner_size();

    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: wgpu::Backends::all(),
        ..Default::default()
    });
    let surface = instance.create_surface(window.clone()).unwrap();
    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        compatible_surface: Some(&surface),
        force_fallback_adapter: false,
    }))
    .unwrap();
    let (device, queue) = pollster::block_on(adapter.request_device(
        &wgpu::DeviceDescriptor {
            label: Some("device"),
            required_features: wgpu::Features::empty(),
            required_limits: wgpu::Limits::default(),
        },
        None,
    ))
    .unwrap();

    let swapchain_caps = surface.get_capabilities(&adapter);
    let swap_format = swapchain_caps.formats[0];
    let mut config = wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format: swap_format,
        width: window_size.width.max(1),
        height: window_size.height.max(1),
        present_mode: wgpu::PresentMode::Fifo,
        alpha_mode: wgpu::CompositeAlphaMode::Auto,
        view_formats: vec![],
        desired_maximum_frame_latency: 2,
    };
    surface.configure(&device, &config);

    // Uniform buffer for viewport size (screen coords → clip space)
    let viewport_uniform = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("viewport"),
        contents: bytemuck::bytes_of(&[config.width as f32, config.height as f32]),
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
    });
    let viewport_bind_group_layout =
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("viewport bgl"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });
    let viewport_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("viewport bg"),
        layout: &viewport_bind_group_layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: viewport_uniform.as_entire_binding(),
        }],
    });

    // Fullscreen quad pipeline for colored rects
    let rect_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("rect shader"),
        source: wgpu::ShaderSource::Wgsl(include_str!("shaders/rect.wgsl").into()),
    });
    let rect_pipeline_layout =
        device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("rect pl"),
            bind_group_layouts: &[&viewport_bind_group_layout],
            push_constant_ranges: &[],
        });
    let rect_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("rect pipeline"),
        layout: Some(&rect_pipeline_layout),
        vertex: wgpu::VertexState {
            module: &rect_shader,
            entry_point: "vs_main",
            buffers: &[wgpu::VertexBufferLayout {
                array_stride: 24,
                step_mode: wgpu::VertexStepMode::Vertex,
                attributes: &wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x4],
            }],
        },
        fragment: Some(wgpu::FragmentState {
            module: &rect_shader,
            entry_point: "fs_main",
            targets: &[Some(wgpu::ColorTargetState {
                format: swap_format,
                blend: Some(wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING),
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        primitive: wgpu::PrimitiveState::default(),
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        multiview: None,
    });

    // Canvas textured-quad pipeline
    let canvas_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("canvas shader"),
        source: wgpu::ShaderSource::Wgsl(include_str!("shaders/canvas.wgsl").into()),
    });
    let canvas_texture_bgl =
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("canvas tex bgl"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });
    let canvas_pipeline_layout =
        device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("canvas pl"),
            bind_group_layouts: &[&viewport_bind_group_layout, &canvas_texture_bgl],
            push_constant_ranges: &[],
        });
    let canvas_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("canvas pipeline"),
        layout: Some(&canvas_pipeline_layout),
        vertex: wgpu::VertexState {
            module: &canvas_shader,
            entry_point: "vs_main",
            buffers: &[wgpu::VertexBufferLayout {
                array_stride: 16,
                step_mode: wgpu::VertexStepMode::Vertex,
                attributes: &wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x2],
            }],
        },
        fragment: Some(wgpu::FragmentState {
            module: &canvas_shader,
            entry_point: "fs_main",
            targets: &[Some(wgpu::ColorTargetState {
                format: swap_format,
                blend: Some(wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING),
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        primitive: wgpu::PrimitiveState::default(),
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        multiview: None,
    });

    let canvas_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some("canvas sampler"),
        address_mode_u: wgpu::AddressMode::ClampToEdge,
        address_mode_v: wgpu::AddressMode::ClampToEdge,
        address_mode_w: wgpu::AddressMode::ClampToEdge,
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Nearest,
        mipmap_filter: wgpu::FilterMode::Nearest,
        ..Default::default()
    });

    // Glyphon text rendering
    let mut font_system = FontSystem::new();
    let mut swash_cache = SwashCache::new();
    let mut atlas = TextAtlas::new(&device, &queue, swap_format);
    let mut text_renderer =
        TextRenderer::new(&mut atlas, &device, wgpu::MultisampleState::default(), None);

    let mut compositor = UiCompositor::new(config.width as f32, config.height as f32);
    let mut canvas_texture: Option<(wgpu::Texture, wgpu::TextureView, wgpu::BindGroup)> = None;
    let canvas_image = Arc::new(Mutex::new(Image::new(1, 1)));
    let mut brush_engine = BrushEngine::new(20.0, 1.0, 0.5, 0.8);
    compositor.drawer.opacity = brush_engine.opacity;
    compositor.drawer.flow = brush_engine.flow;
    compositor.drawer.hardness = brush_engine.hardness;
    let mut painting = false;
    let mut last_pos: Option<(f32, f32)> = None;
    let mut cursor_screen: (f32, f32) = (0.0, 0.0);
    let mut canvas_scale = 1.0f32;
    let mut canvas_offset_x = 0.0f32;
    let mut canvas_offset_y = 0.0f32;
    let mut panning = false;
    let mut pan_start: Option<(f32, f32)> = None;
    let mut image_dirty = true;
    event_loop.set_control_flow(ControlFlow::Poll);
    let _ = event_loop.run(move |event, target| {
        match event {
            Event::WindowEvent { window_id, event } if window_id == window.id() => {
                // Painting input
                let mut repaint = false;
                match &event {
                    WindowEvent::CursorMoved { position, .. } => {
                        let (ox, oy) = compositor.canvas_origin();
                        let (cw, ch) = compositor.canvas_viewport();
                        let sx = position.x as f32;
                        let sy = position.y as f32;
                        let cx = (sx - ox) / canvas_scale + canvas_offset_x;
                        let cy = (sy - oy) / canvas_scale + canvas_offset_y;
                        if painting {
                            let ts = || std::time::SystemTime::now()
                                .duration_since(std::time::UNIX_EPOCH)
                                .unwrap()
                                .as_millis() as u64;
                            let cx_clamped = cx.clamp(0.0, (cw - 1).max(0) as f32);
                            let cy_clamped = cy.clamp(0.0, (ch - 1).max(0) as f32);
                            let mut img = canvas_image.lock().unwrap();
                            if let Some((lx, ly)) = last_pos {
                                let dist = ((cx_clamped - lx) * (cx_clamped - lx) + (cy_clamped - ly) * (cy_clamped - ly)).sqrt();
                                if dist > 0.0 {
                                    let steps = (dist / 2.0).ceil() as u32;
                                    for i in 0..=steps {
                                        let t = i as f32 / steps as f32;
                                        let x = lx + (cx_clamped - lx) * t;
                                        let y = ly + (cy_clamped - ly) * t;
                                        let evt = StrokeEvent { timestamp: ts(), x, y, pressure: 1.0 };
                                        brush_engine.apply(&[evt], &mut *img);
                                    }
                                }
                            } else {
                                let evt = StrokeEvent { timestamp: ts(), x: cx_clamped, y: cy_clamped, pressure: 1.0 };
                                brush_engine.apply(&[evt], &mut *img);
                            }
                            image_dirty = true;
                            repaint = true;
                        }
                        if panning {
                            if let Some((psx, psy)) = pan_start {
                                let dx = sx - psx;
                                let dy = sy - psy;
                                canvas_offset_x -= dx / canvas_scale;
                                canvas_offset_y -= dy / canvas_scale;
                                repaint = true;
                            }
                            pan_start = Some((sx, sy));
                        }
                        last_pos = Some((cx, cy));
                        cursor_screen = (sx, sy);
                    }
                    WindowEvent::MouseWheel { delta, .. } => {
                        let factor = match delta {
                            MouseScrollDelta::LineDelta(_, y) => 1.0 + y * 0.1,
                            MouseScrollDelta::PixelDelta(pos) => 1.0 + pos.y as f32 * 0.01,
                        };
                        if factor != 1.0 {
                            let old_scale = canvas_scale;
                            canvas_scale = (canvas_scale * factor).clamp(0.1, 32.0);
                            let (ox, oy) = compositor.canvas_origin();
                            let sx = cursor_screen.0 - ox;
                            let sy = cursor_screen.1 - oy;
                            let cx = sx / old_scale + canvas_offset_x;
                            let cy = sy / old_scale + canvas_offset_y;
                            canvas_offset_x = cx - sx / canvas_scale;
                            canvas_offset_y = cy - sy / canvas_scale;
                            repaint = true;
                        }
                    }
                    WindowEvent::MouseInput { state, button, .. } => {
                        match (state, button) {
                            (ElementState::Pressed, MouseButton::Left) => {
                                painting = true;
                                repaint = true;
                            }
                            (ElementState::Released, MouseButton::Left) => {
                                painting = false;
                                last_pos = None;
                                repaint = true;
                            }
                            (ElementState::Pressed, MouseButton::Middle) => {
                                panning = true;
                                pan_start = Some((cursor_screen.0, cursor_screen.1));
                                repaint = true;
                            }
                            (ElementState::Released, MouseButton::Middle) => {
                                panning = false;
                                pan_start = None;
                                repaint = true;
                            }
                            (ElementState::Pressed, MouseButton::Right) => {
                                // Color pick: sample pixel from canvas
                                let ox = compositor.canvas_origin().0;
                                let oy = compositor.canvas_origin().1;
                                let px = ((cursor_screen.0 - ox) / canvas_scale + canvas_offset_x) as u32;
                                let py = ((cursor_screen.1 - oy) / canvas_scale + canvas_offset_y) as u32;
                                let img = canvas_image.lock().unwrap();
                                if let Some(pixel) = img.get_pixel(px, py) {
                                    brush_engine.color = pixel;
                                }
                                repaint = true;
                            }
                            _ => {}
                        }
                    }
                    _ => {}
                }
                // Keyboard shortcuts for brush control
                if let WindowEvent::KeyboardInput { event: ke, .. } = &event {
                    if ke.state == ElementState::Pressed && !ke.repeat {
                        match &ke.logical_key {
                            keyboard::Key::Character(c) if c == "[" => {
                                brush_engine.brush_size = (brush_engine.brush_size - 4.0).max(1.0);
                                repaint = true;
                            }
                            keyboard::Key::Character(c) if c == "]" => {
                                brush_engine.brush_size = (brush_engine.brush_size + 4.0).min(500.0);
                                repaint = true;
                            }
                            keyboard::Key::Character(c) if c == "c" => {
                                brush_engine.color = [255.0, 0.0, 0.0];
                                repaint = true;
                            }
                            _ => {}
                        }
                    }
                }
                if repaint {
                    window.request_redraw();
                }
                if compositor.handle_event(&event) {
                    return;
                }
                match &event {
                    WindowEvent::CloseRequested => target.exit(),
                    WindowEvent::Resized(size) => {
                        config.width = size.width.max(1);
                        config.height = size.height.max(1);
                        surface.configure(&device, &config);
                        queue.write_buffer(
                            &viewport_uniform,
                            0,
                            bytemuck::bytes_of(&[config.width as f32, config.height as f32]),
                        );
                        compositor.resize(config.width as f32, config.height as f32);
                    }
                    _ => {}
                }
            }
            Event::AboutToWait => {
                compositor.layout();
                let (cw, ch) = compositor.canvas_viewport();
                {
                    let mut img = canvas_image.lock().unwrap();
                    if img.width != cw || img.height != ch {
                        let mut new_img = Image::new(cw.max(1), ch.max(1));
                        let min_w = img.width.min(new_img.width);
                        let min_h = img.height.min(new_img.height);
                        for y in 0..min_h {
                            for x in 0..min_w {
                                if let Some(p) = img.get_pixel(x, y) {
                                    new_img.set_pixel(x, y, p);
                                }
                            }
                        }
                        *img = new_img;
                        image_dirty = true;
                    }
                }
                // Apply palette color selection
                if let Some(picked) = compositor.take_clicked_color() {
                    brush_engine.color = picked;
                }
                // Sync brush settings from drawer sliders to engine
                brush_engine.opacity = compositor.drawer.opacity;
                brush_engine.flow = compositor.drawer.flow;
                brush_engine.hardness = compositor.drawer.hardness;
                compositor.set_canvas_image_arc(canvas_image.clone());
                let c = brush_engine.color;
                compositor.brush_info = format!(
                    "Brush: {:.0}px  Opacity: {:.0}%  Flow: {:.0}%  Hardness: {:.0}%  Zoom: {:.0}%  RGB({:.0},{:.0},{:.0})",
                    brush_engine.brush_size,
                    brush_engine.opacity * 100.0,
                    brush_engine.flow * 100.0,
                    brush_engine.hardness * 100.0,
                    canvas_scale * 100.0,
                    c[0], c[1], c[2],
                );
                compositor.render(&device, &queue);
                window.request_redraw();
            }
            Event::WindowEvent {
                event: WindowEvent::RedrawRequested,
                ..
            } => {
                let (rects, texts) = compositor.draw_data();
                let (verts, indices) = build_rect_geometry(rects);
                let vertex_buf = if !verts.is_empty() {
                    Some(device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some("rect verts"),
                        contents: bytemuck::cast_slice(&verts),
                        usage: wgpu::BufferUsages::VERTEX,
                    }))
                } else {
                    None
                };
                let index_buf = if !indices.is_empty() {
                    Some(device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some("rect idx"),
                        contents: bytemuck::cast_slice(&indices),
                        usage: wgpu::BufferUsages::INDEX,
                    }))
                } else {
                    None
                };
                let num_idx = indices.len() as u32;

                // Canvas texture – create or upload only when dirty
                if image_dirty {
                    let cv_img = canvas_image.lock().unwrap();
                    let tex_changed = match &canvas_texture {
                        Some((t, _, _)) => t.width() != cv_img.width || t.height() != cv_img.height,
                        None => true,
                    };
                    let rgba = image_to_rgba(&*cv_img);
                    if tex_changed {
                        let tex = device.create_texture(&wgpu::TextureDescriptor {
                            label: Some("canvas tex"),
                            size: wgpu::Extent3d {
                                width: cv_img.width,
                                height: cv_img.height,
                                depth_or_array_layers: 1,
                            },
                            mip_level_count: 1,
                            sample_count: 1,
                            dimension: wgpu::TextureDimension::D2,
                            format: wgpu::TextureFormat::Rgba8UnormSrgb,
                            usage: wgpu::TextureUsages::TEXTURE_BINDING
                                | wgpu::TextureUsages::COPY_DST,
                            view_formats: &[],
                        });
                        queue.write_texture(
                            wgpu::ImageCopyTexture {
                                texture: &tex,
                                mip_level: 0,
                                origin: wgpu::Origin3d::ZERO,
                                aspect: wgpu::TextureAspect::All,
                            },
                            &rgba,
                            wgpu::ImageDataLayout {
                                offset: 0,
                                bytes_per_row: Some(cv_img.width * 4),
                                rows_per_image: Some(cv_img.height),
                            },
                            wgpu::Extent3d {
                                width: cv_img.width,
                                height: cv_img.height,
                                depth_or_array_layers: 1,
                            },
                        );
                        let view = tex.create_view(&wgpu::TextureViewDescriptor::default());
                        let bg = device.create_bind_group(&wgpu::BindGroupDescriptor {
                            label: Some("canvas bg"),
                            layout: &canvas_texture_bgl,
                            entries: &[
                                wgpu::BindGroupEntry {
                                    binding: 0,
                                    resource: wgpu::BindingResource::TextureView(&view),
                                },
                                wgpu::BindGroupEntry {
                                    binding: 1,
                                    resource: wgpu::BindingResource::Sampler(&canvas_sampler),
                                },
                            ],
                        });
                        canvas_texture = Some((tex, view, bg));
                    } else if let Some((ref tex, _, _)) = canvas_texture {
                        queue.write_texture(
                            wgpu::ImageCopyTexture {
                                texture: tex,
                                mip_level: 0,
                                origin: wgpu::Origin3d::ZERO,
                                aspect: wgpu::TextureAspect::All,
                            },
                            &rgba,
                            wgpu::ImageDataLayout {
                                offset: 0,
                                bytes_per_row: Some(cv_img.width * 4),
                                rows_per_image: Some(cv_img.height),
                            },
                            wgpu::Extent3d {
                                width: cv_img.width,
                                height: cv_img.height,
                                depth_or_array_layers: 1,
                            },
                        );
                    }
                    image_dirty = false;
                }

                // Build canvas quad geometry before the render pass
                let (cv_verts, cv_idx) = if let Some((ref tex, _, _)) = canvas_texture {
                    let cv = compositor.canvas_rect();
                    let img_w = tex.width() as f32;
                    let img_h = tex.height() as f32;
                    build_canvas_quad(cv, canvas_scale, canvas_offset_x, canvas_offset_y, img_w, img_h)
                } else {
                    (Vec::new(), Vec::new())
                };
                let cv_vb = if !cv_verts.is_empty() {
                    Some(device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some("canvas verts"),
                        contents: bytemuck::cast_slice(&cv_verts),
                        usage: wgpu::BufferUsages::VERTEX,
                    }))
                } else {
                    None
                };
                let cv_ib = if !cv_idx.is_empty() {
                    Some(device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some("canvas idx"),
                        contents: bytemuck::cast_slice(&cv_idx),
                        usage: wgpu::BufferUsages::INDEX,
                    }))
                } else {
                    None
                };

                // Build text buffers before the render pass
                let mut text_buffers: Vec<Buffer> = Vec::new();
                for dt in texts {
                    let mut buf = Buffer::new(
                        &mut font_system,
                        Metrics::new(dt.font_size, dt.font_size * 1.4),
                    );
                    buf.set_size(
                        &mut font_system,
                        dt.rect.size.width.max(1.0),
                        dt.rect.size.height.max(1.0),
                    );
                    buf.set_text(
                        &mut font_system,
                        &dt.text,
                        Attrs::new().family(Family::SansSerif),
                        Shaping::Basic,
                    );
                    buf.shape_until_scroll(&mut font_system);
                    text_buffers.push(buf);
                }
                let text_areas: Vec<TextArea> = texts
                    .iter()
                    .zip(text_buffers.iter())
                    .map(|(dt, buf)| TextArea {
                        buffer: buf,
                        left: dt.rect.origin.x,
                        top: dt.rect.origin.y,
                        scale: 1.0,
                        bounds: TextBounds {
                            left: 0,
                            top: 0,
                            right: config.width as i32,
                            bottom: config.height as i32,
                        },
                        default_color: GlyphonColor::rgba(
                            (dt.color[0] * 255.0) as u8,
                            (dt.color[1] * 255.0) as u8,
                            (dt.color[2] * 255.0) as u8,
                            (dt.color[3] * 255.0) as u8,
                        ),
                    })
                    .collect();

                let frame = surface.get_current_texture().unwrap();
                let view = frame
                    .texture
                    .create_view(&wgpu::TextureViewDescriptor::default());
                let mut encoder =
                    device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                        label: Some("enc"),
                    });
                {
                    let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: Some("main"),
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view: &view,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear(wgpu::Color {
                                    r: 0.086,
                                    g: 0.094,
                                    b: 0.118,
                                    a: 1.0,
                                }),
                                store: wgpu::StoreOp::Store,
                            },
                        })],
                        depth_stencil_attachment: None,
                        timestamp_writes: None,
                        occlusion_query_set: None,
                    });

                    // 1. Draw canvas textured quad
                    if let (Some(ref vb), Some(ref ib)) = (&cv_vb, &cv_ib) {
                        if let Some((_, _, ref bg)) = canvas_texture {
                            rpass.set_pipeline(&canvas_pipeline);
                            rpass.set_bind_group(0, &viewport_bind_group, &[]);
                            rpass.set_bind_group(1, bg, &[]);
                            rpass.set_vertex_buffer(0, vb.slice(..));
                            rpass.set_index_buffer(ib.slice(..), wgpu::IndexFormat::Uint32);
                            rpass.draw_indexed(0..cv_idx.len() as u32, 0, 0..1);
                        }
                    }

                    // 2. Draw UI rects
                    if let (Some(ref vb), Some(ref ib)) = (&vertex_buf, &index_buf) {
                        rpass.set_pipeline(&rect_pipeline);
                        rpass.set_bind_group(0, &viewport_bind_group, &[]);
                        rpass.set_vertex_buffer(0, vb.slice(..));
                        rpass.set_index_buffer(ib.slice(..), wgpu::IndexFormat::Uint32);
                        rpass.draw_indexed(0..num_idx, 0, 0..1);
                    }

                    // 3. Draw text
                    if !text_areas.is_empty() {
                        text_renderer
                            .prepare(
                                &device,
                                &queue,
                                &mut font_system,
                                &mut atlas,
                                Resolution {
                                    width: config.width,
                                    height: config.height,
                                },
                                text_areas,
                                &mut swash_cache,
                            )
                            .unwrap();
                        text_renderer.render(&atlas, &mut rpass).unwrap();
                    }
                }
                queue.submit(std::iter::once(encoder.finish()));
                frame.present();
                atlas.trim();
            }
            _ => {}
        }
    });
}

fn build_rect_geometry(rects: &[DrawRect]) -> (Vec<f32>, Vec<u32>) {
    let mut verts = Vec::new();
    let mut indices = Vec::new();
    for r in rects {
        let x0 = r.rect.origin.x;
        let y0 = r.rect.origin.y;
        let x1 = r.rect.max_x();
        let y1 = r.rect.max_y();
        let c = r.color;
        let base = (verts.len() / 6) as u32;
        verts.extend_from_slice(&[x0, y0, c[0], c[1], c[2], c[3]]);
        verts.extend_from_slice(&[x1, y0, c[0], c[1], c[2], c[3]]);
        verts.extend_from_slice(&[x0, y1, c[0], c[1], c[2], c[3]]);
        verts.extend_from_slice(&[x1, y1, c[0], c[1], c[2], c[3]]);
        indices.extend_from_slice(&[base, base + 1, base + 2, base + 1, base + 3, base + 2]);
    }
    (verts, indices)
}

fn build_canvas_quad(
    rect: RectF,
    scale: f32,
    offset_x: f32,
    offset_y: f32,
    img_w: f32,
    img_h: f32,
) -> (Vec<f32>, Vec<u32>) {
    let x0 = rect.origin.x;
    let y0 = rect.origin.y;
    let x1 = rect.max_x();
    let y1 = rect.max_y();
    let (cw, ch) = (x1 - x0, y1 - y0);
    let u0 = offset_x / img_w;
    let v0 = offset_y / img_h;
    let u1 = (offset_x + cw / scale) / img_w;
    let v1 = (offset_y + ch / scale) / img_h;
    let verts = vec![
        x0, y0, u0, v0, //
        x1, y0, u1, v0, //
        x0, y1, u0, v1, //
        x1, y1, u1, v1, //
    ];
    let indices = vec![0u32, 1, 2, 1, 3, 2];
    (verts, indices)
}

fn image_to_rgba(img: &easel_core::Image) -> Vec<u8> {
    let num_px = (img.width * img.height) as usize;
    let mut rgba = Vec::with_capacity(num_px * 4);
    for chunk in img.data.chunks_exact(3) {
        rgba.push(chunk[0].clamp(0.0, 255.0) as u8);
        rgba.push(chunk[1].clamp(0.0, 255.0) as u8);
        rgba.push(chunk[2].clamp(0.0, 255.0) as u8);
        rgba.push(255u8);
    }
    rgba
}
