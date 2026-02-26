# 238: The Hum

## Overview

"The Hum" is a mysterious, low-frequency vibration that pervades the colony. To most, it is just background noise or entirely inaudible. To those with the `Sensitive` trait, it is a voice, a song, or a maddening drill in the skull.

This feature introduces a new layer of psychological horror and emergent behavior. `Sensitive` pops can hear "Hum Sources" (e.g., Ancient Artifacts, improperly shielded Generators, or specific geological features). When the Hum is loud, they feel an urge to seek it out, often abandoning their duties to "listen" at the source.

**Mechanics:**
- **HumMap**: A global grid tracking "Resonance" levels (0.0 - 1.0).
- **HumSource**: Component for entities that emit Resonance.
- **Sensitive Trait**: Pops with this trait perceive the Hum.
- **ListenToTheHum Action**: A new Utility AI action where `Sensitive` pops stand near a Hum Source.
    - **Effect**: Restores `Leisure` (they are "socializing" with the void) but increases `Stress`.
    - **Emergence**: A cult of "Listeners" forms around the reactor or a strange rock.

## Dependencies

- `001` Architecture Setup (Map/Grid).
- `016` Utility AI System (Action evaluation).
- `084` Pop Traits (Adding `Sensitive`).
- `060` Acoustic Simulation (Inspiration for the grid implementation).

## RED Phase: Tests First

Write these tests BEFORE any implementation. They will initially FAIL.

```rust
// src/layer1/hum_tests.rs

#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::map::{GridPosition, TerrainGrid};
    use crate::layer1::hum::{HumMap, HumSource, update_hum_system};
    use crate::layer1::pop::{Pop, Needs};
    use crate::layer1::traits::{Trait, Traits};
    use crate::layer1::utility_ai::{ActionType, evaluate_single_pop, UtilityAIBuffer, PopEvalData, WorldContext};
    use crate::layer1::utility_types::UtilityWeights;

    #[test]
    fn test_hum_map_propagation() {
        let mut world = World::new();
        world.insert_resource(HumMap::new(10, 10));
        world.insert_resource(TerrainGrid::new(10, 10));

        // Spawn a Hum Source at (5, 5)
        world.spawn((
            HumSource { radius: 3.0, intensity: 1.0 },
            GridPosition { x: 5, y: 5 },
        ));

        // Run propagation
        update_hum_system(&mut world);

        let map = world.resource::<HumMap>();

        // Center should be max intensity
        assert!((map.get(5, 5) - 1.0).abs() < f32::EPSILON);

        // Falloff
        assert!(map.get(6, 5) < 1.0);
        assert!(map.get(6, 5) > 0.0);

        // Out of range
        assert_eq!(map.get(9, 9), 0.0);
    }

    #[test]
    fn test_sensitive_trait_exists() {
        // Just verify the enum variant exists and compiles
        let sensitive = Trait::Sensitive;
        assert_eq!(sensitive.label(), "Sensitive");
    }

    #[test]
    fn test_evaluate_listen_to_hum_sensitive_pop() {
        // Setup AI context
        let mut buffer = UtilityAIBuffer::default();
        let hum_pos = GridPosition { x: 5, y: 5 };

        // Add a Hum Source to the buffer (assuming we add this field)
        // For the test, we can mock the evaluator's input directly if needed,
        // but ideally we test the integration.
        // Let's assume we pass the HumMap in the WorldContext.

        let pop_pos = GridPosition { x: 0, y: 0 };
        let mut traits = Traits::default();
        traits.add(Trait::Sensitive);

        let data = PopEvalData {
            entity: Entity::from_raw(0),
            pos: pop_pos,
            needs: Needs::default(), // Not stressed yet
            traits: Some(&traits),
            weights: UtilityWeights::default(),
            ..Default::default()
        };

        // Context with HumMap
        let mut map = HumMap::new(10, 10);
        map.set(5, 5, 1.0); // Loud hum at target

        // We need a way to pass Hum info to the evaluator.
        // Option A: Add HumMap to WorldContext.
        // Option B: Collect high-resonance tiles into UtilityAIBuffer.
        // Let's assume Option B: buffer.hum_sources containing (Entity, GridPosition, Intensity)

        buffer.hum_sources.push((Entity::from_raw(1), hum_pos, 1.0));

        // This function must be implemented in the Green phase
        let (action, score, target) = crate::layer1::actions::hum::evaluate_listen_to_hum(&data, &buffer);

        assert_eq!(action, ActionType::ListenToTheHum);
        assert!(score > 0.0);
        assert_eq!(target, Some(Entity::from_raw(1)));
    }

    #[test]
    fn test_evaluate_listen_to_hum_normal_pop_ignores_it() {
        let mut buffer = UtilityAIBuffer::default();
        let hum_pos = GridPosition { x: 5, y: 5 };
        buffer.hum_sources.push((Entity::from_raw(1), hum_pos, 1.0));

        let data = PopEvalData {
            pos: GridPosition { x: 0, y: 0 },
            traits: Some(&Traits::default()), // Normal pop
            ..Default::default()
        };

        let (action, score, _) = crate::layer1::actions::hum::evaluate_listen_to_hum(&data, &buffer);

        // Should return Idle/None or very low score
        if action == ActionType::ListenToTheHum {
            assert_eq!(score, 0.0);
        }
    }
}
```

## GREEN Phase: Minimal Implementation

### 1. Define Components and Resources

```rust
// src/layer1/hum.rs

use bevy_ecs::prelude::*;

#[derive(Resource)]
pub struct HumMap {
    pub width: usize,
    pub height: usize,
    pub values: Vec<f32>,
}

impl HumMap {
    pub fn new(width: usize, height: usize) -> Self {
        Self { width, height, values: vec![0.0; width * height] }
    }
    // get/set implementation...
}

#[derive(Component)]
pub struct HumSource {
    pub radius: f32,
    pub intensity: f32,
}
```

### 2. Update `Trait` Enum

```rust
// src/layer1/traits.rs
pub enum Trait {
    // ... existing ...
    Sensitive,
}
```

### 3. Update `ActionType` Enum

```rust
// src/layer1/utility_types.rs
pub enum ActionType {
    // ... existing ...
    ListenToTheHum,
}
```

### 4. Implement Evaluator

```rust
// src/layer1/actions/hum.rs

use crate::layer1::utility_types::ActionType;
use crate::layer1::utility_eval_types::{PopEvalData, UtilityAIBuffer};
use crate::layer1::traits::Trait;
use crate::layer1::utility_types::calculate_context_score;

pub fn evaluate_listen_to_hum(
    data: &PopEvalData,
    buffer: &UtilityAIBuffer,
) -> (ActionType, f32, Option<Entity>) {
    // 1. Check Trait
    if !data.traits.as_ref().map_or(false, |t| t.has(Trait::Sensitive)) {
        return (ActionType::ListenToTheHum, 0.0, None);
    }

    // 2. Find best source
    let mut best_score = 0.0;
    let mut best_target = None;

    for (entity, pos, intensity) in &buffer.hum_sources {
        // Base score driven by intensity and leisure need (they "enjoy" it in a weird way)
        // High stress also increases attraction ("The Hum soothes me")
        let desire = intensity * (1.0 - data.needs.leisure) + (data.stress * 0.5);

        let context_score = calculate_context_score(data.pos, Some(*pos), 10, 0, &data.weights);
        let total_score = desire * context_score;

        if total_score > best_score {
            best_score = total_score;
            best_target = Some(*entity);
        }
    }

    (ActionType::ListenToTheHum, best_score, best_target)
}
```

### 5. Hook into System

- Add `hum_sources` to `UtilityAIBuffer`.
- Populate it in `populate_ai_buffer`.
- Call `evaluate_listen_to_hum` in `evaluate_single_pop`.

## REFACTOR Phase: Quality & Design

- **Optimization**: `HumMap` should update infrequently (e.g., every 10 ticks) or only when sources change, as it is a global "atmosphere" rather than a fast-moving agent.
- **Integration**:
    - Add `HumSource` to specific buildings (e.g., `Generator` with `Malfunction` status).
    - Create `HumEvent` for UI notifications when a pop starts listening.
- **Visuals**: Add a subtle screen shake or audio cue when viewing a high-hum area (if `Sensitive` trait was on the player... but the player is the colony manager).
- **Expansion**:
    - `The Hum` could be a precursor to `232 Protest Crowds` (mobs forming).
    - High-level "Listeners" might start building "Resonant Structures" (`110 Spontaneous Architecture`).

## Acceptance Criteria

- [ ] `HumMap` exists and propagates values.
- [ ] `Trait::Sensitive` is added.
- [ ] `ActionType::ListenToTheHum` is added.
- [ ] Sensitive pops are attracted to Hum Sources via Utility AI.
- [ ] Normal pops ignore Hum Sources.
- [ ] Tests pass.
