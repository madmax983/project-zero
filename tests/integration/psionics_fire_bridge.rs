use bevy::prelude::*;
use scale::layer1::map::GridPosition;
use scale::layer1::nature::fire::Fire;
use scale::layer1::psychology::psionics::FireEvent;

#[test]
fn test_psionic_fire_event_spawns_fire() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_event::<FireEvent>();
    app.add_systems(
        Update,
        scale::layer1::core::integration::psionic_fire_bridge_system,
    );

    // Initial state: no Fire entities
    assert_eq!(
        app.world_mut().query::<&Fire>().iter(app.world()).count(),
        0
    );

    let test_pos = GridPosition { x: 4, y: 7 };

    // Emit the FireEvent
    app.world_mut().send_event(FireEvent { position: test_pos });

    app.update();

    // Verify a Fire entity was spawned at the correct position
    let mut query = app.world_mut().query::<(&Fire, &GridPosition)>();
    let mut found = false;
    for (_, pos) in query.iter(app.world()) {
        if pos.x == test_pos.x && pos.y == test_pos.y {
            found = true;
            break;
        }
    }

    assert!(
        found,
        "A Fire entity should be spawned at the position specified by FireEvent"
    );
}
