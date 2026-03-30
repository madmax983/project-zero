use bevy_app::App;
use bevy_ecs::prelude::*;
use scale::layer1::whispering_ore::{MineSealedEvent, Rebelling, ResonantTrait};
use scale::layer1::chronicle::{AddChronicleEvent, EventImportance};
use scale::layer1::pop::Pop;
use scale::layer1::integration::whispering_ore_rebellion_chronicle_bridge;

#[test]
fn test_whispering_ore_rebellion_chronicle_bridge() {
    let mut app = App::new();
    app.add_event::<MineSealedEvent>();
    app.add_event::<AddChronicleEvent>();

    app.add_systems(bevy_app::Update, (
        scale::layer1::whispering_ore::handle_mine_sealing_system,
        whispering_ore_rebellion_chronicle_bridge
    ).chain());

    let pop = app.world_mut().spawn((Pop, ResonantTrait)).id();

    app.world_mut().send_event(MineSealedEvent { vein_id: 1 });
    app.update();

    assert!(app.world().get::<Rebelling>(pop).is_some());

    let events = app.world().get_resource::<Events<AddChronicleEvent>>().unwrap();
    let mut reader = events.get_cursor();
    let iter: Vec<_> = reader.read(events).collect();

    assert_eq!(iter.len(), 1);
    assert_eq!(iter[0].importance, EventImportance::Legendary);
}
