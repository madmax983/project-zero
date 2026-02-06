//! GPU compute for utility AI evaluation via wgpu.
//!
//! Offloads the O(N pops x M buildings) utility scoring to the GPU
//! using a WGSL compute shader.

/// GPU-aligned buffer types and CPU↔GPU marshalling.
pub mod buffers;
/// wgpu device/queue initialization.
pub mod context;
/// GPU-based replacement for `evaluate_actions_system`.
pub mod evaluate;
/// Compute pipeline and bind group layout.
pub mod pipeline;
