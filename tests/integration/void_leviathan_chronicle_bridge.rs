use bevy_app::App;
use bevy_ecs::prelude::*;
use scale::layer1::core::chronicle::AddChronicleEvent;
use scale::layer2::integration::void_leviathan_chronicle_bridge;
use scale::layer2::void_leviathan::VoidLeviathan;

#[test]
fn test_void_leviathan_chronicle_bridge() {
    let mut app = App::new();
    app.add_event::<AddChronicleEvent>();
    app.insert_resource(VoidLeviathan { active: false, duration: 0 });
    app.add_systems(bevy_app::Update, void_leviathan_chronicle_bridge);

    app.update();

    app.world_mut().resource_mut::<VoidLeviathan>().active = true;
    app.update();

    let chronicle_events = app.world().resource::<Events<AddChronicleEvent>>();
    let mut iter = chronicle_events.get_cursor();
    let mut found_arrival = false;
    for event in iter.read(chronicle_events) {
        if event.text.contains("A colossal Void Leviathan has entered the system") {
            found_arrival = true;
        }
    }
    assert!(found_arrival, "Should log arrival");

    app.world_mut().resource_mut::<VoidLeviathan>().active = false;
    app.update();

    let chronicle_events2 = app.world().resource::<Events<AddChronicleEvent>>();
    let mut iter2 = chronicle_events2.get_cursor();
    let mut found_departure = false;
    for event in iter2.read(chronicle_events2) {
        if event.text.contains("The Void Leviathan has departed the system") {
            found_departure = true;
        }
    }
    assert!(found_departure, "Should log departure");
}
