# 352: Stellar Drift

## 1. Overview

The galaxy is in motion. Maps are not static.

**Stellar Drift** introduces a mechanic where star systems drift relative to each other over long timescales. This causes hyperlane connections between systems to break and new ones to form. Distances change, altering trade efficiency, travel times, and potentially changing who your strategic neighbors are. A safe backwater colony protected by distance could suddenly drift within jump-range of a hostile faction.

## 2. Dependencies

- `094` System View Architecture (for drawing/managing nodes and edges)
- `095` System Generation (for the initial galaxy map state)

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::prelude::*;
    use crate::layer3::galaxy_map::{StarSystem, Hyperlane, GalaxyMap};
    use crate::layer1::time::SimulationTime;

    #[test]
    fn test_systems_drift_over_time() {
        let mut world = World::new();
        let mut schedule = Schedule::default();
        schedule.add_systems(stellar_drift_system);

        world.insert_resource(SimulationTime { tick: 0, ..Default::default() });

        let sys1 = world.spawn(StarSystem { x: 10.0, y: 10.0, drift_vx: 0.1, drift_vy: -0.1 }).id();
        let sys2 = world.spawn(StarSystem { x: 20.0, y: 20.0, drift_vx: -0.2, drift_vy: 0.0 }).id();

        // Run simulation for 100 ticks
        world.resource_mut::<SimulationTime>().tick = 100;
        schedule.run(&mut world);

        let s1 = world.get::<StarSystem>(sys1).unwrap();
        let s2 = world.get::<StarSystem>(sys2).unwrap();

        assert_eq!(s1.x, 20.0); // 10 + (0.1 * 100)
        assert_eq!(s1.y, 0.0);  // 10 + (-0.1 * 100)
        assert_eq!(s2.x, 0.0);  // 20 + (-0.2 * 100)
        assert_eq!(s2.y, 20.0); // 20 + (0.0 * 100)
    }

    #[test]
    fn test_hyperlanes_snap_when_distance_exceeds_max() {
        let mut world = World::new();
        let mut schedule = Schedule::default();
        schedule.add_systems(hyperlane_maintenance_system);

        let sys1 = world.spawn(StarSystem { x: 0.0, y: 0.0, drift_vx: -1.0, drift_vy: 0.0 }).id();
        let sys2 = world.spawn(StarSystem { x: 10.0, y: 0.0, drift_vx: 1.0, drift_vy: 0.0 }).id();

        let lane = world.spawn(Hyperlane { source: sys1, target: sys2 }).id();

        // Move them far apart manually to simulate drift
        world.get_mut::<StarSystem>(sys1).unwrap().x = -50.0;
        world.get_mut::<StarSystem>(sys2).unwrap().x = 50.0; // Distance = 100

        schedule.run(&mut world);

        // Assuming max distance is 50.0, the lane should be snapped (despawned)
        assert!(world.get_entity(lane).is_err() || world.get_entity(lane).unwrap().is_despawned());
    }

    #[test]
    fn test_new_hyperlanes_form_when_systems_drift_close() {
        let mut world = World::new();
        let mut schedule = Schedule::default();
        schedule.add_systems(hyperlane_formation_system);

        let sys1 = world.spawn(StarSystem { x: 0.0, y: 0.0, drift_vx: 0.0, drift_vy: 0.0 }).id();
        let sys2 = world.spawn(StarSystem { x: 10.0, y: 0.0, drift_vx: 0.0, drift_vy: 0.0 }).id();

        // Initially no lanes
        let lane_count = world.query::<&Hyperlane>().iter(&world).count();
        assert_eq!(lane_count, 0);

        schedule.run(&mut world);

        // They are close enough, a lane should form
        let lane_count = world.query::<&Hyperlane>().iter(&world).count();
        assert_eq!(lane_count, 1);
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy_ecs::prelude::*;
use crate::layer1::time::SimulationTime;

// These would likely exist in `src/layer3/galaxy_map.rs`
#[derive(Component, Debug, Clone)]
pub struct StarSystem {
    pub x: f32,
    pub y: f32,
    pub drift_vx: f32,
    pub drift_vy: f32,
}

#[derive(Component)]
pub struct Hyperlane {
    pub source: Entity,
    pub target: Entity,
}

const MAX_HYPERLANE_DISTANCE: f32 = 50.0;

pub fn stellar_drift_system(
    time: Res<SimulationTime>,
    mut query: Query<&mut StarSystem>,
) {
    // Only update periodically to save performance, e.g., once every 100 ticks
    if time.tick % 100 != 0 {
        return;
    }

    for mut sys in query.iter_mut() {
        sys.x += sys.drift_vx * 100.0; // scale by tick interval
        sys.y += sys.drift_vy * 100.0;
    }
}

pub fn hyperlane_maintenance_system(
    mut commands: Commands,
    sys_query: Query<&StarSystem>,
    lane_query: Query<(Entity, &Hyperlane)>,
) {
    for (lane_entity, lane) in lane_query.iter() {
        if let (Ok(s1), Ok(s2)) = (sys_query.get(lane.source), sys_query.get(lane.target)) {
            let dist_sq = (s1.x - s2.x).powi(2) + (s1.y - s2.y).powi(2);
            if dist_sq > MAX_HYPERLANE_DISTANCE.powi(2) {
                commands.entity(lane_entity).despawn();
                // Alternatively, trigger a HyperlaneCollapsedEvent
            }
        }
    }
}

pub fn hyperlane_formation_system(
    mut commands: Commands,
    sys_query: Query<(Entity, &StarSystem)>,
    lane_query: Query<&Hyperlane>,
) {
    // Note: O(N^2) comparison. See Refactor section for optimization.
    let mut existing_lanes = std::collections::HashSet::new();
    for lane in lane_query.iter() {
        let min_ent = lane.source.min(lane.target);
        let max_ent = lane.source.max(lane.target);
        existing_lanes.insert((min_ent, max_ent));
    }

    let systems: Vec<(Entity, &StarSystem)> = sys_query.iter().collect();

    for i in 0..systems.len() {
        for j in (i+1)..systems.len() {
            let (e1, s1) = systems[i];
            let (e2, s2) = systems[j];

            let dist_sq = (s1.x - s2.x).powi(2) + (s1.y - s2.y).powi(2);
            if dist_sq <= MAX_HYPERLANE_DISTANCE.powi(2) {
                let min_ent = e1.min(e2);
                let max_ent = e1.max(e2);

                if !existing_lanes.contains(&(min_ent, max_ent)) {
                    commands.spawn(Hyperlane { source: min_ent, target: max_ent });
                }
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design

- **Performance:** `hyperlane_formation_system` is $O(N^2)$, which is terrible if the galaxy has 1,000 stars. Since we only run this occasionally (e.g., every 100 or 1000 ticks), it might be acceptable for MVP. For production, consider using a spatial partition (like a quadtree or simple grid-based bucketing) to only check neighboring stars.
- **Events:** Add `HyperlaneCollapsedEvent` and `HyperlaneFormedEvent`. These should hook into the notification system and pathfinding cache invalidation (so fleets re-route).
- **Visuals:** Slowly fading out a snapping hyperlane or drawing the stars moving over time in the UI creates a strong sense of a dynamic galaxy.
- **Predictability:** Expose the drift vectors visually in the system view so players can anticipate when a lane will snap or form.

## 6. Acceptance Criteria

- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code
- [ ] Star positions update based on their velocity vectors over time.
- [ ] Hyperlanes despawn if the systems drift too far apart.
- [ ] New hyperlanes spawn if systems drift close enough together.

## 7. Technical Guidance

- Register these systems in `Layer3SystemSet::Simulation` or equivalent.
- Ensure the drift calculations are scaled correctly with `SimulationTime` to make movement imperceptibly slow on a daily basis but meaningful over years.
- Add an integration test that checks pathfinding cache invalidation when a lane breaks.

## 8. Questions

*Builder: add questions here if spec is unclear.*
