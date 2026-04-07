# Specification: 856 Space Lanes

## 1. Overview
**Layer:** 2
**Fantasy:** Blazing a trail through the dark. Space isn't an open ocean; it's a series of highways. Controlling the highway means controlling the system's economy.
**Mechanic:** Repeated ship travel between two stellar nodes creates a "Space Lane." Lanes provide faster and safer travel due to established navigation data. However, lanes decay if unused. Furthermore, high-traffic lanes naturally attract pirate blockades, and players can build Toll Stations on them. Excessive tolls or pirate activity will drive trade to alternate, slower paths, causing the lane to decay.

## 2. Dependencies
- Base Layer 2 Ship Movement (`src/layer2/ship.rs` / `src/layer2/movement.rs`)
- Node-to-Node Graph / Coordinates (`src/layer2/stellar_map.rs` or equivalent)

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_space_lane_creation_and_decay() {
        let mut app = App::new();
        app.add_plugins(SpaceLanesPlugin);

        let node_a = app.world_mut().spawn(StellarNode).id();
        let node_b = app.world_mut().spawn(StellarNode).id();

        // Simulate travel between Node A and Node B
        app.world_mut().send_event(ShipTraveledEvent {
            from: node_a,
            to: node_b,
        });
        app.update();

        // Assert lane was created with initial traffic
        let mut lane_query = app.world_mut().query::<&SpaceLane>();
        let lane = lane_query.iter(app.world()).next().expect("Lane should be created");
        assert!(lane.traffic > 0.0);
        assert_eq!(lane.endpoints, (node_a, node_b));

        // Simulate time passing with no travel
        app.world_mut().resource_mut::<Time>().advance_by(Duration::from_secs(10));
        app.update();

        // Assert lane traffic decayed
        let decayed_lane = lane_query.iter(app.world()).next().expect("Lane should still exist");
        assert!(decayed_lane.traffic < lane.traffic);
    }

    #[test]
    fn test_space_lane_speed_bonus() {
        let mut app = App::new();
        app.add_plugins(SpaceLanesPlugin);

        let node_a = app.world_mut().spawn(StellarNode).id();
        let node_b = app.world_mut().spawn(StellarNode).id();

        app.world_mut().spawn(SpaceLane {
            endpoints: (node_a, node_b),
            traffic: 100.0, // High traffic lane
        });

        // Spawn a ship moving on this lane
        let ship = app.world_mut().spawn((
            Ship,
            Navigating { from: node_a, to: node_b },
            BaseSpeed(10.0),
        )).id();

        app.update();

        // Speed should be modified by the lane's efficiency
        let effective_speed = app.world().get::<EffectiveSpeed>(ship).unwrap();
        assert!(effective_speed.0 > 10.0, "Speed should be increased by the lane");
    }

    #[test]
    fn test_lane_avoidance_due_to_tolls_or_pirates() {
        let mut app = App::new();
        app.add_plugins(SpaceLanesPlugin);

        let node_a = app.world_mut().spawn(StellarNode).id();
        let node_b = app.world_mut().spawn(StellarNode).id();

        let lane = app.world_mut().spawn((
            SpaceLane {
                endpoints: (node_a, node_b),
                traffic: 100.0,
            },
            PirateThreat(50.0), // High pirate threat
        )).id();

        // Simulate pathfinding request
        app.world_mut().send_event(PathfindingRequest {
            ship: Entity::PLACEHOLDER, // Dummy
            from: node_a,
            to: node_b,
        });

        app.update();

        // Verify the pathfinder prefers an alternate route or penalizes the high-threat lane
        let path_response = app.world().resource::<LatestPathResponse>();
        assert!(path_response.penalty_applied, "Pathfinder should penalize lanes with high pirate threat");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct SpaceLane {
    pub endpoints: (Entity, Entity),
    pub traffic: f32,
}

#[derive(Event)]
pub struct ShipTraveledEvent {
    pub from: Entity,
    pub to: Entity,
}

#[derive(Component)]
pub struct PirateThreat(pub f32);

pub struct SpaceLanesPlugin;

impl Plugin for SpaceLanesPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ShipTraveledEvent>()
           .add_systems(Update, (
               record_travel_system,
               decay_lanes_system,
               apply_lane_speed_bonus_system,
           ));
    }
}

fn record_travel_system(
    mut commands: Commands,
    mut events: EventReader<ShipTraveledEvent>,
    mut lanes: Query<(Entity, &mut SpaceLane)>,
) {
    for event in events.read() {
        let mut found = false;
        for (_, mut lane) in lanes.iter_mut() {
            if (lane.endpoints.0 == event.from && lane.endpoints.1 == event.to) ||
               (lane.endpoints.1 == event.from && lane.endpoints.0 == event.to) {
                lane.traffic += 10.0;
                found = true;
                break;
            }
        }
        if !found {
            commands.spawn(SpaceLane {
                endpoints: (event.from, event.to),
                traffic: 10.0,
            });
        }
    }
}

fn decay_lanes_system(
    mut commands: Commands,
    mut lanes: Query<(Entity, &mut SpaceLane)>,
    time: Res<Time>,
) {
    for (entity, mut lane) in lanes.iter_mut() {
        lane.traffic -= time.delta_seconds();
        if lane.traffic <= 0.0 {
            commands.entity(entity).despawn();
        }
    }
}

// Dummy component for test
#[derive(Component)]
pub struct BaseSpeed(pub f32);

#[derive(Component)]
pub struct EffectiveSpeed(pub f32);

#[derive(Component)]
pub struct Navigating {
    pub from: Entity,
    pub to: Entity,
}

fn apply_lane_speed_bonus_system(
    mut commands: Commands,
    ships: Query<(Entity, &BaseSpeed, &Navigating)>,
    lanes: Query<&SpaceLane>,
) {
    for (ship_entity, base_speed, nav) in ships.iter() {
        let mut speed_mult = 1.0;
        for lane in lanes.iter() {
            if (lane.endpoints.0 == nav.from && lane.endpoints.1 == nav.to) ||
               (lane.endpoints.1 == nav.from && lane.endpoints.0 == nav.to) {
                // simple scaling
                speed_mult += lane.traffic * 0.01;
            }
        }
        commands.entity(ship_entity).insert(EffectiveSpeed(base_speed.0 * speed_mult));
    }
}

// Dummy stubs for pathfinding
#[derive(Event)]
pub struct PathfindingRequest {
    pub ship: Entity,
    pub from: Entity,
    pub to: Entity,
}

#[derive(Resource, Default)]
pub struct LatestPathResponse {
    pub penalty_applied: bool,
}
```

## 5. REFACTOR Phase: Quality & Design
- Extract the lane search logic into a helper function or map resource instead of doing O(N) linear iteration over all lanes.
- Integrate the traffic values into the actual A* or Dijkstra pathfinding weights for ship routing logic.
- Ensure the lane IDs are deterministic (e.g. sorted tuples of Entity IDs) so lookups can be normalized.
- Separate Toll mechanics and Pirate Threat generation into their own modular systems driven by lane traffic thresholds.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage >= 85% for the space lanes module.
- [ ] Traveling between nodes increases lane traffic, and lack of travel decays it.
- [ ] Ships traveling on lanes with traffic receive a measurable speed bonus.

## 7. Technical Guidance
- **Edge cases:** A ship traveling round trip should augment the same bidirectional lane, not create two directional lanes. Sort the tuple `(A, B)` to ensure consistency.
- Implement the `PirateThreat` as a separate observer that ticks up based on the `SpaceLane` traffic amount.
- Ensure `SpaceLane` entities are despawned if traffic hits `0.0` to avoid bloat in the ECS.

## 8. Questions
*Builder: Add questions here if spec is unclear.*
