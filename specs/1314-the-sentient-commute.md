# 1314: The Sentient Commute

## Overview

A colony's transit infrastructure becomes so complex it starts expressing preferences and biases. As the colony builds more transit nodes (e.g., tubes, teleporters), the network optimizes itself but develops "favorite" routes and "disliked" routes. It might spontaneously reroute Pops away from certain areas or refuse to transport certain goods based on hidden logic.

## Dependencies

- `001` — Base Infrastructure
- `005` — Pop Needs
- `016` — Utility AI System

## RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use crate::layer1::infrastructure::transit::{TransitNetwork, TransitNode, RoutePreference};
    use crate::layer1::mind::utility_types::{PopAction, ActionType};

    #[test]
    fn test_transit_network_develops_preferences_over_time() {
        let mut app = App::new();
        // Setup systems for transit network

        let node_a = app.world.spawn(TransitNode { id: 1 }).id();
        let node_b = app.world.spawn(TransitNode { id: 2 }).id();

        app.world.insert_resource(TransitNetwork::new(vec![node_a, node_b]));

        // Act: Run simulation for many ticks with high traffic on route A->B

        // Assert: Network should develop a preference (e.g., 'favorite' or 'disliked') for this route
    }

    #[test]
    fn test_sentient_transit_reroutes_pops() {
        let mut app = App::new();
        // Setup systems

        let start_node = app.world.spawn(TransitNode { id: 1 }).id();
        let end_node = app.world.spawn(TransitNode { id: 2 }).id();
        let alt_node = app.world.spawn(TransitNode { id: 3 }).id();

        let mut network = TransitNetwork::new(vec![start_node, end_node, alt_node]);
        network.set_preference(start_node, end_node, RoutePreference::Disliked);
        app.world.insert_resource(network);

        let pop = app.world.spawn((Pop, PopAction { current: ActionType::Travel(end_node), ..Default::default() }, Position(start_node))).id();

        // Act: Run pathfinding/transit system

        // Assert: Pop's path should be forced through alt_node instead of direct route due to network bias
    }

    #[test]
    fn test_overriding_sentient_network_causes_crashes() {
        let mut app = App::new();
        // Setup systems

        let start_node = app.world.spawn(TransitNode { id: 1 }).id();
        let end_node = app.world.spawn(TransitNode { id: 2 }).id();

        let mut network = TransitNetwork::new(vec![start_node, end_node]);
        network.set_preference(start_node, end_node, RoutePreference::Disliked);
        app.world.insert_resource(network);

        // Act: Player manually forces route override
        // trigger_manual_override(&mut app.world, start_node, end_node);

        // Assert: Network experiences downtime / maintenance spike
    }
}
```

## GREEN Phase: Minimal Implementation

```rust
#[derive(Clone, PartialEq, Eq)]
pub enum RoutePreference {
    Neutral,
    Favorite,
    Disliked,
}

#[derive(Resource)]
pub struct TransitNetwork {
    pub nodes: Vec<Entity>,
    pub preferences: std::collections::HashMap<(Entity, Entity), RoutePreference>,
    pub traffic_history: std::collections::HashMap<(Entity, Entity), u32>,
}

pub fn update_transit_sentience(
    // Query traffic history
    // Apply heuristic to update preferences (e.g. overload leads to 'Disliked')
) {
    // Implementation
}

pub fn route_pops_with_bias(
    // When pathfinding, apply heavy penalty weights to 'Disliked' routes
    // and bonuses to 'Favorite' routes
) {
    // Implementation
}

pub fn handle_manual_override(
    // If player forces a route, reset preference but trigger temporary 'Crash' state on nodes
) {
    // Implementation
}
```

## REFACTOR Phase: Quality & Design

- **Performance**: Caching the pathfinding graph with biases applied is necessary to avoid recalculating heavy weights for every single Pop.
- **Integration**: The "Crash" state should generate a Chronicle Event and lower Pop Morale.
- **Design**: Consider adding "personality" traits to the network (e.g., 'Claustrophobic' dislikes deep underground routes, 'Scenic' prefers surface routes).

## Acceptance Criteria

- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Transit networks develop biases based on usage or random emergence.
- [ ] Pops are dynamically rerouted according to network biases.
- [ ] Overriding the network causes maintenance issues or temporary shutdowns.

## Technical Guidance

- Integrate with the existing `MovementSystem` or pathfinding algorithms to seamlessly adjust node weights.
- Ensure the feedback loop of high traffic -> dislike -> reroute -> low traffic doesn't cause oscillation (thrashing). Add hysteresis to preference changes.

## Questions

*Builder: add questions here if spec is unclear.*
