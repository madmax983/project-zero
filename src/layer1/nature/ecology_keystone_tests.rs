#[cfg(test)]
mod tests {
    use crate::layer1::ecology::{
        biome_collapse_system, handle_keystone_death, BiomeAnchor, DependentOn, Species,
    };
    use crate::layer1::health::{Dead, Health};
    use crate::layer1::map::GridPosition;
    use crate::layer1::terrain::{TerrainGrid, TerrainType};
    use bevy_ecs::prelude::*;

    #[test]
    fn test_keystone_removal_triggers_collapse() {
        let mut world = World::new();

        // Arrange: Keystone entity and dependent entity
        let keystone = world
            .spawn((
                Species {
                    name: "Giant Cactus".to_string(),
                    is_keystone: true,
                },
                Health {
                    current: 10.0,
                    max: 10.0,
                    has_rust_lung: false,
                },
                GridPosition { x: 5, y: 5 },
            ))
            .id();

        let dependent = world
            .spawn((
                Species {
                    name: "Grazer Beetle".to_string(),
                    is_keystone: false,
                },
                DependentOn(keystone),
                Health {
                    current: 5.0,
                    max: 5.0,
                    has_rust_lung: false,
                },
                GridPosition { x: 6, y: 5 },
            ))
            .id();

        // Act: Kill the keystone
        // We simulate death by adding the Dead component, which is what the health system does.
        world.entity_mut(keystone).insert(Dead);

        // Run ecology system
        let mut schedule = Schedule::default();
        schedule.add_systems(biome_collapse_system);
        schedule.run(&mut world);

        // Assert: Dependent entity should be dead or despawned
        // The system might despawn it directly or damage it to death.
        // Spec says: "Dependent dies."
        // Let's assume it despawns or adds Dead component.
        // For RED phase, we check if it is gone or marked Dead.

        let dependent_entity = world.get_entity(dependent);
        if let Ok(e) = dependent_entity {
            // If it still exists, it must have Dead component or 0 health
            let is_dead = e.contains::<Dead>();
            let health_zero = world
                .get::<Health>(dependent)
                .is_some_and(|h| h.current <= 0.0);
            assert!(is_dead || health_zero, "Dependent should be dead");
        }
        // If it's Err, it was despawned, which is also valid success.
    }

    #[test]
    fn test_biome_degradation() {
        let mut world = World::new();
        // Setup terrain
        let mut tiles = vec![TerrainType::Grass; 100];
        // Set (5,5) to Forest (Tree)
        tiles[55] = TerrainType::Tree;

        world.insert_resource(TerrainGrid {
            width: 10,
            height: 10,
            tiles,
        });

        let keystone = world
            .spawn((
                Species {
                    name: "Ironwood Tree".to_string(),
                    is_keystone: true,
                },
                BiomeAnchor {
                    radius: 0, // Current implementation only affects self
                    target_terrain: TerrainType::Tree,
                    fallback_terrain: TerrainType::Dirt,
                },
                GridPosition { x: 5, y: 5 },
                Health::default(),
            ))
            .id();

        // Kill keystone by adding Dead component
        world.entity_mut(keystone).insert(Dead);

        // Run system
        let mut schedule = Schedule::default();
        schedule.add_systems(handle_keystone_death);
        schedule.run(&mut world);

        // Verify terrain changed to fallback (Dirt)
        let terrain = world.resource::<TerrainGrid>();
        assert_eq!(terrain.get(5, 5).unwrap(), TerrainType::Dirt);
    }

    #[test]
    fn test_dependent_dies_if_anchor_despawned() {
        let mut world = World::new();

        // Keystone existed but was despawned completely
        let keystone_id = world.spawn_empty().id(); // Get an ID
        world.despawn(keystone_id); // Now it's gone

        let dependent = world
            .spawn((
                Species {
                    name: "Grazer Beetle".to_string(),
                    is_keystone: false,
                },
                DependentOn(keystone_id),
                Health {
                    current: 5.0,
                    max: 5.0,
                    has_rust_lung: false,
                },
            ))
            .id();

        // Run system
        let mut schedule = Schedule::default();
        schedule.add_systems(biome_collapse_system);
        schedule.run(&mut world);

        // Assert: Dependent should be dead/despawned
        let dependent_entity = world.get_entity(dependent);
        if let Ok(e) = dependent_entity {
            let is_dead = e.contains::<Dead>();
            assert!(
                is_dead,
                "Dependent should be marked Dead if anchor is missing"
            );
        }
    }
}
