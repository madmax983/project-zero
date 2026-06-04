use bevy::prelude::*;
use scale::layer1::entities::pop::Pop;
use scale::layer1::needs::Needs;
use scale::layer1::social::secret_societies::{
    secret_society_formation_system, society_action_system, SecretSociety, SecretSocietyMember,
    SocietyAction, SocietyType,
};
use scale::layer1::traits::{Trait, Traits};

#[test]
fn test_secret_society_formation() {
    let mut app = App::new();
    app.add_systems(Update, secret_society_formation_system);

    // Spawn 3 pops with the EngineCultist trait and low morale
    for _ in 0..3 {
        let mut traits = Traits::default();
        traits.add(Trait::EngineCultist);

        // Morale is the average of needs. Let's make them all low.
        // Morale < 0.40 -> needs average < 0.40
        app.world_mut().spawn((
            Pop,
            traits,
            Needs {
                hunger: 0.2,
                rest: 0.2,
                leisure: 0.2,
                hygiene: 0.2,
                isolation: 0.0,
            },
        ));
    }

    app.update();

    // A secret society should have formed
    let mut society_query = app.world_mut().query::<&SecretSociety>();
    let societies: Vec<_> = society_query.iter(app.world()).collect();

    assert_eq!(societies.len(), 1);
    assert!(societies[0].is_hidden);
}

#[test]
fn test_society_performs_hidden_action() {
    let mut app = App::new();
    app.add_event::<SocietyAction>();
    app.add_systems(Update, society_action_system);

    // Create a society and members
    let society_id = app
        .world_mut()
        .spawn(SecretSociety {
            society_type: SocietyType::MachineCult,
            is_hidden: true,
            action_timer: Timer::from_seconds(1.0, TimerMode::Once),
        })
        .id();

    app.world_mut()
        .spawn((Pop, SecretSocietyMember { society_id }));

    // Fast forward time to trigger action
    let mut time: Time = Time::default();
    time.advance_by(std::time::Duration::from_secs(2));
    app.world_mut().insert_resource(time);

    app.update();

    // Society should have performed an action (e.g., hoarding resources or buffing a machine)
    // We test for an event being fired
    let action_events = app.world().resource::<Events<SocietyAction>>();
    assert!(!action_events.is_empty());
}

#[test]
fn test_society_discovery() {
    // Tests that a society can be uncovered by police/inspection, changing `is_hidden` to false
    use scale::layer1::core::chronicle::AddChronicleEvent;
    use scale::layer1::core::integration::secret_society_discovery_bridge_system;
    use scale::layer1::law::justice::Inmate;

    let mut app = App::new();
    app.add_event::<AddChronicleEvent>();
    app.add_systems(Update, secret_society_discovery_bridge_system);

    // Create a hidden society
    let society_id = app
        .world_mut()
        .spawn(SecretSociety {
            society_type: SocietyType::MachineCult,
            is_hidden: true,
            action_timer: Timer::from_seconds(1.0, TimerMode::Once),
        })
        .id();

    let pop_id = app
        .world_mut()
        .spawn((Pop, SecretSocietyMember { society_id }))
        .id();

    // Simulate arrest
    app.world_mut().entity_mut(pop_id).insert(Inmate {
        sentence_ticks: 100,
    });

    app.update();

    // The society should have been despawned
    assert!(app.world().get::<SecretSociety>(society_id).is_none());

    // The pop should no longer be a member
    assert!(app.world().get::<SecretSocietyMember>(pop_id).is_none());

    // A chronicle event should have been emitted
    let chronicle_events = app.world().resource::<Events<AddChronicleEvent>>();
    assert_eq!(chronicle_events.len(), 1);
}

#[test]
fn test_society_suspicion() {
    use scale::layer1::core::integration::society_suspicion_bridge_system;
    use scale::layer1::energy::PowerConsumer;
    use scale::layer1::law::predictive_policing::{PredictionConfig, PredictiveModel, Suspect};

    let mut app = App::new();
    app.insert_resource(PredictionConfig {
        enabled: true,
        threshold: 0.8,
    });

    // Spawn an active predictive model
    app.world_mut().spawn((
        PredictiveModel,
        PowerConsumer {
            active: true,
            demand: 10.0,
        },
    ));

    app.add_systems(Update, society_suspicion_bridge_system);

    // Create a hidden society
    let society_id = app
        .world_mut()
        .spawn(SecretSociety {
            society_type: SocietyType::MachineCult,
            is_hidden: true,
            action_timer: Timer::from_seconds(1.0, TimerMode::Once),
        })
        .id();

    let pop_id = app
        .world_mut()
        .spawn((Pop, SecretSocietyMember { society_id }))
        .id();

    app.update();

    // The pop should be marked as a suspect
    let suspect = app
        .world()
        .get::<Suspect>(pop_id)
        .expect("Pop should be a suspect");
    assert_eq!(suspect.predicted_crime, "Secret Society Conspiracy");
    assert!(suspect.probability > 0.8);
}
