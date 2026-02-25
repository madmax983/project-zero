use crate::layer2::fleet::{Fleet, InTransit};
use bevy_ecs::prelude::*;
use rand::Rng;

/// Component representing the number of barnacles attached to a fleet.
#[derive(Component, Default, Debug, Clone, Copy)]
pub struct SpaceBarnacles {
    /// The number of barnacles currently attached.
    pub count: u32,
}

/// The maximum number of barnacles that can accumulate on a fleet.
pub const MAX_BARNACLES: u32 = 1000;
/// The speed reduction per barnacle (0.0005 = 0.05%).
pub const DRAG_PER_BARNACLE: f32 = 0.0005;
/// The minimum speed multiplier a fleet can be reduced to (10%).
pub const MIN_SPEED: f32 = 0.1;

/// Calculates the speed multiplier (0.1 to 1.0).
#[must_use]
pub fn calculate_speed_modifier(count: u32) -> f32 {
    #[allow(clippy::cast_precision_loss)]
    let penalty = count as f32 * DRAG_PER_BARNACLE;
    (1.0 - penalty).max(MIN_SPEED)
}

/// System to randomly add barnacles to fleets in transit.
pub fn barnacle_accumulation_system(mut query: Query<(&mut SpaceBarnacles, &InTransit)>) {
    let mut rng = rand::thread_rng();
    for (mut barnacles, _transit) in &mut query {
        // 5% chance per tick to gain a barnacle
        if rng.gen_bool(0.05) {
            barnacles.count = (barnacles.count + 1).min(MAX_BARNACLES);
        }
    }
}

/// Helper to clean barnacles (called via command or interaction).
pub fn clean_barnacles(world: &mut World, fleet_entity: Entity) {
    if let Some(mut barnacles) = world.get_mut::<SpaceBarnacles>(fleet_entity) {
        barnacles.count = 0;
    }
}

/// System to ensure all fleets have the `SpaceBarnacles` component.
pub fn ensure_barnacles_component_system(
    mut commands: Commands,
    query: Query<Entity, (With<Fleet>, Without<SpaceBarnacles>)>,
) {
    for entity in query.iter() {
        commands.entity(entity).insert(SpaceBarnacles::default());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer2::fleet::{Fleet, InOrbit, InTransit};
    use bevy_ecs::prelude::*;

    // 1. Test Barnacle Accumulation
    #[test]
    fn test_barnacle_accumulation() {
        let mut world = World::new();
        let mut schedule = Schedule::default();
        schedule.add_systems(barnacle_accumulation_system);

        // Spawn a fleet in transit
        let fleet = world
            .spawn((
                Fleet,
                InTransit {
                    origin: Entity::from_raw(0),
                    destination: Entity::from_raw(1),
                    progress: 0.0,
                    duration: 100.0,
                },
                SpaceBarnacles { count: 0 },
            ))
            .id();

        // Run system multiple times to simulate time passing
        for _ in 0..100 {
            schedule.run(&mut world);
        }

        let barnacles = world.get::<SpaceBarnacles>(fleet).unwrap();
        assert!(
            barnacles.count > 0,
            "Barnacles should accumulate during transit"
        );
    }

    // 2. Test Drag Effect on Speed (Unit Test)
    #[test]
    fn test_barnacle_drag_modifier() {
        let clean_speed = calculate_speed_modifier(0);
        let dirty_speed = calculate_speed_modifier(100);

        assert!(
            (clean_speed - 1.0).abs() < f32::EPSILON,
            "Clean speed should be 1.0"
        );
        assert!(dirty_speed < 1.0, "Barnacles should reduce speed");
        assert!(
            dirty_speed > 0.1,
            "Should not completely stop (min speed cap)"
        );
    }

    // 3. Test Cleaning
    #[test]
    fn test_barnacle_cleaning() {
        let mut world = World::new();
        let fleet = world
            .spawn((
                Fleet,
                InOrbit {
                    parent: Entity::from_raw(0),
                }, // Must be in orbit to clean
                SpaceBarnacles { count: 50 },
            ))
            .id();

        // Run cleaning command/system
        clean_barnacles(&mut world, fleet);

        let barnacles = world.get::<SpaceBarnacles>(fleet).unwrap();
        assert_eq!(barnacles.count, 0, "Cleaning should remove all barnacles");
    }

    #[test]
    fn test_barnacle_slows_movement() {
        let mut world = World::new();
        let mut schedule = Schedule::default();
        schedule.add_systems(crate::layer2::fleet::fleet_movement_system);

        let origin = world.spawn_empty().id();
        let destination = world.spawn_empty().id();

        // Spawn a clean fleet
        let clean_fleet = world
            .spawn((
                Fleet,
                InTransit {
                    origin,
                    destination,
                    progress: 0.0,
                    duration: 100.0,
                },
                SpaceBarnacles { count: 0 },
            ))
            .id();

        // Spawn a dirty fleet (500 barnacles = 25% penalty -> 0.75 speed)
        let dirty_fleet = world
            .spawn((
                Fleet,
                InTransit {
                    origin,
                    destination,
                    progress: 0.0,
                    duration: 100.0,
                },
                SpaceBarnacles { count: 500 },
            ))
            .id();

        // Run one tick
        schedule.run(&mut world);

        let clean_transit = world.get::<InTransit>(clean_fleet).unwrap();
        let dirty_transit = world.get::<InTransit>(dirty_fleet).unwrap();

        assert!(
            clean_transit.progress > dirty_transit.progress,
            "Clean fleet should be faster"
        );

        // Check exact values
        // Clean: 1.0/100.0 = 0.01
        // Dirty: (1.0/100.0) * (1.0 - 500 * 0.0005) = 0.01 * (1.0 - 0.25) = 0.01 * 0.75 = 0.0075
        assert!((clean_transit.progress - 0.01).abs() < 0.00001);
        assert!((dirty_transit.progress - 0.0075).abs() < 0.00001);
    }

    #[test]
    fn test_ensure_barnacles_component() {
        let mut world = World::new();
        let mut schedule = Schedule::default();
        schedule.add_systems(ensure_barnacles_component_system);

        let fleet = world.spawn(Fleet).id();

        schedule.run(&mut world);

        assert!(
            world.get::<SpaceBarnacles>(fleet).is_some(),
            "Fleet should have SpaceBarnacles component"
        );
    }
}
