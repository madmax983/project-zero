#[cfg(test)]
mod tests {
    use crate::layer1::building::{Building, BuildingType};
    use crate::layer1::map::GridPosition;
    use crate::layer1::tech::rhythm::{update_rhythm_system, MachineRhythm};
    use bevy_ecs::prelude::*;

    #[test]
    fn test_sync_bonus_adjacent_machines() {
        let mut world = World::new();

        // Spawn Machine A (finishes at tick 100)
        let entity_a = world
            .spawn((
                Building {
                    building_type: BuildingType::Refinery,
                },
                GridPosition { x: 5, y: 5 },
                MachineRhythm {
                    cycle_end_tick: 100,
                    last_sync_bonus: 0.0,
                },
            ))
            .id();

        // Spawn Machine B (finishes at tick 100) - Perfect Sync
        let entity_b = world
            .spawn((
                Building {
                    building_type: BuildingType::Refinery,
                },
                GridPosition { x: 6, y: 5 },
                MachineRhythm {
                    cycle_end_tick: 100,
                    last_sync_bonus: 0.0,
                },
            ))
            .id();

        // Run system at tick 100
        let mut schedule = Schedule::default();
        schedule.add_systems(update_rhythm_system);

        // Mock time
        world.insert_resource(crate::shared::time::SimulationTime {
            tick: 100,
            ..Default::default()
        });

        schedule.run(&mut world);

        // Check bonus
        // Both machines should detect the sync event
        let rhythm_a = world.get::<MachineRhythm>(entity_a).unwrap();
        assert!(rhythm_a.last_sync_bonus > 0.0);
        let rhythm_b = world.get::<MachineRhythm>(entity_b).unwrap();
        assert!(rhythm_b.last_sync_bonus > 0.0);
    }

    #[test]
    fn test_discord_penalty() {
        let mut world = World::new();
        world.insert_resource(crate::shared::time::SimulationTime {
            tick: 100,
            ..Default::default()
        });

        // Machine A finishes now
        let entity_a = world
            .spawn((
                GridPosition { x: 0, y: 0 },
                MachineRhythm {
                    cycle_end_tick: 100,
                    last_sync_bonus: 0.0,
                },
            ))
            .id();

        // Machine B finished long ago (tick 50) - No sync
        world.spawn((
            GridPosition { x: 1, y: 0 },
            MachineRhythm {
                cycle_end_tick: 50,
                last_sync_bonus: 0.0,
            },
        ));

        let mut schedule = Schedule::default();
        schedule.add_systems(update_rhythm_system);
        schedule.run(&mut world);

        // No bonus (or penalty if implemented)
        let rhythm = world.get::<MachineRhythm>(entity_a).unwrap();
        assert_eq!(rhythm.last_sync_bonus, 0.0);
    }

    #[test]
    fn test_sync_bonus_lost_on_discord() {
        let mut world = World::new();

        world.insert_resource(crate::shared::time::SimulationTime {
            tick: 100,
            ..Default::default()
        });

        // Spawn Machine A (finishes at tick 100) with an existing bonus
        let entity_a = world
            .spawn((
                Building {
                    building_type: BuildingType::Refinery,
                },
                GridPosition { x: 5, y: 5 },
                MachineRhythm {
                    cycle_end_tick: 100,
                    last_sync_bonus: 10.0,
                },
            ))
            .id();

        // Spawn Machine B (finished long ago)
        world.spawn((
            Building {
                building_type: BuildingType::Refinery,
            },
            GridPosition { x: 6, y: 5 },
            MachineRhythm {
                cycle_end_tick: 50,
                last_sync_bonus: 0.0,
            },
        ));

        let mut schedule = Schedule::default();
        schedule.add_systems(update_rhythm_system);
        schedule.run(&mut world);

        // Machine A should lose its bonus because it didn't sync this cycle
        let rhythm_a = world.get::<MachineRhythm>(entity_a).unwrap();
        assert_eq!(rhythm_a.last_sync_bonus, 0.0);
    }

    #[test]
    fn test_rhythm_sync_distances() {
        let test_cases = vec![
            (GridPosition { x: 0, y: 0 }, GridPosition { x: 1, y: 1 }, true, "Adjacent (1) should sync"),
            (GridPosition { x: 0, y: 0 }, GridPosition { x: 2, y: 0 }, false, "Distance 2 should not sync"),
            (GridPosition { x: 0, y: 0 }, GridPosition { x: 0, y: 0 }, false, "Same position (self) handled gracefully/ignored based on other_pos == pos"),
        ];

        for (pos_a, pos_b, expected_sync, msg) in test_cases {
            let mut world = World::new();
            world.insert_resource(crate::shared::time::SimulationTime {
                tick: 100,
                ..Default::default()
            });

            let entity_a = world.spawn((
                pos_a,
                MachineRhythm {
                    cycle_end_tick: 100,
                    last_sync_bonus: 0.0,
                },
            )).id();

            world.spawn((
                pos_b,
                MachineRhythm {
                    cycle_end_tick: 100,
                    last_sync_bonus: 0.0,
                },
            ));

            let mut schedule = Schedule::default();
            schedule.add_systems(update_rhythm_system);
            schedule.run(&mut world);

            let rhythm_a = world.get::<MachineRhythm>(entity_a).unwrap();
            if expected_sync {
                assert!(rhythm_a.last_sync_bonus > 0.0, "{}", msg);
            } else {
                assert_eq!(rhythm_a.last_sync_bonus, 0.0, "{}", msg);
            }
        }
    }

    #[test]
    fn test_rhythm_sync_time_windows() {
        let test_cases = vec![
            (100, 100, true, "Exact match should sync"),
            (100, 102, true, "Diff 2 (ahead) should sync"),
            (100, 98, true, "Diff 2 (behind) should sync"),
            (100, 103, false, "Diff 3 (ahead) should not sync"),
            (100, 97, false, "Diff 3 (behind) should not sync"),
        ];

        for (tick_a, tick_b, expected_sync, msg) in test_cases {
            let mut world = World::new();
            world.insert_resource(crate::shared::time::SimulationTime {
                tick: tick_a,
                ..Default::default()
            });

            let entity_a = world.spawn((
                GridPosition { x: 0, y: 0 },
                MachineRhythm {
                    cycle_end_tick: tick_a,
                    last_sync_bonus: 0.0,
                },
            )).id();

            world.spawn((
                GridPosition { x: 1, y: 0 },
                MachineRhythm {
                    cycle_end_tick: tick_b,
                    last_sync_bonus: 0.0,
                },
            ));

            let mut schedule = Schedule::default();
            schedule.add_systems(update_rhythm_system);
            schedule.run(&mut world);

            let rhythm_a = world.get::<MachineRhythm>(entity_a).unwrap();
            if expected_sync {
                assert!(rhythm_a.last_sync_bonus > 0.0, "{}", msg);
            } else {
                assert_eq!(rhythm_a.last_sync_bonus, 0.0, "{}", msg);
            }
        }
    }
}
