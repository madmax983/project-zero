use bevy::prelude::*;
use rand::Rng;

use crate::layer1::utility_types::ActionType;

// --- Components and Resources ---
#[derive(Component)]
pub struct WhisperBroker;

#[derive(Component)]
pub struct Secret;

#[derive(Resource, Default)]
pub struct ColonySecrets {
    pub count: u32,
}

#[derive(Resource, Default)]
pub struct GlobalParanoia {
    pub level: f32,
}

#[derive(Event)]
pub struct TradeSecretEvent {
    pub broker_entity: Entity,
    pub secret_value: u32,
}

#[derive(Component)]
pub struct WhisperUnrestTracker {
    pub amount: f32,
}

#[derive(Resource, Default)]
pub struct WhisperTradeConfig {
    pub generation_chance: f64,
}

// --- Systems ---
pub fn generate_secrets_system(
    mut secrets: ResMut<ColonySecrets>,
    config: Option<Res<WhisperTradeConfig>>,
    query: Query<&crate::layer1::utility_types::PopAction, With<crate::layer1::Pop>>,
) {
    let mut rng = rand::thread_rng();
    let chance = config.map(|c| c.generation_chance).unwrap_or(0.01);

    // Instead of using CurrentAction (which does not exist) or JobTenure (which was unlinked),
    // we use `PopAction` which tracks the high-level goal, e.g. Socialize or Work.
    // This is safe and relies on core game components.
    for action in query.iter() {
        if (action.current == ActionType::Socialize || action.current == ActionType::Work)
            && rng.gen_bool(chance)
        {
            secrets.count += 1;
        }
    }
}

pub fn execute_whisper_trade_system(
    mut events: EventReader<TradeSecretEvent>,
    mut secrets: ResMut<ColonySecrets>,
    mut paranoia: ResMut<GlobalParanoia>,
    brokers: Query<&WhisperBroker>,
) {
    for event in events.read() {
        if brokers.get(event.broker_entity).is_ok() && secrets.count >= event.secret_value {
            secrets.count -= event.secret_value;
            paranoia.level += 5.0 * (event.secret_value as f32);
        }
    }
}

pub fn paranoia_unrest_system(
    paranoia: Res<GlobalParanoia>,
    mut query: Query<&mut WhisperUnrestTracker, With<crate::layer1::Pop>>,
) {
    if paranoia.level > 50.0 {
        let unrest_increase = (paranoia.level - 50.0) * 0.1;
        for mut tracker in query.iter_mut() {
            tracker.amount += unrest_increase;
        }
    }
}

pub fn decay_paranoia_system(
    mut paranoia: ResMut<GlobalParanoia>,
) {
    if paranoia.level > 0.0 {
        paranoia.level = (paranoia.level - 0.1).max(0.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use crate::layer1::Pop;
    use crate::layer1::utility_types::PopAction;

    #[test]
    fn test_secret_generation_in_tavern() {
        let mut app = App::new();
        app.init_resource::<ColonySecrets>();
        app.insert_resource(WhisperTradeConfig { generation_chance: 1.0 }); // Guaranteed for test
        app.add_systems(Update, generate_secrets_system);

        let mut action = PopAction::default();
        action.current = ActionType::Socialize; // Used in taverns

        let _pop = app.world_mut().spawn((
            Pop,
            action,
        )).id();

        app.update(); // Only 1 update needed with 100% chance

        let has_secret = app.world_mut().get_resource::<ColonySecrets>().map_or(0, |res| res.count) > 0;
        assert!(has_secret, "A secret should have been generated.");
    }

    #[test]
    fn test_broker_trade_increases_paranoia() {
        let mut app = App::new();
        app.insert_resource(ColonySecrets { count: 1 });
        app.insert_resource(GlobalParanoia { level: 0.0 });
        app.add_event::<TradeSecretEvent>();
        app.add_systems(Update, execute_whisper_trade_system);

        let broker = app.world_mut().spawn(WhisperBroker).id();

        app.world_mut().send_event(TradeSecretEvent {
            broker_entity: broker,
            secret_value: 1
        });
        app.update();

        assert_eq!(app.world().resource::<ColonySecrets>().count, 0);
        assert!(app.world().resource::<GlobalParanoia>().level > 0.0);
    }

    #[test]
    fn test_broker_trade_fails_if_no_broker() {
        let mut app = App::new();
        app.insert_resource(ColonySecrets { count: 1 });
        app.insert_resource(GlobalParanoia { level: 0.0 });
        app.add_event::<TradeSecretEvent>();
        app.add_systems(Update, execute_whisper_trade_system);

        app.world_mut().send_event(TradeSecretEvent {
            broker_entity: Entity::PLACEHOLDER,
            secret_value: 1
        });
        app.update();

        assert_eq!(app.world().resource::<ColonySecrets>().count, 1);
        assert_eq!(app.world().resource::<GlobalParanoia>().level, 0.0);
    }

    #[test]
    fn test_high_paranoia_causes_unrest() {
        let mut app = App::new();
        app.insert_resource(GlobalParanoia { level: 90.0 });
        app.add_systems(Update, paranoia_unrest_system);

        let pop = app.world_mut().spawn((Pop, WhisperUnrestTracker { amount: 0.0 })).id();

        app.update();

        let unrest = app.world().get::<WhisperUnrestTracker>(pop).unwrap().amount;
        assert!(unrest > 0.0);
    }

    #[test]
    fn test_paranoia_decay() {
        let mut app = App::new();
        app.insert_resource(GlobalParanoia { level: 10.0 });
        app.add_systems(Update, decay_paranoia_system);

        app.update();

        assert!(app.world().resource::<GlobalParanoia>().level < 10.0);
    }
}
