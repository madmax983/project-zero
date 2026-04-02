# 778: Hyperlane Collapse

## 1. Overview
The "Hyperlane Collapse" feature brings a dramatic and unpredictable macro-level crisis to the game. It allows hyperlanes (the connective tissue of the galaxy) to destabilize and sever connections between star systems on Layer 3.

When a hyperlane collapses, affected systems become isolated "Islands," severing trade routes, preventing standard fleet movement, and forcing civilizations to rely on dangerous experimental jump drives or slow-warp travel. This creates a compelling tension between relying on centralized, highly efficient trade hubs and building resilient, decentralized networks.

## 2. Dependencies
- Layer 3 Map and Node Graph (`StarSystem`, `Hyperlane`)
- Layer 3 Logistics & Pathfinding (A* or similar pathing for fleets and trade routes)
- Layer 3 Diplomatic and Trade Systems (for handling severed connections)

## 3. RED Phase: Tests First
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::prelude::*;
    use crate::layer3::map::{Hyperlane, StarSystem};
    use crate::layer3::movement::Pathfinding;
    use crate::shared::time::SimulationTime;

    fn setup_app() -> App {
        let mut app = App::new();
        app.insert_resource(SimulationTime::default());
        app.add_event::<HyperlaneCollapseEvent>();
        app.add_systems(Update, (
            trigger_hyperlane_collapse_system,
            process_hyperlane_collapse_system,
            recalculate_trade_routes_system
        ));
        app
    }

    #[test]
    fn test_hyperlane_collapse_severs_connection() {
        let mut app = setup_app();

        let sys_a = app.world_mut().spawn(StarSystem { id: 1 }).id();
        let sys_b = app.world_mut().spawn(StarSystem { id: 2 }).id();

        // Spawn a hyperlane connecting A and B
        let lane = app.world_mut().spawn(Hyperlane {
            start: sys_a,
            end: sys_b,
            stability: 100.0,
        }).id();

        // Trigger a collapse event
        app.world_mut().send_event(HyperlaneCollapseEvent {
            lane_entity: lane,
        });

        app.update();

        // The hyperlane should be despawned or marked as collapsed
        assert!(app.world().get::<Hyperlane>(lane).is_none(), "Hyperlane should be destroyed after a collapse");
    }

    #[test]
    fn test_fleet_pathfinding_fails_when_lane_collapses() {
        let mut app = setup_app();

        let sys_a = app.world_mut().spawn(StarSystem { id: 1 }).id();
        let sys_b = app.world_mut().spawn(StarSystem { id: 2 }).id();

        let lane = app.world_mut().spawn(Hyperlane {
            start: sys_a,
            end: sys_b,
            stability: 100.0,
        }).id();

        // We assume Pathfinding is a resource or component that can verify connectivity
        // For the test, we mock a check that A and B are connected
        app.world_mut().send_event(HyperlaneCollapseEvent { lane_entity: lane });
        app.update();

        // After update, recalculate_trade_routes_system or similar should run
        // Pathfinding from sys_a to sys_b should now fail
        // This is a conceptual assertion; actual implementation depends on `Pathfinding` module
        // assert!(!app.world().resource::<Pathfinding>().is_connected(sys_a, sys_b));
    }
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
use bevy_ecs::prelude::*;
use crate::layer3::map::{Hyperlane, StarSystem};

#[derive(Event)]
pub struct HyperlaneCollapseEvent {
    pub lane_entity: Entity,
}

#[derive(Event)]
pub struct TradeRouteSeveredEvent {
    pub system_a: Entity,
    pub system_b: Entity,
}

pub fn trigger_hyperlane_collapse_system(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Hyperlane)>,
    mut collapse_events: EventWriter<HyperlaneCollapseEvent>,
) {
    // In a real scenario, this would check stability decay or random events
    // For MVP, we just watch for lanes with 0 stability
    for (entity, lane) in query.iter() {
        if lane.stability <= 0.0 {
            collapse_events.send(HyperlaneCollapseEvent { lane_entity: entity });
        }
    }
}

pub fn process_hyperlane_collapse_system(
    mut commands: Commands,
    mut events: EventReader<HyperlaneCollapseEvent>,
    lane_query: Query<&Hyperlane>,
    mut severed_events: EventWriter<TradeRouteSeveredEvent>,
) {
    for ev in events.read() {
        if let Ok(lane) = lane_query.get(ev.lane_entity) {
            severed_events.send(TradeRouteSeveredEvent {
                system_a: lane.start,
                system_b: lane.end,
            });
            commands.entity(ev.lane_entity).despawn_recursive();
        }
    }
}

pub fn recalculate_trade_routes_system(
    mut events: EventReader<TradeRouteSeveredEvent>,
    // Requires access to the trade route registry/graph
) {
    for _ev in events.read() {
        // Trigger a global recalculation of paths and trade networks
        // If a route cannot be reformed, emit a Starvation/Shortage event for affected colonies
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Pathing Updates:** Despawning a hyperlane requires notifying the Layer 3 pathfinding graph to rebuild. The `recalculate_trade_routes_system` must invalidate cached paths for fleets currently en route.
- **Fleet Stranding:** Fleets currently traversing a hyperlane when it collapses should be pushed back to their origin system, destroyed, or dumped into "slow warp" deep space.
- **Stability Decay:** Implement a system where `Hyperlane.stability` naturally decays based on heavy use (too many massive fleets), specific Layer 3 weapons, or random cosmic events.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures for new code.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for `src/layer3/map/hyperlane.rs` (or equivalent).
- [ ] When a `HyperlaneCollapseEvent` fires, the `Hyperlane` entity is despawned.
- [ ] A `TradeRouteSeveredEvent` is correctly emitted indicating the two newly disconnected systems.

## 7. Technical Guidance
- **Graph State:** The Layer 3 map relies heavily on graph connectivity. Ensure that removing a hyperlane safely updates any caching mechanisms (like an A* heuristic cache or a minimal spanning tree for trade).
- **Chronicle Event:** Add a `ChronicleEvent` when a lane collapses so the player receives a chilling notification: "The hyperlane to Sirius has collapsed. The sector is now cut off."
- **Layer Bridge:** Be mindful of how a severed trade route on Layer 3 impacts `ColonyResources` on Layer 1. A colony dependent on food imports must instantly recognize the loss and enter a famine state if no alternative route exists.

## 8. Questions
*Builder: add questions here if spec is unclear.*
