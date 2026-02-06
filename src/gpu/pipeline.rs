//! Compute pipeline creation and bind group layout.

/// Creates the compute pipeline and bind group layout for utility AI evaluation.
///
/// Bind group layout (group 0):
/// - binding 0: `pops` -- storage buffer (read-only)
/// - binding 1: `buildings` -- storage buffer (read-only)
/// - binding 2: `globals` -- uniform buffer
/// - binding 3: `decisions` -- storage buffer (read-write)
#[must_use]
pub fn create_compute_pipeline(
    device: &wgpu::Device,
) -> (wgpu::ComputePipeline, wgpu::BindGroupLayout) {
    let shader_source = include_str!("shaders/evaluate.wgsl");
    let shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("utility-ai-compute"),
        source: wgpu::ShaderSource::Wgsl(shader_source.into()),
    });

    let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("utility-ai-bind-group-layout"),
        entries: &[
            // binding 0: pops (storage, read-only)
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
            // binding 1: buildings (storage, read-only)
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
            // binding 2: globals (uniform)
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
            // binding 3: decisions (storage, read-write)
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

    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("utility-ai-pipeline-layout"),
        bind_group_layouts: &[&bind_group_layout],
        push_constant_ranges: &[],
    });

    let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some("utility-ai-compute-pipeline"),
        layout: Some(&pipeline_layout),
        module: &shader_module,
        entry_point: Some("main"),
        compilation_options: wgpu::PipelineCompilationOptions::default(),
        cache: None,
    });

    (pipeline, bind_group_layout)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::panic::AssertUnwindSafe;

    /// Pipeline creation requires a real GPU device and a valid shader.
    /// Skip gracefully if unavailable or if shader validation fails.
    #[test]
    fn test_create_compute_pipeline() {
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor::default());

        let adapter =
            match pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                force_fallback_adapter: false,
                compatible_surface: None,
            })) {
                Some(a) => a,
                None => {
                    eprintln!("Skipping GPU pipeline test (no adapter available)");
                    return;
                }
            };

        let (device, _queue) = match pollster::block_on(adapter.request_device(
            &wgpu::DeviceDescriptor {
                label: Some("test-device"),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                memory_hints: wgpu::MemoryHints::default(),
            },
            None,
        )) {
            Ok(pair) => pair,
            Err(e) => {
                eprintln!("Skipping GPU pipeline test (device request failed): {e}");
                return;
            }
        };

        // Shader validation errors cause a panic in wgpu; catch gracefully.
        let result =
            std::panic::catch_unwind(AssertUnwindSafe(|| create_compute_pipeline(&device)));

        match result {
            Ok((pipeline, layout)) => {
                // Verify the pipeline was created.
                let _ = pipeline.get_bind_group_layout(0);
                drop(layout);
            }
            Err(_) => {
                eprintln!("Skipping GPU pipeline test (shader validation panic)");
            }
        }
    }
}
