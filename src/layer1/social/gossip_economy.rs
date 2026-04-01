use bevy_ecs::prelude::*;
use crate::layer1::social::morale::Morale;
use crate::layer1::social::rumor::RumorTopic;

#[derive(Event)]
pub struct GossipEvent {
    pub pop: Entity,
    pub rumor: RumorTopic,
}

#[derive(Resource, Default)]
pub struct IntelTokens(pub u32);

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
pub struct Gossiping(pub u32);

pub fn process_gossip(
    mut commands: Commands,
    mut gossip_events: EventReader<GossipEvent>,
    mut intel: ResMut<IntelTokens>,
    mut pops: Query<(&mut Morale,)>,
) {
    for event in gossip_events.read() {
        if let Ok((mut morale,)) = pops.get_mut(event.pop) {
            match event.rumor {
                RumorTopic::EventNews(_) => { // General
                    intel.0 += 1;
                }
                RumorTopic::DoomProphecy => { // Dark Secret
                    intel.0 += 2;
                    morale.value -= 10.0;
                }
                _ => { // Other rumors might behave like general
                    intel.0 += 1;
                }
            }

            // Add Gossiping component to temporarily halt work
            commands.entity(event.pop).insert(Gossiping(1));
        }
    }
}

pub fn spend_intel(
    mut purchase_events: EventReader<BrokerPurchaseEvent>,
    mut intel: ResMut<IntelTokens>,
) {
    for event in purchase_events.read() {
        if intel.0 >= event.cost {
            intel.0 -= event.cost;
            // Broker logic executed here
        }
    }
}

pub fn decrement_gossiping_system(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Gossiping)>,
) {
    for (entity, mut gossiping) in &mut query {
        if gossiping.0 > 0 {
            gossiping.0 -= 1;
        }
        if gossiping.0 == 0 {
            commands.entity(entity).remove::<Gossiping>();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use crate::layer1::pop::Pop;
    use crate::layer1::execution::general_work::calculate_work_amount;
    use crate::layer1::designation::DesignationType;

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
        let pop = app.world_mut().spawn((Pop, Morale { value: 50.0, modifiers: vec![] })).id();
        app.world_mut().resource_mut::<Events<GossipEvent>>().send(GossipEvent { pop, rumor: RumorTopic::EventNews("Juicy Gossip".to_string()) });

        app.update();

        assert_eq!(app.world().resource::<IntelTokens>().0, 1);
    }

    #[test]
    fn test_gossiping_reduces_productivity() {
        let mut app = setup_app();
        let pop = app.world_mut().spawn((
            Pop,
            Morale { value: 50.0, modifiers: vec![] },
            Gossiping(1),
        )).id();

        app.world_mut().resource_mut::<Events<GossipEvent>>().send(GossipEvent { pop, rumor: RumorTopic::EventNews("Juicy Gossip".to_string()) });

        app.update();

        let work = calculate_work_amount(app.world(), pop, DesignationType::Mine, None, 1.0, 1.0, 1.0);
        assert_eq!(work, 0.0, "Pop should not progress work while gossiping");
    }

    #[test]
    fn test_dark_secret_gossip_lowers_morale() {
        let mut app = setup_app();
        let pop = app.world_mut().spawn((
            Pop,
            Morale { value: 50.0, modifiers: vec![] },
        )).id();

        app.world_mut().resource_mut::<Events<GossipEvent>>().send(GossipEvent { pop, rumor: RumorTopic::DoomProphecy });

        app.update();

        let morale = app.world().get::<Morale>(pop).unwrap();
        assert!(morale.value < 50.0, "Dark secret rumor should lower morale");
        assert_eq!(app.world().resource::<IntelTokens>().0, 2, "Dark secrets generate more intel");
    }

    #[test]
    fn test_spend_intel_at_broker() {
        let mut app = setup_app();
        app.world_mut().resource_mut::<IntelTokens>().0 = 10;

        app.world_mut().resource_mut::<Events<BrokerPurchaseEvent>>().send(BrokerPurchaseEvent { item: BrokerItem::RareTech, cost: 5 });

        app.update();

        assert_eq!(app.world().resource::<IntelTokens>().0, 5, "Intel tokens should be deducted");
    }
}
