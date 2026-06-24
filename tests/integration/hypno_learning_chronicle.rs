use bevy_ecs::prelude::*;
use scale::layer1::core::chronicle::AddChronicleEvent;
use scale::layer1::pop::PopName;
use scale::layer1::tech::hypno_learning::MentalFog;
use scale::layer1::core::integration::hypno_learning_chronicle_bridge;

#[test]
fn test_hypno_learning_chronicle_bridge() {
    let mut app = bevy_app::App::new();
    app.add_event::<AddChronicleEvent>();
    app.add_systems(bevy_app::Update, hypno_learning_chronicle_bridge);

    let _pop = app
        .world_mut()
        .spawn((PopName("Test Pop".to_string()), MentalFog::default()))
        .id();

    app.update();

    let chronicle_events = app.world().resource::<Events<AddChronicleEvent>>();
    let mut iter = chronicle_events.get_cursor();
    let mut found = false;

    for event in iter.read(chronicle_events) {
        if event.text.contains("Test Pop") && event.text.contains("Hypno-Learning") {
            found = true;
            break;
        }
    }

    assert!(found, "Chronicle event for Hypno-Learning should have been emitted");
}
