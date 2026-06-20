//! Blind Auction system for rare artifacts and vault access.
//!
//! This module manages events and types associated with the Blind Auction mechanic,
//! where the colony can bid resources to unlock sealed vaults or acquire unique
//! assets from passing merchant fleets. Since the exact contents are unknown
//! during the bidding phase, outcomes carry significant risk.
//!
//! # Examples
//!
//! Triggering a blind auction from a custom system:
//! ```
//! use scale::layer2::auction::BlindAuctionTriggeredEvent;
//! use bevy_ecs::prelude::*;
//!
//! fn trigger_auction_system(mut ev_auction: EventWriter<BlindAuctionTriggeredEvent>) {
//!     ev_auction.send(BlindAuctionTriggeredEvent);
//! }
//! ```

use crate::layer1::economy::resources::{ColonyResources, ResourceType};
use crate::layer1::void_weed::{MerchantArrivalEvent, MerchantType};
use bevy::prelude::*;

/// Event fired when a blind auction becomes available to the colony.
///
/// This typically happens when a specific merchant arrives or a vault is discovered.
#[derive(Event, Debug, Clone)]
pub struct BlindAuctionTriggeredEvent;

/// Event sent when the player places a bid on an active blind auction.
///
/// The amount is deducted from the colony's resources. If the bid is
/// the highest, the colony will win the auction when it resolves.
#[derive(Event, Debug, Clone)]
pub struct PlaceBidEvent {
    /// The quantity of the resource being bid.
    pub amount: f32,
    /// The specific type of resource offered.
    pub resource_type: ResourceType,
}

/// The possible results of winning a blind auction for a sealed vault.
///
/// Because it is a blind auction, winning may result in a powerful boon
/// or a devastating consequence for the colony.
#[derive(Clone, Copy, Debug)]
pub enum VaultOutcome {
    /// The vault contained advanced knowledge, providing a permanent buff.
    TechBoost,
    /// The vault contained a hazard, causing damage or negative effects.
    CatastrophicAnomaly,
}

#[derive(Event, Debug, Clone)]
pub struct VaultOpenedEvent {
    pub outcome: VaultOutcome,
}

#[derive(Event, Debug, Clone)]
pub struct TemporalAnomalyEvent;

pub fn check_for_blind_auction_trigger(
    mut arrivals: EventReader<MerchantArrivalEvent>,
    mut trigger_ew: EventWriter<BlindAuctionTriggeredEvent>,
) {
    for arrival in arrivals.read() {
        if arrival.merchant_type == MerchantType::Enigmatic {
            trigger_ew.send(BlindAuctionTriggeredEvent);
        }
    }
}

pub fn handle_blind_auction_bids(
    mut bids: EventReader<PlaceBidEvent>,
    mut resources: ResMut<ColonyResources>,
) {
    for bid in bids.read() {
        if bid.resource_type == ResourceType::Stone && resources.stone >= bid.amount {
            resources.stone -= bid.amount;
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::economy::resources::{ColonyResources, ResourceType};
    use crate::layer1::void_weed::{MerchantArrivalEvent, MerchantType};

    #[test]
    fn test_blind_auction_event_spawns() {
        let mut app = bevy_app::App::new();
        app.add_event::<MerchantArrivalEvent>();
        app.add_event::<BlindAuctionTriggeredEvent>();
        app.add_systems(bevy_app::Update, check_for_blind_auction_trigger);

        // Act: Enigmatic merchant arrives
        app.world_mut().send_event(MerchantArrivalEvent {
            merchant_type: MerchantType::Enigmatic,
        });
        app.update();

        // Assert: Blind auction starts
        let events = app.world().resource::<Events<BlindAuctionTriggeredEvent>>();
        let mut reader = events.get_cursor();
        assert_eq!(
            reader.read(events).count(),
            1,
            "Auction should trigger upon Enigmatic arrival."
        );
    }

    #[test]
    fn test_bidding_subtracts_resources() {
        let mut app = bevy_app::App::new();
        let res = ColonyResources {
            stone: 1000.0,
            ..Default::default()
        };
        app.insert_resource(res);
        app.add_event::<PlaceBidEvent>();
        app.add_systems(bevy_app::Update, handle_blind_auction_bids);

        // Act: Bid placed
        app.world_mut().send_event(PlaceBidEvent {
            amount: 500.0,
            resource_type: ResourceType::Stone,
        });
        app.update();

        // Assert: Resources decremented
        let resources = app.world().resource::<ColonyResources>();
        assert_eq!(
            resources.stone, 500.0,
            "Bidding should deduct resources immediately."
        );
    }

    #[test]
    fn test_vault_opening_outcome() {
        let mut app = bevy_app::App::new();
        app.add_event::<VaultOpenedEvent>();
        app.add_event::<TemporalAnomalyEvent>();
        app.add_systems(bevy_app::Update, process_vault_outcome);

        // Act: Vault opened with negative outcome generated
        app.world_mut().send_event(VaultOpenedEvent {
            outcome: VaultOutcome::CatastrophicAnomaly,
        });
        app.update();

        // Assert: Anomaly spawns
        let events = app.world().resource::<Events<TemporalAnomalyEvent>>();
        let mut reader = events.get_cursor();
        assert_eq!(
            reader.read(events).count(),
            1,
            "Catastrophic outcome should spawn anomaly event."
        );
    }
}
