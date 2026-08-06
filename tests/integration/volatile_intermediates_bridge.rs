use bevy_app::prelude::*;
use bevy_ecs::prelude::*;
use scale::layer1::core::chronicle::AddChronicleEvent;
use scale::layer1::environment::volatile::ExplosionEvent;
use scale::layer1::core::integration::volatile_explosion_chronicle_bridge;
use scale::layer1::map::GridPosition;

#[test]
fn test_volatile_explosion_chronicle_bridge() {
    let mut app = App::new();
    app.add_event::<ExplosionEvent>();
    app.add_event::<AddChronicleEvent>();
    app.add_systems(Update, volatile_explosion_chronicle_bridge);

    let mut events = app.world_mut().resource_mut::<Events<ExplosionEvent>>();
    events.send(ExplosionEvent {
        center: GridPosition { x: 5, y: 5 },
        damage: 50.0,
        radius: 2,
    });

    app.update();

    let chronicle_events = app.world().resource::<Events<AddChronicleEvent>>();
    let mut reader = chronicle_events.get_cursor();
    let events_list: Vec<_> = reader.read(chronicle_events).collect();

    assert_eq!(events_list.len(), 1, "Should emit one Chronicle event");
    assert!(events_list[0].text.contains("A volatile intermediate catastrophically destabilized at (5, 5)"));
}
