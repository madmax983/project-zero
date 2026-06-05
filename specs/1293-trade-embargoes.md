# 1293: Trade Embargoes

## 1. Overview
Economic warfare through trade embargoes allows starving the enemy without firing a shot. A faction can use its Influence with the Galactic Council to "Embargo" a specific resource. This prevents the resource from being traded on the Galactic Market, creating monopolies and forcing opposing factions into desperate situations such as attacking to break blockades. This mechanism introduces a tension between free market profit and economic weaponization.

## 2. Dependencies
- `layer3::diplomacy::Influence`
- `layer3::market::GalacticMarket`
- `layer3::market::Resource`

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use crate::layer3::diplomacy::{Influence, FactionId};
    use crate::layer3::market::{GalacticMarket, ResourceType};

    #[test]
    fn test_trade_embargo_prevents_trading() {
        let mut app = App::new();
        app.add_systems(Update, apply_trade_embargo_system);

        let faction_id = FactionId(1);
        let mut market = GalacticMarket::default();
        market.add_listing(ResourceType::Durasteel, 100);
        app.insert_resource(market);

        // Spawn an embargo entity
        app.world_mut().spawn(TradeEmbargo {
            resource: ResourceType::Durasteel,
            enforcing_faction: faction_id,
        });

        // Act
        app.update();

        // Assert
        let market = app.world().resource::<GalacticMarket>();
        assert!(market.is_embargoed(ResourceType::Durasteel));
        assert!(!market.can_trade(ResourceType::Durasteel, FactionId(2)));
    }

    #[test]
    fn test_trade_embargo_costs_influence() {
        let mut app = App::new();
        app.add_systems(Update, declare_trade_embargo_system);

        let faction_id = FactionId(1);
        app.insert_resource(Influence { amount: 500, faction: faction_id });

        // Send event to declare embargo
        app.add_event::<DeclareEmbargoEvent>();
        app.world_mut().send_event(DeclareEmbargoEvent {
            faction: faction_id,
            resource: ResourceType::Durasteel,
        });

        // Act
        app.update();

        // Assert
        let influence = app.world().resource::<Influence>();
        assert_eq!(influence.amount, 0); // Assuming it costs 500

        // Embargo should be created
        let mut query = app.world_mut().query::<&TradeEmbargo>();
        let embargo_count = query.iter(app.world()).count();
        assert_eq!(embargo_count, 1);
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;
use crate::layer3::diplomacy::{Influence, FactionId};
use crate::layer3::market::{GalacticMarket, ResourceType};

#[derive(Component)]
pub struct TradeEmbargo {
    pub resource: ResourceType,
    pub enforcing_faction: FactionId,
}

#[derive(Event)]
pub struct DeclareEmbargoEvent {
    pub faction: FactionId,
    pub resource: ResourceType,
}

pub fn apply_trade_embargo_system(
    mut market: ResMut<GalacticMarket>,
    embargoes: Query<&TradeEmbargo>,
) {
    // Clear previous embargoes first
    market.clear_embargoes();

    for embargo in embargoes.iter() {
        market.set_embargoed(embargo.resource, true);
    }
}

pub fn declare_trade_embargo_system(
    mut commands: Commands,
    mut influence: Option<ResMut<Influence>>,
    mut events: EventReader<DeclareEmbargoEvent>,
) {
    let cost = 500;

    for event in events.read() {
        if let Some(ref mut inf) = influence {
            if inf.faction == event.faction && inf.amount >= cost {
                inf.amount -= cost;

                commands.spawn(TradeEmbargo {
                    resource: event.resource.clone(),
                    enforcing_faction: event.faction,
                });
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Market Integration**: Update the actual `GalacticMarket` implementation to respect the `is_embargoed` and `can_trade` flags appropriately across all sub-systems like `TradeRoute` processing.
- **Duration/Upkeep**: Embargoes should likely have a duration or an influence upkeep cost per tick instead of just a one-time flat fee.
- **Breaking Blockades**: Factions need a way to break these embargoes through diplomacy or fleet action.

## 6. Acceptance Criteria
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage >= 85%.
- [ ] Declaring an embargo deducts influence and creates an embargo entity.
- [ ] Active embargoes prevent the embargoed resource from being traded by other factions on the market.

## 7. Technical Guidance
- If `GalacticMarket` doesn't currently support embargo flags natively, you may need to add `HashSet<ResourceType>` to it to track currently embargoed items, alongside standard pricing logic.

## 8. Questions
*Builder: add questions here if spec is unclear.*
