use crate::layer1::core::chronicle::{AddChronicleEvent, EventImportance};
use crate::layer1::economy::resources::{ColonyResources, ResourceType};
use crate::layer1::environment::disasters::DisasterType;
use crate::layer1::tech::Tech;
use bevy::time::Time;
use bevy_app::Plugin;
use bevy_ecs::prelude::*;

#[derive(Clone, Debug, PartialEq)]
pub enum VaultContents {
    Tech(Tech),
    Disaster(DisasterType),
    Resources(f32),
}

impl VaultContents {
    #[must_use]
    pub fn random() -> Self {
        // Simplified random for now
        Self::Resources(100.0)
    }
}

pub struct BlindAuctionPlugin;

impl Plugin for BlindAuctionPlugin {
    fn build(&self, app: &mut bevy_app::App) {
        app.add_event::<TriggerBlindAuction>()
            .add_event::<SubmitBid>()
            .add_event::<VaultOpened>()
            .add_systems(
                bevy_app::Update,
                (
                    generate_blind_auction_system,
                    process_bids_system,
                    resolve_auction_system,
                    update_auction_time_system,
                ),
            );
    }
}

#[derive(Component)]
pub struct BlindAuction {
    pub vault_contents: VaultContents,
    pub current_highest_bid: f32,
    pub winning_faction: Option<Entity>,
    pub previous_resource_type: Option<ResourceType>,
    pub time_remaining: f32,
}

#[derive(Event)]
pub struct TriggerBlindAuction;

#[derive(Event)]
pub struct SubmitBid {
    pub auction: Entity, // The target auction
    pub faction: Entity,
    pub resource_type: ResourceType,
    pub amount: f32,
}

#[derive(Event)]
pub struct VaultOpened {
    pub faction: Entity,
    pub contents: VaultContents,
}

pub fn generate_blind_auction_system(
    mut commands: Commands,
    mut event_reader: EventReader<TriggerBlindAuction>,
) {
    for _ in event_reader.read() {
        let _ = commands.spawn(BlindAuction {
            vault_contents: VaultContents::random(),
            current_highest_bid: 0.0,
            winning_faction: None,
            previous_resource_type: None,
            time_remaining: 10.0,
        });
    }
}

pub fn process_bids_system(
    mut auction_query: Query<&mut BlindAuction>,
    mut bid_reader: EventReader<SubmitBid>,
    mut resources: ResMut<ColonyResources>,
) {
    for bid in bid_reader.read() {
        if let Ok(mut auction) = auction_query.get_mut(bid.auction) {
            if bid.amount > auction.current_highest_bid
                && resources.try_consume(bid.resource_type, bid.amount)
            {
                // Refund previous bidder if there was one
                if let (Some(_previous_faction), Some(prev_resource_type)) =
                    (auction.winning_faction, &auction.previous_resource_type)
                {
                    resources.add_resource(prev_resource_type, auction.current_highest_bid);
                }

                auction.current_highest_bid = bid.amount;
                auction.winning_faction = Some(bid.faction);
                auction.previous_resource_type = Some(bid.resource_type);
            }
        }
    }
}

pub fn resolve_auction_system(
    mut commands: Commands,
    mut auction_query: Query<(Entity, &BlindAuction)>,
    mut events: EventWriter<VaultOpened>,
    mut chronicle_events: EventWriter<AddChronicleEvent>,
) {
    for (entity, auction) in auction_query.iter_mut() {
        if auction.time_remaining <= 0.0 {
            if let Some(winner) = auction.winning_faction {
                events.send(VaultOpened {
                    faction: winner,
                    contents: auction.vault_contents.clone(),
                });

                chronicle_events.send(AddChronicleEvent {
                    text: "A blind auction was won.".to_string(),
                    importance: EventImportance::Standard,
                });
            }
            if let Some(mut entity_commands) = commands.get_entity(entity) {
                entity_commands.despawn();
            }
        }
    }
}

pub fn update_auction_time_system(mut auction_query: Query<&mut BlindAuction>, time: Res<Time>) {
    for mut auction in auction_query.iter_mut() {
        auction.time_remaining -= time.delta_secs();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_blind_auction_event_generation() {
        let mut app = bevy_app::App::new();
        app.add_event::<TriggerBlindAuction>();
        app.add_systems(bevy_app::Update, generate_blind_auction_system);

        app.world_mut().send_event(TriggerBlindAuction);
        app.update();

        let auctions = app
            .world_mut()
            .query::<&BlindAuction>()
            .iter(app.world())
            .collect::<Vec<_>>();
        assert_eq!(auctions.len(), 1);
    }

    #[test]
    fn test_blind_auction_bidding() {
        let mut app = bevy_app::App::new();
        app.add_event::<SubmitBid>();
        app.add_systems(bevy_app::Update, process_bids_system);

        app.world_mut()
            .insert_resource(ColonyResources::default().with_metal(100.0));

        let faction = app.world_mut().spawn_empty().id();
        let auction_id = app
            .world_mut()
            .spawn(BlindAuction {
                vault_contents: VaultContents::random(),
                current_highest_bid: 10.0,
                winning_faction: None,
                previous_resource_type: None,
                time_remaining: 10.0,
            })
            .id();

        app.world_mut().send_event(SubmitBid {
            auction: auction_id,
            faction,
            resource_type: ResourceType::Metal,
            amount: 50.0,
        });

        app.update();

        let auction = app.world_mut().query::<&BlindAuction>().single(app.world());
        assert_eq!(auction.current_highest_bid, 50.0);
        assert_eq!(auction.winning_faction, Some(faction));

        let res = app.world_mut().resource::<ColonyResources>();
        assert_eq!(res.metal, 50.0);
    }

    #[test]
    fn test_blind_auction_resolution_positive() {
        let mut app = bevy_app::App::new();
        app.add_event::<VaultOpened>();
        app.add_event::<AddChronicleEvent>();
        app.add_systems(bevy_app::Update, resolve_auction_system);

        let faction = app.world_mut().spawn_empty().id();
        let auction_entity = app
            .world_mut()
            .spawn(BlindAuction {
                vault_contents: VaultContents::Tech(Tech::Terraforming),
                current_highest_bid: 50.0,
                winning_faction: Some(faction),
                previous_resource_type: Some(ResourceType::Metal),
                time_remaining: 0.0,
            })
            .id();

        app.update();

        assert!(app.world().get_entity(auction_entity).is_err());

        let opened_events = app.world_mut().resource::<Events<VaultOpened>>();
        assert_eq!(opened_events.get_cursor().len(opened_events), 1);

        let chronicle_events = app.world_mut().resource::<Events<AddChronicleEvent>>();
        assert_eq!(chronicle_events.get_cursor().len(chronicle_events), 1);
    }

    #[test]
    fn test_blind_auction_resolution_negative() {
        let mut app = bevy_app::App::new();
        app.add_event::<VaultOpened>();
        app.add_event::<AddChronicleEvent>();
        app.add_systems(bevy_app::Update, resolve_auction_system);

        let faction = app.world_mut().spawn_empty().id();
        let auction_entity = app
            .world_mut()
            .spawn(BlindAuction {
                vault_contents: VaultContents::Disaster(DisasterType::ViolentUprising),
                current_highest_bid: 50.0,
                winning_faction: Some(faction),
                previous_resource_type: Some(ResourceType::Metal),
                time_remaining: 0.0,
            })
            .id();

        app.update();

        assert!(app.world().get_entity(auction_entity).is_err());

        let opened_events = app.world_mut().resource::<Events<VaultOpened>>();
        assert_eq!(opened_events.get_cursor().len(opened_events), 1);
    }
}
