# Easel — Development Roadmap

## Project Goal
A high-performance, non-destructive creative environment unifying painterly brush engines, photo editing, procedural material workflows, and comic production under a single node-graph architecture.

## Phase 1: Engine Foundation & Input (v0.1.0 — ✅ Complete)
- ✅ Core pixel buffer (f32 RGB Image with color spaces, save/load, serde)
- ✅ DAG node graph engine (topological sort, event buffer, 6 node types)
- ✅ Async stylus driver with InputBridge (tokio-based)
- ✅ Pluggable brush engine (6 nozzles, 2 deformers, DSP pipeline)
- ✅ PixolBuffer for per-pixel surface data (depth, normal, material_id)
- ✅ Custom wgpu UI framework (widget trait, theme, 5 panels)
- ✅ Canvas zoom/pan, stroke interpolation, dirty-tracked rendering
- ✅ Project serialization (JSON format)
- ✅ 41 tests covering all modules

## Phase 2: Hybrid Workflow Abstraction (Current — In Progress)
- **Goal:** Unify Painting, Inking, and Photo Editing.
- **Key Tasks:**
    - [ ] Layer stack UI (right drawer list with add/remove/reorder/visibility)
    - [ ] Undo system (snapshot-based on stroke end)
    - [ ] Tool selection (brush/eraser/fill switching in palette)
    - [ ] File save/load with native dialogs
    - [ ] Brush preview cursor
    - [ ] Advanced blend modes and masking in graph engine
    - [ ] Non-destructive filter/adjustment mask system

## Phase 3: AI & Advanced Materiality
- **Goal:** Integrate Generative AI and PBR/RR workflows.
- **Key Tasks:**
    - [ ] Wacom stylus tilt/speed support driving brush parameters
    - [ ] Physics-simulated brush media (oil bleed, watercolor diffusion)
    - [ ] GPU compute shaders for procedural texturing
    - [ ] Material Node Graph for PBR material creation
    - [ ] 3D projection node (2D → 3D UV mapping)
    - [ ] Local diffusion node containers for inpainting/outpainting
    - [ ] PBR/RR material authoring pipeline
    - [ ] Universal viewport for real-time material preview

## Phase 4: Production Pipeline & Specialization
- **Goal:** Deliver specialized tools for comics and professional output.
- **Key Tasks:**
    - [ ] Comic production toolset (panels, layouts, speech bubbles)
    - [ ] High-fidelity CMYK/RGB color management and print-ready export
    - [ ] Plugin API for batch automation pipelines
