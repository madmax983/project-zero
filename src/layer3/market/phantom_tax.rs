use crate::layer1::economy::resources::ColonyResources;
use crate::layer1::psychology::traits::{Trait, Traits};
use crate::layer3::market::ephemeral_market::MarketTradeEvent;
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

pub const PHANTOM_TAX_RATE: u32 = 1;

pub fn accumulate_phantom_tax_system(
    mut slush_fund: ResMut<SlushFund>,
    mut trade_events: EventReader<MarketTradeEvent>,
) {
    for _event in trade_events.read() {
        slush_fund.0 = slush_fund.0.saturating_add(PHANTOM_TAX_RATE);
    }
}

pub fn execute_hack_system(
    mut commands: Commands,
    mut hack_events: EventReader<HackSlushFundEvent>,
    mut slush_fund: ResMut<SlushFund>,
    mut colony_resources: ResMut<ColonyResources>,
    query_traits: Query<&Traits>,
) {
    for event in hack_events.read() {
        if let Ok(traits) = query_traits.get(event.hacker_entity) {
            if traits.has(Trait::Hacker) {
                colony_resources.add_credits(slush_fund.0 as f32);
                slush_fund.0 = 0;

                commands.spawn((
                    Name::new("Repo Fleet"),
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
    use crate::layer1::economy::resources::ColonyResources;
    use crate::layer1::psychology::traits::{Trait, Traits};
    use crate::layer3::market::ephemeral_market::MarketTradeEvent;

    fn setup_app() -> App {
        let mut app = App::new();
        app.insert_resource(SlushFund::default());
        app.insert_resource(ColonyResources::default());
        app.add_event::<MarketTradeEvent>();
        app.add_event::<HackSlushFundEvent>();
        app.add_systems(Update, (accumulate_phantom_tax_system, execute_hack_system));
        app
    }

    #[test]
    fn test_market_trades_accumulate_phantom_tax() {
        let mut app = setup_app();

        // Setup Galactic Market and SlushFund resource
        // Execute a trade using MarketTradeEvent
        app.world_mut().send_event(MarketTradeEvent {
            market_entity: Entity::PLACEHOLDER,
            buyer_entity: Entity::PLACEHOLDER,
            trade_index: 0,
        });

        app.update();

        // Assert Slush Fund increases appropriately
        let slush_fund = app.world().resource::<SlushFund>();
        assert_eq!(slush_fund.0, 1);
    }

    #[test]
    fn test_hacker_pop_can_drain_slush_fund() {
        let mut app = setup_app();

        let mut traits = Traits::default();
        traits.add(Trait::Hacker);
        let hacker_entity = app.world_mut().spawn(traits).id();
        let colony_entity = app.world_mut().spawn_empty().id();

        app.world_mut().resource_mut::<SlushFund>().0 = 100;

        app.world_mut().send_event(HackSlushFundEvent {
            hacker_entity,
            colony_entity,
        });

        app.update();

        let slush_fund = app.world().resource::<SlushFund>();
        assert_eq!(slush_fund.0, 0);

        let colony_resources = app.world().resource::<ColonyResources>();
        assert_eq!(colony_resources.credits, 100.0);
    }

    #[test]
    fn test_non_hacker_pop_cannot_drain_slush_fund() {
        let mut app = setup_app();

        let traits = Traits::default();
        let hacker_entity = app.world_mut().spawn(traits).id();
        let colony_entity = app.world_mut().spawn_empty().id();

        app.world_mut().resource_mut::<SlushFund>().0 = 100;

        app.world_mut().send_event(HackSlushFundEvent {
            hacker_entity,
            colony_entity,
        });

        app.update();

        let slush_fund = app.world().resource::<SlushFund>();
        assert_eq!(slush_fund.0, 100); // Unchanged

        let colony_resources = app.world().resource::<ColonyResources>();
        assert_eq!(colony_resources.credits, 0.0);
    }

    #[test]
    fn test_hack_triggers_repo_fleet_spawn() {
        let mut app = setup_app();

        let mut traits = Traits::default();
        traits.add(Trait::Hacker);
        let hacker_entity = app.world_mut().spawn(traits).id();
        let colony_entity = app.world_mut().spawn_empty().id();

        app.world_mut().resource_mut::<SlushFund>().0 = 100;

        app.world_mut().send_event(HackSlushFundEvent {
            hacker_entity,
            colony_entity,
        });

        app.update();

        let repo_fleets = app
            .world_mut()
            .query::<&RepoFleet>()
            .iter(app.world())
            .count();
        assert_eq!(repo_fleets, 1);
    }
}
