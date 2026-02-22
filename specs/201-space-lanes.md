# 201: Space Lanes

## Overview

Introduces **Space Lanes** to Layer 2. Repeated travel between two systems strengthens the "Lane" connecting them, increasing travel speed for future fleets. Unused lanes decay over time. This creates emergent "Highways" in space, encouraging players to optimize trade routes and patrol specific corridors.

## Dependencies

- `099` — Fleet Movement (for `Fleet`, `InTransit` components)
- `094` — System View Architecture (for `OrbitalBody`)

## RED Phase: Tests First

Write these tests in `src/layer2/lanes_tests.rs`. They will initially FAIL.

```rust
#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer2::fleet::{Fleet, InTransit};
    use crate::layer2::lanes::{SpaceLaneGraph, update_lane_decay_system, apply_lane_speed_bonus};

    #[test]
    fn test_lane_strengthening() {
        let mut world = World::new();
        let mut graph = SpaceLaneGraph::default();
        let origin = Entity::from_raw(1);
        let dest = Entity::from_raw(2);

        // Simulate a fleet completing a journey
        graph.record_travel(origin, dest);

        assert!(graph.get_strength(origin, dest) > 0.0);
        assert_eq!(graph.get_strength(origin, dest), graph.get_strength(dest, origin)); // Bidirectional
    }

    #[test]
    fn test_lane_decay() {
        let mut world = World::new();
        let mut graph = SpaceLaneGraph::default();
        let origin = Entity::from_raw(1);
        let dest = Entity::from_raw(2);

        graph.record_travel(origin, dest);
        let initial_strength = graph.get_strength(origin, dest);

        // Run decay
        world.insert_resource(graph);
        let mut schedule = Schedule::default();
        schedule.add_systems(update_lane_decay_system);
        schedule.run(&mut world);

        let graph = world.resource::<SpaceLaneGraph>();
        assert!(graph.get_strength(origin, dest) < initial_strength);
    }

    #[test]
    fn test_speed_bonus_application() {
        let mut world = World::new();
        let mut graph = SpaceLaneGraph::default();
        let origin = Entity::from_raw(1);
        let dest = Entity::from_raw(2);

        // Max out lane strength (1.0)
        for _ in 0..100 {
            graph.record_travel(origin, dest);
        }
        world.insert_resource(graph);

        let fleet = world.spawn((
            Fleet,
            InTransit {
                origin,
                destination: dest,
                progress: 0.0,
                duration: 100.0, // Base duration
                base_duration: 100.0, // New field needed in InTransit? Or separate component?
            }
        )).id();

        // Run speed bonus system
        let mut schedule = Schedule::default();
        schedule.add_systems(apply_lane_speed_bonus);
        schedule.run(&mut world);

        let transit = world.get::<InTransit>(fleet).unwrap();
        // Assuming max lane gives 50% speed boost (duration * 0.5)
        assert!(transit.duration < 100.0);
        assert!((transit.duration - 50.0).abs() < 10.0); // Rough check
    }

    #[test]
    fn test_decay_removes_weak_lanes() {
        let mut world = World::new();
        let mut graph = SpaceLaneGraph::default();
        let origin = Entity::from_raw(1);
        let dest = Entity::from_raw(2);

        // Weak lane
        graph.record_travel(origin, dest);
        // Force decay below threshold
        graph.manually_set_strength(origin, dest, 0.01);

        world.insert_resource(graph);
        let mut schedule = Schedule::default();
        schedule.add_systems(update_lane_decay_system);
        schedule.run(&mut world);

        let graph = world.resource::<SpaceLaneGraph>();
        assert_eq!(graph.get_strength(origin, dest), 0.0);
    }
}
```

## GREEN Phase: Minimal Implementation

### 1. Define `SpaceLaneGraph` Resource

```rust
// src/layer2/lanes.rs

use bevy_ecs::prelude::*;
use std::collections::HashMap;

/// Tracks the strength of travel lanes between entities.
/// Key is a sorted tuple of Entities (min, max) to ensure bidirectionality.
#[derive(Resource, Default, Debug, Clone)]
pub struct SpaceLaneGraph {
    lanes: HashMap<(Entity, Entity), f32>,
}

impl SpaceLaneGraph {
    fn key(a: Entity, b: Entity) -> (Entity, Entity) {
        if a < b { (a, b) } else { (b, a) }
    }

    pub fn record_travel(&mut self, a: Entity, b: Entity) {
        let key = Self::key(a, b);
        let strength = self.lanes.entry(key).or_insert(0.0);
        *strength = (*strength + 0.1).min(1.0); // Cap at 1.0
    }

    pub fn get_strength(&self, a: Entity, b: Entity) -> f32 {
        *self.lanes.get(&Self::key(a, b)).unwrap_or(&0.0)
    }

    pub fn manually_set_strength(&mut self, a: Entity, b: Entity, val: f32) {
        self.lanes.insert(Self::key(a, b), val);
    }

    pub fn decay(&mut self, rate: f32) {
        self.lanes.retain(|_, strength| {
            *strength -= rate;
            *strength > 0.0
        });
    }
}
```

### 2. Update `InTransit` Component

Modify `src/layer2/fleet.rs` to include `base_duration`.

```rust
// In src/layer2/fleet.rs (Spec 099 update)
pub struct InTransit {
    // ... existing fields
    pub base_duration: f32, // The duration without modifiers
}
```

### 3. Implement Systems

```rust
// src/layer2/lanes.rs

use crate::layer2::fleet::{Fleet, InTransit};

pub fn update_lane_decay_system(mut graph: ResMut<SpaceLaneGraph>) {
    graph.decay(0.001); // Slow decay per tick
}

pub fn apply_lane_speed_bonus(
    graph: Res<SpaceLaneGraph>,
    mut query: Query<&mut InTransit, (With<Fleet>, Added<InTransit>)>,
) {
    for mut transit in &mut query {
        let strength = graph.get_strength(transit.origin, transit.destination);
        if strength > 0.0 {
            // Formula: Duration = Base * (1.0 - Strength * 0.5)
            // Max strength (1.0) = 50% duration reduction
            transit.duration = transit.base_duration * (1.0 - strength * 0.5);
        }
    }
}

// Hook into fleet arrival to record travel
// This requires modifying `fleet_movement_system` in `fleet.rs` or adding a new system reacting to arrival events.
// For MVP, we'll assume `fleet_movement_system` calls `graph.record_travel`.
```

## REFACTOR Phase: Quality & Design

- **Visualization**: Draw lines between systems on the map. Thickness/Brightness = Strength.
- **Event Integration**: Instead of coupling `fleet_movement_system` to `SpaceLaneGraph`, emit a `FleetArrived` event and handle recording in a separate `lane_recording_system`.
- **Curve**: Use a non-linear curve for strength accumulation (easy to start, hard to master).

## Acceptance Criteria

- [ ] `SpaceLaneGraph` resource exists and tracks connections.
- [ ] Traveling increases lane strength.
- [ ] Lane strength decays over time.
- [ ] Strong lanes reduce travel time for new fleets.
- [ ] Bidirectional support (A->B strengthens B->A).
- [ ] Tests pass.

## Technical Guidance

- Use `Added<InTransit>` filter to apply the speed bonus only once when the trip starts.
- Ensure the `Fleet` entity has the `InTransit` component added *after* the `FleetOrder` is processed, so the change detection works.
