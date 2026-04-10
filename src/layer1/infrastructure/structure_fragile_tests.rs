use bevy_ecs::prelude::*;
    use crate::layer1::structure::{fragile_decay_system, Fragile, Structure};
    use crate::layer1::GridPosition;


    fn setup_world() -> World {
        World::new()
    }

    #[test]
    fn test_fragile_decay_reduces_hp() {
        let mut world = setup_world();
        let building = world
            .spawn((
                Structure {
                    current_hp: 100.0,
                    max_hp: 100.0,
                },
                Fragile { stacks: 100 }, // High stacks to guarantee decay if probabilistic
                GridPosition { x: 0, y: 0 },
            ))
            .id();

        // Run system multiple times to catch probabilistic decay
        for _ in 0..100 {
            fragile_decay_system(&mut world);
        }

        let structure = world.get::<Structure>(building).unwrap();
        assert!(
            structure.current_hp < 100.0,
            "Fragile building should decay over time"
        );
    }

    #[test]
    fn test_fragile_decay_scales_with_stacks() {
        let mut world = setup_world();

        // Low stacks
        let low_stacks = world
            .spawn((
                Structure {
                    current_hp: 1000.0,
                    max_hp: 1000.0,
                },
                Fragile { stacks: 1 },
                GridPosition { x: 0, y: 0 },
            ))
            .id();

        // High stacks
        let high_stacks = world
            .spawn((
                Structure {
                    current_hp: 1000.0,
                    max_hp: 1000.0,
                },
                Fragile { stacks: 50 },
                GridPosition { x: 1, y: 0 },
            ))
            .id();

        // Run for a while
        for _ in 0..100 {
            fragile_decay_system(&mut world);
        }

        let hp_low = world.get::<Structure>(low_stacks).unwrap().current_hp;
        let hp_high = world.get::<Structure>(high_stacks).unwrap().current_hp;

        assert!(
            hp_high < hp_low,
            "High stacks should decay faster/more often. Low: {}, High: {}",
            hp_low,
            hp_high
        );
    }