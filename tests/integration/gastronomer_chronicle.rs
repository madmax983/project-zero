use bevy::prelude::*;
use scale::layer1::culture::gastronomers::CulinarySingularityEvent;
use scale::layer1::core::chronicle::AddChronicleEvent;
use scale::layer1::core::integration::gastronomer_chronicle_bridge;

#[test]
fn test_gastronomer_chronicle_bridge() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_event::<CulinarySingularityEvent>();
    app.add_event::<AddChronicleEvent>();

    app.add_systems(Update, gastronomer_chronicle_bridge);

    app.world_mut().send_event(CulinarySingularityEvent);
    app.update();

    let events = app.world().resource::<Events<AddChronicleEvent>>();
    let mut cursor = events.get_cursor();
    let ev = cursor.read(events).next().expect("Expected an AddChronicleEvent");
    assert_eq!(ev.text, "The Gastronomers have achieved the Culinary Singularity!");
}
