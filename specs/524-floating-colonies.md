# 524. Floating Colonies

## 1. Overview
**Layer:** 2 -> 1
**Fantasy:** Living in the clouds of a gas giant, never touching the ground.
**Mechanic:** On Gas Giants, there is no terrain. You build on "Platform" tiles that float. If a platform is destroyed (storm/attack), everything on it falls into the crusher depths.
**Emergence:** A hurricane separates your power plant platform from the housing platform.
**Tension:** Verticality and stability vs. Resource extraction.

## 2. Dependencies
- `002` Basic Map (terrain grid)
- `153` Geological Instability (for platform destruction events)
- `079` Weather Events (hurricanes)

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use crate::layer1::map::{TerrainGrid, TerrainType};
    use crate::layer1::building::Building;
    use crate::layer1::events::PlatformDestroyedEvent;

    #[test]
    fn test_platform_destruction_drops_buildings() {
        let mut app = App::new();
        // Arrange: A Gas Giant map with a Platform and a Building on it
        app.insert_resource(TerrainGrid::new(10, 10, TerrainType::GasCloud));
        app.world_mut().spawn((
            Platform { x: 5, y: 5 },
            Building, // The power plant
        ));

        // Act: Platform is destroyed by a storm
        app.add_event::<PlatformDestroyedEvent>();
        app.world_mut().send_event(PlatformDestroyedEvent { x: 5, y: 5 });
        app.update();

        // Assert: The building at (5, 5) should be despawned (fallen)
        let building_exists = app.world().query::<&Building>().iter(app.world()).next().is_some();
        assert!(!building_exists, "Building on a destroyed platform should fall and be despawned.");
    }

    #[test]
    fn test_platform_separation_breaks_paths() {
        let mut app = App::new();
        // Arrange: Two adjacent platforms with a path between them
        app.insert_resource(TerrainGrid::new(10, 10, TerrainType::Platform));
        let path = vec![(5, 5), (6, 5)];

        // Act: Platform at (5, 5) drifts away or is destroyed
        app.add_event::<PlatformDestroyedEvent>();
        app.world_mut().send_event(PlatformDestroyedEvent { x: 5, y: 5 });
        app.update();

        // Assert: Pathfinding between (5, 5) and (6, 5) fails
        let grid = app.world().resource::<TerrainGrid>();
        assert!(!grid.is_pathable(5, 5, 6, 5), "Path should break when a connecting platform is gone.");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
// The SIMPLEST code that makes tests pass
// In src/layer1/platform.rs

#[derive(Component)]
pub struct Platform {
    pub x: u32,
    pub y: u32,
}

#[derive(Event)]
pub struct PlatformDestroyedEvent {
    pub x: u32,
    pub y: u32,
}

pub fn handle_platform_destruction_system(
    mut commands: Commands,
    mut events: EventReader<PlatformDestroyedEvent>,
    mut grid: ResMut<TerrainGrid>,
    buildings: Query<(Entity, &Platform), With<Building>>,
) {
    for event in events.read() {
        // Destroy the terrain
        grid.set(event.x, event.y, TerrainType::GasCloud);

        // Despawn buildings on that platform
        for (entity, platform) in buildings.iter() {
            if platform.x == event.x && platform.y == event.y {
                commands.entity(entity).despawn_recursive();
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Code Smells:** The minimal implementation only despawns the building entity. It needs to despawn any Pops currently standing on that tile as well.
- **Visual Feedback:** Simply despawning entities is jarring. Add a falling animation or particle effect to show them sinking into the clouds.
- **Architectural Change:** Instead of `Platform` being a component on the `Building`, `Platform` should ideally be a tile property in the `TerrainGrid` that *supports* buildings. The event should target the tile coordinate, and all entities at that coordinate are processed for falling.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage >= 85% for new code
- [ ] Platform destruction correctly updates the `TerrainGrid` to `GasCloud`.
- [ ] All entities (Buildings, Pops, Items) on a destroyed platform are correctly despawned.
- [ ] Pathfinding correctly updates to avoid `GasCloud` tiles.

## 7. Technical Guidance
- **Integration Points:** Connect the `PlatformDestroyedEvent` to the `079` Weather Events system so severe hurricanes can trigger the destruction of weak platforms. Connect to `010` Chronicle System to generate a "Colony Section Lost" event.
- **Gotchas:** Ensure that when a platform falls, resources stored in stockpiles on that platform are actually deducted from global inventory. Do not leave ghost resources that the player can spend but don't physically exist.

## 8. Questions
*Builder: add questions here if spec is unclear.*
