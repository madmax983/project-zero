//! Hygiene system.
//!
//! Handles filth accumulation, hygiene decay, and showering.

use crate::layer1::items::{Item, ItemType};
use crate::layer1::map::GridPosition;
use crate::layer1::needs::Needs;
use crate::layer1::pop::Job;
use crate::layer1::resources::ColonyResources;
use crate::layer1::utility_types::{ActionType, AssignmentType, PopAction};
use bevy_ecs::prelude::*;

/// Tracks physical filth on a Pop.
///
/// *   **0.0**: Clean.
/// *   **100.0**: Filthy.
#[derive(Component, Debug, Clone)]
pub struct Filth {
    /// Current filth level (0.0 to max).
    pub current: f32,
    /// Maximum filth level (usually 100.0).
    pub max: f32,
}

impl Default for Filth {
    fn default() -> Self {
        Self {
            current: 0.0,
            max: 100.0,
        }
    }
}

/// Base rate of filth accumulation per tick.
pub const BASE_FILTH_RATE: f32 = 0.01;
/// Multiplier for dirty jobs (Mining, Farming).
pub const DIRTY_JOB_MULTIPLIER: f32 = 5.0;
/// Multiplier for clean jobs (Research, Admin).
pub const CLEAN_JOB_MULTIPLIER: f32 = 0.5;
/// Water cost per shower tick.
pub const SHOWER_WATER_COST: f32 = 0.1;
/// Hygiene restored per tick in shower.
pub const HYGIENE_RESTORE_RATE: f32 = 0.5;
/// Filth cleaned per tick in shower.
pub const FILTH_CLEAN_RATE: f32 = 1.0;

/// Accumulates filth based on current action and job.
pub fn filth_accumulation_system(mut query: Query<(&mut Filth, &PopAction, Option<&Job>)>) {
    query.par_iter_mut().for_each(|(mut filth, action, job)| {
        let mut rate = BASE_FILTH_RATE;

        // If working, adjust based on job type
        if action.current == ActionType::Work {
            if let Some(j) = job {
                match j.job_type {
                    AssignmentType::FarmWorker | AssignmentType::Funeral => {
                        rate *= DIRTY_JOB_MULTIPLIER;
                    }
                    AssignmentType::LibraryWorker
                    | AssignmentType::Administrator
                    | AssignmentType::ObservatoryWorker => {
                        rate *= CLEAN_JOB_MULTIPLIER;
                    }
                    _ => {
                        // Standard rate for others
                    }
                }
            } else {
                // Working but no job assigned (e.g. general labor like hauling/construction)
                // Hauling is usually dirty? Or maybe average.
                // Let's assume average (1.0 multiplier)
            }
        } else if action.current == ActionType::Idle {
            rate *= 0.2; // Idle accumulates very slowly
        }

        filth.current = (filth.current + rate).min(filth.max);
    });
}

/// Decays hygiene based on filth level.
pub fn hygiene_decay_system(mut query: Query<(&mut Needs, &Filth)>) {
    query.par_iter_mut().for_each(|(mut needs, filth)| {
        // Filth accelerates hygiene loss.
        // Base decay is 0.0 (handled here instead of needs.rs? No needs.rs has generic decay?)
        // Spec says: "Filth accelerates hygiene loss"
        // Let's check needs.rs again. I didn't add hygiene decay to needs.rs decay_needs_system.
        // I added "Hygiene is decayed separately in hygiene.rs" comment.
        // So I must handle ALL hygiene decay here.

        // Base decay: 0.001 (similar to hunger)
        // Filth penalty: up to +0.005
        let base_decay = 0.001;
        let filth_penalty = (filth.current / filth.max) * 0.005;

        needs.hygiene = (needs.hygiene - (base_decay + filth_penalty)).max(0.0);
    });
}

/// Handles pops using the shower.
pub fn shower_use_system(
    mut commands: Commands,
    mut resources: ResMut<ColonyResources>,
    mut query: Query<(Entity, &mut Needs, &mut Filth, &PopAction, &GridPosition)>,
) {
    // Check global water availability first (optimization)
    if resources.water < SHOWER_WATER_COST {
        return;
    }

    for (_entity, mut needs, mut filth, action, pos) in &mut query {
        if action.current == ActionType::UseShower {
            // Double check water per pop (in case we ran out mid-loop, though unlikely with f32)
            if resources.water >= SHOWER_WATER_COST {
                resources.water -= SHOWER_WATER_COST;

                needs.hygiene = (needs.hygiene + HYGIENE_RESTORE_RATE).min(1.0);
                filth.current = (filth.current - FILTH_CLEAN_RATE).max(0.0);

                // Produce Waste (Integration with Recycling)
                // Spawn a waste item every 10 filth cleaned
                if filth.current > 0.0 && filth.current % 10.0 < FILTH_CLEAN_RATE {
                    commands.spawn((
                        Item {
                            item_type: ItemType::Waste,
                        },
                        *pos,
                    ));
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::building::{Building, BuildingType};
    use crate::layer1::needs::Needs;
    use crate::layer1::pop::Job;
    use crate::layer1::pop::Pop;
    use crate::layer1::resources::ColonyResources;
    use crate::layer1::utility_types::{ActionType, AssignmentType, PopAction};

    fn setup_world() -> World {
        let mut world = World::new();
        crate::setup::init_task_pools();
        world.insert_resource(ColonyResources {
            water: 100.0,
            ..Default::default()
        });
        world
    }

    #[test]
    fn test_filth_accumulation_idle() {
        let mut world = setup_world();
        let pop = world
            .spawn((
                Pop,
                PopAction {
                    current: ActionType::Idle,
                    ..Default::default()
                },
                Filth {
                    current: 0.0,
                    max: 100.0,
                },
            ))
            .id();

        // Run system
        let mut schedule = Schedule::default();
        schedule.add_systems(filth_accumulation_system);
        schedule.run(&mut world);

        let filth = world.get::<Filth>(pop).unwrap();
        assert!(
            filth.current > 0.0,
            "Filth should accumulate slowly even when idle"
        );
    }

    #[test]
    fn test_filth_accumulation_dirty_job() {
        let mut world = setup_world();
        let mine = world.spawn_empty().id();
        let lab = world.spawn_empty().id();

        let miner = world
            .spawn((
                Pop,
                PopAction {
                    current: ActionType::Work,
                    ..Default::default()
                },
                Job {
                    workplace: mine,
                    job_type: AssignmentType::FarmWorker,
                }, // Dirty
                Filth {
                    current: 0.0,
                    max: 100.0,
                },
            ))
            .id();

        let researcher = world
            .spawn((
                Pop,
                PopAction {
                    current: ActionType::Work,
                    ..Default::default()
                },
                Job {
                    workplace: lab,
                    job_type: AssignmentType::LibraryWorker,
                }, // Clean
                Filth {
                    current: 0.0,
                    max: 100.0,
                },
            ))
            .id();

        let mut schedule = Schedule::default();
        schedule.add_systems(filth_accumulation_system);
        schedule.run(&mut world);

        let miner_filth = world.get::<Filth>(miner).unwrap().current;
        let researcher_filth = world.get::<Filth>(researcher).unwrap().current;

        assert!(
            miner_filth > researcher_filth,
            "FarmWorkers should get dirtier than Researchers"
        );
    }

    #[test]
    fn test_hygiene_decay_from_filth() {
        let mut world = setup_world();
        let clean_pop = world
            .spawn((
                Needs {
                    hygiene: 1.0,
                    ..Default::default()
                },
                Filth {
                    current: 0.0,
                    ..Default::default()
                },
            ))
            .id();

        let dirty_pop = world
            .spawn((
                Needs {
                    hygiene: 1.0,
                    ..Default::default()
                },
                Filth {
                    current: 100.0,
                    ..Default::default()
                },
            ))
            .id();

        let mut schedule = Schedule::default();
        schedule.add_systems(hygiene_decay_system);
        schedule.run(&mut world);

        let clean_hygiene = world.get::<Needs>(clean_pop).unwrap().hygiene;
        let dirty_hygiene = world.get::<Needs>(dirty_pop).unwrap().hygiene;

        assert!(
            dirty_hygiene < clean_hygiene,
            "High filth should accelerate hygiene decay"
        );
    }

    #[test]
    fn test_shower_usage() {
        let mut world = setup_world();

        let pop = world
            .spawn((
                Needs {
                    hygiene: 0.1,
                    oxygen: 100.0,
                    ..Default::default()
                },
                Filth {
                    current: 80.0,
                    ..Default::default()
                },
                PopAction {
                    current: ActionType::UseShower,
                    ..Default::default()
                },
                GridPosition { x: 0, y: 0 },
            ))
            .id();

        let _shower = world
            .spawn((Building {
                building_type: BuildingType::Shower,
            },))
            .id();

        let mut schedule = Schedule::default();
        schedule.add_systems(shower_use_system);
        schedule.run(&mut world);

        // Check Water Consumption
        let resources = world.resource::<ColonyResources>();
        assert!(resources.water < 100.0, "Shower should consume water");

        // Check Hygiene/Filth
        let needs = world.get::<Needs>(pop).unwrap();
        let filth = world.get::<Filth>(pop).unwrap();

        assert!(needs.hygiene > 0.1, "Hygiene should increase");
        assert!(filth.current < 80.0, "Filth should decrease");
    }
}
