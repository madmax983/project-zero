#[cfg(test)]
mod tests {
    use crate::layer1::beauty::{update_beauty_grid_system, BeautyGrid, BeautySource};
    use crate::layer1::energy::PowerConsumer;
    use crate::layer1::hologram::{
        apply_disillusionment_system, update_holograms_system, HoloProjector, HologramFailureEvent,
    };
    use crate::layer1::map::GridPosition;
    use crate::layer1::morale::Morale;
    use crate::layer1::pop::Pop;
    use crate::layer1::terrain::{TerrainGrid, TerrainType};
    use bevy_ecs::prelude::*;

    fn setup_world() -> World {
        let mut world = World::new();
        // Setup Grid for BeautySystem
        let terrain = TerrainGrid {
            width: 10,
            height: 10,
            tiles: vec![TerrainType::Grass; 100],
        };
        world.insert_resource(terrain);
        world.insert_resource(BeautyGrid::new(10, 10));
        world.insert_resource(Events::<HologramFailureEvent>::default());
        // need to also insert events for apply_disillusionment
        world.init_resource::<Events<HologramFailureEvent>>();
        world
    }

    #[test]
    fn test_hologram_adds_beauty_when_powered() {
        let mut world = setup_world();

        // Spawn powered HoloProjector
        world.spawn((
            HoloProjector {
                active_beauty: 50.0,
                radius: 5.0,
                is_active: false, // Initially off
            },
            BeautySource::default(), // Required for refactored system
            PowerConsumer {
                demand: 10.0,
                active: true, // Powered
                ..Default::default()
            },
            GridPosition { x: 5, y: 5 },
        ));

        // We also need to spawn the query that is matched in update_beauty_grid_system
        // The query is `items: Query<(&GridPosition, &ResourceItem)>` which will be empty.
        // The issue is likely that we need a larger `Schedule` or more robust setup.

        // Run system
        let mut schedule = Schedule::default();
        schedule.add_systems((update_holograms_system, update_beauty_grid_system).chain());
        schedule.run(&mut world);

        let beauty = world.resource::<BeautyGrid>();
        // Base beauty for Grass is 1.0. Hologram adds 50.0.
        // update_beauty_grid_system handles adding.
        // We check center point.
        assert!(
            beauty.get(5, 5) >= 51.0,
            "Expected beauty >= 51.0, got {}",
            beauty.get(5, 5)
        );
    }

    #[test]
    fn test_hologram_removes_beauty_when_unpowered() {
        let mut world = setup_world();

        // Spawn unpowered HoloProjector that WAS active
        world.spawn((
            HoloProjector {
                active_beauty: 50.0,
                radius: 5.0,
                is_active: true, // Was active
            },
            BeautySource {
                value: 50.0,
                radius: 5.0,
            }, // Currently active
            PowerConsumer {
                demand: 10.0,
                active: false, // Power cut
                ..Default::default()
            },
            GridPosition { x: 5, y: 5 },
        ));

        // Run system
        let mut schedule = Schedule::default();
        schedule.add_systems((update_holograms_system, update_beauty_grid_system).chain());
        schedule.run(&mut world);

        let beauty = world.resource::<BeautyGrid>();
        // Should drop to ambient (1.0 for Grass)
        // because update_holograms_system should set BeautySource.value to 0.0
        assert!(
            beauty.get(5, 5) <= 1.0 + f32::EPSILON,
            "Expected beauty <= 1.0 + EPSILON, got {}",
            beauty.get(5, 5)
        );
    }

    #[test]
    fn test_disillusionment_shock() {
        let mut world = setup_world();

        // Spawn HoloProjector losing power
        world.spawn((
            HoloProjector {
                active_beauty: 50.0,
                radius: 5.0,
                is_active: true,
            },
            BeautySource {
                value: 50.0,
                radius: 5.0,
            },
            PowerConsumer {
                demand: 10.0,
                active: false,
                ..Default::default()
            },
            GridPosition { x: 5, y: 5 },
        ));

        // Spawn Pop nearby
        let pop_id = world
            .spawn((
                Pop,
                GridPosition { x: 5, y: 5 },
                Morale { modifiers: vec![],
                    value: 0.8,
                    ..Default::default()
                },
            ))
            .id();

        // Run system
        let mut schedule = Schedule::default();
        schedule.add_systems(update_holograms_system);
        // Add shock system
        schedule.add_systems(apply_disillusionment_system);
        schedule.run(&mut world);
        schedule.run(&mut world);

        let morale = world.get::<Morale>(pop_id).unwrap();
        // Check modifiers
        let found = morale
            .modifiers
            .iter()
            .any(|m| m.label == "Disillusionment" && m.value == -0.2);
        assert!(found, "Disillusionment modifier not found");
    }
}
