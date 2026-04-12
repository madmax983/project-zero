# Spec 972: Corporate Rivals

## 1. Overview
A hostile takeover. The planet isn't big enough for both of us. A rival faction lands a colony on your map (or region), claims territory, mines resources, and competes for trade contracts. You can sabotage, negotiate, or war with them.

**Fantasy:** A rival corporation lands right next to you, and immediately starts draining the local resources you planned to use.

## 2. Dependencies
- Layer 1 Economy System
- Layer 1 Diplomacy/Factions (`src/layer1/factions/`)
- Layer 1 Grid/Territory (`TerrainGrid`)

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    fn setup_app() -> App {
        let mut app = App::new();
        app.insert_resource(TerritoryGrid::default());
        app.insert_resource(RivalColonyAI::default());
        app.add_systems(Update, (
            rival_colony_expansion_system,
            rival_resource_drain_system,
        ));
        app
    }

    #[test]
    fn test_rival_colony_claims_territory() {
        let mut app = setup_app();

        // Spawn a rival colony center
        let rival_entity = app.world_mut().spawn((
            RivalColony {
                faction_id: "OmniCorp".to_string(),
                expansion_points: 100,
            },
            GridPosition { x: 50, y: 50, z: 0 },
        )).id();

        app.update(); // Trigger expansion

        let territory = app.world().resource::<TerritoryGrid>();
        assert_eq!(territory.get_owner(50, 50), Some(rival_entity), "Rival should claim its starting tile");
        assert_eq!(territory.get_owner(51, 50), Some(rival_entity), "Rival should expand its territory using expansion points");
    }

    #[test]
    fn test_rival_colony_drains_resources_from_claimed_tiles() {
        let mut app = setup_app();

        // Setup a resource on the grid
        app.world_mut().resource_mut::<TerritoryGrid>().set_resource(51, 50, ResourceType::Minerals, 100);

        let rival_entity = app.world_mut().spawn((
            RivalColony {
                faction_id: "OmniCorp".to_string(),
                expansion_points: 100,
            },
            RivalStockpile { minerals: 0 },
            GridPosition { x: 50, y: 50, z: 0 },
        )).id();

        app.update(); // Expands and claims tile (51, 50)
        app.update(); // Drains resources from claimed tile

        let territory = app.world().resource::<TerritoryGrid>();
        assert_eq!(territory.get_resource(51, 50, ResourceType::Minerals), 90, "Rival should drain resources from the grid");

        let stockpile = app.world().get::<RivalStockpile>(rival_entity).unwrap();
        assert_eq!(stockpile.minerals, 10, "Rival should accumulate drained resources in its stockpile");
    }
}
```

## 4. GREEN Phase: Minimal Implementation
- Create `src/layer1/factions/rivals.rs`.
- Define `RivalColony` component (`faction_id: String`, `expansion_points: u32`).
- Define `RivalStockpile` component (tracks rival's stolen/gathered resources).
- Define `TerritoryGrid` resource (if not exists) to track ownership of tiles (`HashMap<(i32, i32), Entity>`).
- Implement `rival_colony_expansion_system`:
  - Query all `RivalColony`. If `expansion_points > 10`, spend 10 points to claim an adjacent, unclaimed `TerritoryGrid` tile.
- Implement `rival_resource_drain_system`:
  - For each `RivalColony`, find all tiles it owns in `TerritoryGrid`.
  - If a tile has gatherable resources, decrement the grid resource and increment the `RivalStockpile`.

## 5. REFACTOR Phase: Quality & Design
- Rival AI should prioritize claiming tiles with high-value resources.
- Ensure the `TerritoryGrid` handles contested borders gracefully (e.g., stopping expansion if the player has built a wall or claimed it first).
- Integrate with Layer 1 UI so the player can clearly see rival borders (red outlines on the map).

## 6. Acceptance Criteria
- [ ] Tests compile and pass in RED phase.
- [ ] Rival colonies can successfully expand their borders over time.
- [ ] Rival colonies drain resources from the tiles they own.
- [ ] Test coverage ≥85%.
- [ ] Code passes `clippy -- -D warnings` and `cargo fmt`.

## 7. Technical Guidance
- `TerritoryGrid` may need to interact closely with the existing `TerrainGrid`. Consider adding an `owner: Option<Entity>` field directly to the `TerrainGrid` array if it's more performant than a separate HashMap.
- Rival expansions should probably emit a `BorderExpansionEvent` so the Chronicle or UI can notify the player when they lose access to a region.

## 8. Questions
*Builder: add questions here if spec is unclear.*
