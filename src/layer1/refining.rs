// src/layer1/refining.rs

use bevy_ecs::prelude::*;
use crate::layer1::resources::{ColonyResources, RefiningProgress};
use crate::layer1::building::{Building, BuildingType};
use crate::layer1::GridPosition;
use crate::layer1::pop::Pop;

/// System that processes refining at buildings like Lumber Mills and Stone Masons.
///
/// It iterates over buildings with `RefiningProgress`. If a worker (`Pop`) is nearby
/// and input resources are available, it increments progress. Upon completion,
/// it consumes input resources and produces refined resources.
pub fn process_refining_system(world: &mut World) {
    let worker_positions: Vec<GridPosition> = world
        .query::<(&Pop, &GridPosition)>()
        .iter(world)
        .map(|(_, pos)| *pos)
        .collect();

    let mut updates = Vec::new();

    // Snapshot resources needed for checking conditions to avoid borrowing conflict
    let (wood, planks, max_planks, stone, blocks, max_blocks) = {
        let res = world.resource::<ColonyResources>();
        (res.wood, res.planks, res.max_planks, res.stone, res.blocks, res.max_blocks)
    };

    // Iterate buildings (Immutable query)
    let mut query = world.query::<(Entity, &Building, &GridPosition, &RefiningProgress)>();

    for (entity, building, pos, _) in query.iter(world) {
        // Check worker range (manhattan distance <= 10)
        let has_worker = worker_positions.iter().any(|p| (p.x - pos.x).abs() + (p.y - pos.y).abs() <= 10);

        if !has_worker { continue; }

        let (can_refine, input_cost, output_gain) = match building.building_type {
            BuildingType::LumberMill => {
                (wood >= 1.0 && planks < max_planks,
                 ColonyResources { wood: 1.0, ..Default::default() },
                 ColonyResources { planks: 1.0, ..Default::default() })
            },
            BuildingType::StoneMason => {
                (stone >= 1.0 && blocks < max_blocks,
                 ColonyResources { stone: 1.0, ..Default::default() },
                 ColonyResources { blocks: 1.0, ..Default::default() })
            },
            _ => (false, ColonyResources::default(), ColonyResources::default()),
        };

        if can_refine {
            updates.push((entity, 1.0, input_cost, output_gain));
        }
    }

    // Apply updates
    // We apply progress first. If a job completes, we check resources AGAIN before finalizing.
    // This prevents race conditions where multiple buildings compete for the same resource.

    let mut completing_entities = Vec::new();

    for (entity, work, input, output) in &updates {
        if let Some(mut progress) = world.get_mut::<RefiningProgress>(*entity) {
            progress.current += work;
            if progress.is_complete() {
                // Do not reset yet. Queue for resource check.
                // We clone the input/output resources to process them later.
                completing_entities.push((*entity, input.clone(), output.clone()));
            }
        }
    }

    // Process completions
    // We iterate the queued completions and check resources transactionally.

    if !completing_entities.is_empty() {
        for (entity, input, output) in completing_entities {
            let success = {
                let mut resources = world.resource_mut::<ColonyResources>();
                if resources.try_deduct(&input) {
                    resources.add_planks(output.planks);
                    resources.add_blocks(output.blocks);
                    true
                } else {
                    false
                }
            };

            if success {
                // Reset progress
                if let Some(mut progress) = world.get_mut::<RefiningProgress>(entity) {
                    progress.current = 0.0;
                }
            } else {
                // Resource shortage (race condition hit).
                // Clamp progress to max so it stays "ready" and tries again next tick.
                if let Some(mut progress) = world.get_mut::<RefiningProgress>(entity) {
                    progress.current = progress.max;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::resources::{ColonyResources, RefiningProgress};
    use crate::layer1::building::{Building, BuildingType};
    use crate::layer1::refining::process_refining_system;
    use crate::layer1::pop::Pop;
    use crate::layer1::GridPosition;

    #[test]
    fn test_colony_resources_refined_fields() {
        let resources = ColonyResources::default();
        // New fields
        assert_eq!(resources.planks, 0.0);
        assert_eq!(resources.blocks, 0.0);
        // Default caps (can be same as raw for now)
        assert_eq!(resources.max_planks, 50.0);
        assert_eq!(resources.max_blocks, 20.0);
    }

    #[test]
    fn test_building_type_variants() {
        let lm = BuildingType::LumberMill;
        let sm = BuildingType::StoneMason;
        assert_eq!(lm.label(), "Lumber Mill");
        assert_eq!(sm.label(), "Stone Mason");
    }

    #[test]
    fn test_refining_progress_component() {
        let progress = RefiningProgress {
            current: 0.0,
            max: 100.0,
        };
        assert!(!progress.is_complete());
    }

    #[test]
    fn test_process_refining_lumber_mill() {
        let mut world = World::new();

        // Setup Resources: Has Wood, No Planks
        let mut resources = ColonyResources::default();
        resources.wood = 10.0;
        resources.planks = 0.0;
        world.insert_resource(resources);

        // Spawn Lumber Mill at (5, 5)
        world.spawn((
            Building { building_type: BuildingType::LumberMill },
            GridPosition { x: 5, y: 5 },
            RefiningProgress { current: 0.0, max: 10.0 }, // 10 ticks to refine
        ));

        // Spawn Worker nearby at (5, 6)
        world.spawn((
            Pop,
            GridPosition { x: 5, y: 6 },
        ));

        // Run system
        // 1. Should detect worker
        // 2. Should detect valid input (Wood > 0)
        // 3. Should increment progress
        process_refining_system(&mut world);

        let progress = world.query::<&RefiningProgress>().single(&world);
        assert!(progress.current > 0.0);
    }

    #[test]
    fn test_process_refining_consumes_input_on_complete() {
        let mut world = World::new();

        // Setup Resources
        let mut resources = ColonyResources::default();
        resources.wood = 10.0;
        resources.planks = 0.0;
        world.insert_resource(resources);

        // Spawn Lumber Mill almost done
        world.spawn((
            Building { building_type: BuildingType::LumberMill },
            GridPosition { x: 5, y: 5 },
            RefiningProgress { current: 9.9, max: 10.0 },
        ));

        // Spawn Worker
        world.spawn((Pop, GridPosition { x: 5, y: 6 }));

        // Run system to complete
        process_refining_system(&mut world);

        let res = world.resource::<ColonyResources>();
        // Input consumed
        assert!((res.wood - 9.0).abs() < f32::EPSILON); // 10.0 - 1.0 = 9.0
        // Output produced
        assert!((res.planks - 1.0).abs() < f32::EPSILON); // 0.0 + 1.0 = 1.0

        // Progress reset
        let progress = world.query::<&RefiningProgress>().single(&world);
        assert!(progress.current < 1.0); // Should wrap or reset to 0
    }

    #[test]
    fn test_refining_stops_if_no_input() {
        let mut world = World::new();
        let mut resources = ColonyResources::default();
        resources.wood = 0.0; // No wood
        world.insert_resource(resources);

        world.spawn((
            Building { building_type: BuildingType::LumberMill },
            GridPosition { x: 5, y: 5 },
            RefiningProgress::default(),
        ));
        world.spawn((Pop, GridPosition { x: 5, y: 5 }));

        process_refining_system(&mut world);

        let progress = world.query::<&RefiningProgress>().single(&world);
        assert_eq!(progress.current, 0.0);
    }

    #[test]
    fn test_refining_stops_if_output_full() {
        let mut world = World::new();
        let mut resources = ColonyResources::default();
        resources.wood = 10.0;
        resources.planks = 50.0; // Full
        resources.max_planks = 50.0;
        world.insert_resource(resources);

        world.spawn((
            Building { building_type: BuildingType::LumberMill },
            GridPosition { x: 5, y: 5 },
            RefiningProgress::default(),
        ));
        world.spawn((Pop, GridPosition { x: 5, y: 5 }));

        process_refining_system(&mut world);

        let progress = world.query::<&RefiningProgress>().single(&world);
        assert_eq!(progress.current, 0.0);
    }

    #[test]
    fn test_race_condition_underflow() {
        let mut world = World::new();
        let mut resources = ColonyResources::default();
        resources.wood = 1.0;
        resources.planks = 0.0;
        world.insert_resource(resources);

        // Mill 1
        world.spawn((
            Building { building_type: BuildingType::LumberMill },
            GridPosition { x: 0, y: 0 },
            RefiningProgress { current: 9.9, max: 10.0 },
        ));
        world.spawn((Pop, GridPosition { x: 0, y: 1 }));

        // Mill 2
        world.spawn((
            Building { building_type: BuildingType::LumberMill },
            GridPosition { x: 10, y: 10 },
            RefiningProgress { current: 9.9, max: 10.0 },
        ));
        world.spawn((Pop, GridPosition { x: 10, y: 11 }));

        process_refining_system(&mut world);

        let res = world.resource::<ColonyResources>();
        // With current bug, wood should be -1.0 (1.0 - 1.0 - 1.0)
        assert!(res.wood >= 0.0, "Wood should not underflow: {}", res.wood);
    }
}
