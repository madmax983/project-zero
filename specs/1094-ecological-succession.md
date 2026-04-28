# Spec 1094: Ecological Succession

## 1. Overview
Flora on the colony map undergoes ecological succession. It has distinct growth stages (e.g., Sapling -> Tree). Destroying a mature biome resets the cycle, allowing fast-growing, less valuable pioneer species (like "Fire-Weed") to take over before the climax community (like "Ironwood") can regrow.

**Fantasy:** A forest isn't just trees; it's a slow-motion explosion of life.

## 2. Dependencies
- Layer 1 Terrain & Grid (`TerrainGrid`, `GridPosition`)
- Layer 1 Flora/Entities (`Flora`, `GrowthStage`)

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use crate::layer1::flora::{Flora, FloraType, GrowthStage, EcologicalState};
    use crate::layer1::map::GridPosition;

    fn setup_app() -> App {
        let mut app = App::new();
        app.add_systems(Update, (
            flora_growth_system,
            ecological_succession_system,
        ));
        app
    }

    #[test]
    fn test_flora_progresses_through_stages() {
        let mut app = setup_app();

        let plant = app.world_mut().spawn((
            Flora { f_type: FloraType::Ironwood },
            GrowthStage { current: 0, max: 3, progress: 99.0 },
        )).id();

        // Increment time/progress
        app.world_mut().get_mut::<GrowthStage>(plant).unwrap().progress += 2.0;
        app.update();

        let stage = app.world().get::<GrowthStage>(plant).unwrap();
        assert_eq!(stage.current, 1, "Flora should advance to the next growth stage");
        assert!(stage.progress < 100.0, "Progress should reset after advancing stage");
    }

    #[test]
    fn test_cleared_climax_biome_spawns_pioneer_species() {
        let mut app = setup_app();

        // A tile that recently had its Ironwood cut down
        let tile_pos = GridPosition { x: 5, y: 5, z: 0 };
        app.world_mut().spawn((
            EcologicalState { cleared_recently: true, climax_type: FloraType::Ironwood },
            tile_pos.clone(),
        ));

        app.update();

        // Check if Pioneer species spawned on that tile
        let mut pioneer_found = false;
        for (flora, pos) in app.world().query::<(&Flora, &GridPosition)>().iter(app.world()) {
            if *pos == tile_pos && flora.f_type == FloraType::FireWeed {
                pioneer_found = true;
                break;
            }
        }
        assert!(pioneer_found, "Pioneer species (FireWeed) should spawn where climax species was cleared");
    }
}
```

## 4. GREEN Phase: Minimal Implementation
- Create `src/layer1/ecology.rs` (or extend existing flora modules).
- Define `GrowthStage { current: u8, max: u8, progress: f32 }`.
- Add `FloraType::FireWeed` (pioneer) and `FloraType::Ironwood` (climax) to `FloraType`.
- Define `EcologicalState` component for grid tiles.
- Implement `flora_growth_system`:
  - Increment `progress`. If `progress >= 100.0`, increment `current` and reset `progress`.
- Implement `ecological_succession_system`:
  - Monitor tiles where climax flora was destroyed (e.g., via a `FloraDestroyedEvent`).
  - Mark those tiles with `EcologicalState { cleared_recently: true }`.
  - Spawn `FireWeed` saplings on recently cleared tiles.
  - Over a long period, allow `FireWeed` to die off and be replaced by `Ironwood` saplings.

## 5. REFACTOR Phase: Quality & Design
- **Shade/Canopy Mechanic:** Climax species should ideally only grow in the "shade" or altered soil of the pioneer species, creating a true succession chain.
- **Resource Yields:** Pioneer species should yield low-value resources (or be highly flammable hazards), while climax species yield valuable building materials.

## 6. Acceptance Criteria
- [ ] RED phase tests pass.
- [ ] `cargo test` returns 0 failures.
- [ ] Flora grows through distinct stages.
- [ ] Clearing a forest results in pioneer weeds before trees return.
- [ ] Test coverage >= 85%.

## 7. Technical Guidance
- Tie `progress` increments to the global `SimulationTime` or seasonal tick to control the pacing of growth.
- Ensure harvesting a plant resets the tile's ecological state correctly.

## 8. Questions
*Builder: Add questions here if the mapping between Flora entities and Grid tiles needs clarification.*

- *Architect: The mapping should be 1-to-1 for simplicity in the MVP. Each grid tile can contain at most one Flora entity. Use the terrain grid system to find adjacent tiles.*
