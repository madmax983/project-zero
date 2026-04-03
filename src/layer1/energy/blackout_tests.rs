#[cfg(test)]
mod tests {
    use crate::layer1::energy::{power_grid_system, BlackoutProtocol, PowerConsumer, PowerSource};
    use crate::layer1::lighting::{update_lighting_system, AmbientLight, LightMap, LightSource};
    use crate::layer1::map::GridPosition;
    use crate::layer1::needs::Needs;
    use crate::layer1::pop::{Pop, Speed};
    use crate::layer1::traits::{Trait, Traits};
    use bevy_ecs::prelude::*;
    use bevy_ecs::system::RunSystemOnce;

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
        ));

        // Consumer (Demand 5)
        let consumer = world
            .spawn((
                PowerConsumer {
                    demand: 5.0,
                    active: true,
                }, // Starts active
                GridPosition { x: 0, y: 1 },
            ))
            .id();

        // Run grid system
        world.run_system_once(power_grid_system).unwrap();

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
                is_outdoor: true,
                radius: 5.0,
                intensity: 1.0,
                color: (255, 255, 255),
            },
            PowerConsumer {
                demand: 1.0,
                active: false,
            }, // Inactive due to blackout (simulated here)
            GridPosition { x: 5, y: 5 },
        ));

        // Light Source WITHOUT PowerConsumer (e.g. Torch)
        // Should remain active
        // NOTE: Must be far enough from Lamp (5,5) to avoid overlapping light causing false failure
        // Distance (2,2) to (5,5) is ~4.24, which is < 5.0 radius.
        // Moving to (0,0). Distance to (5,5) is ~7.07 > 5.0.
        world.spawn((
            LightSource {
                is_outdoor: true,
                radius: 5.0,
                intensity: 1.0,
                color: (255, 200, 100),
            },
            GridPosition { x: 0, y: 0 },
        ));

        world.run_system_once(update_lighting_system).unwrap();

        let map = world.resource::<LightMap>();

        // Lamp at 5,5 should be dark
        assert!(map.get(5, 5) < 0.1, "Powered light should be off");

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
                Speed::default(),
                {
                    let mut t = Traits::default();
                    t.add(Trait::Anxious);
                    t
                },
                Needs {
                    leisure: 1.0,
                    ..Default::default()
                },
            ))
            .id();

        // NightOwl Pop (Likes dark)
        let night_owl = world
            .spawn((
                Pop,
                GridPosition { x: 0, y: 0 },
                Speed::default(),
                {
                    let mut t = Traits::default();
                    t.add(Trait::NightOwl);
                    t
                },
                Needs {
                    leisure: 1.0,
                    ..Default::default()
                },
            ))
            .id();

        // Normal Pop
        let normal = world
            .spawn((
                Pop,
                GridPosition { x: 0, y: 0 },
                Speed::default(),
                Traits::default(),
                Needs {
                    leisure: 1.0,
                    ..Default::default()
                },
            ))
            .id();

        // Run lighting penalties system (enhanced)
        // Note: System name matches Spec 053
        world
            .run_system_once(crate::layer1::lighting::apply_lighting_penalties_system)
            .unwrap();

        let n_anxious = world.get::<Needs>(anxious).unwrap();
        let n_nightowl = world.get::<Needs>(night_owl).unwrap();
        let n_normal = world.get::<Needs>(normal).unwrap();

        // Anxious should lose MORE leisure (stress)
        // Normal loses some
        // NightOwl loses LESS or NONE
        assert!(
            n_anxious.leisure < n_normal.leisure,
            "Anxious pop should be more stressed"
        );
        assert!(
            n_normal.leisure < n_nightowl.leisure,
            "Normal pop should be more stressed than NightOwl"
        );
    }
}
