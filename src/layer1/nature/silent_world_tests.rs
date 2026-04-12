#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use bevy_ecs::system::RunSystemOnce;

    use crate::layer1::fauna::{suppress_fauna_system, Fauna};
    use crate::layer1::flora::{consume_silent_flora_system, Flora, FloraType};
    use crate::layer1::map::GridPosition;
    use crate::layer1::needs::Needs;
    use crate::layer1::physics::acoustic::{update_noise_system, NoiseMap, NoiseSource};
    use crate::layer1::pop::Pop;
    use crate::layer1::quirks::{PlanetaryTrait, PlanetaryTraits};
    use crate::layer1::terrain::{TerrainGrid, TerrainType};

    #[test]
    fn test_silent_flora_growth_bonus() {
        // Arrange: A Pop consuming the Silent Flora.
        let mut app = bevy_app::App::new();
        app.add_systems(bevy_app::Update, consume_silent_flora_system);

        // Spawn a hungry pop
        let pop = app
            .world_mut()
            .spawn((
                Pop,
                GridPosition { x: 5, y: 5 },
                Needs {
                    hunger: 0.1,
                    ..Default::default()
                },
            ))
            .id();

        // Spawn a Silent Flora nearby
        app.world_mut().spawn((
            Flora {
                flora_type: FloraType::SilentFlora,
                ..Default::default()
            },
            GridPosition { x: 6, y: 5 },
        ));

        // Act: Advance time.
        app.update();

        // Assert: Pop growth/health bonuses are correctly applied from high nutrition.
        // The pop's hunger should be refilled completely due to the massive nutrition bonus.
        let needs = app.world().get::<Needs>(pop).unwrap();
        assert!(
            needs.hunger > 0.9,
            "SilentFlora should provide massive nutrition bonus"
        );
    }

    #[test]
    fn test_silent_flora_nullifies_sound() {
        // Arrange: A building emitting a noise alert within 5 tiles of Silent Flora.
        let mut app = bevy_app::App::new();
        let grid_size = 10 * 10;
        let terrain = TerrainGrid {
            width: 10,
            height: 10,
            tiles: vec![TerrainType::Grass; grid_size],
        };
        app.insert_resource(terrain);
        app.insert_resource(NoiseMap::new(10, 10));

        app.add_systems(bevy_app::Update, update_noise_system);

        // Spawn a loud machine at (5, 5)
        app.world_mut().spawn((
            NoiseSource {
                radius: 5.0,
                intensity: 1.0,
            },
            GridPosition { x: 5, y: 5 },
        ));

        // Spawn Silent Flora within 5 tiles (at 3, 5)
        app.world_mut().spawn((
            Flora {
                flora_type: FloraType::SilentFlora,
                ..Default::default()
            },
            GridPosition { x: 3, y: 5 },
        ));

        // Act: Try to trigger or propagate the sound alert.
        app.update();

        let noise_map = app.world().resource::<NoiseMap>();

        // Assert: The sound alert fails to propagate or trigger properly due to the flora.
        // It nullifies all sound within a 5-tile radius.
        // The center of the noise source is within 5 tiles of (3, 5) (distance 2).
        // Since it nullifies sound within 5 tiles of the flora, (5,5) should have ambient or 0.0 noise.
        assert!(
            noise_map.get(5, 5) <= 0.1,
            "Sound should be nullified near Silent Flora"
        );
    }

    #[test]
    fn test_silent_flora_no_native_fauna() {
        // Arrange: Generating a Silent World planet.
        let mut app = bevy_app::App::new();
        app.insert_resource(PlanetaryTraits(vec![PlanetaryTrait::SilentWorld]));
        app.add_systems(bevy_app::Update, suppress_fauna_system);

        // Spawn native fauna
        let fauna = app
            .world_mut()
            .spawn((Fauna::default(), GridPosition { x: 5, y: 5 }))
            .id();

        // Act: Spawn planet entities.
        app.update();

        // Assert: Zero native fauna are spawned (despawned).
        assert!(
            app.world().get::<Fauna>(fauna).is_none(),
            "Native fauna should be suppressed on a Silent World"
        );
    }
}
