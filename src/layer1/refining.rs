use bevy_ecs::prelude::*;
use crate::layer1::resources::ColonyResources;
use crate::layer1::building::{Building, BuildingType};
use crate::layer1::GridPosition;
use crate::layer1::pop::Pop;

/// Tracks progress of a refining job (e.g. converting Wood to Planks).
#[derive(Component, Default, Debug)]
pub struct RefiningProgress {
    /// Current work done.
    pub current: f32,
    /// Total work required.
    pub max: f32,
}

impl RefiningProgress {
    /// Returns true if the work is complete.
    #[must_use]
    pub fn is_complete(&self) -> bool {
        self.current >= self.max
    }
}

struct RefiningJob {
    entity: Entity,
    input: ColonyResources,
    output: ColonyResources,
    work: f32,
}

/// System that processes refining logic for buildings.
///
/// It scans for buildings with `RefiningProgress`, checks for nearby workers (within 10 tiles),
/// checks input resource availability, updates progress, and transforms resources upon completion.
pub fn process_refining_system(world: &mut World) {
    // 1. Collect worker positions
    let worker_positions: Vec<GridPosition> = world
        .query::<(&Pop, &GridPosition)>()
        .iter(world)
        .map(|(_, pos)| *pos)
        .collect();

    // 2. Iterate buildings and collect potential updates
    // We can't mutate resources while iterating the query if we access them in the loop.
    // So we calculate needs first.

    let mut jobs = Vec::new();

    // Capture resource snapshot to avoid borrow conflicts
    let (res_wood, res_stone, res_planks, res_blocks, res_max_planks, res_max_blocks) = {
        let r = world.resource::<ColonyResources>();
        (r.wood, r.stone, r.planks, r.blocks, r.max_planks, r.max_blocks)
    };

    // Query for buildings that can refine
    let mut query = world.query::<(Entity, &Building, &GridPosition, &RefiningProgress)>();

    for (entity, building, pos, progress) in query.iter(world) {
        // Must have worker nearby
        let has_worker = worker_positions
            .iter()
            .any(|p| (p.x - pos.x).abs() + (p.y - pos.y).abs() <= 10);

        if !has_worker {
            continue;
        }

        // Check input/output constraints using snapshot
        let (can_refine, input, output) = match building.building_type {
            BuildingType::LumberMill => (
                res_wood >= 1.0 && res_planks < res_max_planks,
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
                res_stone >= 1.0 && res_blocks < res_max_blocks,
                ColonyResources {
                    stone: 1.0,
                    ..Default::default()
                },
                ColonyResources {
                    blocks: 1.0,
                    ..Default::default()
                },
            ),
            _ => (
                false,
                ColonyResources::default(),
                ColonyResources::default(),
            ),
        };

        if can_refine && !progress.is_complete() {
            jobs.push(RefiningJob {
                entity,
                input,
                output,
                work: 1.0, // Fixed work rate for now
            });
        }
    }

    // 3. Apply updates
    for job in jobs {
        // Double check affordability just in case multiple jobs drained the same resource
        let can_afford = world.resource::<ColonyResources>().can_afford(&job.input);
        if !can_afford {
            continue;
        }

        // Update progress
        let completed = if let Some(mut progress) = world.get_mut::<RefiningProgress>(job.entity) {
            progress.current += job.work;
            if progress.is_complete() {
                progress.current = 0.0;
                true
            } else {
                false
            }
        } else {
            false
        };

        if completed {
             let mut resources = world.resource_mut::<ColonyResources>();
             // Deduct input
             resources.deduct(&job.input);
             // Add output
             resources.add_planks(job.output.planks);
             resources.add_blocks(job.output.blocks);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::resources::ColonyResources;
    use crate::layer1::building::{Building, BuildingType};
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
}
