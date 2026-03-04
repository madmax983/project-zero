# 225: Keystone Species

## Overview

Introduces ecological dependencies where specific "Keystone" flora/fauna species support the entire biome. If the Keystone species is removed (over-harvested, killed), the biome collapses: dependent species die, and the terrain may degrade (e.g., Forest -> Scrubland).

## Dependencies

- `002` — Terrain Grid
- `161` — Ecological Succession (Specced, provides `Flora` lifecycle)
- `164` — Modular Fauna (Implemented, provides `Fauna` entities)

## RED Phase: Tests First

```rust
// src/layer1/ecology_tests.rs

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::prelude::*;

    #[test]
    fn test_keystone_removal_triggers_collapse() {
        let mut world = World::new();

        // Arrange: Keystone entity and dependent entity
        let keystone = world.spawn((
            Species::Keystone("Giant Cactus".to_string()),
            Health { current: 10.0, max: 10.0 },
            GridPosition { x: 5, y: 5 },
        )).id();

        let dependent = world.spawn((
            Species::Dependent("Grazer Beetle".to_string()),
            DependentOn(keystone),
            Health { current: 5.0, max: 5.0 },
            GridPosition { x: 6, y: 5 },
        )).id();

        // Act: Kill the keystone
        // (Simulate damage or harvest)
        world.despawn(keystone);

        // Run ecology system
        let mut schedule = Schedule::default();
        schedule.add_systems(biome_collapse_system);
        schedule.run(&mut world);

        // Assert: Dependent entity should be dead or taking damage
        let dependent_health = world.get::<Health>(dependent);
        // Either despawned or health reduced to 0
        assert!(dependent_health.is_none() || dependent_health.unwrap().current <= 0.0);
    }

    #[test]
    fn test_biome_degradation() {
        let mut world = World::new();
        let mut terrain = TerrainGrid::new(10, 10);
        terrain.set(5, 5, TerrainType::Forest);
        world.insert_resource(terrain);

        let keystone = world.spawn((
            Species::Keystone("Ironwood Tree".to_string()),
            BiomeAnchor { radius: 2, target_terrain: TerrainType::Forest, fallback_terrain: TerrainType::Dirt },
            GridPosition { x: 5, y: 5 },
        )).id();

        // Kill keystone
        world.despawn(keystone);

        // Run system
        let mut schedule = Schedule::default();
        schedule.add_systems(biome_collapse_system);
        schedule.run(&mut world);

        // Verify terrain changed to fallback
        let terrain = world.resource::<TerrainGrid>();
        assert_eq!(terrain.get(5, 5).unwrap(), TerrainType::Dirt);
    }
}
```

## GREEN Phase: Minimal Implementation

### Components

```rust
// src/layer1/ecology.rs

#[derive(Component)]
pub struct Species {
    pub name: String,
    pub is_keystone: bool,
}

#[derive(Component)]
pub struct DependentOn(pub Entity);

#[derive(Component)]
pub struct BiomeAnchor {
    pub radius: u32,
    pub target_terrain: TerrainType, // Keeps it this way
    pub fallback_terrain: TerrainType, // Reverts to this if anchor dies
}
```

### Systems

```rust
// src/layer1/ecology.rs

pub fn biome_collapse_system(
    mut commands: Commands,
    mut terrain: ResMut<TerrainGrid>,
    // Query for dependents whose anchor is missing
    dependents: Query<(Entity, &DependentOn)>,
    // Query for anchors that were just removed (Need event or check existence)
    // Actually, easier to check if DependentOn target exists.
    all_entities: Query<Entity>,
) {
    // 1. Handle Dependents
    for (entity, dependency) in dependents.iter() {
        if !all_entities.contains(dependency.0) {
            // Anchor is gone. Dependent dies.
            commands.entity(entity).despawn_recursive();
            // Or apply 'Starving' component
        }
    }

    // 2. Handle Terrain Degradation
    // This requires tracking death events or "Keystone just died" events.
    // For MVP Green, we can use an event reader.
}

pub fn handle_keystone_death(
    mut events: EventReader<EntityDeathEvent>,
    query: Query<(&GridPosition, &BiomeAnchor)>,
    mut terrain: ResMut<TerrainGrid>,
) {
    for event in events.read() {
        if let Ok((pos, anchor)) = query.get(event.entity) {
            // Revert terrain in radius
            // For MVP, just the single tile
            if let Some(tile) = terrain.get_mut(pos.x, pos.y) {
                *tile = anchor.fallback_terrain;
            }
        }
    }
}
```

## REFACTOR Phase: Quality & Design

- **Optimization**: Don't query all entities every frame. Use `EntityDeathEvent` (from Spec 010 or 034) to trigger checks.
- **Grace Period**: Dependents shouldn't die instantly. Apply a `Starvation` effect that drains health over time.
- **Visuals**: Add a "Withered" state for terrain before it fully converts.
- **UI**: Warn player when targeting a Keystone species ("Warning: Ecological Anchor").

## Acceptance Criteria

- [ ] Keystone component defined.
- [ ] Dependent component defined.
- [ ] Removing Keystone kills/damages Dependents.
- [ ] Removing Keystone degrades Terrain (via event).
- [ ] Test coverage > 85%.

## Technical Guidance

- Use the existing `Health` and `Damage` systems.
- Integrate with `TerrainGrid` carefully; don't overwrite buildings.
- Ensure circular dependencies (A depends on B, B depends on A) don't crash the game (though they would cause mutual destruction).

## Questions

*Builder: add questions here if spec is unclear.*
*Architect:* See related specifications for design details. MVP implementation should follow standard conventions.
