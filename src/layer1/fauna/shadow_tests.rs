#[cfg(test)]
mod tests {
    use crate::layer1::energy::load_limits::PowerCable;
    use crate::layer1::fauna::shadow::{
        shadow_visibility_system, spawn_shadow_fauna_system, ShadowEntity, ShadowType,
    };
    use crate::layer1::map::GridPosition;
    use crate::layer1::tech::infinite_archive::Archive;
    use bevy_ecs::prelude::*;

    #[test]
    fn test_high_energy_spawns_static_mites() {
        let mut world = World::new();
        // Setup high energy cable at (5,5)
        world.spawn((
            PowerCable {
                capacity: 100.0,
                current_load: 1000.0, // Very high load
            },
            GridPosition { x: 5, y: 5 },
        ));

        // Ensure no other entities interfere
        world.insert_resource(Archive::default());

        // Run spawn system
        let mut schedule = Schedule::default();
        schedule.add_systems(spawn_shadow_fauna_system);
        schedule.run(&mut world);

        // Check for entity
        let mut query = world.query::<(&ShadowEntity, &GridPosition)>();
        let found = query
            .iter(&world)
            .any(|(e, pos)| e.entity_type == ShadowType::StaticMite && pos.x == 5 && pos.y == 5);
        assert!(found);
    }

    #[test]
    fn test_high_data_density_spawns_data_rot() {
        let mut world = World::new();

        world.insert_resource(Archive {
            capacity: 100.0,
            used: 1000.0,
            efficiency_multiplier: 0.1,
        });

        // Run spawn system
        let mut schedule = Schedule::default();
        schedule.add_systems(spawn_shadow_fauna_system);
        schedule.run(&mut world);

        // Check for DataRot entity
        let mut query = world.query::<&ShadowEntity>();
        let found = query
            .iter(&world)
            .any(|e| e.entity_type == ShadowType::DataRot);
        assert!(found);
    }

    #[test]
    fn test_shadow_feed_reduces_power_efficiency() {
        // Implementation check to verify Mite reduces local machine output
        use crate::layer1::fauna::shadow::shadow_feed_system;

        let mut world = World::new();
        let cable = world
            .spawn((
                PowerCable {
                    capacity: 100.0,
                    current_load: 50.0,
                },
                GridPosition { x: 5, y: 5 },
            ))
            .id();

        world.spawn((
            ShadowEntity {
                entity_type: ShadowType::StaticMite,
                hunger: 100.0,
                visible: false,
            },
            GridPosition { x: 5, y: 5 },
        ));

        let mut schedule = Schedule::default();
        schedule.add_systems(shadow_feed_system);
        schedule.run(&mut world);

        let c = world.get::<PowerCable>(cable).unwrap();
        // Since the static mite is feeding, the current_load should increase.
        assert!(c.current_load > 50.0);
    }

    #[test]
    fn test_data_rot_feed_corrupts() {
        use crate::layer1::fauna::shadow::shadow_feed_system;

        let mut world = World::new();
        world.insert_resource(Archive {
            capacity: 100.0,
            used: 10.0,
            efficiency_multiplier: 1.0,
        });

        world.spawn((
            ShadowEntity {
                entity_type: ShadowType::DataRot,
                hunger: 100.0,
                visible: false,
            },
            GridPosition { x: 5, y: 5 },
        ));

        let mut schedule = Schedule::default();
        schedule.add_systems(shadow_feed_system);
        schedule.run(&mut world);

        let archive = world.resource::<Archive>();
        // Data rot consumes data and corrupts efficiency multiplier.
        assert!(archive.efficiency_multiplier < 1.0);
    }

    #[test]
    fn test_visibility_toggle() {
        let mut world = World::new();
        use crate::layer1::nature::weather::{WeatherState, WeatherType};
        world.insert_resource(WeatherState {
            current_weather: WeatherType::MagneticStorm,
            duration_remaining: 100,
        });

        let e = world
            .spawn((
                ShadowEntity {
                    entity_type: ShadowType::StaticMite,
                    hunger: 100.0,
                    visible: false,
                },
                GridPosition { x: 5, y: 5 },
            ))
            .id();

        let mut schedule = Schedule::default();
        schedule.add_systems(shadow_visibility_system);
        schedule.run(&mut world);

        let shadow = world.get::<ShadowEntity>(e).unwrap();
        assert!(shadow.visible);
    }
}
