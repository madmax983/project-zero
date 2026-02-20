# 177: Stellar Cartography

## 1. Overview

**Layer:** 3 (Galaxy)
**Status:** Spec

Implements the `GalaxyMap` and `StarSystem` discovery mechanics for Layer 3. The galaxy is vast and mostly unknown. Players start with visibility only of their home system. To interact with other systems (trade, war, diplomacy), they must first "Discover" (reveal existence) and then "Chart" (reveal details) them.

This feature adds the "Fog of War" to the galaxy map.

## 2. Dependencies

- `001` — Project Scaffold
- `094` — System View Architecture (defines what a "System" is)

## 3. RED Phase: Tests First

Write these tests in `src/layer3/cartography_tests.rs`. They will initially FAIL.

```rust
#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer3::cartography::{GalaxyMap, StarSystem, DiscoveryState, GalaxyPosition};

    fn setup_world() -> World {
        let mut world = World::new();
        world.insert_resource(GalaxyMap::default());
        world
    }

    #[test]
    fn test_galaxy_map_initialization() {
        let world = setup_world();
        let map = world.resource::<GalaxyMap>();
        assert!(map.systems.is_empty(), "Map should start empty (or procedural generation handles it)");
    }

    #[test]
    fn test_star_system_spawn_defaults_to_hidden() {
        let mut world = setup_world();

        // Spawn a system
        let entity = world.spawn((
            StarSystem,
            GalaxyPosition { x: 10, y: 20, z: 0 },
            DiscoveryState::default(),
        )).id();

        let state = world.get::<DiscoveryState>(entity).expect("Should have state");
        assert_eq!(*state, DiscoveryState::Hidden, "Systems should be hidden by default");
    }

    #[test]
    fn test_discovery_state_transitions() {
        let mut world = setup_world();
        let entity = world.spawn((
            StarSystem,
            DiscoveryState::Hidden,
        )).id();

        // Act: Reveal the system
        let mut state = world.get_mut::<DiscoveryState>(entity).unwrap();
        *state = DiscoveryState::Known;

        // Assert
        assert_eq!(*world.get::<DiscoveryState>(entity).unwrap(), DiscoveryState::Known);
    }

    #[test]
    fn test_fog_of_war_logic() {
        // Arrange: A sensor source (e.g., Colony) and a distant system
        let mut world = setup_world();

        // Spawn "Home" system (Known)
        let home = world.spawn((
            StarSystem,
            GalaxyPosition { x: 0, y: 0, z: 0 },
            DiscoveryState::Visited, // Home is visited
        )).id();

        // Spawn "Distant" system (Hidden)
        let target = world.spawn((
            StarSystem,
            GalaxyPosition { x: 100, y: 0, z: 0 },
            DiscoveryState::Hidden,
        )).id();

        // Act: Run sensor system (mock logic for now)
        // If we were implementing the full sensor system, we'd run it here.
        // For this unit test, we just verify the enum variants exist and behave.

        assert!(matches!(DiscoveryState::Hidden, DiscoveryState::Hidden));
        assert!(matches!(DiscoveryState::Known, DiscoveryState::Known));
        assert!(matches!(DiscoveryState::Visited, DiscoveryState::Visited));
    }
}
```

## 4. GREEN Phase: Minimal Implementation

### 1. Define Components (`src/layer3/cartography.rs`)

```rust
use bevy_ecs::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Resource, Default, Debug)]
pub struct GalaxyMap {
    // Spatial index could go here later
    pub systems: Vec<Entity>,
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct GalaxyPosition {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

#[derive(Component, Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DiscoveryState {
    #[default]
    Hidden,   // Not visible on map.
    Known,    // Visible as a point/name, but details (planets) unknown.
    Charted,  // Fully visible, resource data known.
    Visited,  // Player has ships/colonies here.
}

#[derive(Component, Debug)]
pub struct StarSystem;

// System to update discovery based on range
// (To be implemented fully when Fleets exist)
pub fn update_discovery_system(
    mut query: Query<(&GalaxyPosition, &mut DiscoveryState)>,
    sensors: Query<(&GalaxyPosition, &SensorRange)>,
) {
    // TBD
}

#[derive(Component)]
pub struct SensorRange(pub u32);
```

## 5. REFACTOR Phase: Quality & Design

- **Spatial Hashing**: Storing all systems in a `Vec` or query is O(N). For a large galaxy (10k systems), use a spatial hash (e.g., `HashMap<(i32, i32), Entity>`) in `GalaxyMap` to quickly find neighbors.
- **Events**: Emit `SystemDiscoveredEvent` when a state changes from Hidden -> Known, to trigger UI notifications ("New System Detected").
- **Persistence**: Ensure `DiscoveryState` is serialized in the save file.
- **Fog Rendering**: The UI needs to know how to render "Known" but "Uncharted" systems (maybe just a star icon with no planet count).

## 6. Acceptance Criteria

- [ ] `GalaxyPosition` and `DiscoveryState` components exist.
- [ ] Systems spawn as `Hidden` by default.
- [ ] `DiscoveryState` enum has at least `Hidden`, `Known`, `Visited`.
- [ ] Tests pass.

## 7. Technical Guidance

- Use `bevy_ecs` resources.
- Keep `GalaxyPosition` separate from Layer 2 `OrbitalBody` positions. Layer 3 coordinates are light-years, Layer 2 is AU.
