# 1158: The Butterfly Effect of the Local Market

## Overview

A specific Layer 1 colony produces a highly sought-after luxury resource. The Layer 3 galactic economy heavily depends on this resource for high-end diplomacy. If a local disruption (like a strike or a localized plague) halts production, the ripple effect instantly tanks the Layer 3 market, causing your closest ally, unable to afford the resource needed to appease their own elites, to fall into a civil war, leaving your flank entirely exposed to an invasion.

## Dependencies

- None (Base Layer 1 production and Layer 3 market mechanics assumed)

## RED Phase: Tests First

```rust
// Define the tests that will drive implementation
// These should FAIL initially

#[test]
fn test_layer1_production_halt_triggers_market_shock() {
    // Arrange: Setup a Layer 1 colony producing a luxury resource and a Layer 3 market
    // Act: Trigger a disruption (e.g., a strike) that halts luxury resource production
    // Assert: The Layer 3 market price for the luxury resource skyrockets
}

#[test]
fn test_market_shock_causes_ally_civil_war() {
    // Arrange: Setup an allied Layer 3 faction dependent on the luxury resource
    // Act: Trigger a market shock (price spike) for the luxury resource
    // Assert: The allied faction experiences a civil war event
}

#[test]
fn test_ally_civil_war_removes_defensive_buffs() {
    // Arrange: Setup an allied faction providing defensive buffs to the player
    // Act: The allied faction enters a civil war
    // Assert: The player's defensive buffs from that ally are removed, exposing flanks
}

#[test]
fn test_production_restoration_stabilizes_market() {
    // Arrange: A halted production state with a spiked market price
    // Act: Restore luxury resource production
    // Assert: The market price begins to stabilize over time
}
```

## GREEN Phase: Minimal Implementation

```rust
// The SIMPLEST code that makes tests pass
// No optimization, no extras
// Just enough to turn RED -> GREEN

pub fn monitor_luxury_production_system(
    mut market: ResMut<GalacticMarket>,
    colonies: Query<&LuxuryProduction>,
) {
    let mut total_production = 0.0;
    for production in colonies.iter() {
        total_production += production.amount;
    }

    if total_production == 0.0 {
        market.luxury_price *= 2.0; // Price skyrockets
    } else {
        market.luxury_price = (market.luxury_price * 0.9).max(10.0); // Stabilize
    }
}

pub fn trigger_ally_civil_war_system(
    market: Res<GalacticMarket>,
    mut allies: Query<(Entity, &mut FactionStatus, &Dependencies)>,
    mut commands: Commands,
) {
    if market.luxury_price > 100.0 {
        for (entity, mut status, deps) in allies.iter_mut() {
            if deps.needs_luxury && *status == FactionStatus::Stable {
                *status = FactionStatus::CivilWar;
                commands.entity(entity).remove::<DefensivePact>(); // Flank exposed
            }
        }
    }
}
```

## REFACTOR Phase: Quality & Design

- **Optimization:** Use an event-driven architecture to notify the market of production changes rather than polling all colonies every tick.
- **Code Smell:** Avoid hardcoded price multipliers (`2.0`, `0.9`) and thresholds (`100.0`); these should be loaded from a configuration or balanced dynamically based on galactic supply/demand.
- **Integration:** Ensure the market shock is visually communicated in the UI so the player understands the consequence of the local strike.

## Acceptance Criteria

- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code in `src/layer3/economy/market_shock.rs`
- [ ] The transition to civil war creates a Chronicle entry explaining the cause (luxury shortage).
- [ ] UI reflects the loss of the defensive pact.

## Technical Guidance

### Components

```rust
#[derive(Component)]
pub struct LuxuryProduction {
    pub amount: f32,
}

#[derive(Component)]
pub struct Dependencies {
    pub needs_luxury: bool,
}

#[derive(PartialEq, Component)]
pub enum FactionStatus {
    Stable,
    CivilWar,
}

#[derive(Component)]
pub struct DefensivePact;
```

### Systems

```rust
pub fn update_market_prices(
    // Calculate global supply/demand and adjust prices
) {
    // ...
}

pub fn handle_faction_stability(
    // Monitor resource needs vs market prices, trigger civil wars if unaffordable
) {
    // ...
}
```

### Integration Points

- **Layer 1 to Layer 3 Bridge:** This requires ensuring that Layer 1 production aggregates correctly into the Layer 3 `GalacticMarket` resource or component.
- **Diplomacy System:** The removal of `DefensivePact` must trigger updates to the player's threat map, inviting invasions from hostile neighbors.

## Questions

*Builder: add questions here if spec is unclear. Architect will address.*
