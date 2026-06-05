use bevy::prelude::*;
use scale::layer1::chronicle::AddChronicleEvent;
use scale::layer1::geology::fracking::FrackEvent;
use scale::layer1::core::integration::tectonic_fracking_chronicle_bridge;

#[test]
pub fn test_tectonic_fracking_chronicle_bridge() {
    let mut app = App::new();

    app.add_event::<FrackEvent>();
    app.add_event::<AddChronicleEvent>();

    app.add_systems(Update, tectonic_fracking_chronicle_bridge);

    let entity = app.world_mut().spawn_empty().id();
    app.world_mut().send_event(FrackEvent { entity });

    app.update();

    let chronicle_events = app.world().resource::<Events<AddChronicleEvent>>();
    assert!(
        !chronicle_events.is_empty(),
        "Fracking should trigger a chronicle event"
    );
}
