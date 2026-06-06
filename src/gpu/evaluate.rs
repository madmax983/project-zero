//! GPU-based replacement for `evaluate_actions_system`.
//!
//! Orchestrates: extract ECS data → upload to GPU → dispatch compute → readback → apply to ECS.

use bevy_ecs::prelude::*;

use super::buffers::{
    extract_building_inputs, extract_global_state, extract_pop_inputs, GpuBuffers,
    GpuPersistentBuffers, GpuPopDecision,
};
use crate::gpu::context::GpuContext;
use crate::layer1::utility_ai::evaluate_actions_system;
use crate::layer1::utility_ai::{ActionType, PopAction, StartPlan};

/// GPU-accelerated action evaluation system.
///
/// Drop-in replacement for `evaluate_actions_system` with signature `fn(&mut World)`.
/// Falls back to the CPU `evaluate_actions_system` if `GpuContext` is not available.
#[allow(clippy::cast_possible_truncation)]
pub fn gpu_evaluate_actions(world: &mut World) {
    // Fall back to CPU path if no GPU context
    if !world.contains_resource::<GpuContext>() {
        evaluate_actions_system(world);
        return;
    }

    // Ensure GpuBuffers resource exists
    if !world.contains_resource::<GpuBuffers>() {
        world.init_resource::<GpuBuffers>();
    }

    // Extract persistent buffers (if they exist) or init default
    let mut persistent_buffers = world
        .remove_resource::<GpuPersistentBuffers>()
        .unwrap_or_default();

    // Use resource_scope to get mutable access to buffers while keeping world access
    world.resource_scope::<GpuBuffers, _>(|world, mut buffers| {
        // Deref Mut<GpuBuffers> once to allow splitting borrows on fields
        let buffers = &mut *buffers;

        // 1. Extract data from ECS
        extract_pop_inputs(world, &mut buffers.pop_entities, &mut buffers.pop_inputs);

        if buffers.pop_inputs.is_empty() {
            return;
        }

        extract_building_inputs(
            world,
            &mut buffers.building_entities,
            &mut buffers.building_inputs,
        );

        let global_state = extract_global_state(
            world,
            buffers.pop_inputs.len() as u32,
            buffers.building_inputs.len() as u32,
        );

        // 2. Get GPU context
        // We can access GpuContext because we are inside resource_scope, so world still has it
        let gpu = world.resource::<GpuContext>();

        // 3. Dispatch and readback
        let decisions = dispatch_and_readback(
            gpu,
            &mut persistent_buffers,
            &buffers.pop_inputs,
            &buffers.building_inputs,
            &global_state,
        );

        let Some(decisions) = decisions else {
            return;
        };

        // 4. Apply decisions to ECS
        apply_decisions(
            world,
            &buffers.pop_entities,
            &buffers.building_entities,
            &decisions,
        );
    });

    // Put persistent buffers back
    world.insert_resource(persistent_buffers);
}

/// Helper to ensure a buffer exists and has sufficient capacity.
/// Returns true if the buffer was (re)created.
fn ensure_buffer(
    device: &wgpu::Device,
    buffer: &mut Option<wgpu::Buffer>,
    capacity: &mut u64,
    required_size: u64,
    usage: wgpu::BufferUsages,
    label: &str,
) -> bool {
    // If buffer exists and fits, keep it.
    if let Some(buf) = buffer {
        if *capacity >= required_size {
            return false;
        }
        // Buffer is too small, drop it (will happen when we overwrite the Option)
        let _ = buf;
    }

    // Allocate new buffer
    // For stability, we can round up to next power of two, but strictly required size is also safe.
    // Using next_power_of_two avoids frequent reallocations if size grows slowly.
    let new_capacity = required_size.next_power_of_two().max(32);

    let new_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some(label),
        size: new_capacity,
        usage,
        mapped_at_creation: false,
    });

    *buffer = Some(new_buffer);
    *capacity = new_capacity;
    true
}

/// Upload data, dispatch the compute shader, and read back results.
#[allow(clippy::cast_possible_truncation, clippy::too_many_lines)]
fn dispatch_and_readback(
    gpu: &GpuContext,
    cache: &mut GpuPersistentBuffers,
    pop_inputs: &[super::buffers::GpuPopInput],
    building_inputs: &[super::buffers::GpuBuildingInput],
    global_state: &super::buffers::GpuGlobalState,
) -> Option<Vec<GpuPopDecision>> {
    let mut bind_group_dirty = false;

    // 1. Pop Buffer
    let pop_size = std::mem::size_of_val(pop_inputs) as u64;
    if ensure_buffer(
        &gpu.device,
        &mut cache.pop_buffer,
        &mut cache.pop_capacity,
        pop_size,
        wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        "pop-input-buffer",
    ) {
        bind_group_dirty = true;
    }

    let pop_buffer = match cache.pop_buffer.as_ref() {
        Some(b) => b,
        None => {
            log::error!("Missing pop buffer");
            return None;
        }
    };

    gpu.queue
        .write_buffer(pop_buffer, 0, bytemuck::cast_slice(pop_inputs));

    // 2. Building Buffer
    let building_size = if building_inputs.is_empty() {
        32
    } else {
        std::mem::size_of_val(building_inputs) as u64
    };

    if ensure_buffer(
        &gpu.device,
        &mut cache.building_buffer,
        &mut cache.building_capacity,
        building_size,
        wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        "building-input-buffer",
    ) {
        bind_group_dirty = true;
    }

    let building_buffer = match cache.building_buffer.as_ref() {
        Some(b) => b,
        None => {
            log::error!("Missing building buffer");
            return None;
        }
    };

    if !building_inputs.is_empty() {
        gpu.queue
            .write_buffer(building_buffer, 0, bytemuck::cast_slice(building_inputs));
    }

    // 3. Global Buffer
    let global_size = std::mem::size_of::<super::buffers::GpuGlobalState>() as u64;
    if ensure_buffer(
        &gpu.device,
        &mut cache.global_buffer,
        &mut cache.global_capacity,
        global_size,
        wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        "global-state-buffer",
    ) {
        bind_group_dirty = true;
    }

    let global_buffer = match cache.global_buffer.as_ref() {
        Some(b) => b,
        None => {
            log::error!("Missing global buffer");
            return None;
        }
    };

    gpu.queue
        .write_buffer(global_buffer, 0, bytemuck::bytes_of(global_state));

    // 4. Decision Buffer (Output)
    let decision_size =
        (pop_inputs.len() * std::mem::size_of::<GpuPopDecision>()) as wgpu::BufferAddress;
    if ensure_buffer(
        &gpu.device,
        &mut cache.decision_buffer,
        &mut cache.decision_capacity,
        decision_size,
        wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
        "decision-output-buffer",
    ) {
        bind_group_dirty = true;
    }

    let decision_buffer = match cache.decision_buffer.as_ref() {
        Some(b) => b,
        None => {
            log::error!("Missing decision buffer");
            return None;
        }
    };

    // 5. Staging Buffer
    ensure_buffer(
        &gpu.device,
        &mut cache.staging_buffer,
        &mut cache.staging_capacity,
        decision_size,
        wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
        "decision-staging-buffer",
    );

    let staging_buffer = match cache.staging_buffer.as_ref() {
        Some(b) => b,
        None => {
            log::error!("Missing staging buffer");
            return None;
        }
    };

    // 6. Bind Group
    if bind_group_dirty || cache.bind_group.is_none() {
        let bg = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("utility-ai-bind-group"),
            layout: &gpu.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: pop_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: building_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: global_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: decision_buffer.as_entire_binding(),
                },
            ],
        });
        cache.bind_group = Some(bg);
    }

    let bind_group = match cache.bind_group.as_ref() {
        Some(bg) => bg,
        None => {
            log::error!("Missing bind group");
            return None;
        }
    };

    // 7. Dispatch
    let workgroup_count = (pop_inputs.len() as u32).div_ceil(64);
    let mut encoder = gpu
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("utility-ai-encoder"),
        });

    {
        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("utility-ai-pass"),
            timestamp_writes: None,
        });
        pass.set_pipeline(&gpu.pipeline);
        pass.set_bind_group(0, bind_group, &[]);
        pass.dispatch_workgroups(workgroup_count, 1, 1);
    }

    // Copy to staging
    encoder.copy_buffer_to_buffer(decision_buffer, 0, staging_buffer, 0, decision_size);
    gpu.queue.submit(std::iter::once(encoder.finish()));

    // 8. Blocking Readback
    let buffer_slice = staging_buffer.slice(..decision_size); // Slice only what we need
    let (tx, rx) = std::sync::mpsc::channel();
    buffer_slice.map_async(
        wgpu::MapMode::Read,
        move |result: Result<(), wgpu::BufferAsyncError>| {
            let _ = tx.send(result);
        },
    );
    gpu.device.poll(wgpu::Maintain::Wait);

    if rx.recv().ok()?.is_err() {
        // Map failed
        return None;
    }

    let data = buffer_slice.get_mapped_range();
    let decisions: Vec<GpuPopDecision> = bytemuck::cast_slice(&data).to_vec();
    drop(data);
    staging_buffer.unmap();

    Some(decisions)
}

/// Apply GPU decisions back to the ECS world.
fn apply_decisions(
    world: &mut World,
    pop_entities: &[Entity],
    building_entities: &[Entity],
    decisions: &[GpuPopDecision],
) {
    for (i, decision) in decisions.iter().enumerate() {
        if decision.switched != 1 {
            continue;
        }

        let pop_entity = pop_entities[i];
        let action_type = action_type_from_u32(decision.best_action);

        if let Some(mut pop_action) = world.get_mut::<PopAction>(pop_entity) {
            pop_action.current = action_type;
            pop_action.current_utility = decision.best_utility;
            pop_action.ticks_committed = 0;
        }

        let target = if decision.target_index == u32::MAX {
            None
        } else {
            #[allow(clippy::cast_possible_truncation)]
            building_entities
                .get(decision.target_index as usize)
                .copied()
        };

        world.entity_mut(pop_entity).insert(StartPlan {
            action: action_type,
            target,
        });
    }
}

/// Maps a u32 action index back to an `ActionType`.
const fn action_type_from_u32(v: u32) -> ActionType {
    match v {
        0 => ActionType::SatisfyHunger,
        1 => ActionType::SatisfyRest,
        2 => ActionType::Socialize,
        3 => ActionType::Explore,
        4 => ActionType::Work,
        5 => ActionType::Repair,
        6 => ActionType::Research,
        7 => ActionType::Haul,
        8 => ActionType::SeekMedicalCare,
        9 => ActionType::BuryCorpse,
        10 => ActionType::FetchTool,
        12 => ActionType::Vandalize,
        13 => ActionType::Binge,
        14 => ActionType::Daze,
        15 => ActionType::Fight,
        16 => ActionType::Refine,
        17 => ActionType::Farm,
        18 => ActionType::Warden,
        19 => ActionType::Sleepwalking,
        20 => ActionType::Tame,
        21 => ActionType::FireStarting,
        22 => ActionType::HideInRoom,
        23 => ActionType::SadWander,
        24 => ActionType::FetchClothing,
        25 => ActionType::Surgery,
        30 => ActionType::PreCrimeArrest,
        31 => ActionType::ConsumeChemical,
        39 => ActionType::ExtinguishFire,
        40 => ActionType::TreatWounds,
        41 => ActionType::Flee,
        48 => ActionType::MemeticObsession,
        _ => ActionType::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_action_type_from_u32_roundtrip() {
        assert_eq!(action_type_from_u32(0), ActionType::SatisfyHunger);
        assert_eq!(action_type_from_u32(1), ActionType::SatisfyRest);
        assert_eq!(action_type_from_u32(2), ActionType::Socialize);
        assert_eq!(action_type_from_u32(3), ActionType::Explore);
        assert_eq!(action_type_from_u32(4), ActionType::Work);
        assert_eq!(action_type_from_u32(5), ActionType::Repair);
        assert_eq!(action_type_from_u32(6), ActionType::Research);
        assert_eq!(action_type_from_u32(7), ActionType::Haul);
        assert_eq!(action_type_from_u32(8), ActionType::SeekMedicalCare);
        assert_eq!(action_type_from_u32(9), ActionType::BuryCorpse);
        assert_eq!(action_type_from_u32(10), ActionType::FetchTool);
        assert_eq!(action_type_from_u32(11), ActionType::Idle);
        assert_eq!(action_type_from_u32(12), ActionType::Vandalize);
        assert_eq!(action_type_from_u32(13), ActionType::Binge);
        assert_eq!(action_type_from_u32(14), ActionType::Daze);
        assert_eq!(action_type_from_u32(15), ActionType::Fight);
        // Out of range defaults to Idle
        assert_eq!(action_type_from_u32(99), ActionType::Idle);
    }

    #[test]
    fn test_gpu_evaluate_noop_without_context() {
        crate::setup::init_task_pools();
        let mut world = World::new();
        world.insert_resource(crate::layer1::utility_types::UtilityConfig::default());
        world.insert_resource(crate::layer1::resources::ColonyResources::default());
        world.insert_resource(crate::layer1::day_night::DayNightCycle::default());
        world.insert_resource(crate::layer1::taboo::TabooState::default());

        // No GpuContext resource — should return without panicking
        gpu_evaluate_actions(&mut world);
    }

    #[test]
    fn test_gpu_evaluate_noop_no_pops() {
        crate::setup::init_task_pools();
        let mut world = World::new();
        world.insert_resource(crate::layer1::utility_types::UtilityConfig::default());
        world.insert_resource(crate::layer1::resources::ColonyResources::default());
        world.insert_resource(crate::layer1::day_night::DayNightCycle::default());
        world.insert_resource(crate::layer1::taboo::TabooState::default());

        // No pops — should return without panicking even if GpuContext existed
        gpu_evaluate_actions(&mut world);
    }
}
