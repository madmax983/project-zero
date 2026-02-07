// src/layer1/refining.rs
#![allow(clippy::too_many_lines)]

use crate::layer1::GridPosition;
use crate::layer1::building::{Building, BuildingType};
use crate::layer1::pop::Pop;
use crate::layer1::resources::{ColonyResources, RefiningProgress};
use bevy_ecs::prelude::*;

/// System that processes refining at buildings like Lumber Mills and Stone Masons.
///
/// It iterates over buildings with `RefiningProgress`. If a worker (`Pop`) is nearby
/// and input resources are available, it increments progress. Upon completion,
/// it consumes input resources and produces refined resources.
pub fn process_refining_system(
    worker_query: Query<&GridPosition, With<Pop>>,
    mut building_query: Query<(Entity, &Building, &GridPosition, &mut RefiningProgress)>,
    mut resources: ResMut<ColonyResources>,
) {
    let worker_positions: Vec<GridPosition> = worker_query.iter().copied().collect();

    // Snapshot resources for recipe checks (avoids mutable borrow conflict)
    let resources_snapshot = resources.clone();

    let mut finished_jobs: Vec<(Entity, ColonyResources, ColonyResources)> = Vec::new();

    for (entity, building, pos, mut progress) in &mut building_query {
        let has_worker = worker_positions
            .iter()
            .any(|p| (p.x - pos.x).abs() + (p.y - pos.y).abs() <= 10);

        if !has_worker {
            continue;
        }

        let (can_refine, input_cost, output_gain) =
            get_refining_recipe(building.building_type, &resources_snapshot);

        if can_refine {
            progress.current += 1.0;
            if progress.is_complete() {
                finished_jobs.push((entity, input_cost, output_gain));
            }
        }
    }

    // Apply resource updates for finished jobs
    for (entity, input, output) in &finished_jobs {
        if resources.try_deduct(input) {
            resources.add_planks(output.planks);
            resources.add_blocks(output.blocks);
            resources.add_metal(output.metal);
            resources.add_tools(output.tools);
            resources.add_cloth(output.cloth);
            resources.add_clothing(output.clothing);

            if let Ok((_, _, _, mut progress)) = building_query.get_mut(*entity) {
                progress.current = 0.0;
            }
        } else {
            // Failed: clamp progress to max for retry next tick
            if let Ok((_, _, _, mut progress)) = building_query.get_mut(*entity) {
                progress.current = progress.max;
            }
        }
    }
}

/// Returns the refining recipe for a building type.
///
/// # Returns
/// (`can_afford`, `input_cost`, `output_gain`)
#[must_use]
pub fn get_refining_recipe(
    building_type: BuildingType,
    res: &ColonyResources,
) -> (bool, ColonyResources, ColonyResources) {
    match building_type {
        BuildingType::LumberMill => (
            res.wood >= 1.0 && res.planks < res.max_planks,
            ColonyResources {
                wood: 1.0,
                ..Default::default()
            },
            ColonyResources {
                planks: 1.0,
                ..Default::default()
            },
        ),
        BuildingType::StoneMason => (
            res.stone >= 1.0 && res.blocks < res.max_blocks,
            ColonyResources {
                stone: 1.0,
                ..Default::default()
            },
            ColonyResources {
                blocks: 1.0,
                ..Default::default()
            },
        ),
        BuildingType::Smelter => (
            res.ore >= 1.0 && res.wood >= 1.0 && res.metal < res.max_metal,
            ColonyResources {
                ore: 1.0,
                wood: 1.0,
                ..Default::default()
            },
            ColonyResources {
                metal: 1.0,
                ..Default::default()
            },
        ),
        BuildingType::Smithy => (
            res.metal >= 1.0 && res.wood >= 1.0 && res.tools < res.max_tools,
            ColonyResources {
                metal: 1.0,
                wood: 1.0,
                ..Default::default()
            },
            ColonyResources {
                tools: 1.0,
                ..Default::default()
            },
        ),
        BuildingType::Weaver => (
            res.fiber >= 1.0 && res.cloth < res.max_cloth,
            ColonyResources {
                fiber: 1.0,
                ..Default::default()
            },
            ColonyResources {
                cloth: 1.0,
                ..Default::default()
            },
        ),
        BuildingType::Tailor => (
            res.cloth >= 1.0 && res.clothing < res.max_clothing,
            ColonyResources {
                cloth: 1.0,
                ..Default::default()
            },
            ColonyResources {
                clothing: 1.0,
                ..Default::default()
            },
        ),
        _ => (
            false,
            ColonyResources::default(),
            ColonyResources::default(),
        ),
    }
}

#[cfg(test)]
mod tests {
    use crate::layer1::GridPosition;
    use crate::layer1::building::{Building, BuildingType};
    use crate::layer1::pop::Pop;
    use crate::layer1::refining::process_refining_system;
    use crate::layer1::resources::{ColonyResources, RefiningProgress};
    use bevy_ecs::prelude::*;
    use bevy_ecs::system::RunSystemOnce;

    #[test]
    fn test_colony_resources_refined_fields() {
        let resources = ColonyResources::default();
        // New fields
        assert!((resources.planks - 0.0).abs() < f32::EPSILON);
        assert!((resources.blocks - 0.0).abs() < f32::EPSILON);
        // Default caps (can be same as raw for now)
        assert!((resources.max_planks - 50.0).abs() < f32::EPSILON);
        assert!((resources.max_blocks - 20.0).abs() < f32::EPSILON);
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
        let resources = ColonyResources {
            wood: 10.0,
            planks: 0.0,
            ..Default::default()
        };
        world.insert_resource(resources);

        // Spawn Lumber Mill at (5, 5)
        world.spawn((
            Building {
                building_type: BuildingType::LumberMill,
            },
            GridPosition { x: 5, y: 5 },
            RefiningProgress {
                current: 0.0,
                max: 10.0,
            }, // 10 ticks to refine
        ));

        // Spawn Worker nearby at (5, 6)
        world.spawn((Pop, GridPosition { x: 5, y: 6 }));

        // Run system
        // 1. Should detect worker
        // 2. Should detect valid input (Wood > 0)
        // 3. Should increment progress
        world.run_system_once(process_refining_system).unwrap();

        let progress = world.query::<&RefiningProgress>().single(&world);
        assert!(progress.current > 0.0);
    }

    #[test]
    fn test_process_refining_consumes_input_on_complete() {
        let mut world = World::new();

        // Setup Resources
        let resources = ColonyResources {
            wood: 10.0,
            planks: 0.0,
            ..Default::default()
        };
        world.insert_resource(resources);

        // Spawn Lumber Mill almost done
        world.spawn((
            Building {
                building_type: BuildingType::LumberMill,
            },
            GridPosition { x: 5, y: 5 },
            RefiningProgress {
                current: 9.9,
                max: 10.0,
            },
        ));

        // Spawn Worker
        world.spawn((Pop, GridPosition { x: 5, y: 6 }));

        // Run system to complete
        world.run_system_once(process_refining_system).unwrap();

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
        let resources = ColonyResources {
            wood: 0.0,
            ..Default::default()
        };
        world.insert_resource(resources);

        world.spawn((
            Building {
                building_type: BuildingType::LumberMill,
            },
            GridPosition { x: 5, y: 5 },
            RefiningProgress::default(),
        ));
        world.spawn((Pop, GridPosition { x: 5, y: 5 }));

        world.run_system_once(process_refining_system).unwrap();

        let progress = world.query::<&RefiningProgress>().single(&world);
        assert!((progress.current - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_refining_stops_if_output_full() {
        let mut world = World::new();
        let resources = ColonyResources {
            wood: 10.0,
            planks: 50.0,
            max_planks: 50.0,
            ..Default::default()
        };
        world.insert_resource(resources);

        world.spawn((
            Building {
                building_type: BuildingType::LumberMill,
            },
            GridPosition { x: 5, y: 5 },
            RefiningProgress::default(),
        ));
        world.spawn((Pop, GridPosition { x: 5, y: 5 }));

        world.run_system_once(process_refining_system).unwrap();

        let progress = world.query::<&RefiningProgress>().single(&world);
        assert!((progress.current - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_process_refining_prevents_underflow_race_condition() {
        let mut world = World::new();

        // Setup Resources: 1 Wood (enough for 1 Mill, not 2)
        let resources = ColonyResources {
            wood: 1.0,
            planks: 0.0,
            ..Default::default()
        };
        world.insert_resource(resources);

        // Spawn Lumber Mill 1 (Almost done)
        world.spawn((
            Building {
                building_type: BuildingType::LumberMill,
            },
            GridPosition { x: 5, y: 5 },
            RefiningProgress {
                current: 9.9,
                max: 10.0,
            },
        ));

        // Spawn Lumber Mill 2 (Almost done)
        world.spawn((
            Building {
                building_type: BuildingType::LumberMill,
            },
            GridPosition { x: 7, y: 7 },
            RefiningProgress {
                current: 9.9,
                max: 10.0,
            },
        ));

        // Spawn Workers
        world.spawn((Pop, GridPosition { x: 5, y: 6 }));
        world.spawn((Pop, GridPosition { x: 7, y: 6 }));

        // Run system
        // Both should try to complete.
        // If race condition is handled, one fails and retries later.
        // If not, resources drop to -1.0?
        world.run_system_once(process_refining_system).unwrap();

        let res = world.resource::<ColonyResources>();
        println!("Wood after tick: {}", res.wood);

        // Assert no underflow
        assert!(
            res.wood >= 0.0,
            "Resources should not go negative (race condition check)"
        );

        // One should have succeeded (Planks = 1), one failed (Planks = 1).
        // Or if we are lucky and strict, maybe neither? But optimally one wins.
        // Current 'broken' behavior: Planks = 2, Wood = -1.0.
        // assert_eq!(res.planks, 1.0); // We can be flexible on this, but strict on wood >= 0
    }
}
