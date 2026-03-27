use bevy::prelude::*;
use scale::layer1::flora::{Flora, FloraType};
use scale::layer1::integration::{
    diplomatic_reflection_kill_bridge, diplomatic_reflection_plant_bridge,
};
use scale::layer1::map::GridPosition;
use scale::layer1::pop::PopDied;
use scale::layer3::diplomacy_reflection::{EntityKilledEvent, FloraPlantedEvent};

#[test]
fn test_pop_died_triggers_entity_killed() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    app.add_event::<PopDied>();
    app.add_event::<EntityKilledEvent>();

    app.add_systems(Update, diplomatic_reflection_kill_bridge);

    app.world_mut().send_event(PopDied {
        entity: Entity::PLACEHOLDER,
        name: "Test Pop".to_string(),
        reason: "Unknown".to_string(),
        tick: 0,
    });
    app.update();

    let events = app.world().resource::<Events<EntityKilledEvent>>();
    assert_eq!(events.len(), 1);
}

#[test]
fn test_flora_added_triggers_flora_planted() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    app.add_event::<FloraPlantedEvent>();

    app.add_systems(Update, diplomatic_reflection_plant_bridge);

    let mut flora = Flora::default();
    flora.flora_type = FloraType::XenoMoss;

    app.world_mut().spawn((flora, GridPosition { x: 0, y: 0 }));
    app.update();

    let events = app.world().resource::<Events<FloraPlantedEvent>>();
    assert_eq!(events.len(), 1);
}
