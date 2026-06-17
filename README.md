# Easel: The Universal Creative Operating System

## Overview
**Easel** is the definitive, all-in-one creative environment designed to serve every facet of visual fine arts. By unifying advanced painterly brush engines, professional photo editing, comic production, and next-generation procedural material workflows under a single, non-destructive **Node-Graph Engine**, Easel acts as a universal hub for all visually creative work (excluding creative coding).

Easel is built for the professional artist, featuring native, low-latency Wacom integration and a hardware-accelerated core that ensures your tools respond as fast as your imagination.

## Core Pillars

### 1. The Universal Hybrid Engine
* **Painterly & Illustrator Workflow:** Industry-leading brush engines designed for natural media simulation, inking, and high-fidelity drawing.
* **Imaging & Photo Editing:** Non-destructive layer-stack abstraction over a powerful node backend for professional-grade photo manipulation and complex compositing.
* **Materiality (PBR/RR):** Comprehensive support for Physically Based Rendering (PBR) and Reflective/Relief (RR) workflows, allowing for seamless texture authoring in 2D, 3D, and 3D-projection contexts.

### 2. AI-Native Integration
* **Local Diffusion Models:** Direct integration of local AI diffusion for inpainting, outpainting, and procedural generation, treated as first-class citizens in the node graph.

### 3. Specialized Pipeline Support
* **Comic Production:** Purpose-built toolsets for panel layout, speech bubbles, sequential storytelling, and high-volume asset management.
* **Professional Digital Art:** Unified workspace handling everything from initial sketch to final print-ready production, including color management and CMYK/RGB conversion.

### 4. Technical Architecture
* **Non-Destructive Node-Graph:** Every action is a modular, editable node, providing infinite iteration without data loss.
* **Intent-Driven Stack UI:** The interface provides a familiar "Layer Stack" experience for rapid, tactile work while the underlying engine maintains the robust node-based logic.
* **Wacom Optimized:** Precision-engineered for low-latency stylus input, pressure sensitivity, and customizable hardware-mapped shortcuts.

## Current Implementation Status (v0.1.0)

Easel is currently a functional **desktop painting application** with a full custom UI framework, pluggable brush engine, and node-graph backend. Below is what's actually built and working today.

### What Works

- **Custom UI shell:** winit + wgpu window with dark theme (DaVinci Resolve / ZBrush style). Title bar, toolbar, left palette, right drawer, and canvas area all rendered via wgpu with zero external UI dependencies.
- **Brush engine:** Trait-based `BrushNozzle` system with 6 nozzle types (Circle, Ellipse, Star, Diamond, Texture, AngleNozzle), 2 deformers (PushPull, Smooth), and a composable `BrushPipeline`. Full stroke interpolation at 2px steps. Configurable size, opacity, flow, hardness.
- **Canvas painting:** Mouse-drag painting with smooth stroke interpolation, dirty-tracked texture upload, and canvas content preserved on resize.
- **Zoom & pan:** Scroll-to-zoom (0.1x–32x, centered on cursor), middle-click-drag to pan. Transform applied to painting coordinates, eyedropper sampling, and canvas display UVs.
- **Eyedropper:** Right-click on canvas to sample pixel color.
- **Color swatches:** 14 clickable color swatches in the left palette panel.
- **Brush controls:** Opacity, Flow, and Hardness sliders in the right drawer with click-and-drag interaction.
- **PixolBuffer:** ZBrush-inspired per-pixel surface data (depth, normal, material_id, angle) with displacement and normal recomputation.
- **Node graph engine:** Full DAG with topological sort, 6 node types (Source, Stroke, Painting, Layer, Adjustment, Composite), event buffer for real-time stroke injection.
- **Stylus input pipeline:** Async `StylusDriver` with `InputBridge` connecting to the graph engine.
- **GPU round-tripping:** wgpu-based texture upload/download via `Renderer`.
- **Project serialization:** JSON-based save/load (`EaselProject` with layers, canvas size, metadata).
- **41 tests passing:** Unit, integration, async, and roundtrip tests across all modules.

### Key Technical Details

- **Dependencies:** wgpu 0.19, winit 0.30, glyphon 0.5, lyon 1.0, cosmic-text 0.11, euclid 0.22, tokio 1.x, serde 1.x
- **UI paradigm:** Custom immediate-mode-like widget tree — widgets push `DrawRect`/`DrawText` commands each frame, compositor flushes them via wgpu
- **Architecture:** `main.rs` owns the winit event loop, wgpu device/queue, `UiCompositor`, `BrushEngine`, and shared `Arc<Mutex<Image>>` canvas

### What's Planned Next

- Layer stack UI in the right drawer (add/remove/reorder, visibility toggle)
- Undo system (snapshot-based on stroke end)
- Tool selection (brush/eraser/fill switching in palette)
- File save/load integration with native dialogs
- Brush preview cursor
- GPU compute shaders for advanced effects

---
*Easel: Everything for the artist, all in one place.*
