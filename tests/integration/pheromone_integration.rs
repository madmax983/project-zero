#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use scale::layer1::building::OccupiedTiles;
    use scale::layer1::flora::{flora_spread_system, Flora, FloraType};
    use scale::layer1::map::GridPosition;
    use scale::layer1::morale::Morale;
    use scale::layer1::pheromone::{pheromone_emission_system, PheromoneEmitter};
    use scale::layer1::pop::Pop;
    use scale::layer1::anomalies::{spawn_initial_anomalies, Anomaly, AnomalyType};
    use scale::layer1::terrain::{TerrainGrid, TerrainType};

    fn setup_world() -> World {
        let mut world = World::new();
        let width = 20;
        let height = 20;
        let tiles = vec![TerrainType::Grass; width * height];

        world.insert_resource(TerrainGrid {
            width,
            height,
            tiles,
        });
        world.insert_resource(OccupiedTiles::default());
        world.insert_resource(scale::shared::time::SimulationTime::default());
        world
    }

    #[test]
    fn test_strange_flora_has_pheromones() {
        let mut world = setup_world();

        // Spawn anomalies until we get StrangeFlora
        // Since spawn_initial_anomalies is random, we might need to try a few times or force seed?
        // But we can't easily force seed here as it uses thread_rng.
        // Instead, let's manually spawn the Anomaly entity AS IF it was spawned by the system,
        // to test the *system's logic*? No, we want to test the spawner itself.

        // We can just spawn a lot and find one.
        spawn_initial_anomalies(&mut world, 50);

        let strange_flora = world
            .query::<(&Anomaly, Option<&PheromoneEmitter>)>()
            .iter(&world)
            .find(|(a, _)| a.anomaly_type == AnomalyType::StrangeFlora);

        assert!(
            strange_flora.is_some(),
            "Should have spawned at least one StrangeFlora"
        );
        let (_, emitter) = strange_flora.unwrap();

        // This assertion will FAIL initially
        assert!(
            emitter.is_some(),
            "StrangeFlora should have PheromoneEmitter"
        );

        // Verify Emitter Config
        let emitter = emitter.unwrap();
        assert_eq!(emitter.effect.label, "Strange Scent");
        assert!(emitter.effect.value > 0.0);
    }

    #[test]
    fn test_xenomoss_spread_has_pheromones() {
        let mut world = setup_world();

        // Spawn initial Flora (XenoMoss)
        world.spawn((
            Flora {
                flora_type: FloraType::XenoMoss,
                growth_timer: 0,    // Immediate spread
                spread_chance: 1.0, // Guaranteed
                ..Default::default()
            },
            GridPosition { x: 10, y: 10 },
        ));

        // Run spread system
        let mut schedule = Schedule::default();
        schedule.add_systems(flora_spread_system);
        schedule.run(&mut world);

        // Find the new Flora
        let _new_flora = world
            .query::<(Entity, &Flora, Option<&PheromoneEmitter>)>()
            .iter(&world)
            .find(|(_, f, _)| f.flora_type == FloraType::XenoMoss);

        // Since we started with one, we should find at least one. But we want the NEW one.
        // Actually, the initial one doesn't have PheromoneEmitter either in this test setup.
        // So checking *any* XenoMoss having it is a failure in the current code (since none do).
        // But ideally the *spawner* attaches it.

        // Let's check if ANY flora has it.
        let has_emitter = world
            .query::<(&Flora, &PheromoneEmitter)>()
            .iter(&world)
            .any(|(f, _)| f.flora_type == FloraType::XenoMoss);

        // This assertion will FAIL initially
        assert!(has_emitter, "XenoMoss should have PheromoneEmitter");
    }

    #[test]
    fn test_pheromone_affects_morale() {
        // This tests the SEAM (Emitter -> Morale)
        // We manually attach emitter to prove the connection works IF the component is present.
        let mut world = setup_world();

        let pop = world
            .spawn((Pop, GridPosition { x: 10, y: 10 }, Morale::default()))
            .id();

        world.spawn((
            GridPosition { x: 10, y: 11 }, // Adjacent
            PheromoneEmitter {
                radius: 2,
                interval: 1,
                timer: 0,
                effect: scale::layer1::pheromone::PheromoneEffect {
                    label: "Test Scent".to_string(),
                    value: 0.1,
                    duration: 10,
                },
            },
        ));

        let mut schedule = Schedule::default();
        schedule.add_systems(pheromone_emission_system);
        schedule.run(&mut world);

        let morale = world.get::<Morale>(pop).unwrap();
        assert!(morale.modifiers.iter().any(|m| m.label == "Test Scent"));
    }
}
