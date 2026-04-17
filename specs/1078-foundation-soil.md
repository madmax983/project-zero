# The Foundation Soil

## 1. Overview
**Layer:** 1
**Fantasy:** The first generation literally gives their bodies to build the colony's bedrock. The city is grown from the dead.
**Mechanic:** Pops that die of natural causes on the colony's original landing tiles increase the soil fertility and structural integrity of those specific tiles permanently.
**Emergence:** Players might build their most crucial infrastructure or richest farms right over the first graveyard, creating a literal "founding heart" to the city.
**Tension:** Do you expand outward for quick space, or slowly build upward on the hyper-efficient, but macabre, bodies of your ancestors?

## 2. Dependencies
- Core ECS `World` setup.
- `Pop` entities with aging/death mechanics.
- Grid/Map tile system (e.g. `TerrainGrid`).
- Some concept of "landing tiles" or colony origin points.

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_foundation_soil_buff_on_natural_death() {
        // Arrange: Setup world with a TerrainGrid and a Pop on a landing tile
        let mut app = App::new();
        // Setup Grid
        // ... grid logic setup ...

        let entity = app.world_mut().spawn((
            Pop { age: 100 }, // old age
            Position { x: 0, y: 0 },
            // landing tile tag or we track original tiles in the grid
        )).id();

        // Act: Trigger natural death
        app.update(); // Pop dies of old age

        // Assert: The tile at (0,0) should have increased fertility/integrity
        // let grid = app.world().resource::<TerrainGrid>();
        // let tile = grid.get(0, 0);
        // assert!(tile.fertility > base_fertility);
        // assert!(tile.structural_integrity > base_integrity);
        // assert!(tile.has_foundation_soil_buff);
    }

    #[test]
    fn test_no_buff_for_unnatural_death() {
        // Arrange: Pop dies of starvation/violence
        // ...
        // Act: Pop dies
        // ...
        // Assert: Tile at death location does not receive buff
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
// Minimal impl: listen to DeathEvent
// If DeathEvent::Natural and location is a "landing tile",
// apply FoundationSoil component/buff to the TerrainGrid at that location.
```

## 5. REFACTOR Phase: Quality & Design
- Consider performance of checking landing tile status. Maybe cache landing tiles in a `HashSet` resource.
- Extract the logic into a dedicated system `apply_foundation_soil_system`.

## 6. Acceptance Criteria
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Natural deaths on landing tiles increase tile fertility/integrity.

## 7. Technical Guidance
- Integrate with existing `PopDied` or similar event.
- Check how the map/grid currently stores tile data to apply the buffs.

## 8. Questions
*Builder: add questions here if spec is unclear.*
