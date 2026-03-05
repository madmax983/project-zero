# Specification: The Chrono-Stutter (Layer 1)

## 1. Overview
A localized "Time Anomaly" slowly moves across the map. Tiles inside the anomaly run at an accelerated (or decelerated) speed. This affects Pop movement, aging, crop growth, and machine wear. The tension arises from exploiting the time acceleration for rapid production versus the risk of catastrophic rapid aging and immediate machine degradation.

## 2. Dependencies
- `065` Day/Night Cycle (for tick rate tracking).
- `062` Pop Lifecycle (Aging).
- `032` Entropy/Spoilage (Machine/Item degradation).

## 3. RED Phase: Tests First

```rust
// src/layer1/chrono_stutter_tests.rs

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::prelude::*;
    use crate::layer1::grid::{Transform, TilePos};
    use crate::layer1::pop::Pop;

    fn setup_world() -> World {
        let mut world = World::new();
        // Base setup...
        world
    }

    #[test]
    fn test_chrono_anomaly_applies_time_modifier_to_entities() {
        let mut world = setup_world();

        // Spawn Anomaly at origin
        world.spawn((
            ChronoAnomaly { radius: 5.0, time_multiplier: 10.0 },
            Transform::from_xyz(0.0, 0.0, 0.0),
        ));

        // Spawn Pop inside anomaly
        let pop_in = world.spawn((
            Pop,
            Transform::from_xyz(2.0, 0.0, 0.0),
            TimeModifier::default(),
        )).id();

        // Spawn Pop outside anomaly
        let pop_out = world.spawn((
            Pop,
            Transform::from_xyz(10.0, 0.0, 0.0),
            TimeModifier::default(),
        )).id();

        world.run_system_once(apply_chrono_anomaly_system);

        assert_eq!(world.get::<TimeModifier>(pop_in).unwrap().multiplier, 10.0);
        assert_eq!(world.get::<TimeModifier>(pop_out).unwrap().multiplier, 1.0);
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
// src/layer1/environment/chrono_stutter.rs

use bevy_ecs::prelude::*;
use crate::layer1::grid::Transform;

#[derive(Component)]
pub struct ChronoAnomaly {
    pub radius: f32,
    pub time_multiplier: f32,
}

#[derive(Component)]
pub struct TimeModifier {
    pub multiplier: f32,
}

impl Default for TimeModifier {
    fn default() -> Self {
        Self { multiplier: 1.0 }
    }
}

pub fn apply_chrono_anomaly_system(
    anomaly_query: Query<(&ChronoAnomaly, &Transform)>,
    mut entity_query: Query<(&mut TimeModifier, &Transform)>,
) {
    // Reset all modifiers first
    for (mut modifier, _) in entity_query.iter_mut() {
        modifier.multiplier = 1.0;
    }

    // Apply highest anomaly modifier
    for (anomaly, anomaly_tf) in anomaly_query.iter() {
        for (mut modifier, tf) in entity_query.iter_mut() {
            let dist = (tf.translation.x - anomaly_tf.translation.x).abs()
                     + (tf.translation.y - anomaly_tf.translation.y).abs();

            if dist <= anomaly.radius {
                // In a real implementation we might stack or take max
                modifier.multiplier = anomaly.time_multiplier;
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design

- **Integration:** The `TimeModifier` component needs to be factored into existing systems. For example, `update_age_system` should multiply aging by `TimeModifier.multiplier`. Same for crop growth, machine wear, and movement speed.
- **Anomaly Movement:** Implement a system `move_chrono_anomaly_system` that slowly drifts the anomaly across the map.
- **Visuals:** Add a visual shader or particle effect to clearly delineate the boundaries of the time anomaly.

## 6. Acceptance Criteria

- [ ] All tests pass.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage >= 85%.
- [ ] Entities within the `ChronoAnomaly` radius receive an updated `TimeModifier`.
- [ ] Entities outside the radius have a modifier of `1.0`.

## 7. Technical Guidance
- **Performance:** Iterating all entities vs all anomalies could be $O(N \times M)$. If $M$ (anomalies) is 1-2, it's fine. If more, consider spatial partitioning or a grid-based marker approach where the grid tile holds the modifier.
- **Global Tick:** Ensure global ticks aren't bypassed. The component modifies the *delta* applied during a tick, not the tick itself.

## 8. Questions
*Builder: add questions here if spec is unclear.*
*Architect: I will answer your questions as they come up.*
