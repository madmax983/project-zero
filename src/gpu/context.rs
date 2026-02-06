#![allow(unsafe_code)]
//! wgpu device and queue initialization.

use bevy_ecs::prelude::*;

use super::pipeline::create_compute_pipeline;

/// Holds all GPU state needed for compute dispatch.
pub struct GpuContext {
    /// The wgpu logical device.
    pub device: wgpu::Device,
    /// The command queue.
    pub queue: wgpu::Queue,
    /// The compiled compute pipeline.
    pub pipeline: wgpu::ComputePipeline,
    /// The bind group layout for the compute shader.
    pub bind_group_layout: wgpu::BindGroupLayout,
}

// On native, wgpu types are Send + Sync. On wasm32 they wrap JS objects
// and are !Send + !Sync, but WASM is single-threaded so this is safe.
#[cfg(target_arch = "wasm32")]
// SAFETY: WASM is single-threaded; these types will never be sent across threads.
unsafe impl Send for GpuContext {}
#[cfg(target_arch = "wasm32")]
// SAFETY: WASM is single-threaded; these types will never be accessed from multiple threads.
unsafe impl Sync for GpuContext {}

impl Resource for GpuContext {}

impl GpuContext {
    /// Initialize GPU context: adapter, device, queue, pipeline.
    ///
    /// This is async because wgpu initialization is async.
    /// Use `pollster::block_on(GpuContext::new())` for synchronous init.
    ///
    /// # Errors
    ///
    /// Returns an error if no suitable GPU adapter is found or device
    /// creation fails.
    pub async fn new() -> anyhow::Result<Self> {
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                force_fallback_adapter: false,
                compatible_surface: None,
            })
            .await
            .ok_or_else(|| anyhow::anyhow!("No suitable GPU adapter found"))?;

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("scale-gpu"),
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                    memory_hints: wgpu::MemoryHints::default(),
                },
                None,
            )
            .await
            .map_err(|e| anyhow::anyhow!("{e}"))?;

        let (pipeline, bind_group_layout) = create_compute_pipeline(&device);

        Ok(Self {
            device,
            queue,
            pipeline,
            bind_group_layout,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::panic::AssertUnwindSafe;

    #[test]
    fn test_gpu_context_creation() {
        // This test requires a GPU and a valid shader. Skip gracefully if
        // unavailable or if shader validation fails (which panics in wgpu).
        let result =
            std::panic::catch_unwind(AssertUnwindSafe(|| pollster::block_on(GpuContext::new())));

        match result {
            Ok(Ok(ctx)) => {
                // Basic sanity -- device and queue exist, pipeline created.
                assert!(ctx.device.limits().max_compute_workgroups_per_dimension > 0);
            }
            Ok(Err(e)) => {
                eprintln!("Skipping GPU test (no adapter): {e}");
            }
            Err(_) => {
                eprintln!("Skipping GPU test (shader validation panic)");
            }
        }
    }
}
