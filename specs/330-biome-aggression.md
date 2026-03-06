# Spec 330: Biome Aggression

## 1. Overview
Flora/Fauna tiles have a "Growth/Aggression" rate and actively try to reclaim "Civilized" tiles (buildings/roads). If maintenance is neglected, the wilderness quickly overtakes the colony.

## 2. Dependencies
- Map/Terrain System
- Building System
- Job System (Maintenance)

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_flora_encroaches_on_civilization() {
        let mut app = setup_test_app();

        let terrain = app.world.resource_mut::<TerrainGrid>();
        terrain.set(5, 5, TerrainType::Forest);
        terrain.set(5, 6, TerrainType::Road);

        advance_time(&mut app, 1000.0);

        // Assert forest spread to road
        let terrain = app.world.resource::<TerrainGrid>();
        assert_eq!(terrain.get(5, 6), TerrainType::Forest);
    }

    #[test]
    fn test_maintenance_prevents_encroachment() {
        let mut app = setup_test_app();

        let terrain = app.world.resource_mut::<TerrainGrid>();
        terrain.set(5, 5, TerrainType::Forest);
        let road_entity = app.world.spawn((Building, Position(5, 6), MaintenanceLevel(100.0))).id();

        advance_time(&mut app, 1000.0);

        // Assert road survived due to maintenance
        assert!(app.world.get::<Building>(road_entity).is_some());
    }

    #[test]
    fn test_building_destroyed_by_overgrowth() {
        let mut app = setup_test_app();

        let terrain = app.world.resource_mut::<TerrainGrid>();
        terrain.set(5, 5, TerrainType::Forest);
        // Low maintenance building next to forest
        let building = app.world.spawn((Building, Position(5, 6), MaintenanceLevel(0.0))).id();

        advance_time(&mut app, 1000.0);

        // Assert building is destroyed
        assert!(app.world.get::<Building>(building).is_none());
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
// Run a `biome_encroachment_system` that occasionally checks adjacent tiles to `Forest`/`Jungle`.
// If the target tile is a building with low maintenance, destroy it and turn the tile to `Forest`.
```

## 5. REFACTOR Phase: Quality & Design
- Make encroachment rate dependent on biome type (Jungle > Tundra).
- Fire `BIOME_ENCROACHMENT` chronicle event when a building falls.
- Ensure the "Weeding/Maintenance" job is prioritized when encroachment is near.

## 6. Acceptance Criteria
- [ ] All RED phase tests pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85%
- [ ] Flora spreads to low-maintenance built tiles.
- [ ] Unmaintained buildings are destroyed by overgrowth.

## 7. Technical Guidance
- The grid check should be heavily throttled (e.g., one tile per tick, or a sweep every N ticks) to avoid performance hits.

## 8. Questions
*Builder: Add any questions here.*
