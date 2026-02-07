# 041: Field Science

## Overview

Introduces **Field Science** mechanics where **Anomalies** spawn on the map and can be scanned by pops to gain resources and knowledge.
- **Anomalies**: Special entities (Ruins, Strange Flora, Geodes) scattered on the map.
- **Scanning**: A new action where pops travel to an anomaly and study it.
- **Rewards**: Completing a scan yields Knowledge (for Tech), Food, or Resources.

This encourages exploration and gives utility to the "Explore" action which is currently unused.

## Dependencies

- `002` — Terrain Grid
- `016` — Utility AI (for `ActionType::Explore`)
- `029` — Knowledge System (for Knowledge rewards)

## RED Phase: Tests First

```rust
// src/layer1/science_tests.rs

#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::map::GridPosition;
    use crate::layer1::science::{Anomaly, AnomalyType, ScanProgress, spawn_initial_anomalies};
    use crate::layer1::utility_ai::{ActionType, evaluate_explore};
    use crate::layer1::utility_ai::types::UtilityWeights;
    use crate::layer1::resources::ColonyResources;

    #[test]
    fn test_anomaly_component() {
        let anomaly = Anomaly {
            anomaly_type: AnomalyType::Ruins,
            reward_amount: 10.0,
        };
        assert_eq!(anomaly.anomaly_type, AnomalyType::Ruins);
    }

    #[test]
    fn test_scan_progress_component() {
        let progress = ScanProgress {
            current: 0.0,
            required: 100.0,
        };
        assert!(!progress.is_complete());
    }

    #[test]
    fn test_evaluate_explore_finds_anomaly() {
        let mut world = World::new();
        let pop_pos = GridPosition { x: 5, y: 5 };
        let weights = UtilityWeights::default();

        // Spawn anomaly
        let anomaly = world.spawn((
            Anomaly { anomaly_type: AnomalyType::Geode, reward_amount: 10.0 },
            GridPosition { x: 10, y: 5 }, // Distance 5
            ScanProgress::default(),
        )).id();

        let mut anomalies = world.query::<(Entity, &GridPosition, &Anomaly)>();

        let result = evaluate_explore(&pop_pos, &weights, anomalies.iter(&world));

        assert!(result.is_some());
        let (_, target) = result.unwrap();
        assert_eq!(target, anomaly);
    }

    #[test]
    fn test_evaluate_explore_ignores_scanned() {
        // If we implement a "Scanned" marker, test it here.
        // For now, scanning destroys the entity, so this test might just check if empty query returns None.
        let mut world = World::new();
        let pop_pos = GridPosition { x: 5, y: 5 };
        let weights = UtilityWeights::default();

        let mut anomalies = world.query::<(Entity, &GridPosition, &Anomaly)>();
        let result = evaluate_explore(&pop_pos, &weights, anomalies.iter(&world));

        assert!(result.is_none());
    }

    #[test]
    fn test_spawn_initial_anomalies() {
        let mut world = World::new();
        world.insert_resource(crate::layer1::terrain::TerrainGrid::new(20, 20)); // Mock 20x20
        world.insert_resource(crate::layer1::building::OccupiedTiles::default());

        spawn_initial_anomalies(&mut world, 5); // Spawn 5

        let count = world.query::<&Anomaly>().iter(&world).count();
        assert_eq!(count, 5);
    }
}
```

## GREEN Phase: Minimal Implementation

### 1. Define `Anomaly` and `ScanProgress`

```rust
// src/layer1/science.rs

use bevy_ecs::prelude::*;
use crate::layer1::map::GridPosition;
use crate::layer1::terrain::TerrainGrid;
use crate::layer1::building::OccupiedTiles;
use rand::Rng;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnomalyType {
    Ruins, // Gives Knowledge
    StrangeFlora, // Gives Food
    Geode, // Gives Stone/Ore
}

#[derive(Component)]
pub struct Anomaly {
    pub anomaly_type: AnomalyType,
    pub reward_amount: f32,
}

#[derive(Component)]
pub struct ScanProgress {
    pub current: f32,
    pub required: f32,
}

impl Default for ScanProgress {
    fn default() -> Self {
        Self { current: 0.0, required: 100.0 }
    }
}

impl ScanProgress {
    pub fn is_complete(&self) -> bool {
        self.current >= self.required
    }
}

pub fn spawn_initial_anomalies(world: &mut World, count: usize) {
    let mut rng = rand::thread_rng();
    let (width, height) = {
        let grid = world.resource::<TerrainGrid>();
        (grid.width, grid.height)
    };

    let mut spawned = 0;
    let mut attempts = 0;

    while spawned < count && attempts < count * 10 {
        attempts += 1;
        let x = rng.gen_range(0..width);
        let y = rng.gen_range(0..height);

        // Check occupation (reuse logic from building placement if possible, or manual check)
        let occupied = world.resource::<OccupiedTiles>().0.contains(&(x, y));
        if !occupied {
            let anomaly_type = match rng.gen_range(0..3) {
                0 => AnomalyType::Ruins,
                1 => AnomalyType::StrangeFlora,
                _ => AnomalyType::Geode,
            };

            world.spawn((
                Anomaly {
                    anomaly_type,
                    reward_amount: 20.0
                },
                GridPosition { x, y },
                ScanProgress::default(),
            ));
            spawned += 1;
        }
    }
}
```

### 2. Implement `evaluate_explore` in Utility AI

```rust
// src/layer1/utility_ai.rs

pub fn evaluate_explore<'a>(
    pop_pos: &GridPosition,
    weights: &UtilityWeights,
    anomalies: impl Iterator<Item = (Entity, &'a GridPosition, &'a Anomaly)>,
) -> Option<(f32, Entity)> {
    let mut best: Option<(f32, Entity)> = None;
    let base_utility = 0.55; // Comparable to work

    for (entity, pos, _) in anomalies {
        let context = calculate_context_score(
            *pop_pos,
            Some(*pos),
            1, // Only 1 scanner at a time? Or multiple? Let's say 1 for now.
            0,
            weights,
        );

        // Maybe add bias based on anomaly type?

        let success = calculate_success_modifier(ActionType::Explore, weights);
        let utility = base_utility * context * success;

        if best.is_none_or(|(best_u, _)| utility > best_u) {
            best = Some((utility, entity));
        }
    }
    best
}
```

### 3. Integrate into `evaluate_actions_system`

Add `evaluate_explore` call in the main loop of `evaluate_actions_system`.

### 4. Create `process_scan_system`

Logic similar to `process_refining_system` or `mine_rock`.
- Query pops with `ActionType::Explore`.
- Check if they are at target `Anomaly`.
- Increment `ScanProgress`.
- If complete:
  - Add resources (Knowledge, Food, etc).
  - Despawn `Anomaly`.
  - Log message "Discovery: Found X from Ruins".

## REFACTOR Phase: Quality & Design

- **Visuals**: Anomalies need a distinct character in `map.rs` (e.g., `?` or `*`).
- **Jobs**: Should we restrict this to a "Scientist" job? For MVP, any idle pop can explore.
- **Scanning Speed**: Should depend on Knowledge skill (if added later).
- **Events**: Completion could trigger a pop-up or more complex event.

## Acceptance Criteria

- [ ] `Anomaly` entities spawn on map generation.
- [ ] Pops pick up `ActionType::Explore` to go to anomalies.
- [ ] Pops "work" at the anomaly (wait for duration).
- [ ] Anomaly despawns and grants resources upon completion.
- [ ] Tests pass.

## Technical Guidance

- Use `ActionType::Explore` which is already in the enum.
- Add `process_scan_system` to `simulation.rs`.
- Ensure `spawn_initial_anomalies` is called in `main.rs` startup.
