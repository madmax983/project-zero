use crate::layer1::economy::ColonyResources;
use crate::layer1::pop::Pop;
use crate::layer1::psychology::traits::{Trait, Traits};
use crate::layer3::market::ephemeral_market::{EphemeralMarket, MarketTradeEvent};
use bevy::prelude::*;

#[derive(Resource, Default)]
pub struct SlushFund(pub u32);

#[derive(Component)]
pub struct RepoFleet {
    pub target_colony: Entity,
}

#[derive(Event)]
pub struct HackSlushFundEvent {
    pub hacker_entity: Entity,
    pub colony_entity: Entity,
}

pub fn accumulate_phantom_tax_system(
    mut slush_fund: ResMut<SlushFund>,
    mut trade_events: EventReader<MarketTradeEvent>,
    markets: Query<&EphemeralMarket>,
) {
    for event in trade_events.read() {
        if let Ok(market) = markets.get(event.market_entity) {
            if event.trade_index < market.trades.len() {
                let trade = market.trades[event.trade_index];
                let (_req_res, req_amount, _off_res, _off_amount) = trade;
                // Accumulate tax based on requested amount. Assuming 1%
                let tax = (req_amount * 0.01) as u32;
                slush_fund.0 += tax;
            }
        }
    }
}

pub fn execute_hack_system(
    mut commands: Commands,
    mut hack_events: EventReader<HackSlushFundEvent>,
    mut slush_fund: ResMut<SlushFund>,
    mut colony_resources: ResMut<ColonyResources>,
    pops: Query<&Traits, With<Pop>>,
) {
    for event in hack_events.read() {
        if let Ok(traits) = pops.get(event.hacker_entity) {
            if traits.has(Trait::Hacker) {
                // We don't need to get specific entity's resource as it's a global Resource
                colony_resources.credits += slush_fund.0 as f32;
                slush_fund.0 = 0;

                // Spawn Repo Fleet
                commands.spawn((
                    RepoFleet {
                        target_colony: event.colony_entity,
                    },
                    Transform::default(),
                ));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::resources::ResourceType;

    #[test]
    fn test_market_trades_accumulate_phantom_tax() {
        let mut app = App::new();
        app.add_systems(Update, accumulate_phantom_tax_system);
        app.insert_resource(SlushFund::default());
        app.add_event::<MarketTradeEvent>();

        let market_entity = app
            .world_mut()
            .spawn(EphemeralMarket {
                location: Entity::PLACEHOLDER,
                ticks_remaining: 100,
                trades: vec![(ResourceType::Food, 1000.0, ResourceType::Metal, 50.0)],
            })
            .id();

        app.world_mut().send_event(MarketTradeEvent {
            market_entity,
            buyer_entity: Entity::PLACEHOLDER,
            trade_index: 0,
        });

        app.update();

        let slush_fund = app.world().resource::<SlushFund>();
        assert_eq!(slush_fund.0, 10);
    }

    #[test]
    fn test_hacker_pop_can_drain_slush_fund() {
        let mut app = App::new();
        app.add_systems(Update, execute_hack_system);
        app.insert_resource(SlushFund(5000));
        app.add_event::<HackSlushFundEvent>();

        let mut traits = Traits::default();
        traits.add(Trait::Hacker);
        let hacker_pop = app.world_mut().spawn((Pop, traits)).id();
        let colony = Entity::PLACEHOLDER;
        app.insert_resource(ColonyResources::default());

        app.world_mut().send_event(HackSlushFundEvent {
            hacker_entity: hacker_pop,
            colony_entity: colony,
        });

        app.update();

        let slush_fund = app.world().resource::<SlushFund>();
        assert_eq!(slush_fund.0, 0);

        let resources = app.world().resource::<ColonyResources>();
        assert_eq!(resources.credits, 5000.0);
    }

    #[test]
    fn test_hack_triggers_repo_fleet_spawn() {
        let mut app = App::new();
        app.add_systems(Update, execute_hack_system);
        app.insert_resource(SlushFund(5000));
        app.add_event::<HackSlushFundEvent>();
        app.insert_resource(ColonyResources::default());

        let mut traits = Traits::default();
        traits.add(Trait::Hacker);
        let hacker_pop = app.world_mut().spawn((Pop, traits)).id();
        let colony = Entity::PLACEHOLDER;

        app.world_mut().send_event(HackSlushFundEvent {
            hacker_entity: hacker_pop,
            colony_entity: colony,
        });

        app.update();

        let mut query = app.world_mut().query::<&RepoFleet>();
        let repo_fleets = query.iter(app.world()).count();
        assert_eq!(repo_fleets, 1);
        let fleet = query.iter(app.world()).next().unwrap();
        assert_eq!(fleet.target_colony, colony);
    }
}
