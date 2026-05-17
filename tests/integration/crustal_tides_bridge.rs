use bevy::prelude::*;
use scale::layer1::core::chronicle::AddChronicleEvent;
use scale::layer1::core::integration::crustal_tide_chronicle_bridge;
use scale::layer2::syzygy::TidalForce;

fn setup_app() -> App {
    let mut app = App::new();
    app.init_resource::<Events<AddChronicleEvent>>();
    app.insert_resource(TidalForce { current: 0.5, base: 0.5 });
    app.add_systems(Update, crustal_tide_chronicle_bridge);
    app
}

#[test]
fn test_high_tide_emits_event() {
    let mut app = setup_app();

    app.update();
    let events = app.world().resource::<Events<AddChronicleEvent>>();
    assert_eq!(events.get_cursor().read(events).count(), 0);

    // Shift to high tide
    app.world_mut().resource_mut::<TidalForce>().current = 0.8;
    app.update();

    let events = app.world().resource::<Events<AddChronicleEvent>>();
    assert_eq!(events.get_cursor().read(events).count(), 1);
}

#[test]
fn test_low_tide_emits_event() {
    let mut app = setup_app();

    app.update();
    let events = app.world().resource::<Events<AddChronicleEvent>>();
    assert_eq!(events.get_cursor().read(events).count(), 0);

    // Shift to low tide
    app.world_mut().resource_mut::<TidalForce>().current = 0.2;
    app.update();

    let events = app.world().resource::<Events<AddChronicleEvent>>();
    assert_eq!(events.get_cursor().read(events).count(), 1);
}
