# Easel Technical Development Backlog

**Goal:** Establish an actionable, prioritized technical roadmap for the Easel Node-Graph Super-Art Application.

---

## ✅ Completed (v0.1.0)

### Core Data Types & Brush System
- [x] **Image type** — f32 RGB pixel buffer with PNG/JPEG/EXR save/load, serde, pixel access
- [x] **Color space pipeline** — sRGB, Adobe RGB, ACES2065-1, Linear with matrix-based XYZ conversion
- [x] **PixolBuffer** — per-pixel depth + normal + material_id with displacement and normal recompute
- [x] **Brush engine** — trait-based `BrushNozzle` (6 types: Circle, Ellipse, Star, Diamond, Texture, AngleNozzle)
- [x] **Brush deformers** — PushPull (ZBrush displacement), Smooth (Laplacian)
- [x] **Brush pipeline** — composable nozzle + deformer chain (DSP-like architecture)
- [x] **Stroke interpolation** — 2px step between mouse positions for smooth lines
- [x] **Brush controls** — opacity/flow/hardness via UI sliders, size via `[`/`]` keys

### Node Graph Engine
- [x] **DAG engine** — topological sort, edge resolution, event buffer, `NodeProcessor` trait
- [x] **6 node types** — Source (checkerboard/solid/gradient), Stroke, Painting (persistent canvas), Layer, Adjustment (brightness/contrast with masks), Composite (Normal/Multiply/Screen/Overlay)

### Input Pipeline
- [x] **StylusDriver** — async event source with subscribe mechanism
- [x] **InputBridge** — connects stylus events to graph engine via tokio mpsc
- [x] **StrokeEvent** — timestamp, x, y, pressure data

### GPU & Rendering
- [x] **wgpu setup** — adapter, device, queue, swapchain
- [x] **Canvas textured quad** — WGSL shader with bilinear filtering, zoom/pan UV transform
- [x] **Rect shader** — colored quad rendering for UI elements
- [x] **Glyphon text rendering** — GPU-accelerated text overlay
- [x] **Dirty-tracked texture upload** — only re-upload canvas when painted
- [x] **Canvas resize preserve** — old content copied into new Image on viewport change

### Custom UI
- [x] **Widget trait** — `id()`, `layout()`, `handle_event()`, `render()`
- [x] **Theme system** — dark blue DaVinci Resolve / ZBrush theme
- [x] **TitleBar** — window title with hover states for close/max/min
- [x] **Toolbar** — horizontal tool strip (visual only)
- [x] **Palette** — 14 clickable color swatches
- [x] **Drawer** — brush settings panel with Opacity/Flow/Hardness sliders
- [x] **CanvasArea** — viewport widget with zoom/pan
- [x] **Container** — horizontal/vertical layout

### Project Management
- [x] **EaselProject** — JSON-serializable document with layers, canvas size, metadata
- [x] **41 tests** — unit, integration, async, and roundtrip tests

---

## ⚙️ Core Systems & Foundational Dependencies

### In Progress / Next Up

1.  **Layer Stack UI** — populate the right drawer with a layer list (add/remove/reorder, visibility toggle). *Dependency:* None (UI-only work).
2.  **Undo System** — snapshot-based (store Image snapshots on stroke end). *Dependency:* Layer Stack.
3.  **Tool Selection** — switching between brush/eraser/fill tools via palette. *Dependency:* None.
4.  **File Save/Load** — wire Easel project format to native file dialogs. *Dependency:* None.

### Medium Term

5.  **Advanced Brush Engine Module:**
    - Physics-simulated media emulation (Oil/Watercolor bleed, Charcoal dust scattering)
    - Brush parameters controlled by multiple inputs (Pressure + Tilt + Speed)
    - *Dependency:* Wacom/stylus integration with tilt support
6.  **Layer-as-Node Compositing (Graph Workflow):**
    - Graph nodes for traditional masking, advanced blend modes (Overlay, Soft Light)
    - Non-destructive color grading/LUT application
    - *Dependency:* Core graph engine V2
7.  **Smart Object Transformation Node:**
    - Nodes that transform, scale, or warp entire node groups
    - *Dependency:* Layer-as-Node Compositing

### Longer Term

8.  **Material Node Graph (MNG):** Dedicated subgraph for PBR material creation
9.  **2D Projection Node:** Project UV/texture data onto arbitrary 2D shapes
10. **Generative AI Integration:** Local diffusion nodes for inpainting/outpainting
11. **Comic Production Toolset:** Panel layout, speech bubbles, sequential storytelling
12. **High-Fidelity Color Management:** CMYK/RGB conversion and print-ready export
