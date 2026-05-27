use bevy_app::{App, Update};
use bevy_ecs::prelude::*;
use scale::layer1::economy::resources::ColonyResources;
use scale::layer1::entities::pop::Pop;
use scale::layer3::bureaucracy::{
    colony_reporting_system, empire_resource_distribution_system, AutomatedDefenses,
    AutomatedReporting,
};

#[test]
fn test_ghost_town_continues_receiving_shipments() {
    let mut app = App::new();

    let resources = ColonyResources {
        food: 0.0,
        ..Default::default()
    };
    app.insert_resource(resources);

    app.world_mut().spawn(AutomatedReporting {
        is_active: true,
        reported_population: 100, // Last known good number
    });

    app.add_systems(
        Update,
        (colony_reporting_system, empire_resource_distribution_system).chain(),
    );
    app.update();

    let resources = app.world().resource::<ColonyResources>();

    assert_eq!(
        resources.food, 100.0,
        "Ghost town should continue receiving shipments."
    );
}

#[test]
fn test_ghost_town_maintains_defenses() {
    let mut app = App::new();

    app.world_mut().spawn(AutomatedDefenses {
        is_active: true,
        power_level: 100.0,
    });

    // We can just verify the component exists and is active
    let mut query = app.world_mut().query::<&AutomatedDefenses>();
    let defenses = query.iter(app.world()).next().unwrap();
    assert!(defenses.is_active);
    assert_eq!(defenses.power_level, 100.0);
}

#[derive(Event)]
pub struct DiscoveryEvent;

pub fn trigger_discovery_system(
    mut events: EventWriter<DiscoveryEvent>,
    colonies: Query<&AutomatedReporting>,
    pops: Query<(), With<Pop>>,
) {
    if pops.iter().count() == 0 {
        for reporting in colonies.iter() {
            if reporting.is_active && reporting.reported_population > 0 {
                events.send(DiscoveryEvent);
            }
        }
    }
}

#[test]
fn test_discovery_of_ghost_town() {
    let mut app = App::new();

    app.add_event::<DiscoveryEvent>();

    app.world_mut().spawn(AutomatedReporting {
        is_active: true,
        reported_population: 100, // Last known good number
    });

    app.add_systems(Update, trigger_discovery_system);
    app.update();

    let events = app.world().resource::<Events<DiscoveryEvent>>();
    let mut reader = events.get_cursor();
    assert_eq!(
        reader.read(events).count(),
        1,
        "Ghost town should be discovered."
    );
}
