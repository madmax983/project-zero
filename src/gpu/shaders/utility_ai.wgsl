// Placeholder shader for utility_ai.wgsl
// This file is required for the build to pass.

@group(0) @binding(0) var<storage, read> pops : array<u32>;
@group(0) @binding(1) var<storage, read> buildings : array<u32>;
@group(0) @binding(2) var<uniform> global_state : u32;
@group(0) @binding(3) var<storage, read_write> decisions : array<u32>;

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) global_id : vec3<u32>) {
    // No-op
}
