use crate::layer2::ftl::wakes::SubspaceWakeEvent;
use bevy::prelude::*;

#[derive(Component)]
pub struct SubspaceFlora;

/// Simulates the effect of a subspace wake in the colony.
/// A subspace wake teleportation can manifest Subspace Flora into the colony
/// or teleport random components away. Here we just simulate spawning Subspace Flora hazards
/// near the target system.
pub fn apply_subspace_wake_system(
    mut commands: Commands,
    mut wake_events: EventReader<SubspaceWakeEvent>,
) {
    for wake in wake_events.read() {
        if wake.severity > 0.0 {
            // Apply effect proportional to severity.
            // Spawning SubspaceFlora entities based on the severity.
            let flora_count = (wake.severity * 5.0) as usize;
            for _ in 0..flora_count {
                commands.spawn(SubspaceFlora);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::ecs::system::RunSystemOnce;

    fn setup_world() -> World {
        let mut world = World::new();
        world.insert_resource(Events::<SubspaceWakeEvent>::default());
        world
    }

    #[test]
    fn test_apply_subspace_wake_system_spawns_subspace_flora() {
        let mut world = setup_world();

        world.resource_mut::<Events<SubspaceWakeEvent>>().send(SubspaceWakeEvent {
            system: Entity::PLACEHOLDER,
            severity: 1.0,
        });

        let _ = world.run_system_once(apply_subspace_wake_system);

        let mut query = world.query::<&SubspaceFlora>();
        let spawned_count = query.iter(&world).count();

        assert_eq!(spawned_count, 5, "Should spawn 5 flora for severity 1.0");
    }

    #[test]
    fn test_apply_subspace_wake_system_ignores_low_severity() {
        let mut world = setup_world();

        world.resource_mut::<Events<SubspaceWakeEvent>>().send(SubspaceWakeEvent {
            system: Entity::PLACEHOLDER,
            severity: 0.1, // very low severity => 0.1 * 5.0 = 0 target flora
        });

        let _ = world.run_system_once(apply_subspace_wake_system);

        let mut query = world.query::<&SubspaceFlora>();
        let spawned_count = query.iter(&world).count();

        assert_eq!(spawned_count, 0, "Should spawn 0 flora for low severity");
    }
}
