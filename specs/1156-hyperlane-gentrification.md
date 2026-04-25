# 1156 Hyperlane Gentrification

## 1. Overview
When a new, more efficient Layer 3 hyperlane or gate is constructed, Layer 2 trade fleets immediately reroute to use it. The old, less efficient routes lose all their traffic. Layer 1 colonies that previously thrived by supplying fuel, repairs, and entertainment to passing ships suddenly experience a massive, rapid economic collapse.

This creates tension: Do you prioritize optimal, efficient galactic infrastructure, knowing it will financially ruin and destabilize the legacy colonies that supported you in the early game?

## 2. Dependencies
- Layer 3 `Hyperlane` / routing graph system.
- Layer 2 `TradeFleet` pathfinding mechanics.
- Layer 1 colony economy mechanics (specifically facilities dependent on trade ship arrivals).

## 3. RED Phase: Tests First
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_fleet_reroutes_to_new_hyperlane() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
           .add_systems(Update, update_fleet_routes);

        let start_node = app.world_mut().spawn(SystemNode).id();
        let end_node = app.world_mut().spawn(SystemNode).id();

        // Old slow route
        let old_route = app.world_mut().spawn(Hyperlane {
            source: start_node,
            target: end_node,
            travel_time: 100.0,
        }).id();

        let fleet = app.world_mut().spawn((
            TradeFleet,
            CurrentRoute(old_route),
            OriginDestination { start: start_node, end: end_node },
        )).id();

        app.update();

        // Spawn a new, faster route
        let new_route = app.world_mut().spawn(Hyperlane {
            source: start_node,
            target: end_node,
            travel_time: 20.0,
        }).id();

        app.update(); // Fleet should recalculate and use new route

        let current = app.world().entity(fleet).get::<CurrentRoute>().unwrap();
        assert_eq!(current.0, new_route, "Fleet should reroute to the faster hyperlane");
    }

    #[test]
    fn test_economic_collapse_from_lost_traffic() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
           .add_systems(Update, process_colony_trade_economy);

        // A colony heavily dependent on trade
        let colony = app.world_mut().spawn((
            Colony,
            TradeDependency(100.0), // High dependency
            RecentTraffic(0),       // Lost all traffic
            LocalEconomy(100.0),    // Current economy value
        )).id();

        app.update(); // Economy should plummet due to no traffic

        let economy = app.world().entity(colony).get::<LocalEconomy>().unwrap();
        assert!(economy.0 < 50.0, "Economy should collapse when recent traffic is 0");
    }
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct SystemNode;

#[derive(Component)]
pub struct Hyperlane {
    pub source: Entity,
    pub target: Entity,
    pub travel_time: f32,
}

#[derive(Component)]
pub struct TradeFleet;

#[derive(Component)]
pub struct CurrentRoute(pub Entity);

#[derive(Component)]
pub struct OriginDestination {
    pub start: Entity,
    pub end: Entity,
}

#[derive(Component)]
pub struct Colony;

#[derive(Component)]
pub struct TradeDependency(pub f32);

#[derive(Component)]
pub struct RecentTraffic(pub u32);

#[derive(Component)]
pub struct LocalEconomy(pub f32);

pub fn update_fleet_routes(
    mut fleets: Query<(&mut CurrentRoute, &OriginDestination), With<TradeFleet>>,
    hyperlanes: Query<(Entity, &Hyperlane)>,
) {
    for (mut route, od) in fleets.iter_mut() {
        // Find fastest hyperlane between start and end
        let mut best_route = None;
        let mut min_time = f32::MAX;

        for (hl_ent, hl) in hyperlanes.iter() {
            if hl.source == od.start && hl.target == od.end {
                if hl.travel_time < min_time {
                    min_time = hl.travel_time;
                    best_route = Some(hl_ent);
                }
            }
        }

        if let Some(best) = best_route {
            route.0 = best;
        }
    }
}

pub fn process_colony_trade_economy(
    mut colonies: Query<(&mut LocalEconomy, &RecentTraffic, &TradeDependency), With<Colony>>,
) {
    for (mut economy, traffic, dependency) in colonies.iter_mut() {
        if traffic.0 == 0 && dependency.0 > 50.0 {
            // Massive crash in economy if heavily dependent and traffic stops
            economy.0 -= 60.0;
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Code Smells**: Re-evaluating fleet paths every single tick over all hyperlanes is extremely expensive. Pathfinding should only run when the hyperlane graph changes or a fleet reaches a node (Dijkstra/A* on a navmesh or graph).
- **Architecture**: The `process_colony_trade_economy` is too harsh and immediate. It should probably accumulate a negative modifier over time, triggering `Unrest` or `Riot` events rather than just raw value reduction.
- **Dependencies**: Needs a system to actually count passing ships to update `RecentTraffic` on a periodic basis.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.
- [ ] Layer 2 fleets dynamically switch to faster newly constructed routes.
- [ ] Layer 1 colonies relying on trade lose economic stability when bypassed.

## 7. Technical Guidance
- We need to hook the fleet rerouting logic to a `HyperlaneConstructedEvent` rather than polling `Update` every frame.
- Ensure the legacy colonies receive appropriate warning UI or have a gradual decay so the player has a chance to pivot their local economy before a full riot occurs.

## 8. Questions
*Builder: add questions here if spec is unclear.*
