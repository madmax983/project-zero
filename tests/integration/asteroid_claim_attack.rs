use bevy::prelude::*;
use scale::layer1::core::chronicle::{AddChronicleEvent, EventImportance};
use scale::layer2::orbit::asteroid_claims::{AttackColonyEvent, FactionClaim};

#[test]
fn test_asteroid_claim_attack_chronicle_bridge() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    app.add_event::<AttackColonyEvent>();
    app.add_event::<AddChronicleEvent>();

    app.add_systems(
        Update,
        scale::layer2::integration::attack_colony_chronicle_bridge,
    );

    let attacker = app
        .world_mut()
        .spawn(FactionClaim {
            name: "Pirate Syndicate".to_string(),
            hostility_level: 100,
        })
        .id();

    app.world_mut().send_event(AttackColonyEvent { attacker });

    app.update();

    let chronicle_events = app.world().resource::<Events<AddChronicleEvent>>();
    let mut reader = chronicle_events.get_cursor();

    let mut found = false;
    for event in reader.read(chronicle_events) {
        if event.importance == EventImportance::Major && event.text.contains("Pirate Syndicate") {
            found = true;
        }
    }

    assert!(
        found,
        "Chronicle event should be generated for asteroid claim attack"
    );
}
