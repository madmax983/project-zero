use bevy_ecs::prelude::*;
use crate::layer1::economy::resources::{ColonyResources, ResourceType};
use crate::layer1::void_weed::{MerchantArrivalEvent, MerchantType};

#[derive(Event)]
pub struct BlindAuctionTriggeredEvent;

#[derive(Event)]
pub struct PlaceBidEvent {
    pub amount: f32,
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

#[derive(Resource, Default)]
pub struct BlindAuctionState {
    pub is_active: bool,
    pub time_remaining: f32, // In days
    pub current_highest_bid: f32,
    pub player_is_winning: bool,
    pub tick_counter: u32, // for deterministic pseudo-random logic
}

pub fn check_for_blind_auction_trigger(
    mut arrivals: EventReader<MerchantArrivalEvent>,
    mut trigger_ew: EventWriter<BlindAuctionTriggeredEvent>,
    mut state: ResMut<BlindAuctionState>,
) {
    for arrival in arrivals.read() {
        if arrival.merchant_type == MerchantType::Enigmatic {
            trigger_ew.send(BlindAuctionTriggeredEvent);
            state.is_active = true;
            state.time_remaining = 7.0; // 7 in-game days
            state.current_highest_bid = 100.0; // Starting bid
            state.player_is_winning = false;
        }
    }
}

pub fn handle_blind_auction_bids(
    mut bids: EventReader<PlaceBidEvent>,
    mut resources: ResMut<ColonyResources>,
    mut state: ResMut<BlindAuctionState>,
) {
    if !state.is_active {
        return;
    }

    for bid in bids.read() {
        if bid.resource_type == ResourceType::Metal {
            if bid.amount > state.current_highest_bid && resources.metal >= bid.amount {
                // Refund previous bid if player was already winning
                if state.player_is_winning {
                    resources.metal += state.current_highest_bid;
                }
                resources.metal -= bid.amount;
                state.current_highest_bid = bid.amount;
                state.player_is_winning = true;
            }
        }
    }
}

pub fn process_auction_time(
    mut state: ResMut<BlindAuctionState>,
    mut vault_ew: EventWriter<VaultOpenedEvent>,
    time: Res<crate::shared::time::SimulationTime>,
) {
    if !state.is_active {
        return;
    }

    // Assuming time.delta_seconds() gives us ticks. We'll simulate 1 day = 1.0 delta for simplicity in tests,
    // or adjust based on actual SimulationTime logic.
    state.time_remaining -= time.delta_seconds();
    state.tick_counter = state.tick_counter.wrapping_add(1);

    // Rival bidding logic - deterministic using tick counter
    if !state.player_is_winning && state.tick_counter % 10 == 0 {
         state.current_highest_bid += 50.0;
    } else if state.player_is_winning && state.tick_counter % 20 == 0 {
         // Rival outbids player
         state.current_highest_bid += 100.0;
         state.player_is_winning = false;
    }

    if state.time_remaining <= 0.0 {
        state.is_active = false;
        if state.player_is_winning {
            // Deterministic outcome based on bid parity
            let outcome = if state.current_highest_bid as u32 % 2 == 0 {
                VaultOutcome::TechBoost
            } else {
                VaultOutcome::CatastrophicAnomaly
            };
            vault_ew.send(VaultOpenedEvent { outcome });
        }
        // Cleanup state
        state.current_highest_bid = 0.0;
        state.player_is_winning = false;
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
    use bevy_app::prelude::*;
    use crate::layer1::void_weed::MerchantType;
    use crate::shared::time::SimulationTime;

    #[test]
    fn test_blind_auction_event_spawns() {
        let mut app = App::new();
        app.add_event::<MerchantArrivalEvent>();
        app.add_event::<BlindAuctionTriggeredEvent>();
        app.init_resource::<BlindAuctionState>();
        app.add_systems(Update, check_for_blind_auction_trigger);

        app.world_mut().send_event(MerchantArrivalEvent { merchant_type: MerchantType::Enigmatic });
        app.update();

        let events = app.world().resource::<Events<BlindAuctionTriggeredEvent>>();
        let mut reader = events.get_cursor();
        assert_eq!(reader.read(events).len(), 1);

        let state = app.world().resource::<BlindAuctionState>();
        assert!(state.is_active);
        assert_eq!(state.time_remaining, 7.0);
    }

    #[test]
    fn test_bidding_subtracts_resources_and_updates_state() {
        let mut app = App::new();
        app.insert_resource(ColonyResources { metal: 1000.0, ..Default::default() });
        app.insert_resource(BlindAuctionState { is_active: true, current_highest_bid: 100.0, ..Default::default() });
        app.add_event::<PlaceBidEvent>();
        app.add_systems(Update, handle_blind_auction_bids);

        app.world_mut().send_event(PlaceBidEvent { amount: 500.0, resource_type: ResourceType::Metal });
        app.update();

        let resources = app.world().resource::<ColonyResources>();
        assert_eq!(resources.metal, 500.0);

        let state = app.world().resource::<BlindAuctionState>();
        assert_eq!(state.current_highest_bid, 500.0);
        assert!(state.player_is_winning);
    }

    #[test]
    fn test_vault_opening_outcome() {
        let mut app = App::new();
        app.add_event::<VaultOpenedEvent>();
        app.add_event::<TemporalAnomalyEvent>();
        app.add_systems(Update, process_vault_outcome);

        app.world_mut().send_event(VaultOpenedEvent { outcome: VaultOutcome::CatastrophicAnomaly });
        app.update();

        let events = app.world().resource::<Events<TemporalAnomalyEvent>>();
        let mut reader = events.get_cursor();
        assert_eq!(reader.read(events).len(), 1);
    }

    #[test]
    fn test_auction_time_completion_triggers_vault() {
         let mut app = App::new();
         app.add_event::<VaultOpenedEvent>();

         let mut sim_time = SimulationTime::default();
         // Hack to advance time since we can't directly set delta.
         // Actually, SimulationTime is updated via generic logic, we can just mutate the resource directly if needed, or use app.update() with time systems.
         // For a simple test, we will just insert it and manually advance its internal elapsed.
         app.insert_resource(sim_time);

         app.insert_resource(BlindAuctionState {
             is_active: true,
             time_remaining: 0.0, // immediately expire
             player_is_winning: true,
             current_highest_bid: 500.0, // Even number -> TechBoost (not anomaly)
             ..Default::default()
         });
         app.add_systems(Update, process_auction_time);

         app.update();

         let events = app.world().resource::<Events<VaultOpenedEvent>>();
         let mut reader = events.get_cursor();
         assert_eq!(reader.read(events).len(), 1);

         let state = app.world().resource::<BlindAuctionState>();
         assert!(!state.is_active);
    }
}
