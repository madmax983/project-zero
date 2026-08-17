use bevy_app::App;
use bevy_ecs::prelude::*;
use scale::layer1::economy::biomass_dividend::RecycleEvent;
use scale::layer1::core::chronicle::{AddChronicleEvent, EventImportance};
use scale::layer1::core::integration::biomass_dividend_chronicle_bridge;

#[test]
fn test_biomass_dividend_chronicle_bridge() {
    let mut app = App::new();

    app.add_event::<RecycleEvent>();
    app.add_event::<AddChronicleEvent>();
    app.add_systems(bevy_app::Update, biomass_dividend_chronicle_bridge);

    let target = app.world_mut().spawn_empty().id();
    let processor = app.world_mut().spawn_empty().id();
    app.world_mut().send_event(RecycleEvent {
        target,
        processor,
    });

    app.update();

    let events = app.world().resource::<Events<AddChronicleEvent>>();
    let mut reader = events.get_cursor();
    let mut count = 0;
    for event in reader.read(events) {
        assert_eq!(event.importance, EventImportance::Minor);
        count += 1;
    }
    assert_eq!(count, 1);
}
