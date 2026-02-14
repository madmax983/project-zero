// src/layer1/refining.rs
#![allow(clippy::too_many_lines, clippy::collapsible_if)]

//! Resource processing and refinement system.
//!
//! This module handles the conversion of raw materials (e.g., Wood, Ore) into
//! refined products (e.g., Planks, Metal) at industrial buildings.
//!
//! # The Refining Loop
//!
//! Refining is a continuous process that occurs tick-by-tick in [`process_refining_system`]:
//!
//! 1.  **Check Requirements**:
//!     *   Is the building active? (Powered, if applicable).
//!     *   Is there a worker AT the building doing [`ActionType::Refine`]?
//!     *   Are there input resources available? (e.g., Wood > 1).
//!     *   Is there storage space for output? (e.g., Planks < Max).
//! 2.  **Progress**:
//!     *   The worker's [`Skills`](crate::layer1::skills::Skills) (Crafting) determine efficiency.
//!     *   Progress accumulates in [`RefiningProgress`].
//! 3.  **Completion**:
//!     *   Inputs are consumed.
//!     *   Outputs are produced.
//!     *   Waste (Pollution) may be generated.
//!     *   Worker gains experience.

use crate::layer1::GridPosition;
use crate::layer1::building::{Building, BuildingType};
use crate::layer1::factions::{FactionMember, FactionState, Factions};
use crate::layer1::pop::Pop;
use crate::layer1::resources::{ColonyResources, RefiningProgress, ResourceItem, ResourceType};
use crate::layer1::skills::{SkillType, Skills, get_skill_efficiency};
use crate::layer1::utility_ai::{ActionType, PopAction};
use bevy_ecs::prelude::*;
use rand::Rng;

/// System that processes refining at buildings like Lumber Mills and Stone Masons.
///
/// # Algorithm
///
/// 1.  **Filter Workers**: Collects all Pops with [`ActionType::Refine`] at a [`GridPosition`].
/// 2.  **Iterate Buildings**: Finds all entities with [`RefiningProgress`] and [`Building`].
/// 3.  **Power Check**: Skips buildings with inactive [`PowerConsumer`](crate::layer1::energy::PowerConsumer) components.
/// 4.  **Match Worker**: Checks if any worker is at the building's position.
/// 5.  **Check Recipe**: Calls [`get_refining_recipe`] to verify resource availability.
/// 6.  **Apply Work**:
///     *   Calculates efficiency based on worker's `Crafting` skill.
///     *   Increments progress.
/// 7.  **Finalize**:
///     *   On completion, atomically deducts input and adds output using [`ColonyResources::try_deduct`].
///     *   Spawns `Waste` with 50% probability.
///     *   Resets progress.
#[doc(alias = "crafting")]
pub fn process_refining_system(world: &mut World) {
    let factions_data = world
        .get_resource::<Factions>()
        .map(|f| f.map.clone());

    // Collect workers who are refining
    let workers: Vec<(Entity, GridPosition)> = world
        .query_filtered::<(
            Entity,
            &GridPosition,
            &PopAction,
            Option<&FactionMember>,
        ), With<Pop>>()
        .iter(world)
        .filter(|(_, _, action, member)| {
            if action.current != ActionType::Refine {
                return false;
            }

            if let Some(map) = &factions_data {
                if let Some(m) = member {
                    if let Some(fid) = m.faction_id {
                        if map.get(&fid).is_some_and(|d| d.state == FactionState::Striking) {
                            return false;
                        }
                    }
                }
            }
            true
        })
        .map(|(e, p, _, _)| (e, *p))
        .collect();

    let resources_snapshot = world
        .get_resource::<ColonyResources>()
        .cloned()
        .unwrap_or_default();

    let mut finished_jobs: Vec<(Entity, ColonyResources, ColonyResources, GridPosition)> =
        Vec::new();
    let mut xp_gains: Vec<Entity> = Vec::new();

    // Iterate buildings
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
            let active = power.is_none_or(|c| c.active);
            (e, b.building_type, *p, prog.current, prog.max, active)
        })
        .collect();

    for (building_entity, building_type, pos, _current_prog, _max_prog, is_active) in buildings {
        if !is_active {
            continue;
        }

        // Find worker AT the building
        let worker_at_building = workers.iter().find(|(_, p)| *p == pos);

        let Some((worker_entity, _)) = worker_at_building else {
            continue;
        };

        // Get skill efficiency
        let efficiency = {
            let skills = world.get::<Skills>(*worker_entity);
            get_skill_efficiency(skills, SkillType::Crafting)
        };

        let (can_refine, input_cost, output_gain) =
            get_refining_recipe(building_type, &resources_snapshot);

        if can_refine {
            if let Some(mut progress) = world.get_mut::<RefiningProgress>(building_entity) {
                progress.current += 1.0 * efficiency;
                if progress.is_complete() {
                    finished_jobs.push((building_entity, input_cost, output_gain, pos));
                }
            }

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
                    resources.add_rations(output.rations);
                    resources.add_planks(output.planks);
                    resources.add_blocks(output.blocks);
                    resources.add_metal(output.metal);
                    resources.add_tools(output.tools);
                    resources.add_cloth(output.cloth);
                    resources.add_clothing(output.clothing);
                    resources.add_fuel(output.fuel);
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
#[must_use]
pub fn get_refining_recipe(
    building_type: BuildingType,
    res: &ColonyResources,
) -> (bool, ColonyResources, ColonyResources) {
    match building_type {
        BuildingType::Smokehouse => (
            res.food >= 5.0 && res.wood >= 1.0 && res.rations < res.max_rations,
            ColonyResources {
                food: 5.0,
                wood: 1.0,
                ..ColonyResources::zeroed()
            },
            ColonyResources {
                rations: 5.0,
                ..ColonyResources::zeroed()
            },
        ),
        BuildingType::LumberMill => (
            res.wood >= 1.0 && res.planks < res.max_planks,
            ColonyResources {
                wood: 1.0,
                ..ColonyResources::zeroed()
            },
            ColonyResources {
                planks: 1.0,
                ..ColonyResources::zeroed()
            },
        ),
        BuildingType::StoneMason => (
            res.stone >= 1.0 && res.blocks < res.max_blocks,
            ColonyResources {
                stone: 1.0,
                ..ColonyResources::zeroed()
            },
            ColonyResources {
                blocks: 1.0,
                ..ColonyResources::zeroed()
            },
        ),
        BuildingType::Smelter => (
            res.ore >= 1.0 && res.wood >= 1.0 && res.metal < res.max_metal,
            ColonyResources {
                ore: 1.0,
                wood: 1.0,
                ..ColonyResources::zeroed()
            },
            ColonyResources {
                metal: 1.0,
                ..ColonyResources::zeroed()
            },
        ),
        BuildingType::Smithy => (
            res.metal >= 1.0 && res.wood >= 1.0 && res.tools < res.max_tools,
            ColonyResources {
                metal: 1.0,
                wood: 1.0,
                ..ColonyResources::zeroed()
            },
            ColonyResources {
                tools: 1.0,
                ..ColonyResources::zeroed()
            },
        ),
        BuildingType::Weaver => (
            res.fiber >= 1.0 && res.cloth < res.max_cloth,
            ColonyResources {
                fiber: 1.0,
                ..ColonyResources::zeroed()
            },
            ColonyResources {
                cloth: 1.0,
                ..ColonyResources::zeroed()
            },
        ),
        BuildingType::Tailor => (
            res.cloth >= 1.0 && res.clothing < res.max_clothing,
            ColonyResources {
                cloth: 1.0,
                ..ColonyResources::zeroed()
            },
            ColonyResources {
                clothing: 1.0,
                ..ColonyResources::zeroed()
            },
        ),
        BuildingType::Refinery => (
            res.ore >= 2.0 && res.fuel < res.max_fuel,
            ColonyResources {
                ore: 2.0,
                ..ColonyResources::zeroed()
            },
            ColonyResources {
                fuel: 1.0,
                ..ColonyResources::zeroed()
            },
        ),
        _ => (false, ColonyResources::zeroed(), ColonyResources::zeroed()),
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
    use crate::layer1::utility_ai::{ActionType, PopAction};
    use bevy_ecs::prelude::*;

    #[test]
    fn test_colony_resources_refined_fields() {
        let resources = ColonyResources::default();
        assert!((resources.planks - 0.0).abs() < f32::EPSILON);
        assert!((resources.blocks - 0.0).abs() < f32::EPSILON);
        assert!((resources.max_planks - 50.0).abs() < f32::EPSILON);
        assert!((resources.max_blocks - 20.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_process_refining_lumber_mill() {
        let mut world = World::new();

        let resources = ColonyResources {
            wood: 10.0,
            planks: 0.0,
            ..Default::default()
        };
        world.insert_resource(resources);

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

        // Spawn Worker AT (5, 5) with correct action
        world.spawn((
            Pop,
            GridPosition { x: 5, y: 5 },
            PopAction {
                current: ActionType::Refine,
                current_utility: 0.5,
                ticks_committed: 1,
            },
        ));

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

        let mut skills = Skills::default();
        skills.add_xp(SkillType::Crafting, 100.0);
        world.spawn((
            Pop,
            GridPosition { x: 5, y: 5 },
            skills,
            PopAction {
                current: ActionType::Refine,
                ..Default::default()
            },
        ));

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
            .spawn((
                Pop,
                GridPosition { x: 5, y: 5 },
                Skills::default(),
                PopAction {
                    current: ActionType::Refine,
                    ..Default::default()
                },
            ))
            .id();

        process_refining_system(&mut world);

        let skills = world.get::<Skills>(worker).unwrap();
        assert_eq!(skills.get_xp(SkillType::Crafting), 1.0);
    }

    #[test]
    fn test_process_refining_consumes_input_on_complete() {
        let mut world = World::new();

        let resources = ColonyResources {
            wood: 10.0,
            planks: 0.0,
            ..Default::default()
        };
        world.insert_resource(resources);

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

        world.spawn((
            Pop,
            GridPosition { x: 5, y: 5 },
            PopAction {
                current: ActionType::Refine,
                ..Default::default()
            },
        ));

        process_refining_system(&mut world);

        let res = world.resource::<ColonyResources>();
        assert!((res.wood - 9.0).abs() < f32::EPSILON);
        assert!((res.planks - 1.0).abs() < f32::EPSILON);

        let progress = world.query::<&RefiningProgress>().single(&world);
        assert!(progress.current < 1.0);
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
        world.spawn((
            Pop,
            GridPosition { x: 5, y: 5 },
            PopAction {
                current: ActionType::Refine,
                ..Default::default()
            },
        ));

        process_refining_system(&mut world);

        let progress = world.query::<&RefiningProgress>().single(&world);
        assert!((progress.current - 0.0).abs() < f32::EPSILON);
    }
}
// DEBUG
