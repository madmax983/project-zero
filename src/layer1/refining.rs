// src/layer1/refining.rs
#![allow(clippy::too_many_lines, clippy::collapsible_if)]

use crate::layer1::GridPosition;
use crate::layer1::building::{Building, BuildingType};
use crate::layer1::pop::Pop;
use crate::layer1::resources::{ColonyResources, RefiningProgress, ResourceItem, ResourceType};
use crate::layer1::skills::{SkillType, Skills, get_skill_efficiency};
use bevy_ecs::prelude::*;
use rand::Rng;

/// System that processes refining at buildings like Lumber Mills and Stone Masons.
///
/// It iterates over buildings with `RefiningProgress`. If a worker (`Pop`) is nearby
/// and input resources are available, it increments progress. Upon completion,
/// it consumes input resources and produces refined resources.
pub fn process_refining_system(world: &mut World) {
    let workers: Vec<(Entity, GridPosition)> = world
        .query_filtered::<(Entity, &GridPosition), With<Pop>>()
        .iter(world)
        .map(|(e, p)| (e, *p))
        .collect();

    let resources_snapshot = world
        .get_resource::<ColonyResources>()
        .cloned()
        .unwrap_or_default();

    let mut finished_jobs: Vec<(Entity, ColonyResources, ColonyResources, GridPosition)> =
        Vec::new();
    let mut xp_gains: Vec<Entity> = Vec::new();

    // Iterate buildings
    // We collect entities to avoid borrow conflict when accessing skills later
    // INT-006: Added PowerConsumer check. If building has PowerConsumer, it must be active.
    let buildings: Vec<(Entity, BuildingType, GridPosition, f32, f32, bool)> = world
        .query::<(
            Entity,
            &Building,
            &GridPosition,
            &RefiningProgress,
            Option<&crate::layer1::energy::PowerConsumer>,
        )>()
        .iter(world)
        .map(|(e, b, p, prog, power)| {
            // If no power consumer, assume active (e.g. LumberMill). If present, check active.
            let active = power.is_none_or(|c| c.active);
            (e, b.building_type, *p, prog.current, prog.max, active)
        })
        .collect();

    for (building_entity, building_type, pos, _current_prog, _max_prog, is_active) in buildings {
        if !is_active {
            continue;
        }

        // Find nearest worker
        let nearest_worker = workers
            .iter()
            .min_by_key(|(_, p)| (p.x - pos.x).abs() + (p.y - pos.y).abs());

        let Some((worker_entity, worker_pos)) = nearest_worker else {
            continue;
        };

        // Check range (10 tiles)
        if (worker_pos.x - pos.x).abs() + (worker_pos.y - pos.y).abs() > 10 {
            continue;
        }

        // Get skill efficiency
        let efficiency = {
            let skills = world.get::<Skills>(*worker_entity);
            get_skill_efficiency(skills, SkillType::Crafting)
        };

        let (can_refine, input_cost, output_gain) =
            get_refining_recipe(building_type, &resources_snapshot);

        if can_refine {
            // Update progress
            // We need to write back to RefiningProgress
            // Since we collected immutable data, we need to get_mut now.
            if let Some(mut progress) = world.get_mut::<RefiningProgress>(building_entity) {
                progress.current += 1.0 * efficiency;
                if progress.is_complete() {
                    finished_jobs.push((building_entity, input_cost, output_gain, pos));
                }
            }

            // Add XP
            xp_gains.push(*worker_entity);
        }
    }

    // Apply XP gains
    for worker_entity in xp_gains {
        if let Some(mut skills) = world.get_mut::<Skills>(worker_entity) {
            skills.add_xp(SkillType::Crafting, 1.0);
        }
    }

    // Apply resource updates for finished jobs
    for (entity, input, output, pos) in finished_jobs {
        let success = world
            .get_resource_mut::<ColonyResources>()
            .is_some_and(|mut resources| {
                if resources.try_deduct(&input) {
                    resources.add_planks(output.planks);
                    resources.add_blocks(output.blocks);
                    resources.add_metal(output.metal);
                    resources.add_tools(output.tools);
                    resources.add_cloth(output.cloth);
                    resources.add_clothing(output.clothing);
                    true
                } else {
                    false
                }
            });

        if success {
            // Spawn Waste (50% chance)
            let mut rng = rand::thread_rng();
            if rng.gen_bool(0.5) {
                world.spawn((
                    ResourceItem {
                        resource_type: ResourceType::Waste,
                        amount: 1.0,
                    },
                    pos,
                ));
            }

            if let Some(mut progress) = world.get_mut::<RefiningProgress>(entity) {
                progress.current = 0.0;
            }
        } else if let Some(mut progress) = world.get_mut::<RefiningProgress>(entity) {
            progress.current = progress.max;
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
    use crate::layer1::skills::{SkillType, Skills};
    use bevy_ecs::prelude::*;

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
        process_refining_system(&mut world);

        let progress = world.query::<&RefiningProgress>().single(&world);
        assert!(progress.current > 0.0);
    }

    #[test]
    fn test_process_refining_skills_efficiency() {
        let mut world = World::new();
        world.insert_resource(ColonyResources {
            wood: 10.0,
            ..Default::default()
        });

        world.spawn((
            Building {
                building_type: BuildingType::LumberMill,
            },
            GridPosition { x: 5, y: 5 },
            RefiningProgress {
                current: 0.0,
                max: 10.0,
            },
        ));

        // Worker with Level 1 Crafting -> 1.1 efficiency
        let mut skills = Skills::default();
        skills.add_xp(SkillType::Crafting, 100.0);
        world.spawn((Pop, GridPosition { x: 5, y: 6 }, skills));

        process_refining_system(&mut world);

        let progress = world.query::<&RefiningProgress>().single(&world);
        assert!((progress.current - 1.1).abs() < f32::EPSILON);
    }

    #[test]
    fn test_process_refining_skills_xp() {
        let mut world = World::new();
        world.insert_resource(ColonyResources {
            wood: 10.0,
            ..Default::default()
        });

        world.spawn((
            Building {
                building_type: BuildingType::LumberMill,
            },
            GridPosition { x: 5, y: 5 },
            RefiningProgress {
                current: 0.0,
                max: 10.0,
            },
        ));

        let worker = world
            .spawn((Pop, GridPosition { x: 5, y: 6 }, Skills::default()))
            .id();

        process_refining_system(&mut world);

        let skills = world.get::<Skills>(worker).unwrap();
        assert_eq!(skills.get_xp(SkillType::Crafting), 1.0);
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

        process_refining_system(&mut world);

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

        process_refining_system(&mut world);

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
        process_refining_system(&mut world);

        let res = world.resource::<ColonyResources>();
        println!("Wood after tick: {}", res.wood);

        // Assert no underflow
        assert!(
            res.wood >= 0.0,
            "Resources should not go negative (race condition check)"
        );
    }
}
