# 1040 - The Blind Auction

## 1. Overview
The ultimate gamble. An enigmatic merchant fleet arrives offering a "Sealed Precursor Vault" to the highest bidder. Players must bid large amounts of Layer 1 resources (e.g., alloys, food) without knowing what's inside. Winning the bid opens the vault, which could grant endgame tech or unleash a catastrophic localized disaster directly into the capital.

## 2. Dependencies
- Layer 1 Economy / Resources (`ColonyResources`, etc.)
- Layer 2 Fleet/Merchant Arrivals
- UI Event notification system

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use scale::layer1::economy::ColonyResources;
    use scale::layer2::fleet::MerchantArrivalEvent;

    #[test]
    fn test_blind_auction_event_spawns() {
        let mut app = App::new();
        app.add_event::<MerchantArrivalEvent>();
        app.add_event::<BlindAuctionTriggeredEvent>();
        app.add_systems(Update, check_for_blind_auction_trigger);

        // Act: Enigmatic merchant arrives
        app.world_mut().send_event(MerchantArrivalEvent { faction: "Enigmatic".to_string() });
        app.update();

        // Assert: Blind auction starts
        let events = app.world().resource::<Events<BlindAuctionTriggeredEvent>>();
        let mut reader = events.get_reader();
        assert_eq!(reader.read(events).len(), 1, "Auction should trigger upon Enigmatic arrival.");
    }

    #[test]
    fn test_bidding_subtracts_resources() {
        let mut app = App::new();
        app.insert_resource(ColonyResources { alloys: 1000, ..default() });
        app.add_event::<PlaceBidEvent>();
        app.add_systems(Update, handle_blind_auction_bids);

        // Act: Bid placed
        app.world_mut().send_event(PlaceBidEvent { amount: 500, resource_type: ResourceType::Alloys });
        app.update();

        // Assert: Resources decremented
        let resources = app.world().resource::<ColonyResources>();
        assert_eq!(resources.alloys, 500, "Bidding should deduct resources immediately.");
    }

    #[test]
    fn test_vault_opening_outcome() {
        let mut app = App::new();
        app.add_event::<VaultOpenedEvent>();
        app.add_event::<TemporalAnomalyEvent>();
        app.add_systems(Update, process_vault_outcome);

        // Act: Vault opened with negative outcome generated
        app.world_mut().send_event(VaultOpenedEvent { outcome: VaultOutcome::CatastrophicAnomaly });
        app.update();

        // Assert: Anomaly spawns
        let events = app.world().resource::<Events<TemporalAnomalyEvent>>();
        let mut reader = events.get_reader();
        assert_eq!(reader.read(events).len(), 1, "Catastrophic outcome should spawn anomaly event.");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
// scale/layer1/core/integration.rs or scale/layer2/auction.rs

use bevy::prelude::*;
use crate::layer1::economy::{ColonyResources, ResourceType};
use crate::layer2::fleet::MerchantArrivalEvent;

#[derive(Event)]
pub struct BlindAuctionTriggeredEvent;

#[derive(Event)]
pub struct PlaceBidEvent {
    pub amount: u32,
    pub resource_type: ResourceType,
}

#[derive(Clone, Copy)]
pub enum VaultOutcome {
    TechBoost,
    CatastrophicAnomaly,
}

#[derive(Event)]
pub struct VaultOpenedEvent {
    pub outcome: VaultOutcome,
}

#[derive(Event)]
pub struct TemporalAnomalyEvent;

pub fn check_for_blind_auction_trigger(
    mut arrivals: EventReader<MerchantArrivalEvent>,
    mut trigger_ew: EventWriter<BlindAuctionTriggeredEvent>,
) {
    for arrival in arrivals.read() {
        if arrival.faction == "Enigmatic" {
            trigger_ew.send(BlindAuctionTriggeredEvent);
        }
    }
}

pub fn handle_blind_auction_bids(
    mut bids: EventReader<PlaceBidEvent>,
    mut resources: ResMut<ColonyResources>,
) {
    for bid in bids.read() {
        match bid.resource_type {
            ResourceType::Alloys => {
                if resources.alloys >= bid.amount {
                    resources.alloys -= bid.amount;
                }
            }
            _ => {}
        }
    }
}

pub fn process_vault_outcome(
    mut openings: EventReader<VaultOpenedEvent>,
    mut anomaly_ew: EventWriter<TemporalAnomalyEvent>,
) {
    for opening in openings.read() {
        if let VaultOutcome::CatastrophicAnomaly = opening.outcome {
            anomaly_ew.send(TemporalAnomalyEvent);
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- Currently hardcoded "Enigmatic" faction string. Change to enum or specific Component tag.
- Outcomes need to tie into actual gameplay. Temporal anomaly should freeze pops, TechBoost should actually unlock tech.
- Create UI popups for event prompts rather than just sending events implicitly.
- Prevent bids if resources aren't sufficient.
- Handle rival empires outbidding the player over time.

## 6. Acceptance Criteria
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Coverage for new systems ≥85%.
- [ ] A rival bidder can outbid the player or a specific bidding duration logic exists.
- [ ] The outcome of the vault has actual gameplay effects (good or bad).

## 7. Technical Guidance
- Bidding logic should probably occur in a specific App State (e.g. `AppState::AuctionActive`) to prevent regular economic consumption from ruining bids mid-process.
- `TemporalAnomalyEvent` will need listeners in Layer 1 to freeze pop logic components if triggered.

## 8. Questions
- How long should the bidding period last?
  - *Architect:* The bidding period should last exactly 7 in-game days.
- Should we allow bidding multi-resource baskets?
  - *Architect:* No, for the MVP, bidding is restricted to a single resource type (e.g., just Alloys).
