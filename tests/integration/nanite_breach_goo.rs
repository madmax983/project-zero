use bevy::app::App;
use scale::layer1::integration::nanite_breach_goo_bridge;
use scale::layer1::map::GridPosition;
use scale::layer1::nanite_fabrication::{ContainmentBreachEvent, GreyGoo};

#[test]
fn test_nanite_breach_spawns_grey_goo() {
    let mut app = App::new();

    app.add_event::<ContainmentBreachEvent>();
    app.add_systems(bevy::app::Update, nanite_breach_goo_bridge);

    let forge_entity = app.world_mut().spawn_empty().id();
    let position = GridPosition { x: 50, y: 50 };

    app.world_mut().send_event(ContainmentBreachEvent {
        source_entity: forge_entity,
        position,
    });

    app.update();

    // The forge should be despawned
    assert!(app.world().get_entity(forge_entity).is_err());

    // A GreyGoo should be spawned at the position
    let mut goo_found = false;
    let mut query = app.world_mut().query::<(&GreyGoo, &GridPosition)>();
    for (goo, pos) in query.iter(app.world()) {
        if pos.x == position.x && pos.y == position.y {
            assert_eq!(goo.replication_progress, 0.0);
            goo_found = true;
            break;
        }
    }
    assert!(goo_found, "GreyGoo was not spawned by the breach event");
}
