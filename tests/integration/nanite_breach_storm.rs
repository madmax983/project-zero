use bevy::app::App;
use bevy::ecs::prelude::*;
use bevy::math::Vec2;
use scale::layer1::map::GridPosition;
use scale::layer1::nanite_fabrication::ContainmentBreachEvent;
use scale::layer1::nanite_storms::{ActiveNaniteStorm, NaniteStormType};
use scale::layer1::integration::nanite_breach_storm_bridge;

#[test]
fn test_nanite_breach_spawns_storm() {
    let mut app = App::new();

    app.add_event::<ContainmentBreachEvent>();
    app.add_systems(bevy::app::Update, nanite_breach_storm_bridge);

    app.world_mut().send_event(ContainmentBreachEvent {
        source_entity: Entity::PLACEHOLDER,
        position: GridPosition { x: 50, y: 50 },
    });

    app.update();

    let storm = app
        .world()
        .get_resource::<ActiveNaniteStorm>()
        .expect("Storm should be spawned");
    assert!(storm.storm_type == NaniteStormType::Grey);
    assert!(storm.affected_area.contains(Vec2::new(50.0, 50.0)));
}
