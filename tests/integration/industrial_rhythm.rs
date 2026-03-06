#[cfg(test)]
mod integration_tests {
    use bevy_ecs::prelude::*;
    use scale::layer1::building::{Building, BuildingType};
    use scale::layer1::integration::industrial_rhythm_morale_bridge;
    use scale::layer1::map::GridPosition;
    use scale::layer1::morale::Morale;
    use scale::layer1::pop::Pop;
    use scale::layer1::tech::rhythm::{update_rhythm_system, MachineRhythm, RhythmManager};
    use scale::shared::time::SimulationTime;

    #[test]
    fn industrial_rhythm_boosts_pop_morale() {
        let mut world = World::new();
        world.insert_resource(SimulationTime {
            tick: 100,
            speed: scale::shared::time::SimSpeed::Normal,
        });
        world.insert_resource(RhythmManager::default());

        // Setup schedule
        let mut schedule = Schedule::default();
        schedule.add_systems((
            update_rhythm_system,
            industrial_rhythm_morale_bridge.after(update_rhythm_system),
        ));

        // Spawn Machine A (finishes at tick 100)
        world.spawn((
            Building {
                building_type: BuildingType::Refinery,
            },
            GridPosition { x: 5, y: 5 },
            MachineRhythm {
                cycle_end_tick: 100,
                last_sync_bonus: 0.0,
            },
        ));

        // Spawn Machine B (finishes at tick 100) - Perfect Sync
        world.spawn((
            Building {
                building_type: BuildingType::Refinery,
            },
            GridPosition { x: 6, y: 5 },
            MachineRhythm {
                cycle_end_tick: 100,
                last_sync_bonus: 0.0,
            },
        ));

        // Spawn a Pop nearby (dist <= 3)
        let pop_near = world
            .spawn((
                Pop::default(),
                GridPosition { x: 5, y: 6 },
                Morale::default(),
            ))
            .id();

        // Spawn a Pop far away
        let pop_far = world
            .spawn((
                Pop::default(),
                GridPosition { x: 20, y: 20 },
                Morale::default(),
            ))
            .id();

        // Run systems
        schedule.run(&mut world);

        // Assert nearby pop got the modifier
        let morale_near = world.get::<Morale>(pop_near).unwrap();
        let has_bonus = morale_near
            .modifiers
            .iter()
            .any(|m| m.label == "Industrial Rhythm" && m.value > 0.0);
        assert!(
            has_bonus,
            "Nearby pop should have gained the Industrial Rhythm modifier."
        );

        // Assert far pop did not get the modifier
        let morale_far = world.get::<Morale>(pop_far).unwrap();
        let far_has_bonus = morale_far
            .modifiers
            .iter()
            .any(|m| m.label == "Industrial Rhythm");
        assert!(
            !far_has_bonus,
            "Far pop should not have the Industrial Rhythm modifier."
        );
    }
}
