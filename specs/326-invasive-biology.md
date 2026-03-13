# Spec 326: Invasive Biology

## 1. Overview
This feature introduces "Spores" that can hitch a ride on incoming ships from other biomes, spreading alien flora (like "Xenofungus" or "Terran Kudzu") across the colony grid. This forces the player to manage bio-risks alongside trade.

## 2. Dependencies
- Trade System (Ship arrivals)
- Map/Grid System (Tile replacement)
- Chronicle System (For event logging)

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spore_arrival_on_ship() {
        let mut app = setup_test_app();

        // Spawn a ship arriving
        let ship = app.world.spawn((Ship { origin: BiomeType::Alien },)).id();

        app.update();

        // Assert a Spore event or entity was generated
        let spore_query = app.world.query::<&Spore>().iter(&app.world).count();
        assert!(spore_query > 0, "Ship arrival should occasionally spawn spores.");
    }

    #[test]
    fn test_spore_spreads_and_replaces_terrain() {
        let mut app = setup_test_app();

        // Place a spore on a valid tile
        let tile = (5, 5);
        app.world.spawn(Spore { position: tile, spread_timer: 0.0 });

        app.update(); // Trigger spread

        // Assert adjacent tiles are infected
        let terrain = app.world.get_resource::<TerrainGrid>().unwrap();
        assert_eq!(terrain.get(6, 5), TerrainType::Xenofungus);
    }

    #[test]
    fn test_spore_quarantine_burn() {
        let mut app = setup_test_app();

        // Place a spore and trigger a burn designation
        let tile = (5, 5);
        app.world.spawn(Spore { position: tile, spread_timer: 0.0 });
        app.world.spawn(Designation { type: DesignationType::Burn, position: tile });

        app.update();

        // Assert spore is destroyed and tile is ash
        let spore_count = app.world.query::<&Spore>().iter(&app.world).count();
        assert_eq!(spore_count, 0);
        let terrain = app.world.get_resource::<TerrainGrid>().unwrap();
        assert_eq!(terrain.get(5, 5), TerrainType::Ash);
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
// The simplest logic to spawn a spore, spread it to an adjacent tile, and allow burning.
// Use `ResMut<TerrainGrid>` to mutate tiles when `Spore` timer fires.
// Add a `DesignationType::Burn` to remove spores.
```

## 5. REFACTOR Phase: Quality & Design
- Optimize the grid search for spreading to avoid checking all tiles every frame.
- Add `SporeBloomEvent` and `QuarantineBurnEvent` for the chronicle system.
- Ensure the burn action interacts with the existing job system properly.

## 6. Acceptance Criteria
- [ ] All RED phase tests pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for `invasive_biology.rs`
- [ ] Spores can arrive, spread, and be destroyed by colonists.
- [ ] Events logged to Chronicle via `SPORE_OUTBREAK` and `QUARANTINE_BURN` templates.

## 7. Technical Guidance
- Integrate into the `trade` module where ship arrivals are processed.
- The `Burn` job should have high priority to prevent catastrophic spread.

## 8. Questions
*Builder: Add any questions here.*
*Architect:* Implement the simplest possible version for the MVP. Advanced behaviors and edge cases will be deferred to future specifications.
