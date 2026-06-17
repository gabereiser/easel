# Easel — Architectural Specification (as Implemented)

## I. Core Engine & Data Flow (Rust Implementation)
- **Core Structure:** Application state is managed by the winit event loop in `main.rs`. The `GraphEngine` module provides DAG-based node graph execution using Rust's ownership system. Node resolution uses topological sort of the dependency graph.
- **Data Abstraction:** All raster data flows as `Image` (f32 RGB pixel buffer with color space, serde, save/load). Textured data uses WGSL shaders on the GPU. The `PixolBuffer` provides per-pixel depth/normal/material_id for 3D-aware brush workflows.
- **Input Layer:** The `StylusDriver` module runs asynchronously (tokio) and translates raw (X, Y, time) data into `StrokeEvent` structs with pressure. The `InputBridge` connects driver events to the graph engine's event buffer. Currently driven by mouse input in the main app.

## II. Stylus & Canvas Interaction
- **Input Handling:** The winit event loop feeds `CursorMoved` events with mouse button state into the painting system. Stroke events are interpolated at 2px steps and passed to `BrushEngine::process_stroke` or directly to `GraphEngine::push_stroke_event`.
- **Painting Workflow:** The primary painting action is handled by `BrushEngine::apply` in `main.rs`, which directly modifies an `Arc<Mutex<Image>>` shared with the compositor. A node-graph `PaintingNode` alternative exists for graph-based workflows.

## III. Painting Module (Krita/Corel Fidelity)
- **Module Structure:** `BrushEngine` is decoupled from graph resolution. It takes stroke data and calculates final visible pixels based on the selected `BrushNozzle` and `BrushDeformer` pipeline.
- **Nozzle System:** Trait-based `BrushNozzle` with 6 implementations (Circle, Ellipse, Star, Diamond, Texture, AngleNozzle). Each nozzle implements `sample(dx, dy, radius, hardness, ctx) -> f32` — an alpha mask at a given point relative to brush center. The `AngleNozzle` wraps another nozzle and modulates alpha based on surface angle from `SurfaceSample`.
- **Deformer System:** `BrushDeformer` trait with `PushPullDeformer` (ZBrush displacement) and `SmoothDeformer` (Laplacian smoothing). Applied post-nozzle in the pipeline.
- **DSP Pipeline:** `BrushPipeline` chains a nozzle + optional deformer. `evaluate()` returns the final alpha, which the engine uses to blend color onto the canvas.
- **PixolBuffer (3D Foundation):** Each pixel stores `(color, depth, normal, material_id)`. Supports depth displacement and normal recomputation — designed for 3D-projection workflows (Substance Painter style).

## IV. UI & Rendering
- **Rendering API:** **WGPU 0.19** — cross-platform Vulkan/Metal/DX12 abstraction. Vertex/fragment shaders written in WGSL.
- **Text Rendering:** **Glyphon 0.5** — GPU-accelerated glyph atlas. Text is collected as `DrawText` commands each frame and rendered in a single pass.
- **UI Framework:** Custom immediate-mode-like widget system — `Widget` trait with `layout()`, `handle_event()`, `render()`. Each frame, widgets push `DrawRect` and `DrawText` commands into a `DrawFrame` buffer, which the compositor flushes to the GPU.
- **Canvas Pipeline:**
    1. `CanvasArea` holds `Arc<Mutex<Image>>` shared with painting code
    2. On dirty, `image_to_rgba()` converts f32 Image → Vec<u8> RGBA bytes
    3. Texture uploaded to GPU via queue.write_texture
    4. Canvas quad rendered with zoom/pan UV transform in WGSL vertex shader
    5. `u0 = offset_x / img_width`, `u1 = (offset_x + viewport_width/scale) / img_width`
- **Layout:**
    ```
    TitleBar (32px)
    Toolbar (40px) + brush info HUD
    Palette (48px) | CanvasArea (flex) | Drawer (260px)
    ```

## V. Data Flow Architecture
```
  winit events
       │
       ▼
  main.rs event loop
       │
       ├──► UiCompositor::handle_event() → Widget event dispatch
       │       ├── Palette: click swatch → set brush color
       │       ├── Drawer: click/drag slider → update opacity/flow/hardness
       │       ├── CanvasArea: right-click → eyedropper sample pixel
       │       └── CanvasArea: middle-click → pan, scroll → zoom
       │
       ├──► Painting path (main.rs):
       │       CursorMoved → interpolate → BrushEngine::apply → Image → dirty flag
       │
       └──► Graph path (optional):
               push StrokeEvent → GraphEngine::push_stroke_event → process nodes → Image

  AboutToWait:
       UiCompositor::layout() → resize canvas if needed → sync drawer→engine → render()
       render() queues DrawRects + DrawTexts → stored in DrawFrame

  RedrawRequested:
       Build GPU geometry from DrawFrame → upload canvas texture → begin render pass
       → draw canvas quad → draw UI rects → draw text → present
```

## VI. Current State Summary
- **41 tests passing** across all modules
- **Single-window** desktop painting application
- **Fully functional** brush engine, zoom/pan, color picking, brush settings
- **Modular foundations** for node graph, 3D pixol workflows, and custom UI
- **Next priorities:** Layer stack UI, undo system, tool selection, file save/load
