use crate::layer1::construction::{ConstructionCost, ConstructionProgress};
use bevy_ecs::prelude::*;

#[derive(Component)]
pub struct GreatWork {
    pub name: String,
    pub current_phase: usize,
    pub phase_costs: Vec<ConstructionCost>,
}

impl GreatWork {
    #[must_use]
    pub fn is_completed(&self) -> bool {
        self.current_phase >= self.phase_costs.len()
    }
}

#[derive(Component)]
pub struct OperationalGreatWork;

pub fn spawn_great_work(
    world: &mut World,
    name: &str,
    phase_costs: Vec<ConstructionCost>,
) -> Entity {
    let initial_cost = phase_costs.first().cloned().unwrap_or(ConstructionCost {
        item_type: crate::layer1::items::ItemType::None,
        amount: 1,
    });
    world
        .spawn((
            GreatWork {
                name: name.to_string(),
                current_phase: 0,
                phase_costs,
            },
            ConstructionProgress {
                total_work_required: initial_cost.amount as f32 * 10.0,
                current_work: 0.0,
            },
        ))
        .id()
}

pub fn process_great_work_phases(
    mut commands: Commands,
    mut query: Query<(Entity, &mut GreatWork, &mut ConstructionProgress)>,
) {
    for (entity, mut work, mut progress) in query.iter_mut() {
        if work.is_completed() {
            continue;
        }

        if progress.current_work >= progress.total_work_required {
            work.current_phase += 1;

            if work.is_completed() {
                commands.entity(entity).insert(OperationalGreatWork);
                commands.entity(entity).remove::<ConstructionProgress>();
            } else {
                // Setup next phase
                let next_cost = &work.phase_costs[work.current_phase];
                progress.total_work_required = next_cost.amount as f32 * 10.0;
                progress.current_work = 0.0;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::construction::{ConstructionCost, ConstructionProgress};

    #[test]
    fn test_great_work_initializes_in_phase_one() {
        let mut world = World::new();
        let work_id = spawn_great_work(
            &mut world,
            "Space Elevator",
            vec![
                ConstructionCost {
                    item_type: crate::layer1::items::ItemType::None,
                    amount: 1000,
                },
                ConstructionCost {
                    item_type: crate::layer1::items::ItemType::None,
                    amount: 500,
                },
            ],
        );

        let work = world.get::<GreatWork>(work_id).unwrap();
        assert_eq!(work.current_phase, 0);
        assert!(!work.is_completed());
    }

    #[test]
    fn test_great_work_advances_phase_when_progress_filled() {
        let mut world = World::new();
        let work_id = spawn_great_work(
            &mut world,
            "Planetary Shield",
            vec![
                ConstructionCost {
                    item_type: crate::layer1::items::ItemType::None,
                    amount: 100,
                }, // Phase 0
                ConstructionCost {
                    item_type: crate::layer1::items::ItemType::None,
                    amount: 5,
                }, // Phase 1
            ],
        );

        // Manually complete phase 0
        let mut progress = world.get_mut::<ConstructionProgress>(work_id).unwrap();
        progress.current_work = progress.total_work_required;

        let mut schedule = Schedule::default();
        schedule.add_systems(process_great_work_phases);
        schedule.run(&mut world);

        let work = world.get::<GreatWork>(work_id).unwrap();
        assert_eq!(work.current_phase, 1, "Should have advanced to phase 1.");
        assert!(!work.is_completed(), "Should not be completed yet.");
    }

    #[test]
    fn test_great_work_completion_triggers_buff() {
        let mut world = World::new();
        let work_id = spawn_great_work(
            &mut world,
            "Monument",
            vec![ConstructionCost {
                item_type: crate::layer1::items::ItemType::None,
                amount: 10,
            }],
        );

        // Complete the only phase
        let mut progress = world.get_mut::<ConstructionProgress>(work_id).unwrap();
        progress.current_work = progress.total_work_required;

        let mut schedule = Schedule::default();
        schedule.add_systems(process_great_work_phases);
        schedule.run(&mut world);

        let work = world.get::<GreatWork>(work_id).unwrap();
        assert!(work.is_completed(), "Should be completed.");
        // Verify the component changed state or emitted an event
        assert!(world.get::<OperationalGreatWork>(work_id).is_some());
    }

    #[test]
    fn test_great_work_advances_multiple_phases_correctly() {
        let mut world = World::new();
        let work_id = spawn_great_work(
            &mut world,
            "Planetary Shield",
            vec![
                ConstructionCost {
                    item_type: crate::layer1::items::ItemType::None,
                    amount: 100,
                }, // Phase 0
                ConstructionCost {
                    item_type: crate::layer1::items::ItemType::None,
                    amount: 5,
                }, // Phase 1
            ],
        );

        let mut schedule = Schedule::default();
        schedule.add_systems(process_great_work_phases);

        // Manually complete phase 0
        {
            let mut progress = world.get_mut::<ConstructionProgress>(work_id).unwrap();
            progress.current_work = progress.total_work_required;
        }

        schedule.run(&mut world);

        {
            let work = world.get::<GreatWork>(work_id).unwrap();
            assert_eq!(work.current_phase, 1, "Should have advanced to phase 1.");
            assert!(!work.is_completed(), "Should not be completed yet.");
        }

        // Manually complete phase 1
        {
            let mut progress = world.get_mut::<ConstructionProgress>(work_id).unwrap();
            progress.current_work = progress.total_work_required;
        }

        schedule.run(&mut world);

        {
            let work = world.get::<GreatWork>(work_id).unwrap();
            assert_eq!(work.current_phase, 2, "Should have advanced to phase 2.");
            assert!(work.is_completed(), "Should be completed.");
            assert!(world.get::<OperationalGreatWork>(work_id).is_some());
        }
    }
}
