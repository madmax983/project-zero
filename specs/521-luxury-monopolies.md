# 521. Luxury Monopolies

## 1. Overview
**Layer:** 3
**Fantasy:** "The spice must flow." You control the universe's addiction.
**Mechanic:** Each planet has a unique "Native Resource" (e.g., "Glow-Moss"). If you control the only source in the sector, you set the price. Other civs become dependent on you (Diplomatic leverage) but also covetous (War risk).
**Emergence:** You corner the market on "Stardust". Your economy booms, but a coalition forms specifically to liberate the Stardust mines.
**Tension:** Share the wealth (peace) or Gouge the galaxy (profit/war)?

## 2. Dependencies
- `039` Trade System
- `101` System Mining
- `203` Prohibition & Contraband
- `469` The Galactic Council (implied Layer 3 diplomacy mechanics)

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use crate::layer3::trade::MarketPrices;
    use crate::layer3::diplomacy::DiplomaticRelations;

    #[test]
    fn test_monopoly_price_increase() {
        let mut app = App::new();
        // Arrange: Setup market with a unique resource controlled by player
        app.insert_resource(MarketPrices::default());
        app.world_mut().spawn((
            Civilization::player(),
            ResourceControl { resource: ItemType::Stardust, amount: 100 },
        ));

        // Act: Advance time to trigger market update
        app.update();

        // Assert: Price of Stardust should skyrocket
        let prices = app.world().resource::<MarketPrices>();
        assert!(prices.get(&ItemType::Stardust) > 500.0, "Monopoly price didn't increase.");
    }

    #[test]
    fn test_monopoly_diplomatic_penalty() {
        let mut app = App::new();
        app.insert_resource(DiplomaticRelations::default());
        let player_id = app.world_mut().spawn((
            Civilization::player(),
            ResourceControl { resource: ItemType::GlowMoss, amount: 100 },
        )).id();
        let npc_id = app.world_mut().spawn((
            Civilization::npc(),
            ResourceNeed { resource: ItemType::GlowMoss, urgency: High },
        )).id();

        app.update();

        let relations = app.world().resource::<DiplomaticRelations>();
        assert!(relations.get_score(player_id, npc_id) < 0.0, "Monopoly should cause diplomatic friction.");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
// The SIMPLEST code that makes tests pass
// In src/layer3/trade/market.rs

pub fn update_market_prices_system(
    mut prices: ResMut<MarketPrices>,
    query: Query<&ResourceControl, With<Civilization>>,
) {
    for control in query.iter() {
        // Very basic monopoly check: if player has a lot, increase price
        if control.amount > 50 {
            prices.set(control.resource, 600.0);
        }
    }
}

// In src/layer3/diplomacy/relations.rs
pub fn update_diplomatic_relations_system(
    mut relations: ResMut<DiplomaticRelations>,
    providers: Query<(Entity, &ResourceControl), With<Civilization>>,
    needers: Query<(Entity, &ResourceNeed), With<Civilization>>,
) {
    for (provider_entity, control) in providers.iter() {
        for (needer_entity, need) in needers.iter() {
            if control.resource == need.resource && need.urgency == High {
                 relations.adjust_score(provider_entity, needer_entity, -10.0);
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Code Smells:** The minimal implementation hardcodes values (`50`, `600.0`, `-10.0`). These should be driven by a robust economic simulation based on supply/demand curves and actual monopoly calculation (checking if *only* one civilization controls the resource).
- **Performance:** Calculating monopolies every tick for every civilization and resource could be expensive. Throttle the `update_market_prices_system` to run periodically (e.g., every in-game week) or on a specific schedule.
- **API Improvements:** Introduce a proper `MonopolyStatus` component or resource to explicitly track who holds monopolies, making it easier for UI and other systems (like event generation) to query.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage >= 85% for new code
- [ ] Controlling 100% of a rare resource correctly triggers monopoly pricing.
- [ ] Monopoly status correctly degrades diplomatic relations with dependent factions.

## 7. Technical Guidance
- **Integration Points:** Connect this to `src/layer3/diplomacy.rs` to handle war declarations when relations drop too low due to price gouging. Link to `src/layer2/trade.rs` to allow the player to physically export the resource.
- **Gotchas:** Be careful with the initial state. If a player starts with a monopoly accidentally, they might face early game wars they can't survive. Ensure rare resources require mid/late game tech to extract.

## 8. Questions
*Builder: add questions here if spec is unclear.*
