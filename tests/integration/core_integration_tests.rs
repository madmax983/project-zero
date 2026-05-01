use bevy_ecs::prelude::*;
use scale::layer1::core::chronicle::AddChronicleEvent;
use scale::layer1::economy::remittances::MigrantArrivalEvent;
use scale::layer1::core::integration::beacon_migrant_arrival_bridge;

#[test]
fn test_beacon_migrant_arrival_bridge() {
    let mut world = World::new();
    world.init_resource::<Events<MigrantArrivalEvent>>();
    world.init_resource::<Events<AddChronicleEvent>>();

    let faction = world.spawn_empty().id();

    world.send_event(MigrantArrivalEvent {
        home_faction: faction,
        count: 5,
        criminal_chance: 1.0, // Force criminal
        low_skill_chance: 1.0, // Force low skill
    });

    let mut schedule = Schedule::default();
    schedule.add_systems(beacon_migrant_arrival_bridge);
    schedule.run(&mut world);

    let pops = world.query::<&scale::layer1::entities::pop::Pop>().iter(&world).count();
    assert_eq!(pops, 5);

    let mut traits_query = world.query::<&scale::layer1::psychology::traits::Traits>();
    for traits in traits_query.iter(&world) {
        assert!(traits.has(scale::layer1::psychology::traits::Trait::Greedy));
        assert!(traits.has(scale::layer1::psychology::traits::Trait::Lazy));
    }
}
