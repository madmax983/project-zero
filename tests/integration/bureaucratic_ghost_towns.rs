use bevy::prelude::*;
use scale::layer1::core::chronicle::AddChronicleEvent;
use scale::layer1::economy::resources::ColonyResources;

use scale::layer3::bureaucracy::{
    colony_reporting_system, empire_resource_distribution_system, AutomatedDefenses,
    AutomatedReporting,
};
use scale::layer3::integration::{discover_ghost_town_system, GhostTownDiscovered};

#[test]
fn test_ghost_town_continues_receiving_shipments() {
    let mut app = App::new();
    app.init_resource::<ColonyResources>();
    app.add_systems(
        Update,
        (colony_reporting_system, empire_resource_distribution_system).chain(),
    );

    let _colony_id = app
        .world_mut()
        .spawn((AutomatedReporting {
            is_active: true,
            reported_population: 100, // Still reporting a population
        },))
        .id();

    app.update(); // Tick updates reporting (or fails to), then distributions run

    let resources = app.world().resource::<ColonyResources>();
    assert!(resources.food > 0.0, "Ghost town should have received food");
}

#[test]
fn test_ghost_town_maintains_defenses() {
    let mut app = App::new();

    let colony_id = app
        .world_mut()
        .spawn((AutomatedDefenses {
            power_level: 100.0,
            is_active: true,
        },))
        .id();

    let defense = app.world().get::<AutomatedDefenses>(colony_id).unwrap();
    assert!(defense.is_active, "Defenses should remain active");
    assert_eq!(defense.power_level, 100.0, "Defenses should maintain power");
}

#[test]
fn test_discovery_of_ghost_town() {
    let mut app = App::new();
    app.add_event::<AddChronicleEvent>();
    app.init_resource::<ColonyResources>();
    app.add_systems(Update, discover_ghost_town_system);

    // Start with a dead colony that's still reporting and hoarding
    app.world_mut().resource_mut::<ColonyResources>().food = 1000.0;

    let colony_id = app
        .world_mut()
        .spawn((AutomatedReporting {
            is_active: true,
            reported_population: 100,
        },))
        .id();

    app.update();

    let events = app.world().resource::<Events<AddChronicleEvent>>();
    let mut cursor = events.get_cursor();
    assert!(
        cursor.read(events).next().is_some(),
        "AddChronicleEvent should be triggered"
    );

    let is_discovered = app.world().get::<GhostTownDiscovered>(colony_id).is_some();
    assert!(
        is_discovered,
        "GhostTownDiscovered marker component should be added to the entity"
    );
}
