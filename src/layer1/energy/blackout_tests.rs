#[cfg(test)]
mod tests {
    use crate::layer1::energy::{BlackoutProtocol, PowerConsumer, PowerSource, power_grid_system};
    use crate::layer1::lighting::{AmbientLight, LightMap, LightSource, update_lighting_system};
    use crate::layer1::map::GridPosition;
    use crate::layer1::needs::Needs;
    use crate::layer1::pop::{Pop, Speed};
    use crate::layer1::traits::{Trait, Traits};
    use bevy_ecs::prelude::*;
    use bevy_ecs::system::RunSystemOnce;
    use std::collections::HashSet;

    #[test]
    fn test_blackout_cuts_power() {
        let mut world = World::new();
        world.insert_resource(BlackoutProtocol { active: true });

        // Generator (Output 10)
        world.spawn((
            PowerSource {
                output: 10.0,
                active: true,
            },
            GridPosition { x: 0, y: 0 },
            crate::layer1::building::Building {
                building_type: crate::layer1::building::BuildingType::Generator,
            }, // Needed for grid
        ));

        // Consumer (Demand 5)
        let consumer = world
            .spawn((
                PowerConsumer {
                    demand: 5.0,
                    active: true,
                }, // Starts active
                GridPosition { x: 0, y: 1 },
                crate::layer1::building::Building {
                    building_type: crate::layer1::building::BuildingType::Smelter,
                }, // Needed for grid
                crate::layer1::energy::Conduit, // Connect
            ))
            .id();

        // Run grid system
        // power_grid_system is a regular function taking &mut World, so we can call it directly
        power_grid_system(&mut world);

        let state = world.get::<PowerConsumer>(consumer).unwrap();
        assert!(!state.active, "Consumer should be inactive during blackout");
    }

    #[test]
    fn test_blackout_disables_lights() {
        let mut world = World::new();
        world.insert_resource(LightMap::new(10, 10));
        world.insert_resource(AmbientLight { level: 0.0 }); // Pitch black

        // Light Source WITH PowerConsumer (e.g. Lamp)
        // Should be disabled if PowerConsumer is inactive
        world.spawn((
            LightSource {
                radius: 5.0,
                intensity: 1.0,
                color: (255, 255, 255),
            },
            PowerConsumer {
                demand: 1.0,
                active: false,
            }, // Inactive due to blackout
            GridPosition { x: 9, y: 9 },
        ));

        // Light Source WITHOUT PowerConsumer (e.g. Torch)
        // Should remain active
        world.spawn((
            LightSource {
                radius: 5.0,
                intensity: 1.0,
                color: (255, 200, 100),
            },
            GridPosition { x: 0, y: 0 },
        ));

        // Use RunSystemOnce for system functions requiring queries
        let _ = world.run_system_once(update_lighting_system);

        let map = world.resource::<LightMap>();

        // Lamp at 9,9 should be dark
        assert!(
            map.get(9, 9) < 0.1,
            "Powered light should be off, got {}",
            map.get(9, 9)
        );

        // Torch at 0,0 should be lit
        assert!(map.get(0, 0) > 0.5, "Unpowered light (torch) should be on");
    }

    #[test]
    fn test_panic_in_darkness() {
        // Test Trait-based stress response
        let mut world = World::new();
        world.insert_resource(LightMap::new(10, 10));
        // Pitch black
        {
            let mut map = world.resource_mut::<LightMap>();
            map.tiles.fill(0.0);
        }

        // Anxious Pop (Scared of dark)
        let anxious = world
            .spawn((
                Pop,
                GridPosition { x: 0, y: 0 },
                Traits(HashSet::from([Trait::Anxious])),
                Needs {
                    leisure: 1.0,
                    ..Default::default()
                },
                Speed::default(),
            ))
            .id();

        // NightOwl Pop (Likes dark)
        let night_owl = world
            .spawn((
                Pop,
                GridPosition { x: 0, y: 0 },
                Traits(HashSet::from([Trait::NightOwl])),
                Needs {
                    leisure: 1.0,
                    ..Default::default()
                },
                Speed::default(),
            ))
            .id();

        // Normal Pop
        let normal = world
            .spawn((
                Pop,
                GridPosition { x: 0, y: 0 },
                Traits(HashSet::new()),
                Needs {
                    leisure: 1.0,
                    ..Default::default()
                },
                Speed::default(),
            ))
            .id();

        // Run lighting penalties system
        let _ = world.run_system_once(crate::layer1::lighting::apply_lighting_penalties_system);

        let n_anxious = world.get::<Needs>(anxious).unwrap();
        let n_nightowl = world.get::<Needs>(night_owl).unwrap();
        let n_normal = world.get::<Needs>(normal).unwrap();

        // Anxious should lose MORE leisure (stress)
        // Normal loses some
        // NightOwl loses LESS or NONE
        assert!(
            n_anxious.leisure < n_normal.leisure,
            "Anxious pop should be more stressed. Anxious: {}, Normal: {}",
            n_anxious.leisure,
            n_normal.leisure
        );
        assert!(
            n_normal.leisure < n_nightowl.leisure,
            "Normal pop should be more stressed than NightOwl. Normal: {}, NightOwl: {}",
            n_normal.leisure,
            n_nightowl.leisure
        );
    }
}
