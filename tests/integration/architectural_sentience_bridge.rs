use bevy_app::prelude::*;
use bevy_ecs::prelude::*;
use scale::layer1::architecture_sentience::{
    architectural_sentience_chronicle_bridge, architectural_union_trigger_system,
    sentient_architecture_strike_system, AutomatedInfrastructure,
};
use scale::layer1::core::chronicle::AddChronicleEvent;
use scale::layer1::social::Unrest;

#[test]
fn test_architectural_sentience_chronicle_bridge() {
    let mut app = App::new();
    app.add_plugins(bevy::MinimalPlugins);
    app.add_event::<AddChronicleEvent>();
    app.add_systems(
        Update,
        (
            architectural_union_trigger_system,
            apply_deferred,
            sentient_architecture_strike_system,
            architectural_sentience_chronicle_bridge,
        )
            .chain(),
    );

    app.insert_resource(Unrest {
        level: 90.0,
        ..Default::default()
    });

    app.world_mut().spawn(AutomatedInfrastructure);

    app.update();

    let events = app.world().resource::<Events<AddChronicleEvent>>();
    let reader = events.get_cursor();
    assert!(
        reader.len(events) > 0,
        "Chronicle event should be emitted when architecture gains sentience."
    );
}
