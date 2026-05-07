use bevy::prelude::*;
use scale::layer1::pop::Pop;
use scale::layer1::social::Tavern;
use scale::layer1::culture::nostalgia::{Nostalgia, RumorSpreadEvent};
use scale::layer1::core::integration::nostalgia_tavern_bridge_system;

#[derive(Resource, Default)]
struct EventCounter {
    count: usize,
}

fn count_rumor_events(mut events: EventReader<RumorSpreadEvent>, mut counter: ResMut<EventCounter>) {
    for _ in events.read() {
        counter.count += 1;
    }
}

#[test]
fn test_nostalgia_tavern_bridge_emits_rumor() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_systems(Update, (nostalgia_tavern_bridge_system, count_rumor_events).chain());
    app.add_event::<RumorSpreadEvent>();
    app.init_resource::<EventCounter>();

    let mut tavern = Tavern::default();

    let nostalgic_pop = app.world_mut().spawn((Pop, Nostalgia)).id();
    let normal_pop = app.world_mut().spawn(Pop).id();

    tavern.visitors.push(nostalgic_pop);
    tavern.visitors.push(normal_pop);

    app.world_mut().spawn(tavern);

    // Run multiple times to trigger the 10% chance
    for _ in 0..100 {
        app.update();
    }

    let counter = app.world().resource::<EventCounter>();
    assert!(counter.count > 0, "Should emit RumorSpreadEvent when socializing with nostalgic pop");
}
