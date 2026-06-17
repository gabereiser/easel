// src/core.rs

pub mod color_space;
pub mod image;
pub mod project;
pub mod brush_engine;
pub mod pixol;
pub mod workspace;

// This file now only declares submodules and handles visibility from other modules (e.g., lib.rs).
// Exporting types should happen in src/lib.rs or the consuming module itself.
