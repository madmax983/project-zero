use bevy_ecs::prelude::*;
use rand::Rng;

use crate::layer1::entities::pop::Pop;
use crate::layer1::social::unrest::Unrest;

use crate::layer1::mind::utility_types::AssignmentType;

use crate::layer1::specialization::JobTenure;

#[derive(Component)]
pub struct SecretBroker;

#[derive(Component)]
pub struct AtLocation(pub Entity);

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

pub fn generate_secrets_system(
    mut secrets: Option<ResMut<ColonySecrets>>,
    query: Query<&JobTenure, With<Pop>>,
) {
    let mut rng = rand::thread_rng();
    for tenure in query.iter() {
        if (tenure.get_ticks(AssignmentType::TavernVisitor) > 0
            || tenure.get_ticks(AssignmentType::Administrator) > 0)
            && rng.gen_bool(0.01)
        // 1% chance per tick
        {
            if let Some(sec) = secrets.as_mut() {
                sec.count += 1;
            }
        }
    }
}

pub fn execute_whisper_trade_system(
    mut events: EventReader<TradeSecretEvent>,
    mut secrets: Option<ResMut<ColonySecrets>>,
    mut paranoia: Option<ResMut<GlobalParanoia>>,
) {
    if let (Some(sec), Some(par)) = (secrets.as_mut(), paranoia.as_mut()) {
        for event in events.read() {
            if sec.count >= event.secret_value {
                sec.count -= event.secret_value;
                par.level += 5.0 * (event.secret_value as f32);
            }
        }
    }
}

pub fn paranoia_unrest_system(paranoia: Option<Res<GlobalParanoia>>, mut unrest: ResMut<Unrest>) {
    if let Some(par) = paranoia {
        if par.level > 50.0 {
            let unrest_increase = (par.level - 50.0) * 0.1;
            unrest.level += unrest_increase;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::social::Tavern;
    use crate::layer1::AssignmentType;
    use bevy::prelude::*;

    #[test]
    fn test_secret_generation_in_tavern() {
        let mut app = App::new();
        app.insert_resource(ColonySecrets::default());
        app.add_systems(Update, generate_secrets_system);

        let tavern = app.world_mut().spawn(Tavern::default()).id();
        app.world_mut().spawn((
            Pop,
            JobTenure::with_ticks(AssignmentType::TavernVisitor, 1000),
            AtLocation(tavern),
        ));

        // Act: Run the system enough times to trigger a secret discovery
        for _ in 0..1000 {
            app.update();
        }

        // Assert: The Pop or the Tavern should now hold a Secret component/resource
        let has_secret = app
            .world()
            .get_resource::<ColonySecrets>()
            .map_or(0, |res| res.count)
            > 0;
        assert!(has_secret, "A secret should have been generated.");
    }

    #[test]
    fn test_broker_trade_increases_paranoia() {
        let mut app = App::new();
        app.insert_resource(ColonySecrets { count: 1 });
        app.insert_resource(GlobalParanoia { level: 0.0 });
        app.add_event::<TradeSecretEvent>();
        app.add_systems(Update, execute_whisper_trade_system);

        let broker = app.world_mut().spawn(SecretBroker).id();

        app.world_mut().send_event(TradeSecretEvent {
            broker_entity: broker,
            secret_value: 1,
        });
        app.update();

        assert_eq!(app.world().resource::<ColonySecrets>().count, 0);
        assert!(app.world().resource::<GlobalParanoia>().level > 0.0);
    }

    #[test]
    fn test_high_paranoia_causes_unrest() {
        let mut app = App::new();
        app.insert_resource(GlobalParanoia { level: 90.0 });
        app.add_systems(Update, paranoia_unrest_system);

        app.world_mut().insert_resource(Unrest {
            level: 0.0,
            modifiers: vec![],
        });

        app.update();

        let unrest = app.world().get_resource::<Unrest>().unwrap().level;
        assert!(unrest > 0.0);
    }

    #[test]
    fn test_low_paranoia_no_unrest() {
        let mut app = App::new();
        app.insert_resource(GlobalParanoia { level: 40.0 });
        app.add_systems(Update, paranoia_unrest_system);

        app.world_mut().insert_resource(Unrest {
            level: 0.0,
            modifiers: vec![],
        });

        app.update();

        let unrest = app.world().get_resource::<Unrest>().unwrap().level;
        assert_eq!(
            unrest, 0.0,
            "Unrest should not increase when paranoia is below threshold"
        );
    }

    #[test]
    fn test_missing_paranoia_resource() {
        let mut app = App::new();
        // Do not insert GlobalParanoia resource
        app.add_systems(Update, paranoia_unrest_system);

        app.world_mut().insert_resource(Unrest {
            level: 0.0,
            modifiers: vec![],
        });

        app.update();

        let unrest = app.world().get_resource::<Unrest>().unwrap().level;
        assert_eq!(
            unrest, 0.0,
            "Unrest should not increase if paranoia resource is missing"
        );
    }
}
