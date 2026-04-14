# 1005: Subterranean Biosphere Inversion

## 1. Overview
On worlds with utterly inhospitable surfaces, players must dig deep. The deep crust contains its own hyper-aggressive, closed ecosystem. Digging deeper yields incredible exotic resources, but physically unleashes more dangerous flora and fauna into mining shafts, forcing a constant war of attrition against the planet's immune system. Players must weigh the irresistible lure of deep-crust wealth against the absolute certainty of unearthing biological horrors.

## 2. Dependencies
- Layer 1 `TerrainGrid` (Z-level or depth tracking).
- Layer 1 `Mining` task system.
- Layer 1 `Flora`/`Fauna` entity spawning systems.
- Layer 1 `Combat`/`Hazard` mechanics for Pops.

## 3. RED Phase: Tests First
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use crate::layer1::mining::{MineEvent, MiningSystem};
    use crate::layer1::terrain::GridPosition;
    use crate::layer1::hazards::HazardFlora;

    #[test]
    fn test_deep_mining_spawns_hostile_biosphere() {
        let mut app = App::new();
        app.add_event::<MineEvent>();
        app.add_systems(Update, deep_crust_breach_system);

        // Simulate mining a very deep tile (z = -50)
        app.world_mut().resource_mut::<Events<MineEvent>>().send(MineEvent {
            position: GridPosition { x: 10, y: 10, z: -50 },
            miner: Entity::from_raw(1),
        });

        app.update();

        // Verify that a hostile flora/fauna was spawned at the location
        let mut hazard_query = app.world_mut().query::<(&GridPosition, &HazardFlora)>();
        let mut found_hazard = false;
        for (pos, _hazard) in hazard_query.iter(app.world()) {
            if pos.x == 10 && pos.y == 10 && pos.z == -50 {
                found_hazard = true;
                break;
            }
        }

        assert!(found_hazard, "Mining at depth should spawn hostile flora/fauna.");
    }

    #[test]
    fn test_shallow_mining_does_not_spawn_hostile_biosphere() {
        let mut app = App::new();
        app.add_event::<MineEvent>();
        app.add_systems(Update, deep_crust_breach_system);

        // Simulate mining a shallow tile (z = -2)
        app.world_mut().resource_mut::<Events<MineEvent>>().send(MineEvent {
            position: GridPosition { x: 10, y: 10, z: -2 },
            miner: Entity::from_raw(1),
        });

        app.update();

        // Verify no hazards spawned
        let mut hazard_query = app.world_mut().query::<&HazardFlora>();
        assert_eq!(hazard_query.iter(app.world()).count(), 0, "Shallow mining should not trigger deep crust breaches.");
    }
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
// src/layer1/biosphere_inversion.rs
use bevy::prelude::*;
use crate::layer1::mining::MineEvent;
use crate::layer1::terrain::GridPosition;
use crate::layer1::hazards::HazardFlora;

// A threshold below which the crust becomes hostile
const DEEP_CRUST_THRESHOLD: i32 = -30;

pub fn deep_crust_breach_system(
    mut commands: Commands,
    mut mine_events: EventReader<MineEvent>,
) {
    for event in mine_events.read() {
        if event.position.z <= DEEP_CRUST_THRESHOLD {
            // Spawn a basic hazard at the breached location
            commands.spawn((
                event.position.clone(),
                HazardFlora {
                    lethality: 10.0,
                    spread_rate: 2.0
                },
            ));
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Configurable Depths:** `DEEP_CRUST_THRESHOLD` should be tied to the specific planet's generation config, not a hardcoded constant.
- **RNG/Probability:** Currently, *every* mined tile below the threshold spawns a hazard. This should be probabilistic (e.g., 5% chance per tile mined, scaling with depth).
- **Spawn Variations:** Expand to spawn different types of horrors (acid spitters, fungal blooms) based on depth and biome.
- **Sealing Bulkheads:** Need an integration with building mechanics so players can construct bulkheads to block the `spread_rate` of `HazardFlora`.

## 6. Acceptance Criteria (Testable!)
- [ ] Test `test_deep_mining_spawns_hostile_biosphere` passes.
- [ ] Test `test_shallow_mining_does_not_spawn_hostile_biosphere` passes.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for the new module.

## 7. Technical Guidance
- Place the `deep_crust_breach_system` after the mining execution system but before hazard logic processing in the scheduling.
- Ensure that `HazardFlora` ties into the `TerrainGrid` properly so it can obstruct pathfinding or damage Pops occupying the same tile.

## 8. Questions
*Builder: add questions here if spec is unclear.*
