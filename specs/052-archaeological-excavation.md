# 052: Archaeological Excavation

## Overview

Mining is no longer just about resources; it's about history. When mining natural `Rock`, there is a small chance to uncover a `BuriedAnomaly`. This spawns an `Anomaly` entity (from Spec 041) at the mined location, which can then be studied by scientists to yield Knowledge or special resources.

This feature bridges the gap between the **Mining** (018) and **Field Science** (041) systems, adding a layer of exploration to the industrial expansion loop.

## Dependencies

- `018` — Mining & Resources (for `mine_rock` logic)
- `041` — Field Science (for `Anomaly` entity and types)

## RED Phase: Tests First

Write these tests BEFORE any implementation. They will initially FAIL.

```rust
// src/layer1/mining_tests.rs (add to existing or create new)

#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::resources::{mine_rock, ColonyResources, MiningProgress, ResourceItem, ResourceType};
    use crate::layer1::science::{Anomaly, AnomalyType};
    use crate::layer1::terrain::{TerrainGrid, TerrainType};
    use crate::layer1::map::GridPosition;
    use crate::layer1::designation::{Designation, DesignationType};

    #[test]
    fn test_mine_rock_spawns_anomaly_probabilistically() {
        let mut world = World::new();
        // Setup grid
        world.insert_resource(TerrainGrid { width: 100, height: 100, tiles: vec![TerrainType::Rock; 10000] });
        world.insert_resource(ColonyResources::default());
        world.insert_resource(crate::shared::log::MessageLog::default());

        // We need to run mine_rock many times to trigger the probability (e.g. 5%)
        // Or refactor mine_rock to accept a seed/config.
        // For this test, we run 100 times. Probability of NO anomaly in 100 tries at 5% is ~0.6%.
        // This is acceptable for a game system test.

        let mut anomaly_spawned = false;

        for i in 0..100 {
            let entity = world.spawn((
                Designation { designation_type: DesignationType::Mine },
                MiningProgress { current: 9.0, max: 10.0 },
                GridPosition { x: i % 100, y: 0 },
            )).id();

            // Complete mining
            mine_rock(&mut world, entity, 1.0);

            // Check if Anomaly exists at this position
            let anomalies = world.query::<(&Anomaly, &GridPosition)>().iter(&world).count();
            if anomalies > 0 {
                anomaly_spawned = true;
                break;
            }
        }

        assert!(anomaly_spawned, "Should have spawned at least one anomaly in 100 mining attempts");
    }

    #[test]
    fn test_spawned_anomaly_has_valid_type() {
        // Setup world to FORCE spawn if possible, or check the one that spawned.
        // If we can't force it, we rely on the loop above or a mock.
        // Assuming we rely on the loop:

        let mut world = World::new();
        world.insert_resource(TerrainGrid { width: 100, height: 100, tiles: vec![TerrainType::Rock; 10000] });
        world.insert_resource(ColonyResources::default());
        world.insert_resource(crate::shared::log::MessageLog::default());

        // Mine until anomaly
        let mut anomaly_entity = None;
        for i in 0..100 {
            let entity = world.spawn((
                Designation { designation_type: DesignationType::Mine },
                MiningProgress { current: 10.0, max: 10.0 }, // Instant complete
                GridPosition { x: i, y: 0 },
            )).id();

            mine_rock(&mut world, entity, 10.0);

            if let Some((e, _)) = world.query::<(Entity, &Anomaly)>().iter(&world).next() {
                anomaly_entity = Some(e);
                break;
            }
        }

        if let Some(e) = anomaly_entity {
            let anomaly = world.get::<Anomaly>(e).unwrap();
            // Should be a valid type (Ruins, Geode, etc)
            assert!(matches!(anomaly.anomaly_type, AnomalyType::Ruins | AnomalyType::Geode | AnomalyType::StrangeFlora));
        } else {
            panic!("Failed to spawn anomaly for type check");
        }
    }
}
```

## GREEN Phase: Minimal Implementation

### 1. Update `mine_rock` in `src/layer1/resources.rs`

Inject the probability logic upon completion.

```rust
// src/layer1/resources.rs

use crate::layer1::science::{Anomaly, AnomalyType, ScanProgress}; // Import from Spec 041

// ... inside mine_rock function, inside the `if completed` block ...

// 1. Existing logic (Change Terrain, Spawn Stone, Log)
// ...

// 2. New Logic: Anomaly Spawn Chance
let mut rng = rand::thread_rng();
if rng.gen_bool(0.05) { // 5% chance
    // Determine type
    let anomaly_type = match rng.gen_range(0..3) {
        0 => AnomalyType::Ruins,
        1 => AnomalyType::Geode,
        _ => AnomalyType::StrangeFlora, // Maybe fossilized flora?
    };

    // Spawn Anomaly entity
    world.spawn((
        Anomaly {
            anomaly_type,
            reward_amount: 50.0, // High reward for excavated items
        },
        ScanProgress::default(),
        pos, // Same position as the mined rock
    ));

    if let Some(mut log) = world.get_resource_mut::<MessageLog>() {
        log.add("Discovery: Unearthed an Anomaly!");
    }
}
```

## REFACTOR Phase: Quality & Design

- **Loot Tables**: Move the probability and type selection to a separate `LootTable` config or function (`roll_excavation_reward`) to avoid hardcoding in `mine_rock`.
- **RNG Injection**: For deterministic testing, pass an RNG source into `mine_rock` (e.g., via a `RandomSource` resource or parameter).
- **Buried State**: Consider adding a `Buried` component to `Rock` entities if we want predetermined anomalies (seeds) rather than purely random rolls. This allows for "treasure maps" later.
- **Notification**: Trigger a specific notification (Spec 046) with `Severity::Major` for discoveries.

## Acceptance Criteria

- [ ] Mining rock has a 5% chance to spawn an `Anomaly`.
- [ ] Spawned anomalies have `ScanProgress` and can be interacted with via Spec 041 logic.
- [ ] Log message appears on discovery.
- [ ] Tests pass (probabilistic tests are acceptable for Green phase, determinism preferred in Refactor).

## Technical Guidance

- Reuse `Anomaly` and `AnomalyType` from `crate::layer1::science`.
- Ensure `mine_rock` has access to `Anomaly` struct (public visibility).
- Do not spawn anomaly on top of `ResourceItem` if it causes visual clutter (Z-order handles it, but check logic).
