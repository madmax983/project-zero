#[cfg(test)]
mod integration_tests {
    use bevy::prelude::*;
    use scale::layer1::map::GridPosition;
    use scale::layer1::pop::Pop;
    use scale::layer1::morale::Morale;
    use scale::layer1::tech::rhythm::MachineRhythm;
    use scale::layer1::integration::industrial_rhythm_morale_bridge;
    use scale::shared::time::SimulationTime;

    #[test]
    fn test_industrial_rhythm_morale_boost() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        // Add the bridge system
        app.add_systems(Update, industrial_rhythm_morale_bridge);

        let mut world = app.world_mut();

        // Time simulation
        world.insert_resource(SimulationTime { tick: 100, speed: scale::shared::time::SimSpeed::Normal });

        // Spawn a Machine with high rhythm
        let _machine = world.spawn((
            GridPosition { x: 5, y: 5 },
            MachineRhythm {
                cycle_end_tick: 100,
                last_sync_bonus: 20.0, // High bonus
            },
        )).id();

        // Spawn a Pop nearby
        let pop = world.spawn((
            Pop::default(),
            GridPosition { x: 5, y: 5 }, // Right on it
            Morale { value: 0.5, modifiers: vec![] },
        )).id();

        app.update();

        // Assert the pop received the "Industrial Rhythm" modifier
        let morale = app.world().get::<Morale>(pop).unwrap();
        assert!(
            morale.modifiers.iter().any(|m| m.label == "Industrial Rhythm"),
            "Pop should have received the Industrial Rhythm modifier"
        );
        let modifier = morale.modifiers.iter().find(|m| m.label == "Industrial Rhythm").unwrap();
        assert!(modifier.value > 0.0);
    }
}
