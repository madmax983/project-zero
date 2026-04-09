//! Tests for explosion overflow security
#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use scale::layer1::map::GridPosition;
    use scale::layer1::structure::Structure;
    use scale::layer1::volatile::{handle_explosion_system, ExplosionEvent};

    #[test]
    fn test_explosion_overflow_exploit() {
        let mut world = World::new();
        // Register events
        world.init_resource::<Events<ExplosionEvent>>();

        // Case 1: Center at i32::MAX, Target at i32::MIN
        // Distance is approx 4 billion.
        // i32::MAX - i32::MIN = overflow.
        // If calculated as i32 (wrapping), it might wrap to something small negative.
        // abs() would make it small positive.

        let center = GridPosition { x: i32::MAX, y: 0 };
        // We place the target at a position such that (center.x - target.x) overflows.
        // i32::MAX - (i32::MIN + 10)
        // = 2147483647 - (-2147483648 + 10)
        // = 2147483647 - (-2147483638)
        // = 4294967285
        // As i32, this wraps.
        // 4294967285 % 2^32 = -11 (approx)
        // abs(-11) = 11.
        // So distance is calculated as 11.

        let target_pos = GridPosition {
            x: i32::MIN + 10,
            y: 0,
        };

        // Spawn Structure
        let structure = world
            .spawn((
                Structure {
                    current_hp: 100.0,
                    max_hp: 100.0,
                },
                target_pos,
            ))
            .id();

        // Send Event manually
        let mut events = world.resource_mut::<Events<ExplosionEvent>>();
        events.send(ExplosionEvent {
            center,
            damage: 100.0,
            radius: 20, // Radius 20. If distance is calculated as 11, it HITS.
        });

        // Run handle system
        let mut schedule = Schedule::default();
        schedule.add_systems(handle_explosion_system);
        schedule.run(&mut world);

        let s = world.get::<Structure>(structure).unwrap();

        // If vulnerable, the distance calculation overflowed and wrapped to 11,
        // causing the structure to take damage despite being billions of units away.
        // If secure, distance should be huge, so no damage.
        assert_eq!(
            s.current_hp, 100.0,
            "Structure took damage despite being at extreme distance! Integer overflow detected. HP: {}",
            s.current_hp
        );
    }
}
