use scale::layer1::building::{Building, BuildingType, OccupiedTiles, try_place_building};
use scale::layer1::chronicle::Chronicle;
use scale::layer1::integration::pop_death_notification_system;
use scale::layer1::notifications::{NotificationQueue, NotificationSeverity};
use scale::layer1::pop::PopDied;
use scale::layer1::resources::ColonyResources;
use scale::layer1::tech::{Tech, TechState, unlock_tech};
use scale::layer1::terrain::{TerrainGrid, TerrainType};
use scale::layer1::trade::{MerchantState, TradeDepot, merchant_arrival_system};
use scale::shared::colony::ColonyName;
use scale::shared::log::MessageLog;
use scale::shared::narrative::NarrativeGenerator;
use scale::shared::time::SimulationTime;
use bevy_ecs::prelude::*;
use bevy_ecs::system::RunSystemOnce;

fn setup_world() -> World {
    let mut world = World::new();
    world.insert_resource(NotificationQueue::default());
    world.insert_resource(MessageLog::default());
    world.insert_resource(SimulationTime::default());
    world
}

#[test]
fn test_pop_death_triggers_notification() {
    let mut world = setup_world();
    world.init_resource::<Events<PopDied>>(); // Init event resource
    world.insert_resource(NarrativeGenerator::default());
    world.insert_resource(ColonyName {
        name: "Test Colony".to_string(),
    });

    // Setup event
    let mut events = world.resource_mut::<Events<PopDied>>();
    events.send(PopDied {
        entity: Entity::PLACEHOLDER,
        name: "John Doe".to_string(),
        tick: 100,
        reason: "Starvation".to_string(),
    });

    // We need to register the system or run it once.
    // However, PopDied events are usually handled by a system that reads them.
    // The current integration uses `pop_death_chronicle_bridge`.
    // We will assume we modify it or add a new one.
    // Let's use `pop_death_notification_system` for now, assuming we'll add it.

    world.run_system_once(pop_death_notification_system).unwrap();

    let queue = world.resource::<NotificationQueue>();
    assert_eq!(queue.active.len(), 1, "Should generate a notification for death");
    let notification = &queue.active[0];
    assert!(notification.text.contains("John Doe"));
    assert_eq!(notification.severity, NotificationSeverity::Error); // Death is bad
}

#[test]
fn test_research_completion_triggers_notification() {
    let mut world = setup_world();
    world.insert_resource(TechState::default());
    world.insert_resource(ColonyResources {
        knowledge: 100.0,
        ..ColonyResources::default()
    });

    let success = unlock_tech(&mut world, Tech::Masonry);
    assert!(success);

    let queue = world.resource::<NotificationQueue>();
    assert_eq!(queue.active.len(), 1, "Should generate notification for research");
    assert!(queue.active[0].text.contains("Masonry"));
    assert_eq!(queue.active[0].severity, NotificationSeverity::Success);
}

#[test]
fn test_merchant_arrival_triggers_notification() {
    let mut world = setup_world();
    world.insert_resource(MerchantState {
        cooldown: 0,
        ..Default::default()
    });
    world.insert_resource(Chronicle::default());

    // Spawn Trade Depot
    world.spawn((
        Building { building_type: BuildingType::TradeDepot },
        TradeDepot,
    ));

    merchant_arrival_system(&mut world);

    let queue = world.resource::<NotificationQueue>();
    assert_eq!(queue.active.len(), 1, "Should generate notification for merchant");
    assert!(queue.active[0].text.contains("arrived"));
    assert_eq!(queue.active[0].severity, NotificationSeverity::Info);
}

#[test]
fn test_building_placement_failure_triggers_notification() {
    let mut world = setup_world();
    world.insert_resource(TerrainGrid {
        width: 10,
        height: 10,
        tiles: vec![TerrainType::Water; 100], // All Water
    });
    world.insert_resource(OccupiedTiles::default());
    world.insert_resource(ColonyResources {
        wood: 100.0,
        ..Default::default()
    });
    // TechState is required by try_place_building
    world.insert_resource(TechState::default());

    // Try to place on Water (fails)
    let success = try_place_building(&mut world, 5, 5, BuildingType::Housing);
    assert!(!success);

    let queue = world.resource::<NotificationQueue>();
    assert_eq!(queue.active.len(), 1, "Should generate notification for placement failure");
    assert!(queue.active[0].text.contains("Cannot build"));
    assert_eq!(queue.active[0].severity, NotificationSeverity::Warning);
}
