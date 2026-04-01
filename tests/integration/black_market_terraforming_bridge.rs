use bevy::prelude::*;
use scale::layer1::chronicle::{AddChronicleEvent, EventImportance};
use scale::layer3::integration::black_market_terraforming_bridge;
use scale::layer3::planet::black_market_terraforming::RogueTerraformEvent;
use scale::shared::time::SimulationTime;

#[test]
fn test_black_market_terraforming_bridge() {
    let mut app = App::new();
    app.add_plugins(bevy::MinimalPlugins);
    app.init_resource::<SimulationTime>();
    app.add_event::<RogueTerraformEvent>();
    app.add_event::<AddChronicleEvent>();

    app.add_systems(Update, black_market_terraforming_bridge);

    app.world_mut()
        .send_event(RogueTerraformEvent { target_sector: 42 });

    app.update();

    let chronicle_events = app.world().resource::<Events<AddChronicleEvent>>();
    let mut reader = chronicle_events.get_cursor();
    let events: Vec<&AddChronicleEvent> = reader.read(chronicle_events).collect();

    assert_eq!(
        events.len(),
        1,
        "Should emit exactly one AddChronicleEvent for the rogue terraforming"
    );
    assert_eq!(events[0].importance, EventImportance::Major);
}
