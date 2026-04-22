//! The Bridge to Silicon (GPU Acceleration Module).
//!
//! This module houses the core infrastructure for offloading massive computational tasks
//! from the CPU to the GPU. Specifically, it initializes the `wgpu` state required
//! for executing our parallelized Utility AI evaluation shader.
//!
//! In a colony of thousands of souls, calculating the optimal action (eat, work, sleep, riot)
//! for every individual simultaneously will bring a CPU to its knees. By dispatching these
//! mathematical matrices to the GPU, we unlock the scale necessary for true simulation.

use bevy_ecs::prelude::*;
use wgpu;

/// The central nervous system of our GPU computations.
///
/// `GpuContext` acts as a Bevy `Resource`, holding the critical `wgpu` handles
/// required to communicate with the physical hardware. It sets up the pipeline,
/// the layout for memory bindings, and the execution queues.
///
/// # Examples
///
/// Because `new()` is `async`, it is typically invoked during application setup
/// before the Bevy app begins its normal synchronous loop.
///
/// ```
/// # use scale::gpu::context::GpuContext;
/// # use pollster::block_on;
/// # fn main() -> anyhow::Result<()> {
/// # block_on(async {
/// let context = GpuContext::new().await?;
/// // Now it can be inserted into the Bevy app as a resource.
/// # Ok(())
/// # })
/// # }
/// ```
///
/// # Panics
///
/// The creation itself will return an `Err` if no compatible GPU is found,
/// but it expects valid WGSL shaders to exist at compile time in `src/gpu/shaders/utility_ai.wgsl`.
#[derive(Resource)]
pub struct GpuContext {
    /// The handle to the physical GPU device, used to create buffers and pipelines.
    pub device: wgpu::Device,
    /// The command queue for submitting work and writing to buffers.
    pub queue: wgpu::Queue,
    /// The blueprint for how CPU memory buffers bind to GPU shader inputs.
    pub bind_group_layout: wgpu::BindGroupLayout,
    /// The compiled instructions (the Compute Shader) for evaluating Utility AI.
    pub pipeline: wgpu::ComputePipeline,
}

impl GpuContext {
    /// Bootstraps the connection to the GPU and compiles the compute shaders.
    ///
    /// This function requests a high-performance adapter, sets up the exact memory
    /// binding layout expected by our `utility_ai.wgsl` shader, and compiles the
    /// compute pipeline.
    ///
    /// The pipeline layout expects exactly four bindings:
    /// 1. Pop Inputs (Storage Buffer, Read Only)
    /// 2. Building Inputs (Storage Buffer, Read Only)
    /// 3. Global State (Uniform Buffer, Read Only)
    /// 4. Decisions Output (Storage Buffer, Write Only)
    ///
    /// # Errors
    ///
    /// Returns an error if the system does not have a compatible graphics adapter
    /// or fails to acquire a logical device handle.
    pub async fn new() -> anyhow::Result<Self> {
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: None,
                force_fallback_adapter: false,
            })
            .await
            .ok_or_else(|| anyhow::anyhow!("No GPU adapter found"))?;

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("scale-gpu-device"),
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::downlevel_defaults(),
                    memory_hints: wgpu::MemoryHints::default(),
                },
                None,
            )
            .await?;

        // Create bind group layout for our utility AI shader
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("utility-ai-bind-group-layout"),
            entries: &[
                // Binding 0: Pop Inputs (Storage Buffer, ReadOnly)
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // Binding 1: Building Inputs (Storage Buffer, ReadOnly)
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // Binding 2: Global State (Uniform Buffer)
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // Binding 3: Decisions (Storage Buffer, WriteOnly)
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        // Load shader module
        let shader = device.create_shader_module(wgpu::include_wgsl!("shaders/utility_ai.wgsl"));

        // Create pipeline layout
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("utility-ai-pipeline-layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        // Create compute pipeline
        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("utility-ai-pipeline"),
            layout: Some(&pipeline_layout),
            module: &shader,
            entry_point: Some("main"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            cache: None,
        });

        Ok(Self {
            device,
            queue,
            bind_group_layout,
            pipeline,
        })
    }
}

/// Creates a raw `wgpu::Instance` capable of selecting the best available graphics backend.
///
/// This is a convenience function mostly used internally or for isolated testing
/// without needing the full `GpuContext` setup.
///
/// # Examples
///
/// ```
/// # use scale::gpu::context::create_instance;
/// let instance = create_instance();
/// // The instance can now be used to request physical adapters.
/// ```
pub fn create_instance() -> wgpu::Instance {
    wgpu::Instance::new(&wgpu::InstanceDescriptor {
        backends: wgpu::Backends::all(),
        ..Default::default()
    })
}
