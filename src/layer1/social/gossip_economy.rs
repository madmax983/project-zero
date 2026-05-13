use crate::layer1::social::morale::Morale;
use crate::layer1::social::rumor::RumorTopic;
use bevy_ecs::prelude::*;

#[derive(Event)]
pub struct GossipEvent {
    pub pop: Entity,
    pub rumor: RumorTopic,
}

pub const MAX_INTEL_TOKENS: u32 = 100;

#[derive(Resource, Default)]
pub struct IntelTokens(pub u32);

pub fn intel_decay_system(
    mut intel: ResMut<IntelTokens>,
    time: Res<crate::shared::time::SimulationTime>,
) {
    if time.tick > 0 && time.tick.is_multiple_of(100) && intel.0 > 0 {
        intel.0 -= 1;
    }
}

#[derive(Event)]
pub struct BrokerPurchaseEvent {
    pub item: BrokerItem,
    pub cost: u32,
}

pub enum BrokerItem {
    RareTech,
    RelationsBoost,
}

#[derive(Component)]
pub struct Broker;

pub fn process_gossip(
    mut gossip_events: EventReader<GossipEvent>,
    mut intel: ResMut<IntelTokens>,
    mut pops: Query<(&mut Morale,)>,
) {
    for event in gossip_events.read() {
        if let Ok((mut morale,)) = pops.get_mut(event.pop) {
            match event.rumor {
                RumorTopic::EventNews(_) => {
                    // General
                    intel.0 = (intel.0 + 1).min(MAX_INTEL_TOKENS);
                }
                RumorTopic::DoomProphecy => {
                    // Dark Secret
                    intel.0 = (intel.0 + 2).min(MAX_INTEL_TOKENS);
                    morale.value -= 10.0;
                }
                _ => {
                    // Other rumors might behave like general
                    intel.0 = (intel.0 + 1).min(MAX_INTEL_TOKENS);
                }
            }
        }
    }
}

pub fn spawn_broker_system(
    mut commands: Commands,
    time: Res<crate::shared::time::SimulationTime>,
    brokers: Query<(), With<Broker>>,
) {
    // Only spawn if there is no broker, and spawn sporadically (e.g. at intervals)
    if brokers.is_empty() && time.tick > 0 && time.tick.is_multiple_of(500) {
        commands.spawn(Broker);
    }
}

pub fn spend_intel(
    mut purchase_events: EventReader<BrokerPurchaseEvent>,
    mut intel: ResMut<IntelTokens>,
    brokers: Query<(), With<Broker>>,
) {
    for event in purchase_events.read() {
        if !brokers.is_empty() && intel.0 >= event.cost {
            intel.0 -= event.cost;
            // Broker logic executed here
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::layer1::pop::Pop;
    use bevy::prelude::*;

    fn setup_app() -> App {
        let mut app = App::new();
        app.add_event::<GossipEvent>();
        app.add_event::<BrokerPurchaseEvent>();
        app.add_systems(Update, (process_gossip, spend_intel));
        app.insert_resource(IntelTokens(0));
        app
    }

    #[test]
    fn test_gossiping_generates_intel_tokens() {
        let mut app = setup_app();
        let pop = app
            .world_mut()
            .spawn((
                Pop,
                Morale {
                    value: 50.0,
                    modifiers: vec![],
                },
            ))
            .id();
        app.world_mut()
            .resource_mut::<Events<GossipEvent>>()
            .send(GossipEvent {
                pop,
                rumor: RumorTopic::EventNews("Juicy Gossip".to_string()),
            });

        app.update();

        assert_eq!(app.world().resource::<IntelTokens>().0, 1);
    }

    #[test]
    fn test_dark_secret_gossip_lowers_morale() {
        let mut app = setup_app();
        let pop = app
            .world_mut()
            .spawn((
                Pop,
                Morale {
                    value: 50.0,
                    modifiers: vec![],
                },
            ))
            .id();

        app.world_mut()
            .resource_mut::<Events<GossipEvent>>()
            .send(GossipEvent {
                pop,
                rumor: RumorTopic::DoomProphecy,
            });

        app.update();

        let morale = app.world().get::<Morale>(pop).unwrap();
        assert!(morale.value < 50.0, "Dark secret rumor should lower morale");
        assert_eq!(
            app.world().resource::<IntelTokens>().0,
            2,
            "Dark secrets generate more intel"
        );
    }

    #[test]
    fn test_intel_decays_over_time() {
        let mut app = setup_app();
        app.world_mut().resource_mut::<IntelTokens>().0 = 10;
        app.add_systems(Update, intel_decay_system);

        // Time is 0, no decay
        app.world_mut()
            .insert_resource(crate::shared::time::SimulationTime {
                tick: 0,
                ..Default::default()
            });
        app.update();
        assert_eq!(app.world().resource::<IntelTokens>().0, 10);

        // Time is 50, no decay
        app.world_mut()
            .insert_resource(crate::shared::time::SimulationTime {
                tick: 50,
                ..Default::default()
            });
        app.update();
        assert_eq!(app.world().resource::<IntelTokens>().0, 10);

        // Time is 100, decay
        app.world_mut()
            .insert_resource(crate::shared::time::SimulationTime {
                tick: 100,
                ..Default::default()
            });
        app.update();
        assert_eq!(app.world().resource::<IntelTokens>().0, 9);
    }

    #[test]
    fn test_intel_caps_at_max() {
        let mut app = setup_app();
        app.world_mut().resource_mut::<IntelTokens>().0 = MAX_INTEL_TOKENS;

        let pop = app
            .world_mut()
            .spawn((
                Pop,
                Morale {
                    value: 50.0,
                    modifiers: vec![],
                },
            ))
            .id();
        app.world_mut()
            .resource_mut::<Events<GossipEvent>>()
            .send(GossipEvent {
                pop,
                rumor: RumorTopic::DoomProphecy,
            });
        app.update();

        assert_eq!(app.world().resource::<IntelTokens>().0, MAX_INTEL_TOKENS);
    }

    #[test]
    fn test_spend_intel_at_broker() {
        let mut app = setup_app();
        app.world_mut().resource_mut::<IntelTokens>().0 = 10;
        app.world_mut().spawn(Broker);

        app.world_mut()
            .resource_mut::<Events<BrokerPurchaseEvent>>()
            .send(BrokerPurchaseEvent {
                item: BrokerItem::RareTech,
                cost: 5,
            });

        app.update();

        assert_eq!(
            app.world().resource::<IntelTokens>().0,
            5,
            "Intel tokens should be deducted"
        );
    }

    #[test]
    fn test_broker_spawn_and_spend_failure() {
        let mut app = setup_app();
        app.world_mut().resource_mut::<IntelTokens>().0 = 10;
        app.world_mut()
            .insert_resource(crate::shared::time::SimulationTime {
                tick: 0,
                ..Default::default()
            });
        app.add_systems(Update, spawn_broker_system);

        // Fail to spend intel since no broker exists
        app.world_mut()
            .resource_mut::<Events<BrokerPurchaseEvent>>()
            .send(BrokerPurchaseEvent {
                item: BrokerItem::RareTech,
                cost: 5,
            });

        app.update();
        assert_eq!(app.world().resource::<IntelTokens>().0, 10);

        // Spawn broker on tick 500
        app.world_mut()
            .insert_resource(crate::shared::time::SimulationTime {
                tick: 500,
                ..Default::default()
            });
        app.update();

        let broker_exists = app.world_mut().query::<&Broker>().iter(app.world()).count() > 0;
        assert!(broker_exists);

        app.world_mut()
            .resource_mut::<Events<BrokerPurchaseEvent>>()
            .send(BrokerPurchaseEvent {
                item: BrokerItem::RareTech,
                cost: 5,
            });

        app.update();
        assert_eq!(app.world().resource::<IntelTokens>().0, 5);
    }
}
