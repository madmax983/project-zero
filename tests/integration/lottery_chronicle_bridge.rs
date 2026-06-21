use bevy::prelude::*;
use scale::layer1::administration::edicts::ColonyPolicies;
use scale::layer1::administration::edicts::Policy;
use scale::layer1::core::chronicle::AddChronicleEvent;
use scale::layer1::core::integration::lottery_chronicle_bridge;
use scale::layer1::entities::pop::Pop;
use scale::layer1::psychology::memory::Memories;
use scale::layer1::psychology::traits::Traits;
use scale::layer1::the_lottery::{execute_lottery_system, LotteryExecutedEvent};
use std::collections::HashSet;

#[test]
fn test_lottery_chronicle_bridge() {
    let mut app = App::new();
    app.add_plugins(bevy_time::TimePlugin);

    let mut set = HashSet::new();
    set.insert(Policy::TheLottery);
    app.insert_resource(ColonyPolicies {
        active_policies: set,
        ..Default::default()
    });

    app.add_event::<LotteryExecutedEvent>();
    app.add_event::<AddChronicleEvent>();

    app.add_systems(
        Update,
        (execute_lottery_system, lottery_chronicle_bridge).chain(),
    );

    // Spawn 3 pops
    app.world_mut()
        .spawn((Pop, Memories::default(), Traits::default()));
    app.world_mut()
        .spawn((Pop, Memories::default(), Traits::default()));
    app.world_mut()
        .spawn((Pop, Memories::default(), Traits::default()));

    app.update();

    let chronicle_events = app.world().resource::<Events<AddChronicleEvent>>();
    #[allow(deprecated)]
    let mut reader = chronicle_events.get_reader();
    let events: Vec<_> = reader.read(chronicle_events).collect();

    assert_eq!(
        events.len(),
        1,
        "Should emit exactly 1 chronicle event for the lottery"
    );

    assert!(
        events[0].text.contains("The Lottery was executed."),
        "Event text should mention the lottery execution."
    );
    assert!(
        events[0].text.contains("1 Pop(s) sacrificed"),
        "Event text should mention 1 pop sacrificed."
    );
    assert!(
        events[0].text.contains("2 might live"),
        "Event text should mention 2 survivors."
    );
}
